//! C-output: the text form of the results and the no-good-answer label
//! (STORY-15).

use std::borrow::Cow;
use std::io::{self, Write};

use serde::Serialize;

use crate::failure::Failure;
use crate::scorer::{PROMPT_VERSION, THRESHOLD};
use crate::search::snippet::{Snippet, Source};
use crate::search::walk::{SkipCounts, WalkWarning};

/// The first line on standard output when no result answers well.
pub const NO_GOOD_ANSWER: &str =
    "jag: no good answer. The results below do not answer the question well.";

/// The version of the form of the document (CONSTRAINT-05). Change it only
/// when a field is removed or renamed, in a new major version of jag.
pub const JSON_VERSION: u32 = 1;

/// Each part of the document other than the results (FEATURE-11).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub exit_code: u8,
    pub error: Option<Failure>,
    pub no_good_answer: bool,
    pub results_found: u64,
    pub results_scored: u64,
    pub results_skipped: u64,
    /// FEATURE-09 sets the 4 size-limit fields (decision 16).
    pub left_out: u64,
    pub top_result_cut: u64,
    pub size_limit_fix: String,
    pub files_skipped: SkipCounts,
    pub warnings: Vec<WalkWarning>,
    pub warnings_left_out: u64,
}

/// True when the top score is less than the threshold: the text then shows
/// the no-good-answer label. False when there is no result.
pub fn is_no_good_answer(ranked: &[(Snippet, f64)]) -> bool {
    // A top score of exactly the threshold is a good answer.
    ranked.first().is_some_and(|(_, score)| *score < THRESHOLD)
}

// The document. The fields have the names and the order of the spec. Add the
// name of each new field before `results`, and never remove or rename one in
// one major version of jag (CONSTRAINT-05).
#[derive(Serialize)]
struct Document<'a> {
    version: u32,
    prompt_version: &'static str,
    exit_code: u8,
    error: Option<ErrorField<'a>>,
    no_good_answer: bool,
    results_found: u64,
    results_scored: u64,
    results_skipped: u64,
    left_out: u64,
    top_result_cut: u64,
    size_limit_fix: &'a str,
    files_skipped: FilesSkipped,
    warnings: Vec<WarningField<'a>>,
    warnings_left_out: u64,
    results: Vec<ResultField<'a>>,
}

#[derive(Serialize)]
struct ErrorField<'a> {
    cause: &'a str,
    fix: &'a str,
}

#[derive(Serialize)]
struct FilesSkipped {
    total: u64,
    binary: u64,
    hidden: u64,
    ignored: u64,
}

#[derive(Serialize)]
struct WarningField<'a> {
    path: Option<Cow<'a, str>>,
    text: String,
}

#[derive(Serialize)]
struct ResultField<'a> {
    rank: usize,
    path: Option<Cow<'a, str>>,
    first_line: u64,
    last_line: u64,
    lines: Vec<LineField<'a>>,
}

#[derive(Serialize)]
struct LineField<'a> {
    number: u64,
    text: Cow<'a, str>,
    is_match: bool,
    cut: u64,
}

/// Write the document, with `ranked` as its results in that order, on one
/// line with `\n` at the end. The score is never written.
pub fn write_json(
    out: &mut dyn Write,
    summary: &Summary,
    ranked: &[(Snippet, f64)],
) -> io::Result<()> {
    let results = ranked
        .iter()
        .enumerate()
        .map(|(index, (snippet, _))| ResultField {
            rank: index + 1,
            path: match &snippet.source {
                Source::File(path) => Some(path.to_string_lossy()),
                Source::StandardInput => None,
            },
            first_line: snippet.first_line(),
            last_line: snippet.last_line(),
            lines: snippet
                .lines
                .iter()
                .map(|line| LineField {
                    number: line.number,
                    text: String::from_utf8_lossy(&line.text),
                    is_match: line.is_match,
                    cut: line.cut,
                })
                .collect(),
        })
        .collect();
    let document = Document {
        version: JSON_VERSION,
        prompt_version: PROMPT_VERSION,
        exit_code: summary.exit_code,
        error: summary.error.as_ref().map(|failure| ErrorField {
            cause: &failure.cause,
            fix: &failure.fix,
        }),
        no_good_answer: summary.no_good_answer,
        results_found: summary.results_found,
        results_scored: summary.results_scored,
        results_skipped: summary.results_skipped,
        left_out: summary.left_out,
        top_result_cut: summary.top_result_cut,
        size_limit_fix: &summary.size_limit_fix,
        files_skipped: FilesSkipped {
            total: summary.files_skipped.total(),
            binary: summary.files_skipped.binary,
            hidden: summary.files_skipped.hidden,
            ignored: summary.files_skipped.ignored,
        },
        warnings: summary
            .warnings
            .iter()
            .map(|warning| WarningField {
                path: warning.path.as_deref().map(|path| path.to_string_lossy()),
                text: warning.text(),
            })
            .collect(),
        warnings_left_out: summary.warnings_left_out,
        results,
    };
    serde_json::to_writer(&mut *out, &document)?;
    out.write_all(b"\n")
}

/// Write the results in rank order. Gives true when the label shows.
///
/// A result has the form of `grep -n` with lines around the match:
/// `path:N:text` for a matching line, `path-N-text` for another line, and
/// `--` between two results (decision 27). `path` is the path as typed, or
/// `(standard input)`. `text` is the first bytes of the line as the file holds
/// them. A line that jag cut ends with ` [+N chars]`, with N the count of cut
/// characters.
pub fn write_text(out: &mut dyn Write, ranked: &[(Snippet, f64)]) -> io::Result<bool> {
    let no_good_answer = is_no_good_answer(ranked);
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
    use crate::search::walk::WarningKind;

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

    fn json(summary: &Summary, ranked: &[(Snippet, f64)]) -> String {
        let mut out = Vec::new();
        write_json(&mut out, summary, ranked).expect("write");
        String::from_utf8(out).expect("text")
    }

    fn value(summary: &Summary, ranked: &[(Snippet, f64)]) -> serde_json::Value {
        let text = json(summary, ranked);
        assert!(text.ends_with('\n'));
        assert_eq!(text.matches('\n').count(), 1, "one line");
        serde_json::from_str(&text).expect("json")
    }

    #[test]
    fn a_document_has_one_compact_line() {
        let text = json(&Summary::default(), &[]);
        assert_eq!(
            text,
            "{\"version\":1,\"prompt_version\":\"jag-p1\",\"exit_code\":0,\"error\":null,\
             \"no_good_answer\":false,\"results_found\":0,\"results_scored\":0,\
             \"results_skipped\":0,\"left_out\":0,\"top_result_cut\":0,\"size_limit_fix\":\"\",\
             \"files_skipped\":{\"total\":0,\"binary\":0,\"hidden\":0,\"ignored\":0},\
             \"warnings\":[],\"warnings_left_out\":0,\"results\":[]}\n"
        );
    }

    #[test]
    fn a_result_from_standard_input_has_a_null_path() {
        let mut result = snippet("a", &[(6, "before", false), (7, "found", true)]);
        result.source = Source::StandardInput;
        let document = value(&Summary::default(), &[(result, 0.9)]);
        assert!(document["results"][0]["path"].is_null());
        assert_eq!(document["results"][0]["first_line"], 6);
        assert_eq!(document["results"][0]["last_line"], 7);
    }

    #[test]
    fn the_bytes_of_a_line_are_made_into_text_and_a_return_stays() {
        let mut result = snippet("a", &[(1, "x", true), (2, "y", false)]);
        result.lines[0].text = vec![b'a', 0xff, b'b'];
        result.lines[1].text = b"tail\r".to_vec();
        let text = json(&Summary::default(), &[(result, 0.9)]);
        assert!(text.contains("\"text\":\"a\u{fffd}b\""));
        assert!(text.contains("\"text\":\"tail\\r\""));
    }

    #[test]
    fn a_result_has_a_rank_and_the_numbers_of_its_snippet_and_no_score() {
        let lines: Vec<(u64, &str, bool)> = (7..=17).map(|n| (n, "t", n == 10)).collect();
        let ranked = [
            (snippet("a", &lines), 0.9),
            (snippet("b", &[(1, "x", true)]), 0.5),
        ];
        let document = value(&Summary::default(), &ranked);
        let results = document["results"].as_array().expect("list");
        assert_eq!(results[0]["rank"], 1);
        assert_eq!(results[1]["rank"], 2);
        assert_eq!(results[0]["path"], "a");
        assert_eq!(results[0]["first_line"], ranked[0].0.first_line());
        assert_eq!(results[0]["last_line"], ranked[0].0.last_line());
        assert_eq!(results[0]["lines"].as_array().map(Vec::len), Some(11));
        assert_eq!(results[0]["lines"][3]["is_match"], true);
        assert_eq!(results[0]["lines"][0]["is_match"], false);
        let text = json(&Summary::default(), &ranked);
        assert!(!text.contains("\"score\""));
        assert!(!text.contains("0.9"));
    }

    #[test]
    fn a_cut_is_a_number_and_the_text_has_no_mark() {
        let mut result = snippet("a", &[(1, "kept", true)]);
        result.lines[0].cut = 700;
        let document = value(&Summary::default(), &[(result, 0.9)]);
        assert_eq!(document["results"][0]["lines"][0]["cut"], 700);
        assert_eq!(document["results"][0]["lines"][0]["text"], "kept");
    }

    fn full_summary() -> Summary {
        Summary {
            exit_code: 5,
            error: Some(Failure {
                cause: "the cause.".to_owned(),
                fix: "The fix.".to_owned(),
                exit_code: 5,
            }),
            no_good_answer: true,
            results_found: 3,
            results_scored: 2,
            results_skipped: 1,
            files_skipped: SkipCounts {
                binary: 1,
                hidden: 2,
                ignored: 3,
            },
            warnings: vec![WalkWarning {
                path: Some(PathBuf::from("dir/a.txt")),
                kind: WarningKind::Read(std::io::ErrorKind::PermissionDenied),
            }],
            ..Summary::default()
        }
    }

    /// The names of the keys of an object, sorted. The document has a fixed
    /// order, but an agent must not depend on it.
    fn keys(value: &serde_json::Value) -> Vec<String> {
        let mut names: Vec<String> = value.as_object().expect("object").keys().cloned().collect();
        names.sort();
        names
    }

    fn sorted(names: &[&str]) -> Vec<String> {
        let mut names: Vec<String> = names.iter().map(|name| (*name).to_owned()).collect();
        names.sort();
        names
    }

    // CRITERION-52. Add the name of each new field to these lists. Never
    // remove or rename a name in one major version of jag (CONSTRAINT-05).
    const TOP_LEVEL: &[&str] = &[
        "version",
        "prompt_version",
        "exit_code",
        "error",
        "no_good_answer",
        "results_found",
        "results_scored",
        "results_skipped",
        "left_out",
        "top_result_cut",
        "size_limit_fix",
        "files_skipped",
        "warnings",
        "warnings_left_out",
        "results",
    ];
    const ERROR: &[&str] = &["cause", "fix"];
    const FILES_SKIPPED: &[&str] = &["total", "binary", "hidden", "ignored"];
    const WARNING: &[&str] = &["path", "text"];
    const RESULT: &[&str] = &["rank", "path", "first_line", "last_line", "lines"];
    const LINE: &[&str] = &["number", "text", "is_match", "cut"];

    #[test]
    fn no_field_of_the_form_is_removed_or_renamed() {
        let with_result = value(&full_summary(), &[(snippet("a", &[(1, "x", true)]), 0.9)]);
        assert_eq!(keys(&with_result), sorted(TOP_LEVEL));
        assert_eq!(keys(&with_result["error"]), sorted(ERROR));
        assert_eq!(keys(&with_result["files_skipped"]), sorted(FILES_SKIPPED));
        assert_eq!(keys(&with_result["warnings"][0]), sorted(WARNING));
        assert_eq!(keys(&with_result["results"][0]), sorted(RESULT));
        assert_eq!(keys(&with_result["results"][0]["lines"][0]), sorted(LINE));

        let with_error = value(&full_summary(), &[]);
        assert_eq!(keys(&with_error), sorted(TOP_LEVEL));
        assert_eq!(keys(&with_error["error"]), sorted(ERROR));
        assert_eq!(with_error["results"], serde_json::json!([]));
    }

    #[test]
    fn the_form_version_is_the_whole_number_1() {
        assert_eq!(JSON_VERSION, 1);
        let document = value(&Summary::default(), &[]);
        assert_eq!(document["version"], 1);
        assert!(document["version"].is_u64());
        assert_eq!(document["prompt_version"], PROMPT_VERSION);
    }

    #[test]
    fn a_warning_has_its_path_and_its_text_with_no_jag_prefix() {
        let document = value(&full_summary(), &[]);
        assert_eq!(document["warnings"][0]["path"], "dir/a.txt");
        let text = document["warnings"][0]["text"].as_str().expect("text");
        assert!(text.starts_with("cannot read dir/a.txt"));
        let none = WalkWarning {
            path: None,
            kind: WarningKind::Other,
        };
        let summary = Summary {
            warnings: vec![none],
            ..Summary::default()
        };
        let document = value(&summary, &[]);
        assert!(document["warnings"][0]["path"].is_null());
    }

    #[test]
    fn no_text_of_the_document_starts_with_jag() {
        let document = value(&full_summary(), &[]);
        let mut texts = vec![
            document["error"]["cause"].clone(),
            document["error"]["fix"].clone(),
            document["warnings"][0]["text"].clone(),
        ];
        texts.push(document["size_limit_fix"].clone());
        for text in texts {
            assert!(!text.as_str().expect("text").starts_with("jag: "));
        }
    }

    #[test]
    fn each_number_of_the_document_is_a_whole_number() {
        fn walk(value: &serde_json::Value) {
            match value {
                serde_json::Value::Number(number) => assert!(number.is_u64(), "{number}"),
                serde_json::Value::Array(list) => list.iter().for_each(walk),
                serde_json::Value::Object(map) => map.values().for_each(walk),
                _ => {}
            }
        }
        let ranked = [(snippet("a", &[(1, "x", true)]), 0.9)];
        walk(&value(&full_summary(), &ranked));
    }

    #[test]
    fn the_largest_document_with_no_result_is_less_than_1000_characters() {
        let summary = Summary {
            exit_code: 255,
            error: None,
            no_good_answer: false,
            results_found: u64::MAX,
            results_scored: u64::MAX,
            results_skipped: u64::MAX,
            left_out: u64::MAX,
            top_result_cut: u64::MAX,
            size_limit_fix: "x".repeat(120),
            files_skipped: SkipCounts {
                binary: u64::MAX / 3,
                hidden: u64::MAX / 3,
                ignored: u64::MAX / 3,
            },
            warnings: Vec::new(),
            warnings_left_out: u64::MAX,
        };
        let length = json(&summary, &[]).chars().count();
        assert!(length < 1000, "{length} characters");
    }

    #[test]
    fn no_result_writes_nothing() {
        let (text, label) = write(&[]);
        assert!(text.is_empty());
        assert!(!label);
    }
}
