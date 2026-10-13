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

/// One line of a result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnippetLine {
    /// The number of the line. The first line is 1.
    pub number: u64,
    /// The bytes of the line, with no `\n`. A `\r` at the end stays.
    pub text: Vec<u8>,
    pub is_match: bool,
}

/// One result: a match with the lines around it (IF-6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snippet {
    pub path: PathBuf,
    pub lines: Vec<SnippetLine>,
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

/// Join the matches of each file into windows of 12 lines or less.
///
/// The windows are in the order of `files`, then in line order.
pub fn plan_windows(files: &[FileMatches]) -> Vec<Window> {
    let mut windows = Vec::new();
    for file in files {
        let mut current: Option<Window> = None;
        for line in &file.lines {
            let m = line.line_number;
            current = Some(match current.take() {
                None => open_window(&file.path, m, 1),
                Some(mut window) => {
                    if m <= window.start + (MAX_LINES - 1)
                        && m.saturating_sub(CONTEXT_LINES) <= window.end + 1
                    {
                        window.matches.push(m);
                        window.end = window
                            .end
                            .max(m + CONTEXT_LINES)
                            .min(window.start + (MAX_LINES - 1));
                        window
                    } else {
                        let first_free = window.end + 1;
                        windows.push(window);
                        open_window(&file.path, m, first_free)
                    }
                }
            });
        }
        windows.extend(current);
    }
    windows
}

/// A new window for the match `m`. No line before `first_free` is in it.
fn open_window(path: &std::path::Path, m: u64, first_free: u64) -> Window {
    Window {
        path: path.to_path_buf(),
        start: m.saturating_sub(CONTEXT_LINES).max(first_free),
        end: m + CONTEXT_LINES,
        matches: vec![m],
    }
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
        .map(|number| SnippetLine {
            number,
            text: lines[(number - 1) as usize].to_vec(),
            is_match: matches.binary_search(&number).is_ok(),
        })
        .collect();
    Some(Snippet {
        path: window.path.clone(),
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

    /// The `(start, end)` of each window for one file.
    fn spans(numbers: &[u64]) -> Vec<(u64, u64)> {
        plan_windows(&[file("a", numbers)])
            .iter()
            .map(|window| (window.start, window.end))
            .collect()
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
        let windows = plan_windows(&[file("a", &[10, 13, 16, 40])]);
        assert_eq!(windows[0].matches, [10, 13, 16]);
        assert_eq!(windows[1].matches, [40]);
    }

    #[test]
    fn windows_follow_the_order_of_the_files() {
        let windows = plan_windows(&[file("a", &[1]), file("b", &[1])]);
        assert_eq!(windows[0].path, PathBuf::from("a"));
        assert_eq!(windows[1].path, PathBuf::from("b"));
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
        let windows = plan_windows(&files);
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
}
