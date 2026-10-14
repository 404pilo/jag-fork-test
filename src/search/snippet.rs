//! Results with the lines around a match (STORY-14, CRITERION-21).
//!
//! `plan_windows` joins the matches of each file into windows. It reads no
//! file. `read_snippets` reads each file of the kept windows one time and
//! gives the lines of each window.

use std::borrow::Cow;
use std::path::PathBuf;

use super::walk::{FileMatches, WalkWarning, WarningKind};

/// The lines before and after a match.
pub const CONTEXT_LINES: u64 = 3;
/// The most lines in one result.
pub const MAX_LINES: u64 = 12;

/// The most characters of a line in a request and in the output (decision 21).
pub const MAX_LINE_CHARS: usize = 300;

/// Where the text of a result came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    File(PathBuf),
    StandardInput,
}

impl Source {
    /// The label of standard input, in the place of a path.
    pub const STANDARD_INPUT_LABEL: &'static str = "(standard input)";

    /// The bytes that show the source: the path as typed, or the label.
    pub fn label(&self) -> &[u8] {
        match self {
            Source::File(path) => path.as_os_str().as_encoded_bytes(),
            Source::StandardInput => Self::STANDARD_INPUT_LABEL.as_bytes(),
        }
    }
}

/// One line of a result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnippetLine {
    /// The number of the line. The first line is 1.
    pub number: u64,
    /// The bytes of the line that jag keeps, with no `\n`. A `\r` at the end
    /// stays. A line of more than `MAX_LINE_CHARS` characters is cut.
    pub text: Vec<u8>,
    pub is_match: bool,
    /// The count of characters cut from the end. 0 for no cut.
    pub cut: u64,
}

impl SnippetLine {
    /// A line from its full bytes. The line is cut at `MAX_LINE_CHARS`.
    pub fn new(number: u64, line: &[u8], is_match: bool) -> SnippetLine {
        let (kept, cut) = cut_line(line);
        SnippetLine {
            number,
            text: kept.to_vec(),
            is_match,
            cut,
        }
    }
}

/// Keep the first `MAX_LINE_CHARS` characters of a line. Gives the kept bytes
/// and the count of the other characters. Each bad sequence of bytes counts as
/// one character, as in the request (`from_utf8_lossy` gives one U+FFFD for
/// it).
pub fn cut_line(bytes: &[u8]) -> (&[u8], u64) {
    let mut count = 0usize;
    let mut offset = 0usize;
    let mut kept = 0usize;
    for width in char_widths(bytes) {
        offset += width;
        count += 1;
        if count == MAX_LINE_CHARS {
            kept = offset;
        }
    }
    if count <= MAX_LINE_CHARS {
        (bytes, 0)
    } else {
        (&bytes[..kept], (count - MAX_LINE_CHARS) as u64)
    }
}

/// The width in bytes of each character of `String::from_utf8_lossy(bytes)`.
/// Each bad sequence of bytes is one character. The cut and the count of
/// characters use this one rule.
fn char_widths(bytes: &[u8]) -> impl Iterator<Item = usize> + '_ {
    bytes.utf8_chunks().flat_map(|chunk| {
        let bad = chunk.invalid().len();
        chunk
            .valid()
            .chars()
            .map(char::len_utf8)
            .chain((bad > 0).then_some(bad))
    })
}

/// The first `chars` characters of `bytes`: a prefix that ends at a character,
/// never inside one.
pub fn char_prefix(bytes: &[u8], chars: usize) -> &[u8] {
    let end: usize = char_widths(bytes).take(chars).sum();
    &bytes[..end]
}

/// The count of characters of `String::from_utf8_lossy(bytes)`, with no copy
/// (the unit of the size limit, decision 2 of FEATURE-09).
pub fn char_count(bytes: &[u8]) -> usize {
    char_widths(bytes).count()
}

/// One result: a match with the lines around it (IF-6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snippet {
    pub source: Source,
    pub lines: Vec<SnippetLine>,
}

impl Snippet {
    /// The number of the first line. 0 when the result has no line.
    pub fn first_line(&self) -> u64 {
        self.lines.first().map_or(0, |line| line.number)
    }

    /// The number of the last line. 0 when the result has no line.
    pub fn last_line(&self) -> u64 {
        self.lines.last().map_or(0, |line| line.number)
    }
}

/// The lines around each match (FEATURE-13, decisions 13 to 17).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextLines {
    pub before: u64,
    pub after: u64,
    /// The most lines of one result. `None`: no limit (plain grep).
    pub max_lines: Option<u64>,
}

impl ContextLines {
    /// 3 lines before, 3 lines after, and at most 12 lines (FEATURE-04 spec
    /// decision 4).
    pub const DEFAULT: ContextLines = ContextLines {
        before: CONTEXT_LINES,
        after: CONTEXT_LINES,
        max_lines: Some(MAX_LINES),
    };
}

/// The plan of one result. The plan uses only line numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    pub path: PathBuf,
    pub start: u64,
    pub end: u64,
    /// The numbers of the matching lines in the window, in line order.
    pub matches: Vec<u64>,
}

/// Join the matches of each file into windows of `context.max_lines` lines
/// or less, with `context.before` and `context.after` lines around each
/// match. With no limit, windows that touch or overlap always join.
///
/// The windows are in the order of `files`, then in line order.
pub fn plan_windows(files: &[FileMatches], context: ContextLines) -> Vec<Window> {
    // The last line that a window from `start` can hold.
    let last_line = |start: u64| {
        context
            .max_lines
            .map_or(u64::MAX, |max| start.saturating_add(max.saturating_sub(1)))
    };
    let mut windows = Vec::new();
    for file in files {
        let mut current: Option<Window> = None;
        for line in &file.lines {
            let m = line.line_number;
            current = Some(match current.take() {
                None => open_window(&file.path, m, 1, context),
                Some(mut window) => {
                    if m <= last_line(window.start)
                        && m.saturating_sub(context.before) <= window.end.saturating_add(1)
                    {
                        window.matches.push(m);
                        window.end = window
                            .end
                            .max(m.saturating_add(context.after))
                            .min(last_line(window.start));
                        window
                    } else {
                        let first_free = window.end.saturating_add(1);
                        windows.push(window);
                        open_window(&file.path, m, first_free, context)
                    }
                }
            });
        }
        windows.extend(current);
    }
    windows
}

/// A new window for the match `m`. No line before `first_free` is in it.
fn open_window(path: &std::path::Path, m: u64, first_free: u64, context: ContextLines) -> Window {
    Window {
        path: path.to_path_buf(),
        start: m.saturating_sub(context.before).max(first_free),
        end: m.saturating_add(context.after),
        matches: vec![m],
    }
}

/// One window for each matching line, with no line around it (FEATURE-12,
/// decision 6). The windows are in the order of `files`, then in line order.
pub fn line_windows(files: &[FileMatches]) -> Vec<Window> {
    files
        .iter()
        .flat_map(|file| {
            file.lines.iter().map(|line| Window {
                path: file.path.clone(),
                start: line.line_number,
                end: line.line_number,
                matches: vec![line.line_number],
            })
        })
        .collect()
}

/// The text of the matching lines of each window, in the order of `windows`.
///
/// `windows` must be the output of `plan_windows(files, context)`. That function puts
/// each matching line of `files` in exactly one window, in order. So window
/// k holds the next `windows[k].matches.len()` matching lines. This reads no
/// file.
pub fn match_texts<'a>(files: &'a [FileMatches], windows: &[Window]) -> Vec<Vec<&'a [u8]>> {
    let mut lines = files.iter().flat_map(|file| file.lines.iter());
    windows
        .iter()
        .map(|window| {
            window
                .matches
                .iter()
                .map(|number| {
                    let line = lines.next().expect("a matching line for each window line");
                    assert_eq!(
                        line.line_number, *number,
                        "windows must come from plan_windows"
                    );
                    line.text.as_slice()
                })
                .collect()
        })
        .collect()
}

/// Read each file of the windows one time and give the lines of each window.
///
/// A file that jag cannot read gives a warning and no result. The file can
/// change after the scan, so the end of a window stops at the last line, and
/// a match past the last line is dropped.
pub fn read_snippets(windows: &[Window]) -> (Vec<Snippet>, Vec<WalkWarning>) {
    let mut snippets = Vec::new();
    let mut warnings = Vec::new();
    let mut index = 0;
    while index < windows.len() {
        let path = &windows[index].path;
        let group_end = windows[index..]
            .iter()
            .position(|window| &window.path != path)
            .map_or(windows.len(), |offset| index + offset);
        match std::fs::read(path) {
            Err(error) => warnings.push(WalkWarning {
                path: Some(path.clone()),
                kind: WarningKind::Read(error.kind()),
            }),
            Ok(bytes) => {
                let bytes = decode(&bytes);
                let lines = split_lines(&bytes);
                for window in &windows[index..group_end] {
                    snippets.extend(snippet_of(window, &lines));
                }
            }
        }
        index = group_end;
    }
    (snippets, warnings)
}

/// Give the text as the scan sees it: a byte order mark of UTF-16 means the
/// file is changed to UTF-8, and a mark of UTF-8 is removed. So the line
/// numbers and the lines are the same as in the scan.
fn decode(bytes: &[u8]) -> Cow<'_, [u8]> {
    let little = bytes.starts_with(&[0xFF, 0xFE]);
    if little || bytes.starts_with(&[0xFE, 0xFF]) {
        let body = &bytes[2..];
        let (pairs, _) = body.as_chunks::<2>();
        let mut units: Vec<u16> = pairs
            .iter()
            .map(|pair| {
                if little {
                    u16::from_le_bytes(*pair)
                } else {
                    u16::from_be_bytes(*pair)
                }
            })
            .collect();
        if body.len() % 2 == 1 {
            units.push(0xFFFD);
        }
        return Cow::Owned(String::from_utf16_lossy(&units).into_bytes());
    }
    match bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        Some(rest) => Cow::Borrowed(rest),
        None => Cow::Borrowed(bytes),
    }
}

/// Split on `\n`. A file that ends with `\n` has no empty last line.
fn split_lines(bytes: &[u8]) -> Vec<&[u8]> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<&[u8]> = bytes.split(|byte| *byte == b'\n').collect();
    if bytes.ends_with(b"\n") {
        lines.pop();
    }
    lines
}

fn snippet_of(window: &Window, lines: &[&[u8]]) -> Option<Snippet> {
    let last = lines.len() as u64;
    let matches: Vec<u64> = window
        .matches
        .iter()
        .copied()
        .filter(|m| *m <= last)
        .collect();
    if matches.is_empty() {
        return None;
    }
    let end = window.end.min(last);
    let lines = (window.start..=end)
        .map(|number| {
            SnippetLine::new(
                number,
                lines[(number - 1) as usize],
                matches.binary_search(&number).is_ok(),
            )
        })
        .collect();
    Some(Snippet {
        source: Source::File(window.path.clone()),
        lines,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::path::Path;

    use tempfile::TempDir;

    use super::*;
    use crate::search::walk::MatchLine;

    fn file(path: &str, numbers: &[u64]) -> FileMatches {
        FileMatches {
            path: PathBuf::from(path),
            lines: numbers
                .iter()
                .map(|number| MatchLine {
                    line_number: *number,
                    text: b"x".to_vec(),
                })
                .collect(),
        }
    }

    #[test]
    fn line_windows_gives_one_window_for_each_matching_line() {
        let windows = line_windows(&[file("a", &[1, 3]), file("b", &[2])]);
        let got: Vec<_> = windows
            .iter()
            .map(|window| {
                (
                    window.path.to_str().expect("text"),
                    window.start,
                    window.end,
                    window.matches.clone(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                ("a", 1, 1, vec![1]),
                ("a", 3, 3, vec![3]),
                ("b", 2, 2, vec![2])
            ]
        );
    }

    #[test]
    fn line_windows_read_as_one_line_snippets_that_keep_a_carriage_return() {
        let dir = tempfile::tempdir().expect("folder");
        fs::write(dir.path().join("a.txt"), b"one\r\ntwo\nthree\n").expect("write");
        let path = dir.path().join("a.txt");
        let files = [FileMatches {
            path,
            lines: vec![
                MatchLine {
                    line_number: 1,
                    text: b"one".to_vec(),
                },
                MatchLine {
                    line_number: 3,
                    text: b"three".to_vec(),
                },
            ],
        }];
        let (snippets, warnings) = read_snippets(&line_windows(&files));
        assert!(warnings.is_empty());
        assert_eq!(snippets.len(), 2);
        assert_eq!(snippets[0].lines.len(), 1);
        assert_eq!(snippets[0].lines[0].text, b"one\r");
        assert!(snippets[0].lines[0].is_match);
        assert_eq!(snippets[1].lines[0].number, 3);
    }

    /// The `(start, end)` of each window for one file.
    fn spans(numbers: &[u64]) -> Vec<(u64, u64)> {
        plan_windows(&[file("a", numbers)], ContextLines::DEFAULT)
            .iter()
            .map(|window| (window.start, window.end))
            .collect()
    }

    /// The `(start, end)` of each window for one file, with `context`.
    fn spans_with(
        numbers: &[u64],
        before: u64,
        after: u64,
        max_lines: Option<u64>,
    ) -> Vec<(u64, u64)> {
        let context = ContextLines {
            before,
            after,
            max_lines,
        };
        plan_windows(&[file("a", numbers)], context)
            .iter()
            .map(|window| (window.start, window.end))
            .collect()
    }

    #[test]
    fn only_lines_after_give_no_line_before() {
        assert_eq!(spans_with(&[5], 0, 2, Some(12)), [(5, 7)]);
    }

    #[test]
    fn twelve_lines_on_each_side_give_a_window_of_25_lines() {
        assert_eq!(spans_with(&[30], 12, 12, Some(25)), [(18, 42)]);
    }

    #[test]
    fn with_no_limit_groups_that_touch_join_and_others_do_not() {
        assert_eq!(spans_with(&[4, 6], 1, 1, None), [(3, 7)]);
        assert_eq!(spans_with(&[4, 10], 1, 1, None), [(3, 5), (9, 11)]);
        // Lines 3 to 5 and 6 to 8 touch, so they join. Line 6 is a gap
        // between 3 to 5 and 7 to 9.
        assert_eq!(spans_with(&[4, 7], 1, 1, None), [(3, 8)]);
        assert_eq!(spans_with(&[4, 8], 1, 1, None), [(3, 5), (7, 9)]);
        assert_eq!(spans_with(&[4, 5], 0, 0, None), [(4, 5)]);
        assert_eq!(spans_with(&[4, 10], 0, 0, None), [(4, 4), (10, 10)]);
        let many: Vec<u64> = (1..=100).step_by(2).collect();
        assert_eq!(spans_with(&many, 1, 1, None), [(1, 100)]);
    }

    #[test]
    fn a_very_large_value_does_not_overflow() {
        assert_eq!(spans_with(&[5, 9], 0, u64::MAX, None), [(5, u64::MAX)]);
        assert_eq!(
            spans_with(&[5], u64::MAX, u64::MAX, Some(u64::MAX)),
            [(1, u64::MAX)]
        );
        assert_eq!(
            spans_with(&[u64::MAX], 3, 3, Some(12)),
            [(u64::MAX - 3, u64::MAX)]
        );
    }

    #[test]
    fn the_default_context_gives_the_windows_of_feature_04() {
        assert_eq!(spans_with(&[10, 16], 3, 3, Some(12)), spans(&[10, 16]));
        assert_eq!(ContextLines::DEFAULT.max_lines, Some(MAX_LINES));
    }

    fn cut(bytes: &[u8]) -> (usize, u64) {
        let (kept, cut) = cut_line(bytes);
        (kept.len(), cut)
    }

    #[test]
    fn a_line_of_300_characters_is_not_cut() {
        assert_eq!(cut(&[b'a'; 300]), (300, 0));
        assert_eq!(cut(b""), (0, 0));
    }

    #[test]
    fn a_line_of_more_than_300_characters_is_cut() {
        assert_eq!(cut(&[b'a'; 301]), (300, 1));
        assert_eq!(cut(&[b'a'; 1000]), (300, 700));
    }

    #[test]
    fn a_multibyte_character_counts_as_one_character() {
        let line = "漢".repeat(400);
        assert_eq!(cut(line.as_bytes()), (900, 100));
    }

    #[test]
    fn a_byte_that_is_not_utf8_counts_as_one_character() {
        assert_eq!(cut(&[0xFF; 400]), (300, 100));
    }

    #[test]
    fn a_bad_sequence_counts_as_one_character() {
        let mut line = vec![b'a'; 299];
        line.extend([0xE2, 0x82]);
        line.extend([b'b'; 10]);
        assert_eq!(cut(&line), (301, 10));
    }

    #[test]
    fn a_carriage_return_counts_as_one_character() {
        let mut line = vec![b'a'; 300];
        line.push(b'\r');
        assert_eq!(cut(&line), (300, 1));
    }

    #[test]
    fn the_kept_bytes_of_valid_text_are_valid_text() {
        for line in ["é".repeat(500), "漢".repeat(500), "😀".repeat(500)] {
            let (kept, cut) = cut_line(line.as_bytes());
            assert!(std::str::from_utf8(kept).is_ok());
            assert_eq!(
                std::str::from_utf8(kept).expect("text").chars().count(),
                300
            );
            assert_eq!(cut, 200);
        }
    }

    #[test]
    fn a_new_line_is_cut_at_300_characters() {
        let line = SnippetLine::new(4, &[b'x'; 1000], true);
        assert_eq!((line.number, line.text.len(), line.cut), (4, 300, 700));
        assert!(line.is_match);
    }

    #[test]
    fn a_file_line_of_1000_characters_is_read_as_300_and_a_cut_of_700() {
        let (_dir, path) = folder_with("a.txt", format!("{}\n", "x".repeat(1000)).as_bytes());
        let (snippets, _) = read_snippets(&[window(&path, 1, 1, &[1])]);
        assert_eq!(snippets[0].lines[0].text.len(), 300);
        assert_eq!(snippets[0].lines[0].cut, 700);
    }

    #[test]
    fn a_result_gives_its_first_and_last_line() {
        let snippet = Snippet {
            source: Source::StandardInput,
            lines: (7..=17)
                .map(|n| SnippetLine::new(n, b"x", n == 10))
                .collect(),
        };
        assert_eq!((snippet.first_line(), snippet.last_line()), (7, 17));
        let empty = Snippet {
            source: Source::StandardInput,
            lines: Vec::new(),
        };
        assert_eq!((empty.first_line(), empty.last_line()), (0, 0));
    }

    #[test]
    fn the_label_of_standard_input_is_fixed() {
        assert_eq!(Source::StandardInput.label(), b"(standard input)");
        assert_eq!(Source::File(PathBuf::from("a/b")).label(), b"a/b");
    }

    #[test]
    fn near_matches_join_into_one_window() {
        assert_eq!(spans(&[10, 14]), [(7, 17)]);
    }

    #[test]
    fn a_window_has_at_most_12_lines() {
        assert_eq!(spans(&[10, 16]), [(7, 18)]);
        assert_eq!(spans(&[10, 13, 16]), [(7, 18)]);
    }

    #[test]
    fn a_new_window_does_not_repeat_a_line() {
        assert_eq!(spans(&[1, 4, 7, 9, 11, 14]), [(1, 12), (13, 17)]);
    }

    #[test]
    fn far_matches_give_two_windows() {
        assert_eq!(spans(&[10, 30]), [(7, 13), (27, 33)]);
    }

    #[test]
    fn a_match_on_line_2_starts_at_line_1() {
        assert_eq!(spans(&[2]), [(1, 5)]);
    }

    #[test]
    fn the_matches_of_a_window_are_kept_in_line_order() {
        let windows = plan_windows(&[file("a", &[10, 13, 16, 40])], ContextLines::DEFAULT);
        assert_eq!(windows[0].matches, [10, 13, 16]);
        assert_eq!(windows[1].matches, [40]);
    }

    #[test]
    fn windows_follow_the_order_of_the_files() {
        let windows = plan_windows(&[file("a", &[1]), file("b", &[1])], ContextLines::DEFAULT);
        assert_eq!(windows[0].path, PathBuf::from("a"));
        assert_eq!(windows[1].path, PathBuf::from("b"));
    }

    #[test]
    fn match_texts_gives_the_matching_lines_of_each_window() {
        let mut first = file("a", &[10, 14, 30]);
        first.lines[0].text = b"ten".to_vec();
        first.lines[1].text = b"fourteen".to_vec();
        first.lines[2].text = b"thirty".to_vec();
        let mut second = file("b", &[2]);
        second.lines[0].text = b"two".to_vec();
        let files = [first, second];
        let windows = plan_windows(&files, ContextLines::DEFAULT);
        assert_eq!(windows.len(), 3);
        let texts = match_texts(&files, &windows);
        assert_eq!(
            texts,
            [
                vec![&b"ten"[..], &b"fourteen"[..]],
                vec![&b"thirty"[..]],
                vec![&b"two"[..]]
            ]
        );
    }

    #[test]
    #[should_panic(expected = "windows must come from plan_windows")]
    fn match_texts_refuses_windows_that_do_not_match_the_files() {
        let files = [file("a", &[10])];
        let windows = [Window {
            path: PathBuf::from("a"),
            start: 1,
            end: 5,
            matches: vec![3],
        }];
        let _ = match_texts(&files, &windows);
    }

    fn folder_with(name: &str, text: &[u8]) -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("temporary folder");
        let path = dir.path().join(name);
        fs::write(&path, text).expect("write file");
        (dir, path)
    }

    fn window(path: &Path, start: u64, end: u64, matches: &[u64]) -> Window {
        Window {
            path: path.to_path_buf(),
            start,
            end,
            matches: matches.to_vec(),
        }
    }

    fn numbers(snippet: &Snippet) -> Vec<u64> {
        snippet.lines.iter().map(|line| line.number).collect()
    }

    #[test]
    fn a_match_on_the_last_line_ends_at_the_last_line() {
        let (_dir, path) = folder_with("a.txt", b"1\n2\n3\n4\n5\n");
        let (snippets, warnings) = read_snippets(&[window(&path, 2, 8, &[5])]);
        assert!(warnings.is_empty());
        assert_eq!(numbers(&snippets[0]), [2, 3, 4, 5]);
        let last = snippets[0].lines.last().expect("a last line");
        assert!(last.is_match);
        assert_eq!(last.text, b"5");
    }

    #[test]
    fn a_file_with_no_final_newline_gives_the_same_lines() {
        let (_dir, path) = folder_with("a.txt", b"1\n2\n3\n4\n5");
        let (snippets, _) = read_snippets(&[window(&path, 2, 8, &[5])]);
        assert_eq!(numbers(&snippets[0]), [2, 3, 4, 5]);
    }

    #[test]
    fn a_match_past_the_last_line_is_dropped() {
        let (_dir, path) = folder_with("a.txt", b"1\n2\n3\n");
        let (snippets, _) =
            read_snippets(&[window(&path, 1, 6, &[2, 9]), window(&path, 20, 26, &[23])]);
        assert_eq!(snippets.len(), 1);
        let marks: Vec<bool> = snippets[0].lines.iter().map(|l| l.is_match).collect();
        assert_eq!(marks, [false, true, false]);
    }

    fn utf16(text: &str, little: bool) -> Vec<u8> {
        let mut bytes = if little {
            vec![0xFF, 0xFE]
        } else {
            vec![0xFE, 0xFF]
        };
        for unit in text.encode_utf16() {
            let pair = if little {
                unit.to_le_bytes()
            } else {
                unit.to_be_bytes()
            };
            bytes.extend(pair);
        }
        bytes
    }

    #[test]
    fn a_utf16_file_with_a_mark_gives_the_lines_as_text() {
        for little in [true, false] {
            let (_dir, path) = folder_with("a.txt", &utf16("one\ntwo é\nthree\n", little));
            let (snippets, warnings) = read_snippets(&[window(&path, 1, 3, &[2])]);
            assert!(warnings.is_empty());
            let texts: Vec<&[u8]> = snippets[0]
                .lines
                .iter()
                .map(|line| line.text.as_slice())
                .collect();
            assert_eq!(texts, [&b"one"[..], "two é".as_bytes(), &b"three"[..]]);
            assert!(snippets[0].lines[1].is_match);
        }
    }

    #[test]
    fn a_utf8_mark_is_not_part_of_the_first_line() {
        let (_dir, path) = folder_with("a.txt", b"\xEF\xBB\xBFone\ntwo\n");
        let (snippets, _) = read_snippets(&[window(&path, 1, 2, &[1])]);
        assert_eq!(snippets[0].lines[0].text, b"one");
    }

    #[test]
    fn a_carriage_return_stays_at_the_end_of_a_line() {
        let (_dir, path) = folder_with("a.txt", b"one\r\ntwo\r\n");
        let (snippets, _) = read_snippets(&[window(&path, 1, 2, &[1])]);
        assert_eq!(snippets[0].lines[0].text, b"one\r");
    }

    #[test]
    fn a_file_that_jag_cannot_read_gives_a_warning_and_no_result() {
        let dir = TempDir::new().expect("temporary folder");
        let missing = dir.path().join("missing.txt");
        let (snippets, warnings) = read_snippets(&[window(&missing, 1, 4, &[1])]);
        assert!(snippets.is_empty());
        assert_eq!(
            warnings,
            [WalkWarning {
                path: Some(missing),
                kind: WarningKind::Read(io::ErrorKind::NotFound),
            }]
        );
    }

    #[test]
    fn jag_reads_only_the_files_of_the_kept_windows() {
        let dir = TempDir::new().expect("temporary folder");
        let files: Vec<FileMatches> = (0..250)
            .map(|number| {
                let name = format!("f{number:03}.txt");
                if number < 200 {
                    fs::write(dir.path().join(&name), b"needle\n").expect("write file");
                }
                file(&dir.path().join(name).to_string_lossy(), &[1])
            })
            .collect();
        let windows = plan_windows(&files, ContextLines::DEFAULT);
        assert_eq!(windows.len(), 250);
        let (snippets, warnings) = read_snippets(&windows[..200]);
        assert_eq!(snippets.len(), 200);
        assert!(warnings.is_empty());
    }

    #[test]
    fn two_windows_of_one_file_read_the_file_one_time() {
        let text: String = (1..=40).map(|number| format!("line {number}\n")).collect();
        let (_dir, path) = folder_with("a.txt", text.as_bytes());
        let (snippets, _) =
            read_snippets(&[window(&path, 7, 13, &[10]), window(&path, 27, 33, &[30])]);
        assert_eq!(snippets.len(), 2);
        assert_eq!(numbers(&snippets[1]).first(), Some(&27));
    }

    #[test]
    fn char_count_counts_characters_and_a_bad_sequence_as_one() {
        assert_eq!(char_count(b""), 0);
        assert_eq!(char_count(b"abc"), 3);
        assert_eq!(char_count("é".as_bytes()), 1);
        assert_eq!(char_count(b"a\xffb"), 3);
        assert_eq!(char_count(b"\xe2\x82"), 1);
        assert_eq!(char_count(b"\xe2\x82\n"), 2);
    }

    #[test]
    fn the_count_of_joined_lines_is_the_sum_of_their_counts() {
        let lines: [&[u8]; 3] = [b"a\xe2\x82\n", b"\xffb\n", "é\n".as_bytes()];
        let joined: Vec<u8> = lines.concat();
        let sum: usize = lines.iter().map(|line| char_count(line)).sum();
        assert_eq!(char_count(&joined), sum);
        assert_eq!(sum, String::from_utf8_lossy(&joined).chars().count());
    }
}
