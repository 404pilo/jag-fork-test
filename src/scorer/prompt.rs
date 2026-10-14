//! The request of the prompt `jag-p1`, and the split of the results into calls.
//!
//! Shape B: all results of a call go in `state`, with one `noul` question for
//! each result. There is no `criteria`.

use std::ops::Range;

use serde::Serialize;
use serde_json::{Map, json};

use crate::search::snippet::{MAX_LINE_CHARS, Snippet};

/// The most results in one call (ADR-0002).
pub const MAX_CALL_RESULTS: usize = 32;
/// A call closes before the estimate is more than this (decision 22).
pub const MAX_CALL_ESTIMATE: usize = 24_000;

/// The text of question `n`. It names the snippet number, because the API
/// does not send the question ids to the model.
pub fn instruction(n: usize) -> String {
    format!("Does snippet {n} answer the question?")
}

#[derive(Serialize)]
struct SnippetJson<'a> {
    n: usize,
    path: std::borrow::Cow<'a, str>,
    lines: String,
}

fn snippet_json(n: usize, snippet: &Snippet) -> SnippetJson<'_> {
    let lines: Vec<String> = snippet
        .lines
        .iter()
        .map(|line| {
            String::from_utf8_lossy(&line.text)
                .chars()
                .take(MAX_LINE_CHARS)
                .collect()
        })
        .collect();
    SnippetJson {
        n,
        path: String::from_utf8_lossy(snippet.source.label()),
        lines: lines.join("\n"),
    }
}

/// The body of one call. The ids of the questions are `s1`, `s2`, ...
pub fn request_body(model: &str, question: &str, snippets: &[Snippet]) -> String {
    let list: Vec<SnippetJson> = snippets
        .iter()
        .enumerate()
        .map(|(index, snippet)| snippet_json(index + 1, snippet))
        .collect();
    let mut questions = Map::new();
    for n in 1..=snippets.len() {
        questions.insert(
            format!("s{n}"),
            json!({ "type": "noul", "instructions": instruction(n) }),
        );
    }
    json!({
        "model": model,
        "state": { "question": question, "snippets": list },
        "questions": questions,
    })
    .to_string()
}

/// One call: the results `range` of the input, and its estimate in tokens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub range: Range<usize>,
    pub estimate: usize,
}

/// The estimate: the UTF-8 bytes of the `state` JSON and of the longest
/// question, divided by 3. A CJK character is 3 bytes and about one token, so
/// the estimate is not too low (decision 22).
fn estimate(state_bytes: usize, longest_question_bytes: usize) -> usize {
    (state_bytes + longest_question_bytes) / 3
}

/// Split the results into calls of 32 results or less, each with an estimate
/// of 24,000 or less. One result always fits (decision 22).
pub fn split_batches(snippets: &[Snippet], question: &str) -> Vec<Batch> {
    // The state with no result. It holds the question one time.
    let base = json!({ "question": question, "snippets": [] })
        .to_string()
        .len();
    // The longest question is the one with the largest number.
    let longest = instruction(MAX_CALL_RESULTS).len();

    let mut batches = Vec::new();
    let mut start = 0;
    let mut bytes = base;
    for (index, snippet) in snippets.iter().enumerate() {
        // Two digits for `n` is the most. One more byte is the comma.
        let entry = serde_json::to_string(&snippet_json(MAX_CALL_RESULTS, snippet))
            .map_or(0, |text| text.len())
            + 1;
        let count = index - start;
        if count > 0
            && (count == MAX_CALL_RESULTS || estimate(bytes + entry, longest) > MAX_CALL_ESTIMATE)
        {
            batches.push(Batch {
                range: start..index,
                estimate: estimate(bytes, longest),
            });
            start = index;
            bytes = base;
        }
        bytes += entry;
    }
    if start < snippets.len() {
        batches.push(Batch {
            range: start..snippets.len(),
            estimate: estimate(bytes, longest),
        });
    }
    batches
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::search::snippet::{SnippetLine, Source};

    fn snippet(path: &str, lines: &[&str]) -> Snippet {
        Snippet {
            source: Source::File(PathBuf::from(path)),
            lines: lines
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    SnippetLine::new(index as u64 + 1, text.as_bytes(), index == 0)
                })
                .collect(),
        }
    }

    fn sizes(batches: &[Batch]) -> Vec<usize> {
        batches.iter().map(|batch| batch.range.len()).collect()
    }

    #[test]
    fn the_body_has_the_model_the_state_and_one_question_for_each_snippet() {
        let snippets = [
            snippet("a.txt", &["one", "two"]),
            snippet("b.txt", &["three"]),
        ];
        let body: serde_json::Value =
            serde_json::from_str(&request_body("jev-1.13.0", "which one?", &snippets))
                .expect("json");
        assert_eq!(body["model"], "jev-1.13.0");
        assert_eq!(body["state"]["question"], "which one?");
        assert_eq!(body["state"]["snippets"][0]["n"], 1);
        assert_eq!(body["state"]["snippets"][0]["path"], "a.txt");
        assert_eq!(body["state"]["snippets"][0]["lines"], "one\ntwo");
        assert_eq!(body["state"]["snippets"][1]["n"], 2);
        assert_eq!(body["questions"]["s1"]["type"], "noul");
        assert_eq!(
            body["questions"]["s1"]["instructions"],
            "Does snippet 1 answer the question?"
        );
        assert_eq!(
            body["questions"]["s2"]["instructions"],
            "Does snippet 2 answer the question?"
        );
        assert!(body["questions"]["s1"].get("criteria").is_none());
        assert_eq!(body["questions"].as_object().map(|map| map.len()), Some(2));
    }

    #[test]
    fn a_line_is_cut_at_300_characters_at_a_character_border() {
        let long = "é".repeat(1000);
        let snippets = [snippet("a.txt", &[long.as_str()])];
        let body: serde_json::Value =
            serde_json::from_str(&request_body("m", "q", &snippets)).expect("json");
        let lines = body["state"]["snippets"][0]["lines"]
            .as_str()
            .expect("text");
        assert_eq!(lines.chars().count(), 300);
    }

    #[test]
    fn the_request_holds_the_label_of_standard_input() {
        let mut one = snippet("a.txt", &["x"]);
        one.source = Source::StandardInput;
        let body: serde_json::Value =
            serde_json::from_str(&request_body("m", "q", &[one])).expect("json");
        assert_eq!(body["state"]["snippets"][0]["path"], "(standard input)");
    }

    #[test]
    fn the_request_text_is_the_first_300_characters_of_the_full_line() {
        let mut lines: Vec<Vec<u8>> = vec![
            vec![b'a'; 300],
            vec![b'a'; 301],
            vec![b'a'; 1000],
            "漢".repeat(400).into_bytes(),
            vec![0xFF; 400],
        ];
        let mut mixed = vec![b'a'; 299];
        mixed.extend([0xE2, 0x82]);
        mixed.extend([b'b'; 10]);
        lines.push(mixed);
        let mut returned = vec![b'a'; 300];
        returned.push(b'\r');
        lines.push(returned);
        for full in lines {
            let mut one = snippet("a.txt", &["x"]);
            one.lines[0] = SnippetLine::new(1, &full, true);
            let body: serde_json::Value =
                serde_json::from_str(&request_body("m", "q", &[one])).expect("json");
            let want: String = String::from_utf8_lossy(&full).chars().take(300).collect();
            assert_eq!(body["state"]["snippets"][0]["lines"], want.as_str());
        }
    }

    #[test]
    fn a_line_that_is_not_utf8_is_made_into_text() {
        let mut one = snippet("a.txt", &["x"]);
        one.lines[0].text = vec![b'a', 0xff, b'b'];
        let body: serde_json::Value =
            serde_json::from_str(&request_body("m", "q", &[one])).expect("json");
        assert_eq!(body["state"]["snippets"][0]["lines"], "a\u{fffd}b");
    }

    #[test]
    fn seventy_short_snippets_give_calls_of_32_32_and_6() {
        let snippets: Vec<Snippet> = (0..70).map(|_| snippet("a.txt", &["x"])).collect();
        assert_eq!(sizes(&split_batches(&snippets, "q")), [32, 32, 6]);
    }

    #[test]
    fn no_snippet_gives_no_call() {
        assert!(split_batches(&[], "q").is_empty());
    }

    #[test]
    fn the_batches_cover_each_snippet_one_time_in_order() {
        let snippets: Vec<Snippet> = (0..70).map(|_| snippet("a.txt", &["x"])).collect();
        let batches = split_batches(&snippets, "q");
        assert_eq!(batches[0].range, 0..32);
        assert_eq!(batches[1].range, 32..64);
        assert_eq!(batches[2].range, 64..70);
    }

    #[test]
    fn full_snippets_of_long_lines_give_calls_within_the_estimate() {
        let line = "x".repeat(300);
        let lines: Vec<&str> = (0..12).map(|_| line.as_str()).collect();
        let snippets: Vec<Snippet> = (0..200).map(|_| snippet("a.txt", &lines)).collect();
        let batches = split_batches(&snippets, "q");
        assert!(batches.len() > 7, "{} calls", batches.len());
        for batch in &batches {
            assert!(batch.estimate <= MAX_CALL_ESTIMATE);
            assert!(batch.range.len() <= MAX_CALL_RESULTS);
        }
    }

    #[test]
    fn cjk_snippets_give_two_calls_or_more_within_the_estimate() {
        let line = "漢".repeat(80);
        let lines: Vec<&str> = (0..12).map(|_| line.as_str()).collect();
        let snippets: Vec<Snippet> = (0..32).map(|_| snippet("a.txt", &lines)).collect();
        let batches = split_batches(&snippets, "q");
        assert!(batches.len() >= 2);
        for batch in &batches {
            assert!(batch.estimate <= MAX_CALL_ESTIMATE);
        }
    }

    #[test]
    fn one_snippet_of_the_largest_size_fits_with_the_longest_question() {
        // 12 lines of 300 control characters: JSON writes each as 6 bytes.
        let line = "\u{1}".repeat(300);
        let lines: Vec<&str> = (0..12).map(|_| line.as_str()).collect();
        let question = "q".repeat(12_000);
        let batches = split_batches(&[snippet("a.txt", &lines)], &question);
        assert_eq!(batches.len(), 1);
        assert!(batches[0].estimate <= MAX_CALL_ESTIMATE);
    }

    #[test]
    fn the_estimate_is_the_real_size_of_the_state_over_three_or_more() {
        let snippets: Vec<Snippet> = (0..5).map(|_| snippet("a.txt", &["hello"])).collect();
        let batches = split_batches(&snippets, "q");
        let body: serde_json::Value =
            serde_json::from_str(&request_body("m", "q", &snippets)).expect("json");
        let state_bytes = body["state"].to_string().len();
        assert!(batches[0].estimate >= state_bytes / 3);
    }
}
