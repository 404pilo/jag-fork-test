//! C-prerank: rank the results by word match with BM25, to keep the best
//! ones when a search has more results than the cost limit (STORY-18).
//!
//! The pre-rank reads only the text of the matching lines, so it reads no
//! file. It is own code, with no crate (decisions 8 to 14).

use std::collections::HashMap;

use crate::search::snippet::MAX_LINE_CHARS;

/// BM25 term saturation.
pub const K1: f64 = 1.2;
/// BM25 length normalisation.
pub const B: f64 = 0.75;

/// The words of a text: split on each character that is not alphanumeric,
/// then lower case each word. The empty parts go away.
pub fn words(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// The words of the question, each word one time, in the order of its first
/// use.
pub fn question_words(question: &str) -> Vec<String> {
    let mut seen = Vec::new();
    for word in words(question) {
        if !seen.contains(&word) {
            seen.push(word);
        }
    }
    seen
}

/// The BM25 score of each result. `texts[k]` holds the matching lines of
/// result k. The scores are in the order of `texts`.
pub fn scores(texts: &[Vec<&[u8]>], question: &str) -> Vec<f64> {
    let question = question_words(question);
    let index: HashMap<&str, usize> = question
        .iter()
        .enumerate()
        .map(|(position, word)| (word.as_str(), position))
        .collect();

    // Keep only the length and the count of each question word.
    let mut lengths = Vec::with_capacity(texts.len());
    let mut counts = Vec::with_capacity(texts.len());
    for lines in texts {
        let mut length = 0_usize;
        let mut count = vec![0_usize; question.len()];
        for line in lines {
            let text = String::from_utf8_lossy(line);
            let cut = match text.char_indices().nth(MAX_LINE_CHARS) {
                Some((end, _)) => &text[..end],
                None => &text,
            };
            for word in words(cut) {
                length += 1;
                if let Some(position) = index.get(word.as_str()) {
                    count[*position] += 1;
                }
            }
        }
        lengths.push(length);
        counts.push(count);
    }

    let total = texts.len() as f64;
    let average = lengths.iter().sum::<usize>() as f64 / total;
    if texts.is_empty() || average == 0.0 {
        return vec![0.0; texts.len()];
    }
    let idf: Vec<f64> = (0..question.len())
        .map(|position| {
            let holding = counts.iter().filter(|count| count[position] > 0).count() as f64;
            (1.0 + (total - holding + 0.5) / (holding + 0.5)).ln()
        })
        .collect();

    lengths
        .iter()
        .zip(&counts)
        .map(|(length, count)| {
            let normal = K1 * (1.0 - B + B * *length as f64 / average);
            let mut score = 0.0;
            for (position, found) in count.iter().enumerate() {
                if *found > 0 {
                    let found = *found as f64;
                    score += idf[position] * found * (K1 + 1.0) / (found + normal);
                }
            }
            score
        })
        .collect()
}

/// The indices of the `limit` best results, in increasing order. When the
/// results are not more than the limit, all of them, with no computation.
pub fn best(texts: &[Vec<&[u8]>], question: &str, limit: usize) -> Vec<usize> {
    if texts.len() <= limit {
        return (0..texts.len()).collect();
    }
    let scores = scores(texts, question);
    let mut order: Vec<usize> = (0..texts.len()).collect();
    // A stable sort keeps the scan order of equal scores: the order of the
    // paths on the command line, then the byte order of the path, then the
    // line number.
    order.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));
    order.truncate(limit);
    order.sort_unstable();
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARGIN: f64 = 1e-9;

    fn texts<'a>(lines: &[&'a str]) -> Vec<Vec<&'a [u8]>> {
        lines.iter().map(|line| vec![line.as_bytes()]).collect()
    }

    fn assert_close(got: &[f64], want: &[f64]) {
        assert_eq!(got.len(), want.len());
        for (got, want) in got.iter().zip(want) {
            assert!((got - want).abs() < MARGIN, "{got} is not {want}");
        }
    }

    #[test]
    fn words_split_lower_case_and_keep_unicode() {
        assert_eq!(
            words("Retry-After: 30s, RÉSUMÉ"),
            ["retry", "after", "30s", "résumé"]
        );
        assert_eq!(words("retry_wait"), ["retry", "wait"]);
        assert!(words("???").is_empty());
    }

    #[test]
    fn question_words_keep_the_first_use_of_each_word() {
        assert_eq!(
            question_words("Retry the RETRY, retry_wait?"),
            ["retry", "the", "wait"]
        );
    }

    #[test]
    fn a_word_in_one_of_two_results_scores_ln_2() {
        let scores = scores(&texts(&["retry now", "wait here"]), "retry");
        assert_close(&scores, &[2.0_f64.ln(), 0.0]);
    }

    #[test]
    fn a_shorter_result_with_the_same_word_ranks_higher() {
        let scores = scores(&texts(&["retry", "retry a b", "x y"]), "retry");
        assert_close(
            &scores,
            &[0.590_861_705_337_496_3, 0.390_191_692_204_007, 0.0],
        );
        assert!(scores[0] > scores[1]);
    }

    #[test]
    fn a_word_in_each_result_still_counts() {
        let scores = scores(&texts(&["retry", "retry"]), "retry");
        assert_close(&scores, &[1.2_f64.ln(), 1.2_f64.ln()]);
        assert!(scores[0] > 0.0);
    }

    #[test]
    fn a_repeated_question_word_counts_one_time() {
        let docs = texts(&["retry now", "wait here"]);
        assert_eq!(scores(&docs, "retry retry"), scores(&docs, "retry"));
    }

    #[test]
    fn texts_with_no_word_score_0_and_not_nan() {
        let scores = scores(&texts(&["???", "!!"]), "retry");
        assert_eq!(scores, [0.0, 0.0]);
    }

    #[test]
    fn a_word_after_the_cut_at_300_characters_does_not_count() {
        let line = format!("{}retry", "x ".repeat(150));
        let docs = vec![vec![line.as_bytes()]];
        assert_eq!(scores(&docs, "retry"), [0.0]);
    }

    #[test]
    fn the_matching_lines_of_a_result_make_one_document() {
        let docs = vec![vec![&b"retry"[..], &b"wait"[..]], vec![&b"other"[..]]];
        let both = scores(&docs, "retry wait");
        assert!(both[0] > 0.0);
        assert_eq!(both[1], 0.0);
    }

    #[test]
    fn best_keeps_the_best_then_the_first_of_equal_scores_in_order() {
        let docs = texts(&["needle", "needle", "needle retry"]);
        assert_eq!(best(&docs, "retry", 2), [0, 2]);
    }

    #[test]
    fn a_question_with_no_word_keeps_the_first_results() {
        let docs = texts(&["a", "b", "c"]);
        assert_eq!(best(&docs, "???", 2), [0, 1]);
    }

    #[test]
    fn results_that_are_not_more_than_the_limit_are_all_kept() {
        let docs = texts(&["a", "b", "c"]);
        assert_eq!(best(&docs, "retry", 3), [0, 1, 2]);
        assert_eq!(best(&docs, "retry", 10), [0, 1, 2]);
    }
}
