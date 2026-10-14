//! C-output: the text form of the results and the no-good-answer label
//! (STORY-15).

use std::io::{self, Write};

use crate::scorer::THRESHOLD;
use crate::search::snippet::Snippet;

/// The first line on standard output when no result answers well.
pub const NO_GOOD_ANSWER: &str =
    "jag: no good answer. The results below do not answer the question well.";

/// Write the results in rank order. Gives true when the label shows.
///
/// A result has the form of `grep -n` with lines around the match:
/// `path:N:text` for a matching line, `path-N-text` for another line, and
/// `--` between two results (decision 27). `path` is the path as typed, or
/// `(standard input)`. `text` is the first bytes of the line as the file holds
/// them. A line that jag cut ends with ` [+N chars]`, with N the count of cut
/// characters.
pub fn write_text(out: &mut dyn Write, ranked: &[(Snippet, f64)]) -> io::Result<bool> {
    // A top score of exactly the threshold is a good answer.
    let no_good_answer = ranked.first().is_some_and(|(_, score)| *score < THRESHOLD);
    if no_good_answer {
        writeln!(out, "{NO_GOOD_ANSWER}")?;
    }
    for (index, (snippet, _)) in ranked.iter().enumerate() {
        if index > 0 {
            out.write_all(b"--\n")?;
        }
        let label = snippet.source.label();
        for line in &snippet.lines {
            let separator: &[u8] = if line.is_match { b":" } else { b"-" };
            out.write_all(label)?;
            out.write_all(separator)?;
            write!(out, "{}", line.number)?;
            out.write_all(separator)?;
            out.write_all(&line.text)?;
            if line.cut > 0 {
                write!(out, " [+{} chars]", line.cut)?;
            }
            out.write_all(b"\n")?;
        }
    }
    Ok(no_good_answer)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::search::snippet::{SnippetLine, Source};

    fn snippet(path: &str, lines: &[(u64, &str, bool)]) -> Snippet {
        Snippet {
            source: Source::File(PathBuf::from(path)),
            lines: lines
                .iter()
                .map(|(number, text, is_match)| SnippetLine {
                    number: *number,
                    text: text.as_bytes().to_vec(),
                    is_match: *is_match,
                    cut: 0,
                })
                .collect(),
        }
    }

    fn write(ranked: &[(Snippet, f64)]) -> (String, bool) {
        let mut out = Vec::new();
        let label = write_text(&mut out, ranked).expect("write");
        (String::from_utf8(out).expect("text"), label)
    }

    #[test]
    fn a_result_has_the_form_of_grep_n_with_lines_around() {
        let ranked = [
            (
                snippet(
                    "src/net.rs",
                    &[
                        (38, "    let wait = Duration::from_secs(1);", false),
                        (41, "    // retry one time after a timeout", true),
                        (42, "    retry(call, wait)", false),
                    ],
                ),
                0.9,
            ),
            (
                snippet(
                    "docs/notes.md",
                    &[(7, "The client waits one second, then tries again.", true)],
                ),
                0.8,
            ),
        ];
        let (text, label) = write(&ranked);
        assert!(!label);
        assert_eq!(
            text,
            "src/net.rs-38-    let wait = Duration::from_secs(1);\n\
             src/net.rs:41:    // retry one time after a timeout\n\
             src/net.rs-42-    retry(call, wait)\n\
             --\n\
             docs/notes.md:7:The client waits one second, then tries again.\n"
        );
    }

    #[test]
    fn a_low_top_score_puts_the_label_on_the_first_line() {
        let ranked = [(snippet("a", &[(1, "x", true)]), 0.2)];
        let (text, label) = write(&ranked);
        assert!(label);
        assert_eq!(text, format!("{NO_GOOD_ANSWER}\na:1:x\n"));
    }

    #[test]
    fn a_top_score_of_the_threshold_is_a_good_answer() {
        let (_, label) = write(&[(snippet("a", &[(1, "x", true)]), 0.4)]);
        assert!(!label);
        let (_, label) = write(&[(snippet("a", &[(1, "x", true)]), 0.39)]);
        assert!(label);
    }

    #[test]
    fn the_bytes_of_a_line_are_written_as_the_file_holds_them() {
        let mut result = snippet("a", &[(1, "x", true)]);
        result.lines[0].text = vec![b'a', 0xff, b'\r'];
        let mut out = Vec::new();
        write_text(&mut out, &[(result, 0.9)]).expect("write");
        assert_eq!(out, b"a:1:a\xff\r\n");
    }

    #[test]
    fn a_result_of_lines_7_to_17_starts_and_ends_at_those_lines() {
        let lines: Vec<(u64, &str, bool)> = (7..=17)
            .map(|number| (number, "text", number == 10 || number == 14))
            .collect();
        let (text, _) = write(&[(snippet("a.txt", &lines), 0.9)]);
        let rows: Vec<&str> = text.lines().collect();
        assert_eq!(rows.len(), 11);
        assert!(rows[0].starts_with("a.txt-7-"));
        assert!(rows[10].starts_with("a.txt-17-"));
        assert!(rows[3].starts_with("a.txt:10:"));
        assert!(rows[7].starts_with("a.txt:14:"));
    }

    #[test]
    fn standard_input_shows_its_label_in_the_place_of_a_path() {
        let mut result = snippet("a", &[(6, "before", false), (7, "found", true)]);
        result.source = Source::StandardInput;
        let (text, _) = write(&[(result, 0.9)]);
        assert_eq!(
            text,
            "(standard input)-6-before\n(standard input):7:found\n"
        );
    }

    #[test]
    fn a_cut_line_ends_with_the_count_of_cut_characters() {
        let long = "x".repeat(300);
        let mut result = snippet("a", &[(1, long.as_str(), true), (2, "ctx", false)]);
        result.lines[0].cut = 700;
        result.lines[1].cut = 1;
        let (text, _) = write(&[(result, 0.9)]);
        assert_eq!(
            text,
            format!("a:1:{long} [+700 chars]\na-2-ctx [+1 chars]\n")
        );
    }

    #[test]
    fn a_line_with_no_cut_has_no_mark() {
        let (text, _) = write(&[(snippet("a", &[(1, "x", true)]), 0.9)]);
        assert!(!text.contains("chars"));
    }

    #[test]
    fn no_result_writes_nothing() {
        let (text, label) = write(&[]);
        assert!(text.is_empty());
        assert!(!label);
    }
}
