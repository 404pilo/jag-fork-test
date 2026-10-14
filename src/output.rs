//! C-output: the text form of the results and the no-good-answer label
//! (STORY-15).

pub mod progress;

use std::borrow::Cow;
use std::io::{self, Write};
use std::ops::Range;

use serde::Serialize;

#[cfg(test)]
use crate::config::MIN_SIZE_LIMIT;
use crate::config::{SIZE_LIMIT_OFF_OPTION, SIZE_LIMIT_OPTION};
use crate::failure::Failure;
use crate::scorer::{PROMPT_VERSION, THRESHOLD};
use crate::search::snippet::{Snippet, SnippetLine, Source, char_count, char_prefix};
use crate::search::walk::{FileMatches, SkipCounts, WalkWarning};

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

/// Write the document, with `results` in that order, on one line with `\n` at
/// the end. The document has no score.
pub fn write_json(out: &mut dyn Write, summary: &Summary, results: &[&Snippet]) -> io::Result<()> {
    let results = results
        .iter()
        .enumerate()
        .map(|(index, snippet)| ResultField {
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

/// The byte ranges of the matches in one line. C-cli gives it, so that
/// C-output does not use the matcher (FEATURE-15, decision 10).
pub type Paint<'a> = &'a dyn Fn(&[u8]) -> Vec<Range<usize>>;

/// The colors of GNU grep with its default `GREP_COLORS` (decision 7).
const MATCH: &str = "01;31";
const PATH: &str = "35";
const NUMBER: &str = "32";
const SEPARATOR: &str = "36";

/// Write `item`, in `code` when `colored`. A colored item is
/// `ESC[<code>m ESC[K <item> ESC[m ESC[K`, byte for byte as grep.
fn item(out: &mut dyn Write, colored: bool, code: &str, bytes: &[u8]) -> io::Result<()> {
    if !colored {
        return out.write_all(bytes);
    }
    write!(out, "\x1b[{code}m\x1b[K")?;
    out.write_all(bytes)?;
    out.write_all(b"\x1b[m\x1b[K")
}

/// Write the text of a matching line, with each match in `MATCH`. An empty
/// range gets no color. On a cut line, a range that ends at the last kept
/// byte gets no color, because it may go on in the cut part (decision 8).
fn write_painted(out: &mut dyn Write, text: &[u8], cut: bool, paint: Paint) -> io::Result<()> {
    let mut at = 0;
    for range in paint(text) {
        let ends_at_the_cut = cut && range.end == text.len();
        if range.is_empty() || range.start < at || range.end > text.len() || ends_at_the_cut {
            continue;
        }
        out.write_all(&text[at..range.start])?;
        item(out, true, MATCH, &text[range.clone()])?;
        at = range.end;
    }
    out.write_all(&text[at..])
}

/// What the size limit did to the output of one search.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SizeReport {
    /// The results that jag wrote, with a cut top result.
    pub shown: usize,
    /// The results that the size limit left out.
    pub left_out: usize,
    /// The N of the mark of a cut top result. 0 for no cut.
    pub top_result_cut: u64,
}

impl SizeReport {
    /// True when the size limit left out a result or cut the top result. This
    /// gives exit 8 (decision 17 of FEATURE-09).
    pub fn limited(&self) -> bool {
        self.left_out > 0 || self.top_result_cut > 0
    }
}

/// What `write_text` wrote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextReport {
    pub no_good_answer: bool,
    pub size: SizeReport,
}

/// The largest `k` from 0 to `count` with `size(k) <= limit`. `size(k)` must
/// grow with `k`.
pub fn fitting_count(count: usize, limit: usize, size: impl Fn(usize) -> usize) -> usize {
    let (mut low, mut high) = (0, count);
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if size(middle) <= limit {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low
}

/// The count of characters of a result that a line holds, in the file.
fn full_chars(line: &SnippetLine) -> u64 {
    char_count(&line.text) as u64 + line.cut
}

/// The cut of the top result (STORY-27): the lines that jag keeps, with their
/// true cuts, and N, the count of the characters of the result that the output
/// does not show. `fits` says whether a candidate and its N fit. None when not
/// even the path, the line number, and the mark of the first matching line fit
/// (decision 14).
fn cut_top(top: &Snippet, fits: &dyn Fn(&Snippet, u64) -> bool) -> Option<(Snippet, u64)> {
    let lines = &top.lines;
    let full: Vec<u64> = lines.iter().map(full_chars).collect();
    let all: u64 = full.iter().map(|count| count + 1).sum();
    let matching = lines.iter().position(|line| line.is_match).unwrap_or(0);
    if lines.is_empty() {
        return None;
    }
    // The lines `start..end`, whole, and the count that the output leaves out.
    let whole = |start: usize, end: usize| -> (Snippet, u64) {
        let shown: u64 = full[start..end].iter().map(|count| count + 1).sum();
        let snippet = Snippet {
            source: top.source.clone(),
            lines: lines[start..end].to_vec(),
        };
        (snippet, all - shown + lines[end - 1].cut)
    };
    // Step 1: leave out lines from the top, but never the first matching line.
    let start = (0..=matching)
        .find(|start| {
            let (candidate, left) = whole(*start, matching + 1);
            fits(&candidate, left)
        })
        .unwrap_or(matching);
    let (candidate, left) = whole(start, matching + 1);
    if fits(&candidate, left) {
        // Step 2: whole lines while they fit.
        let mut end = matching + 1;
        while end < lines.len() {
            let (candidate, left) = whole(start, end + 1);
            if !fits(&candidate, left) {
                break;
            }
            end += 1;
        }
        return Some(whole(start, end));
    }
    // Step 3: the first characters of the first matching line.
    let line = &lines[matching];
    let others = all - (full[matching] + 1);
    (0..=char_count(&line.text)).rev().find_map(|chars| {
        let mut kept = line.clone();
        kept.text = char_prefix(&line.text, chars).to_vec();
        kept.cut = full[matching] - chars as u64;
        let left = others + kept.cut;
        let candidate = Snippet {
            source: top.source.clone(),
            lines: vec![kept],
        };
        fits(&candidate, left).then_some((candidate, left))
    })
}

/// The bytes of one result: its lines, with `--` before them when `separator`.
fn block(snippet: &Snippet, separator: bool, paint: Option<Paint>) -> Vec<u8> {
    let mut bytes = Vec::new();
    // A write to a `Vec` does not fail.
    if separator {
        let _ = item(&mut bytes, paint.is_some(), SEPARATOR, b"--");
        bytes.push(b'\n');
    }
    let label = snippet.source.label();
    for line in &snippet.lines {
        let _ = write_line(&mut bytes, label, line, paint);
    }
    bytes
}

/// Write `label`, then the results, inside `limit`. `separator` puts `--`
/// between two results. The fit, the cut, and the report are the same for
/// `write_text` and `write_lines`.
fn write_fitted(
    out: &mut dyn Write,
    label: &[u8],
    results: &[&Snippet],
    separator: bool,
    paint: Option<Paint>,
    limit: Option<usize>,
) -> io::Result<SizeReport> {
    let count = results.len();
    let Some(limit) = limit else {
        out.write_all(label)?;
        for (index, snippet) in results.iter().enumerate() {
            out.write_all(&block(snippet, separator && index > 0, paint))?;
        }
        return Ok(SizeReport {
            shown: count,
            ..SizeReport::default()
        });
    };
    let Some(room) = limit.checked_sub(char_count(label)) else {
        return Ok(SizeReport {
            left_out: count,
            ..SizeReport::default()
        });
    };
    let blocks: Vec<Vec<u8>> = results
        .iter()
        .enumerate()
        .map(|(index, snippet)| block(snippet, separator && index > 0, paint))
        .collect();
    let mut sums = vec![0usize];
    for bytes in &blocks {
        sums.push(sums[sums.len() - 1] + char_count(bytes));
    }
    let kept = fitting_count(count, room, |k| sums[k]);
    if kept > 0 {
        out.write_all(label)?;
        for bytes in &blocks[..kept] {
            out.write_all(bytes)?;
        }
        return Ok(SizeReport {
            shown: kept,
            left_out: count - kept,
            top_result_cut: 0,
        });
    }
    let Some(top) = results.first() else {
        return Ok(SizeReport::default());
    };
    // The room for the mark uses the digits of the full count (decision 13).
    let all: u64 = top.lines.iter().map(|line| full_chars(line) + 1).sum();
    let fits = |candidate: &Snippet, _left: u64| {
        let mut estimate = candidate.clone();
        if let Some(last) = estimate.lines.last_mut() {
            last.cut = all;
        }
        char_count(&block(&estimate, false, paint)) <= room
    };
    out.write_all(label)?;
    match cut_top(top, &fits) {
        Some((mut cut, left)) => {
            if let Some(last) = cut.lines.last_mut() {
                last.cut = left;
            }
            out.write_all(&block(&cut, false, paint))?;
            Ok(SizeReport {
                shown: 1,
                left_out: count - 1,
                top_result_cut: left,
            })
        }
        None => Ok(SizeReport {
            left_out: count,
            ..SizeReport::default()
        }),
    }
}

/// Write the results in rank order, inside the size limit (`None` for no
/// limit). The report says if the label shows and what the limit did.
///
/// A result has the form of `grep -n` with lines around the match:
/// `path:N:text` for a matching line, `path-N-text` for another line, and
/// `--` between two results (decision 27). `path` is the path as typed, or
/// `(standard input)`. `text` is the first bytes of the line as the file holds
/// them. A line that jag cut ends with ` [+N chars]`, with N the count of cut
/// characters.
///
/// With `paint`, the text has the color of `grep --color=always` (decision 7).
/// With `None`, it has no escape code. The codes count toward the limit.
pub fn write_text(
    out: &mut dyn Write,
    ranked: &[(Snippet, f64)],
    paint: Option<Paint>,
    limit: Option<usize>,
) -> io::Result<TextReport> {
    let no_good_answer = is_no_good_answer(ranked);
    let label = if no_good_answer {
        format!("{NO_GOOD_ANSWER}\n")
    } else {
        String::new()
    };
    let results: Vec<&Snippet> = ranked.iter().map(|(snippet, _)| snippet).collect();
    let size = write_fitted(out, label.as_bytes(), &results, true, paint, limit)?;
    Ok(TextReport {
        no_good_answer,
        size,
    })
}

/// Write the lines of a search with no question (FEATURE-12): one
/// `path:N:text` line for each line of `results`, in that order, and no label
/// line. With `groups`, `--` stands between two results, as grep with a
/// context option (FEATURE-13, decision 15). With no `groups`, no `--`, as
/// grep with no context option. The size limit applies as for `write_text`
/// (FEATURE-09).
pub fn write_lines(
    out: &mut dyn Write,
    results: &[Snippet],
    paint: Option<Paint>,
    groups: bool,
    limit: Option<usize>,
) -> io::Result<SizeReport> {
    let results: Vec<&Snippet> = results.iter().collect();
    write_fitted(out, b"", &results, groups, paint, limit)
}
/// Write each of `items` as one whole line, inside `limit` (`None` for no
/// limit). The lines that fit come from the top. The report says how many
/// lines jag left out (FEATURE-13, decision 19 and 20; FEATURE-09).
fn write_whole_lines(
    out: &mut dyn Write,
    items: &[Vec<u8>],
    limit: Option<usize>,
) -> io::Result<SizeReport> {
    let kept = match limit {
        None => items.len(),
        Some(limit) => {
            let mut sums = vec![0usize];
            for bytes in items {
                sums.push(sums[sums.len() - 1] + char_count(bytes));
            }
            fitting_count(items.len(), limit, |k| sums[k])
        }
    };
    for bytes in &items[..kept] {
        out.write_all(bytes)?;
    }
    Ok(SizeReport {
        shown: kept,
        left_out: items.len() - kept,
        top_result_cut: 0,
    })
}

/// The bytes of one line: the parts of `parts` with their color codes when
/// `colored`, and `\n` at the end.
fn colored_line(colored: bool, parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    // A write to a `Vec` does not fail.
    for (code, text) in parts {
        let _ = item(&mut bytes, colored, code, text);
    }
    bytes.push(b'\n');
    bytes
}

/// Write the path of each file with a match, one path on each line, in the
/// order of `files` (FEATURE-13, decision 19). The path is in `PATH` when
/// `colored`. The size limit applies, with each line as one whole item.
pub fn write_files(
    out: &mut dyn Write,
    files: &[FileMatches],
    colored: bool,
    limit: Option<usize>,
) -> io::Result<SizeReport> {
    let items: Vec<Vec<u8>> = files
        .iter()
        .filter(|file| !file.lines.is_empty())
        .map(|file| colored_line(colored, &[(PATH, file.path.as_os_str().as_encoded_bytes())]))
        .collect();
    write_whole_lines(out, &items, limit)
}

/// Write `path:N` for each file of `files`, in that order: N is the count of
/// matching lines (FEATURE-13, decision 20). The path is in `PATH` and the `:`
/// in `SEPARATOR` when `colored`. The size limit applies, with each line as
/// one whole item.
pub fn write_counts(
    out: &mut dyn Write,
    files: &[FileMatches],
    colored: bool,
    limit: Option<usize>,
) -> io::Result<SizeReport> {
    let items: Vec<Vec<u8>> = files
        .iter()
        .map(|file| {
            let count = file.lines.len().to_string();
            let mut line = colored_line(
                colored,
                &[
                    (PATH, file.path.as_os_str().as_encoded_bytes()),
                    (SEPARATOR, b":"),
                ],
            );
            // The line ends with `\n`: put the count before it.
            line.pop();
            line.extend_from_slice(count.as_bytes());
            line.push(b'\n');
            line
        })
        .collect();
    write_whole_lines(out, &items, limit)
}

/// The second sentence of the size notice, and the field `size_limit_fix`.
fn size_fix(limit: usize) -> String {
    format!(
        "To see more, use {SIZE_LIMIT_OPTION} with a number larger than {limit}, or {SIZE_LIMIT_OFF_OPTION}."
    )
}

/// The notice of the size limit, for the error stream. `None` when there is no
/// limit, or when jag left out no result and cut nothing.
pub fn size_notice(size: &SizeReport, limit: Option<usize>) -> Option<String> {
    let limit = limit?;
    if !size.limited() {
        return None;
    }
    let cut = if size.top_result_cut > 0 {
        ", and cut the top result"
    } else {
        ""
    };
    Some(format!(
        "jag: size limit reached: showed {} of {} results, left out {}{cut}. {}",
        size.shown,
        size.shown + size.left_out,
        size.left_out,
        size_fix(limit)
    ))
}

/// The count of characters of the document that `write_json` gives.
fn document_chars(summary: &Summary, results: &[&Snippet]) -> usize {
    let mut buffer = Vec::new();
    // A write to a `Vec` does not fail.
    let _ = write_json(&mut buffer, summary, results);
    char_count(&buffer)
}

/// The paths in a text: each run that starts with `/` and goes to a space, a
/// quote, a bracket, or the end, without the dot, comma, or colon at its end.
fn shorten_paths(text: &str, keep: usize) -> String {
    let mut shortened = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('/') {
        shortened.push_str(&rest[..start]);
        let tail = &rest[start..];
        let end = tail
            .find(|c: char| c.is_whitespace() || "`'\"()".contains(c))
            .unwrap_or(tail.len());
        let run = tail[..end].trim_end_matches(['.', ',', ':', ';']);
        let length = run.chars().count();
        if length > keep {
            shortened.push_str("...");
            let skip = length - (keep - 3);
            shortened.extend(run.chars().skip(skip));
        } else {
            shortened.push_str(run);
        }
        rest = &tail[run.len()..];
    }
    shortened.push_str(rest);
    shortened
}

/// The longest path in the error texts, in characters.
fn longest_path(failure: &Failure) -> usize {
    [&failure.cause, &failure.fix]
        .iter()
        .flat_map(|text| text.split(|c: char| c.is_whitespace() || "`'\"()".contains(c)))
        .filter(|word| word.starts_with('/'))
        .map(|word| word.trim_end_matches(['.', ',', ':', ';']).chars().count())
        .max()
        .unwrap_or(0)
}

/// Write the document of a search inside the size limit (`None` for no
/// limit). `summary` holds the counts; this function sets its 4 size-limit
/// fields, its exit code, and the warnings that stay (STORY-26, STORY-27,
/// CRITERION-104). `exit_code(limited)` gives the exit code, with `limited`
/// true when the limit left out a result or cut the top result. The results
/// fit first, then the warnings. jag never cuts a document, so a document that
/// does not fit with no result and no warning gets shorter paths in its error
/// texts.
pub fn write_document(
    out: &mut dyn Write,
    summary: &mut Summary,
    results: &[&Snippet],
    limit: Option<usize>,
    exit_code: &dyn Fn(bool) -> u8,
) -> io::Result<SizeReport> {
    let count = results.len();
    let warnings = std::mem::take(&mut summary.warnings);
    let total = warnings.len();
    summary.left_out = 0;
    summary.top_result_cut = 0;
    summary.size_limit_fix.clear();
    summary.warnings_left_out = 0;
    summary.exit_code = exit_code(false);
    summary.warnings = warnings.clone();
    let whole = SizeReport {
        shown: count,
        ..SizeReport::default()
    };
    let Some(limit) = limit else {
        write_json(out, summary, results)?;
        return Ok(whole);
    };
    if document_chars(summary, results) <= limit {
        write_json(out, summary, results)?;
        return Ok(whole);
    }

    // Jag leaves out or cuts something. The results fit first, with no warning.
    summary.size_limit_fix = size_fix(limit);
    summary.warnings = Vec::new();
    summary.warnings_left_out = total as u64;
    let size = |summary: &mut Summary, kept: &[&Snippet]| {
        summary.left_out = (count - kept.len()) as u64;
        summary.exit_code = exit_code(summary.left_out > 0);
        document_chars(summary, kept)
    };
    let mut kept = fitting_count(count, limit, |k| size(&mut summary.clone(), &results[..k]));
    let mut cut_snippet = None;
    let mut report = SizeReport {
        shown: kept,
        left_out: count - kept,
        top_result_cut: 0,
    };
    if kept == 0 && count > 0 {
        let top = results[0];
        let fits = |candidate: &Snippet, left: u64| {
            let mut trial = summary.clone();
            trial.left_out = (count - 1) as u64;
            trial.top_result_cut = left;
            trial.exit_code = exit_code(true);
            document_chars(&trial, &[candidate]) <= limit
        };
        match cut_top(top, &fits) {
            Some((candidate, left)) => {
                kept = 1;
                report = SizeReport {
                    shown: 1,
                    left_out: count - 1,
                    top_result_cut: left,
                };
                cut_snippet = Some(candidate);
            }
            None => {
                report = SizeReport {
                    shown: 0,
                    left_out: count,
                    top_result_cut: 0,
                };
            }
        }
    }
    summary.left_out = report.left_out as u64;
    summary.top_result_cut = report.top_result_cut;
    summary.exit_code = exit_code(report.limited());
    let shown: Vec<&Snippet> = match &cut_snippet {
        Some(candidate) => vec![candidate],
        None => results[..kept].to_vec(),
    };

    // An error document that does not fit with no warning gets shorter paths.
    let original = summary.error.clone();
    if document_chars(summary, &shown) > limit
        && let Some(failure) = &original
    {
        let longest = longest_path(failure);
        let fits = |keep: usize, summary: &mut Summary| {
            summary.error = Some(Failure {
                cause: shorten_paths(&failure.cause, keep),
                fix: shorten_paths(&failure.fix, keep),
                exit_code: failure.exit_code,
            });
            document_chars(summary, &shown) <= limit
        };
        let (mut low, mut high) = (3, longest.max(3));
        while low < high {
            // The largest length with which the document fits.
            let middle = low + (high - low).div_ceil(2);
            if fits(middle, &mut summary.clone()) {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        let _ = fits(low, summary);
    }

    // Then the warnings, in order, while the document fits.
    let mut added = 0;
    while added < total {
        summary.warnings = warnings[..=added].to_vec();
        summary.warnings_left_out = (total - added - 1) as u64;
        if document_chars(summary, &shown) > limit {
            break;
        }
        added += 1;
    }
    summary.warnings = warnings[..added].to_vec();
    summary.warnings_left_out = (total - added) as u64;
    write_json(out, summary, &shown)?;
    Ok(report)
}

/// Write one line of a result: `path:N:text` for a matching line, and
/// `path-N-text` for another line. A cut line ends with ` [+N chars]`.
fn write_line(
    out: &mut dyn Write,
    label: &[u8],
    line: &SnippetLine,
    paint: Option<Paint>,
) -> io::Result<()> {
    let colored = paint.is_some();
    let separator: &[u8] = if line.is_match { b":" } else { b"-" };
    item(out, colored, PATH, label)?;
    item(out, colored, SEPARATOR, separator)?;
    item(out, colored, NUMBER, line.number.to_string().as_bytes())?;
    item(out, colored, SEPARATOR, separator)?;
    match paint {
        Some(paint) if line.is_match => {
            write_painted(out, &line.text, line.cut > 0, paint)?;
        }
        _ => out.write_all(&line.text)?,
    }
    if line.cut > 0 {
        write!(out, " [+{} chars]", line.cut)?;
    }
    out.write_all(b"\n")
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
        let label = write_text(&mut out, ranked, None, None)
            .expect("write")
            .no_good_answer;
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
        write_text(&mut out, &[(result, 0.9)], None, None).expect("write");
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
        let results: Vec<&Snippet> = ranked.iter().map(|(snippet, _)| snippet).collect();
        write_json(&mut out, summary, &results).expect("write");
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
        assert!(length < MIN_SIZE_LIMIT, "{length} characters");
    }

    #[test]
    fn no_result_writes_nothing() {
        let (text, label) = write(&[]);
        assert!(text.is_empty());
        assert!(!label);
    }

    /// Every range of `needle` in the line.
    fn find_needle(line: &[u8]) -> Vec<Range<usize>> {
        let word = b"needle";
        (0..line.len())
            .filter(|at| line[*at..].starts_with(word))
            .map(|at| at..at + word.len())
            .collect()
    }

    fn colored(ranked: &[(Snippet, f64)]) -> String {
        let mut out = Vec::new();
        write_text(&mut out, ranked, Some(&find_needle), None).expect("write");
        String::from_utf8(out).expect("text")
    }

    fn paint(code: &str, text: &str) -> String {
        format!("\x1b[{code}m\x1b[K{text}\x1b[m\x1b[K")
    }

    /// `path`, a separator, a number, a separator, as grep colors them.
    fn head(path: &str, separator: &str, number: u64) -> String {
        format!(
            "{}{}{}{}",
            paint("35", path),
            paint("36", separator),
            paint("32", &number.to_string()),
            paint("36", separator)
        )
    }

    #[test]
    fn a_colored_result_has_the_bytes_of_grep_color_always() {
        let ranked = [(
            snippet("a.txt", &[(1, "x needle y", true), (2, "ctx", false)]),
            0.9,
        )];
        assert_eq!(
            colored(&ranked),
            format!(
                "{}x {} y\n{}ctx\n",
                head("a.txt", ":", 1),
                paint("01;31", "needle"),
                head("a.txt", "-", 2)
            )
        );
        // The exact bytes of the spec.
        assert!(colored(&ranked).starts_with(
            "\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K\x1b[32m\x1b[K1\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[Kx \x1b[01;31m\x1b[Kneedle\x1b[m\x1b[K y\n"
        ));
    }

    #[test]
    fn two_results_have_a_colored_double_dash_between_them() {
        let ranked = [
            (snippet("a", &[(1, "needle", true)]), 0.9),
            (snippet("b", &[(1, "needle", true)]), 0.8),
        ];
        assert!(colored(&ranked).contains("\n\x1b[36m\x1b[K--\x1b[m\x1b[K\n"));
    }

    #[test]
    fn two_matches_on_a_line_give_two_colored_items() {
        let ranked = [(snippet("a", &[(1, "needle and needle", true)]), 0.9)];
        let match_item = paint("01;31", "needle");
        assert_eq!(
            colored(&ranked),
            format!("{}{match_item} and {match_item}\n", head("a", ":", 1))
        );
    }

    #[test]
    fn an_empty_range_and_a_line_around_get_no_match_color() {
        let ranked = [(snippet("a", &[(1, "needle", false), (2, "x", true)]), 0.9)];
        let empty = |_: &[u8]| std::iter::once(0..0).collect::<Vec<_>>();
        let mut out = Vec::new();
        write_text(&mut out, &ranked, Some(&empty), None).expect("write");
        let text = String::from_utf8(out).expect("text");
        assert!(!text.contains("01;31"));
        // A line around the match is not searched for matches.
        assert!(text.contains(&format!("{}needle\n", head("a", "-", 1))));
    }

    #[test]
    fn standard_input_has_the_color_of_a_path() {
        let mut result = snippet("a", &[(1, "x", true)]);
        result.source = Source::StandardInput;
        assert!(colored(&[(result, 0.9)]).starts_with(&paint("35", "(standard input)")));
    }

    #[test]
    fn on_a_cut_line_a_match_at_the_cut_has_no_color_and_the_mark_has_none() {
        let mut result = snippet("a", &[(1, "x needle", true)]);
        result.lines[0].cut = 5;
        assert_eq!(
            colored(&[(result, 0.9)]),
            format!("{}x needle [+5 chars]\n", head("a", ":", 1))
        );
        // A match that ends before the end of the kept bytes has color.
        let mut result = snippet("a", &[(1, "needle x", true)]);
        result.lines[0].cut = 5;
        assert_eq!(
            colored(&[(result, 0.9)]),
            format!(
                "{}{} x [+5 chars]\n",
                head("a", ":", 1),
                paint("01;31", "needle")
            )
        );
    }

    #[test]
    fn the_label_line_has_no_color() {
        let ranked = [(snippet("a", &[(1, "needle", true)]), 0.1)];
        assert!(colored(&ranked).starts_with(&format!("{NO_GOOD_ANSWER}\n")));
    }

    fn lines_text(results: &[Snippet], paint: Option<Paint>) -> String {
        let mut out = Vec::new();
        write_lines(&mut out, results, paint, false, None).expect("write");
        String::from_utf8(out).expect("text")
    }

    #[test]
    fn write_lines_gives_one_line_for_each_result_with_no_separator() {
        let results = [
            snippet("a.txt", &[(1, "x", true)]),
            snippet("a.txt", &[(3, "y", true)]),
            snippet("b.txt", &[(2, "z", true)]),
        ];
        assert_eq!(
            lines_text(&results, None),
            "a.txt:1:x\na.txt:3:y\nb.txt:2:z\n"
        );
        assert_eq!(lines_text(&[], None), "");
    }

    #[test]
    fn write_lines_names_standard_input_and_marks_a_cut() {
        let mut input = snippet("a", &[(1, "x", true)]);
        input.source = Source::StandardInput;
        assert_eq!(lines_text(&[input], None), "(standard input):1:x\n");
        let mut cut = snippet("a", &[(1, "xxx", true)]);
        cut.lines[0].cut = 7;
        assert_eq!(lines_text(&[cut], None), "a:1:xxx [+7 chars]\n");
    }

    #[test]
    fn write_lines_gives_the_bytes_of_write_text_for_the_same_result() {
        let find = |line: &[u8]| {
            line.iter()
                .enumerate()
                .filter(|(_, byte)| **byte == b'x')
                .map(|(at, _)| at..at + 1)
                .collect::<Vec<_>>()
        };
        let result = snippet("a.txt", &[(1, "axb", true)]);
        let mut text = Vec::new();
        write_text(&mut text, &[(result.clone(), 0.9)], Some(&find), None).expect("write");
        let lines = lines_text(&[result], Some(&find));
        assert_eq!(lines.as_bytes(), text);
        assert!(lines.contains("\x1b[01;31m"));
    }

    #[test]
    fn write_lines_with_groups_writes_a_double_dash_between_two_results() {
        let results = [
            snippet("a.txt", &[(3, "3", false), (4, "X", true), (5, "5", false)]),
            snippet("b.txt", &[(1, "X", true)]),
        ];
        let mut out = Vec::new();
        write_lines(&mut out, &results, None, true, None).expect("write");
        assert_eq!(
            String::from_utf8(out).expect("text"),
            "a.txt-3-3\na.txt:4:X\na.txt-5-5\n--\nb.txt:1:X\n"
        );
        let mut out = Vec::new();
        write_lines(&mut out, &results, Some(&find_needle), true, None).expect("write");
        let text = String::from_utf8(out).expect("text");
        assert!(text.contains(&format!("\n{}\n", paint("36", "--"))));
        assert!(text.starts_with(&head("a.txt", "-", 3)));
    }

    fn found(path: &str, count: u64) -> FileMatches {
        FileMatches {
            path: PathBuf::from(path),
            lines: (1..=count)
                .map(|number| crate::search::walk::MatchLine {
                    line_number: number,
                    text: b"needle".to_vec(),
                })
                .collect(),
        }
    }

    #[test]
    fn write_files_gives_each_path_with_a_match_one_time_for_each_place() {
        let files = [found("a.txt", 2), found("b.txt", 0), found("a.txt", 1)];
        let mut out = Vec::new();
        write_files(&mut out, &files, false, None).expect("write");
        assert_eq!(out, b"a.txt\na.txt\n");
        let mut out = Vec::new();
        write_files(&mut out, &files[..1], true, None).expect("write");
        assert_eq!(out, b"\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\n");
    }

    #[test]
    fn write_counts_gives_path_and_count_also_for_no_match() {
        let files = [found("a.txt", 2), found("b.txt", 0)];
        let mut out = Vec::new();
        write_counts(&mut out, &files, false, None).expect("write");
        assert_eq!(out, b"a.txt:2\nb.txt:0\n");
        // The bytes of `grep --color=always -c needle a.txt b.txt`.
        let mut out = Vec::new();
        write_counts(&mut out, &files[..1], true, None).expect("write");
        assert_eq!(
            out,
            b"\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K2\n"
        );
    }

    #[test]
    fn write_files_and_write_counts_fit_whole_lines_inside_the_limit() {
        let files = [found("a.txt", 2), found("b.txt", 1), found("c.txt", 1)];
        // `a.txt\n` has 6 characters, so 2 lines fit in 12.
        let mut out = Vec::new();
        let size = write_files(&mut out, &files, false, Some(12)).expect("write");
        assert_eq!(out, b"a.txt\nb.txt\n");
        assert_eq!((size.shown, size.left_out), (2, 1));
        // `a.txt:2\n` has 8 characters, so 1 line fits in 12.
        let mut out = Vec::new();
        let size = write_counts(&mut out, &files, false, Some(12)).expect("write");
        assert_eq!(out, b"a.txt:2\n");
        assert_eq!((size.shown, size.left_out), (1, 2));
        let mut out = Vec::new();
        let size = write_counts(&mut out, &files, false, None).expect("write");
        assert_eq!(size.left_out, 0);
    }

    #[test]
    fn write_json_gives_the_results_in_the_order_it_gets_them() {
        let results = [
            snippet("a.txt", &[(1, "x", true)]),
            snippet("a.txt", &[(3, "y", true)]),
            snippet("b.txt", &[(2, "z", true)]),
        ];
        let summary = Summary {
            results_found: 3,
            ..Summary::default()
        };
        let refs: Vec<&Snippet> = results.iter().collect();
        let mut out = Vec::new();
        write_json(&mut out, &summary, &refs).expect("write");
        let document: serde_json::Value = serde_json::from_slice(&out).expect("json");
        assert_eq!(document["results_found"], 3);
        assert_eq!(document["results_scored"], 0);
        let items = document["results"].as_array().expect("a list");
        assert_eq!(items.len(), 3);
        for (index, item) in items.iter().enumerate() {
            assert_eq!(item["rank"], index + 1);
        }
        assert_eq!(items[1]["first_line"], 3);
        assert_eq!(items[2]["path"], "b.txt");
    }

    // FEATURE-09: the size limit.

    /// A result of `count` lines, each of `width` characters, with line
    /// `matching` as the matching line.
    fn wide(path: &str, count: u64, width: usize, matching: u64) -> Snippet {
        let lines: Vec<SnippetLine> = (1..=count)
            .map(|number| SnippetLine {
                number,
                text: "x".repeat(width).into_bytes(),
                is_match: number == matching,
                cut: 0,
            })
            .collect();
        Snippet {
            source: Source::File(PathBuf::from(path)),
            lines,
        }
    }

    fn limited(ranked: &[(Snippet, f64)], limit: usize) -> (String, TextReport) {
        let mut out = Vec::new();
        let report = write_text(&mut out, ranked, None, Some(limit)).expect("write");
        // A bad sequence of bytes counts as one character, as U+FFFD does.
        (String::from_utf8_lossy(&out).into_owned(), report)
    }

    fn report(shown: usize, left_out: usize, top_result_cut: u64) -> SizeReport {
        SizeReport {
            shown,
            left_out,
            top_result_cut,
        }
    }

    #[test]
    fn fitting_count_gives_the_largest_count_that_fits() {
        let size = |k: usize| k * 10;
        assert_eq!(fitting_count(5, 100, size), 5);
        assert_eq!(fitting_count(5, 49, size), 4);
        assert_eq!(fitting_count(5, 50, size), 5);
        assert_eq!(fitting_count(5, 9, size), 0);
        assert_eq!(fitting_count(0, 9, size), 0);
    }

    // The 30 results of 113 characters of CRITERION-36.
    fn thirty() -> Vec<(Snippet, f64)> {
        (0..30)
            .map(|number| {
                let text = format!("needle {number:02} {}", "x".repeat(90));
                (
                    snippet(&format!("d/r{number:02}.txt"), &[(1, &text, true)]),
                    0.9 - f64::from(number) / 100.0,
                )
            })
            .collect()
    }

    #[test]
    fn whole_results_stay_with_the_lowest_rank_left_out_first() {
        let (text, size) = limited(&thirty(), 1000);
        assert_eq!(text.chars().count(), 925);
        assert_eq!(size.size, report(8, 22, 0));
        assert!(text.contains("d/r07.txt:1:") && !text.contains("d/r08.txt"));
        let (text, size) = limited(&thirty(), 1041);
        assert_eq!(text.chars().count(), 1041);
        assert_eq!(size.size, report(9, 21, 0));
    }

    #[test]
    fn a_large_second_result_keeps_a_small_third_result_out() {
        let ranked = [
            (snippet("a", &[(1, "x", true)]), 0.9),
            (wide("b", 1, 300, 1), 0.8),
            (snippet("c", &[(1, "y", true)]), 0.7),
        ];
        let (text, size) = limited(&ranked, 100);
        assert_eq!(text, "a:1:x\n");
        assert_eq!(size.size, report(1, 2, 0));
    }

    #[test]
    fn the_label_the_separators_and_the_marks_count() {
        let ranked = [
            (snippet("a", &[(1, "x", true)]), 0.2),
            (snippet("b", &[(1, "y", true)]), 0.1),
        ];
        let full = format!("{NO_GOOD_ANSWER}\na:1:x\n--\nb:1:y\n");
        let count = full.chars().count();
        let (text, size) = limited(&ranked, count);
        assert_eq!(text, full);
        assert_eq!(size.size, report(2, 0, 0));
        // One character less leaves out the second result and its `--`.
        let (text, size) = limited(&ranked, count - 1);
        assert_eq!(text, format!("{NO_GOOD_ANSWER}\na:1:x\n"));
        assert_eq!(size.size, report(1, 1, 0));
        assert!(size.no_good_answer);
        // A cut mark counts.
        let mut cut = snippet("a", &[(1, "x", true)]);
        cut.lines[0].cut = 5;
        let (text, _) = limited(&[(cut.clone(), 0.9)], 100);
        assert_eq!(text, "a:1:x [+5 chars]\n");
        // The line with its mark does not fit, so jag cuts the top result.
        let (text, size) = limited(&[(cut, 0.9)], 16);
        assert_eq!(text, "a:1: [+6 chars]\n");
        assert_eq!(size.size, report(1, 0, 6));
    }

    #[test]
    fn no_limit_and_a_limit_of_the_full_count_give_the_same_output() {
        let ranked = thirty();
        let mut free = Vec::new();
        let report_free = write_text(&mut free, &ranked, None, None).expect("write");
        assert_eq!(report_free.size, report(30, 0, 0));
        let count = String::from_utf8_lossy(&free).chars().count();
        let (text, size) = limited(&ranked, count);
        assert_eq!(text.as_bytes(), free);
        assert_eq!(size.size, report(30, 0, 0));
    }

    #[test]
    fn the_output_never_has_more_characters_than_the_limit() {
        let bad = {
            let mut result = snippet("p/\u{e9}.txt", &[(1, "x", true), (2, "y", false)]);
            result.lines[0].text = b"a\xffb\xe2\x82 \xc3\xa9\xc3\xa9".to_vec();
            result.lines[1].cut = 40;
            result
        };
        let lists: [Vec<(Snippet, f64)>; 4] = [
            thirty(),
            vec![
                (bad.clone(), 0.9),
                (bad.clone(), 0.8),
                (wide("w", 7, 60, 4), 0.7),
            ],
            vec![(wide("w", 7, 200, 4), 0.2), (bad, 0.1)],
            vec![(snippet("a", &[(1, "x", true)]), 0.1)],
        ];
        for ranked in &lists {
            for limit in 1..=2000 {
                let (text, size) = limited(ranked, limit);
                assert!(text.chars().count() <= limit, "{limit}: {text:?}");
                assert_eq!(
                    size.size.shown + size.size.left_out,
                    ranked.len(),
                    "{limit}"
                );
            }
        }
    }

    // Example 1 of the spec: lines of 200 characters, line 4 matches.
    #[test]
    fn a_cut_top_result_ends_at_the_marked_line_example_1() {
        let ranked = [
            (wide("d/top.txt", 7, 200, 4), 0.9),
            (snippet("low", &[(1, "x", true)]), 0.5),
        ];
        let (text, size) = limited(&ranked, 1000);
        assert_eq!(text.chars().count(), 865);
        assert_eq!(size.size, report(1, 1, 603));
        let rows: Vec<&str> = text.lines().collect();
        assert_eq!(rows.len(), 4);
        assert!(rows[0].starts_with("d/top.txt-1-"));
        assert!(rows[3].starts_with("d/top.txt:4:") && rows[3].ends_with(" [+603 chars]"));
    }

    // Example 2: lines before the match are left out first.
    #[test]
    fn a_cut_top_result_leaves_out_lines_before_the_match_example_2() {
        let ranked = [(wide("d/top.txt", 7, 300, 4), 0.9)];
        let (text, size) = limited(&ranked, 1000);
        assert_eq!(text.chars().count(), 953);
        assert_eq!(size.size, report(1, 0, 1204));
        let rows: Vec<&str> = text.lines().collect();
        assert!(rows[0].starts_with("d/top.txt-2-"));
        assert!(rows[2].starts_with("d/top.txt:4:") && rows[2].ends_with(" [+1204 chars]"));
    }

    #[test]
    fn a_matching_line_that_does_not_fit_is_cut_at_a_character() {
        let mut top = wide("d/t.txt", 1, 0, 1);
        top.lines[0].text = "é".repeat(300).into_bytes();
        let (text, size) = limited(&[(top, 0.9)], 150);
        let row = text.trim_end();
        assert!(row.starts_with("d/t.txt:1:é"));
        assert!(text.chars().count() <= 150);
        let kept = row.matches('é').count();
        assert!(kept > 100, "{kept}");
        assert!(row.ends_with(&format!(" [+{} chars]", 300 - kept)));
        assert_eq!(size.size, report(1, 0, (300 - kept) as u64));
    }

    #[test]
    fn the_mark_of_the_cut_takes_the_place_of_the_mark_of_the_last_line() {
        let mut top = wide("d/t.txt", 3, 200, 1);
        top.lines[0].cut = 50;
        let (text, size) = limited(&[(top, 0.9)], 300);
        // Line 1 with its mark does not fit with the mark of the cut: line 1
        // is the last line, so its own cut counts in N.
        let rows: Vec<&str> = text.lines().collect();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].ends_with(" chars]"));
        let n = size.size.top_result_cut;
        assert!(rows[0].ends_with(&format!(" [+{n} chars]")));
        assert!(n >= 50 + 2 * 201);
    }

    #[test]
    fn a_room_that_is_too_small_for_the_path_and_the_mark_gives_no_output() {
        let ranked = [
            (wide("d/a-long-path.txt", 1, 50, 1), 0.9),
            (wide("b", 1, 5, 1), 0.8),
        ];
        let (text, size) = limited(&ranked, 20);
        assert_eq!(text, "");
        assert_eq!(size.size, report(0, 2, 0));
    }

    #[test]
    fn a_result_that_starts_with_its_matching_line_leaves_out_no_line_before_it() {
        let ranked = [(wide("d/t.txt", 4, 300, 1), 0.9)];
        let (text, size) = limited(&ranked, 700);
        assert!(text.starts_with("d/t.txt:1:"));
        assert_eq!(size.size.shown, 1);
        assert!(size.size.top_result_cut > 0);
    }

    #[test]
    fn a_limit_of_the_label_alone_leaves_out_each_result() {
        let ranked = [(snippet("a", &[(1, "x", true)]), 0.2)];
        let (text, size) = limited(&ranked, 10);
        assert_eq!(text, "");
        assert_eq!(size.size, report(0, 1, 0));
    }

    #[test]
    fn write_lines_has_the_limit_and_cuts_a_first_line() {
        let lines: Vec<Snippet> = (0..30)
            .map(|number| {
                snippet(
                    &format!("d/r{number:02}.txt"),
                    &[(1, &format!("n {}", "x".repeat(97)), true)],
                )
            })
            .collect();
        let mut out = Vec::new();
        let size = write_lines(&mut out, &lines, None, false, Some(1000)).expect("write");
        assert_eq!(size, report(8, 22, 0));
        assert_eq!(String::from_utf8_lossy(&out).chars().count(), 896);
        let mut out = Vec::new();
        let size = write_lines(&mut out, &lines, None, false, Some(60)).expect("write");
        assert_eq!(size.shown, 1);
        assert_eq!(size.left_out, 29);
        assert!(size.top_result_cut > 0);
        assert!(String::from_utf8_lossy(&out).chars().count() <= 60);
    }

    #[test]
    fn the_notice_has_each_form() {
        let fix = "To see more, use --max-chars with a number larger than 1000, or --no-max-chars.";
        assert_eq!(
            size_notice(&report(8, 22, 0), Some(1000)),
            Some(format!(
                "jag: size limit reached: showed 8 of 30 results, left out 22. {fix}"
            ))
        );
        assert_eq!(
            size_notice(&report(1, 2, 7), Some(1000)),
            Some(format!(
                "jag: size limit reached: showed 1 of 3 results, left out 2, and cut the top result. {fix}"
            ))
        );
        assert_eq!(
            size_notice(&report(1, 0, 7), Some(1000)),
            Some(format!(
                "jag: size limit reached: showed 1 of 1 results, left out 0, and cut the top result. {fix}"
            ))
        );
        assert_eq!(
            size_notice(&report(0, 3, 0), Some(1000)),
            Some(format!(
                "jag: size limit reached: showed 0 of 3 results, left out 3. {fix}"
            ))
        );
        assert_eq!(size_notice(&report(3, 0, 0), Some(1000)), None);
        assert_eq!(size_notice(&report(1, 2, 0), None), None);
    }

    // The document.

    fn document_of(
        summary: &mut Summary,
        results: &[&Snippet],
        limit: Option<usize>,
    ) -> (String, SizeReport) {
        let mut out = Vec::new();
        let size = write_document(&mut out, summary, results, limit, &|limited| {
            if limited { 8 } else { 0 }
        })
        .expect("write");
        (String::from_utf8(out).expect("text"), size)
    }

    #[test]
    fn a_document_with_no_limit_or_inside_the_limit_is_whole() {
        let results = thirty();
        let refs: Vec<&Snippet> = results.iter().map(|(snippet, _)| snippet).collect();
        let (free, size) = document_of(&mut Summary::default(), &refs, None);
        assert_eq!(size, report(30, 0, 0));
        let (text, size) = document_of(&mut Summary::default(), &refs, Some(free.chars().count()));
        assert_eq!(text, free);
        assert_eq!(size, report(30, 0, 0));
        assert!(text.contains("\"size_limit_fix\":\"\""));
    }

    #[test]
    fn a_document_fits_the_limit_and_names_what_it_left_out() {
        let results = thirty();
        let refs: Vec<&Snippet> = results.iter().map(|(snippet, _)| snippet).collect();
        for limit in (1000..7000).step_by(250) {
            let mut summary = Summary::default();
            let (text, size) = document_of(&mut summary, &refs, Some(limit));
            assert!(text.chars().count() <= limit, "{limit}");
            let value: serde_json::Value = serde_json::from_str(&text).expect("json");
            assert_eq!(value["results"].as_array().map(Vec::len), Some(size.shown));
            assert_eq!(value["left_out"], size.left_out);
            assert_eq!(value["exit_code"], if size.left_out > 0 { 8 } else { 0 });
        }
    }

    #[test]
    fn a_cut_top_result_in_a_document_has_no_mark_and_a_cut_field() {
        let top = wide("d/t.txt", 7, 200, 4);
        let low = snippet("low", &[(1, "x", true)]);
        let mut summary = Summary::default();
        let (text, size) = document_of(&mut summary, &[&top, &low], Some(1000));
        assert!(text.chars().count() <= 1000);
        assert_eq!(size.shown, 1);
        assert_eq!(size.left_out, 1);
        assert!(size.top_result_cut > 0);
        assert!(!text.contains("chars]"));
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        assert_eq!(value["top_result_cut"], size.top_result_cut);
        assert_eq!(value["exit_code"], 8);
    }

    #[test]
    fn warnings_fit_after_the_results_and_are_never_cut() {
        let warnings: Vec<WalkWarning> = (0..50)
            .map(|number| WalkWarning {
                path: Some(PathBuf::from(format!("d/locked{number:02}.txt"))),
                kind: WarningKind::Read(io::ErrorKind::PermissionDenied),
            })
            .collect();
        let result = snippet("d/a.txt", &[(1, "needle", true)]);
        let mut summary = Summary {
            warnings: warnings.clone(),
            ..Summary::default()
        };
        let (text, size) = document_of(&mut summary, &[&result], Some(1000));
        assert!(text.chars().count() <= 1000);
        assert_eq!(size, report(1, 0, 0));
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        let kept = value["warnings"].as_array().map_or(0, Vec::len);
        assert!(kept > 0 && kept < 50);
        assert_eq!(value["warnings_left_out"], 50 - kept);
        assert_eq!(value["exit_code"], 0, "a warning is not a result");
        assert!(
            value["size_limit_fix"]
                .as_str()
                .is_some_and(|text| !text.is_empty())
        );
        // The kept warnings are the first ones, in order.
        assert_eq!(value["warnings"][0]["path"], "d/locked00.txt");
    }

    #[test]
    fn an_error_document_with_a_long_path_gets_a_shorter_path() {
        let path = format!("/home/{}/config.yml", "h".repeat(1200));
        let failure = Failure {
            cause: format!("{path} can be read by other users (mode 0644)."),
            fix: format!("Run `chmod 600 {path}`."),
            exit_code: 3,
        };
        let mut summary = Summary {
            error: Some(failure),
            ..Summary::default()
        };
        let mut out = Vec::new();
        write_document(&mut out, &mut summary, &[], Some(1000), &|_| 3).expect("write");
        let text = String::from_utf8(out).expect("text");
        assert!(text.chars().count() <= 1000, "{}", text.chars().count());
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        let cause = value["error"]["cause"].as_str().expect("text");
        assert!(cause.starts_with("...") && cause.contains("config.yml can be read"));
        assert!(
            value["error"]["fix"]
                .as_str()
                .expect("text")
                .contains("`chmod 600 ...")
        );
        assert_eq!(value["exit_code"], 3);
        assert!(
            value["size_limit_fix"]
                .as_str()
                .is_some_and(|text| !text.is_empty())
        );
    }

    #[test]
    fn shorten_paths_keeps_the_end_of_each_path_in_the_same_form() {
        assert_eq!(
            shorten_paths(
                "/aaaaaaaaaa/b.yml is bad. Run `chmod 600 /aaaaaaaaaa/b.yml`.",
                8
            ),
            "...b.yml is bad. Run `chmod 600 ...b.yml`."
        );
        assert_eq!(shorten_paths("/ab is short.", 8), "/ab is short.");
        assert_eq!(shorten_paths("no path here", 5), "no path here");
    }
}
