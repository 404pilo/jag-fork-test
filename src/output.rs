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
/// `--` between two results (decision 27). `text` is the bytes of the line
/// as the file holds them.
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
        let path = snippet.path.as_os_str().as_encoded_bytes();
        for line in &snippet.lines {
            out.write_all(path)?;
            out.write_all(if line.is_match { b":" } else { b"-" })?;
            write!(out, "{}", line.number)?;
            out.write_all(if line.is_match { b":" } else { b"-" })?;
            out.write_all(&line.text)?;
            out.write_all(b"\n")?;
        }
    }
    Ok(no_good_answer)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::search::snippet::SnippetLine;

    fn snippet(path: &str, lines: &[(u64, &str, bool)]) -> Snippet {
        Snippet {
            path: PathBuf::from(path),
            lines: lines
                .iter()
                .map(|(number, text, is_match)| SnippetLine {
                    number: *number,
                    text: text.as_bytes().to_vec(),
                    is_match: *is_match,
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
    fn no_result_writes_nothing() {
        let (text, label) = write(&[]);
        assert!(text.is_empty());
        assert!(!label);
    }
}
