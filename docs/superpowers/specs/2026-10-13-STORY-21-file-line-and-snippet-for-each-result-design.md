---
title: "File, line, and snippet for each result: the path form, the line range, the cut of long lines, and the standard input label"
feature: FEATURE-08
stories: [STORY-21, STORY-22, STORY-23, STORY-24]
kind: build
lifecycle_state: draft
inherits: [IF-1, IF-6, C-search, C-scorer, C-output, CV-3, CV-4]
session: 84086903-189a-4495-bbe8-0cfb867635cd
---

# File, line, and snippet for each result: the path form, the line range, the cut of long lines, and the standard input label

## Problem

Each result shows the file, the line range, and the exact source text, with
each matching line marked. So the person or the agent can open the source at
that line (FEATURE-08, CAPABILITY-03).

- STORY-21: a knowledge seeker sees the file, the line number, and the exact
  source text of each result, with the path in the same form as in grep.
  - CRITERION-31: each result shows the file, the line number, and the
    snippet.
  - CRITERION-32: the path of each result follows the form of the path that
    the user gave, as in grep (DECISION-15).
- STORY-22: an agent builder gets the first and the last line of a joined
  result, and each line that matches the term has a mark.
  - CRITERION-33: a result gives its first and last line, and it marks each
    line that matches the term (DECISION-16).
- STORY-23: jag cuts each line that is longer than a fixed length and marks
  the cut, so that one long line does not fill the output.
  - CRITERION-34: a result shows the exact source text, and jag cuts each
    line that is longer than a fixed length and marks the cut (DECISION-17).
- STORY-24: a result from piped text has the label `(standard input)` and the
  line numbers of the piped text.
  - CRITERION-35: a result from piped text has the label `(standard input)`
    and the line numbers of the piped text (DECISION-18).

The FEATURE-04 code (merge 61a8f22) already writes most of the form. This
spec adds the cut of long lines and its mark, the source type for standard
input, and the range functions of IF-6. It also adds the tests that prove the
path form and the range.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Text form and line range | Keep the `grep -n` form with lines around the match: `path:N:text` for a matching line, `path-N-text` for another line, and `--` between two results. The first and the last line number of a result give its range. No heading line | human:404pilo, 2026-10-13 (Q-ao); FEATURE-04 spec decision 27; DECISION-16 |
| 2 | Cut length | 300 characters, the same length as in the model request. One constant `MAX_LINE_CHARS` in `src/search/snippet.rs`, for the request and the output | human:404pilo, 2026-10-13 (Q-ap); FEATURE-04 spec decision 21; DECISION-17 |
| 3 | Cut mark | ` [+N chars]` after the kept text. N is the count of characters that jag cut | human:404pilo, 2026-10-13 (Q-aq); DECISION-17 |
| 4 | Piped text | Only the output part: the source of a result is a file or standard input, and standard input shows `(standard input)`. A unit test proves it. FEATURE-14 reads standard input and adds the end-to-end test of CRITERION-35 | human:404pilo, 2026-10-13 (Q-ar); DECISION-18 |
| 5 | Path form | The root as the user typed it, joined with the rest of the path, as the walk gives it. A relative root gives relative paths, and an absolute root gives absolute paths. jag does not change the path | DECISION-15; CRITERION-32; FEATURE-05 spec (`FileMatches.path`); GNU grep 3.12, checked on 2026-10-13 |
| 6 | Path on each line | Each line of a result shows the path, also when the user names one file. This is the form of `grep -H`. Plain grep hides the path when it searches one file | CRITERION-31; CAPABILITY-03 |
| 7 | Path bytes | The output writes the bytes of the path, also when they are not UTF-8, as grep does | FEATURE-04 code (`src/output.rs`) |
| 8 | Line bytes | The text of a line is its bytes as the file holds them, also a `\r` at the end. jag only cuts it (decision 2) | FEATURE-04 spec (`read_snippets` step 2, decision 27) |
| 9 | What a character is | A character of the text that `String::from_utf8_lossy` gives. Each UTF-8 character counts as 1. Each bad byte sequence counts as 1, because it becomes one replacement character. For a UTF-8 line, this is the usual count of characters | This spec: the request already counts this way (FEATURE-04 decision 21). So the reader sees the text that the model scored, which is the reason of Q-ap |
| 10 | Place of the cut | `SnippetLine::new` cuts each line one time, and `read_snippets` uses it. `SnippetLine` holds the kept bytes and the count of cut characters. The request and the output read the same `SnippetLine` | IF-6 (the Result record holds the marks on cut lines); Q-ap reason |
| 11 | Mark in the request | The request holds no mark. The request text of each line stays the same as in `jag-p1` | 0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md; FEATURE-04 spec decision 21 |
| 12 | Mark form | Always the word `chars`, also for N = 1. One space before `[`. Plain ASCII | This spec: one fixed form, so a script finds the mark with one pattern; Q-aq |
| 13 | Lines that jag cuts | Each line of a result: the matching lines and the lines around them | DECISION-17 ("each line that is longer than a fixed length") |
| 14 | First and last line | `Snippet::first_line()` and `Snippet::last_line()` give the range. `rank` uses `first_line()`. FEATURE-11 writes them as fields | IF-6; DECISION-16; Q-ao |
| 15 | Rank order with a source | Equal scores sort by the label bytes of the source (a path, or `(standard input)`), then by the first line | FEATURE-04 spec decision 28 |
| 16 | Color | No color in this feature | DECISION-38; FEATURE-13 |
| 17 | Spec grouping | One spec for STORY-21 to STORY-24 | docs/adlc.md: one feature gives one spec |

## Design

### Files

```
src/search/snippet.rs   C-search: Source, MAX_LINE_CHARS, cut_line, SnippetLine.cut,
                        SnippetLine::new, Snippet::first_line, Snippet::last_line
src/scorer/prompt.rs    C-scorer: reads the cut text; uses MAX_LINE_CHARS of snippet.rs
src/scorer.rs           C-scorer: rank sorts by the source label and first_line()
src/output.rs           C-output: the source label and the cut mark
tests/search.rs         the path form, the range, and the long line tests
tests/score.rs          the test helper builds a Source
```

No new crate. `<[u8]>::utf8_chunks` is in `std` since Rust 1.79.

### Results (C-search, IF-6)

```rust
pub const MAX_LINE_CHARS: usize = 300;

pub enum Source { File(PathBuf), StandardInput }

impl Source {
    pub const STANDARD_INPUT_LABEL: &'static str = "(standard input)";
    /// The bytes that the output writes for the source.
    pub fn label(&self) -> &[u8];
}

pub struct SnippetLine {
    pub number: u64,
    pub text: Vec<u8>,   // the kept bytes: the full line, or its first 300 characters
    pub is_match: bool,
    pub cut: u64,        // the count of characters that jag cut; 0 when it cut none
}

impl SnippetLine {
    /// Cut the line with cut_line, and keep the kept bytes and the count.
    pub fn new(number: u64, line: &[u8], is_match: bool) -> SnippetLine;
}

pub struct Snippet { pub source: Source, pub lines: Vec<SnippetLine> }

impl Snippet {
    pub fn first_line(&self) -> u64;
    pub fn last_line(&self) -> u64;
}

pub fn cut_line(bytes: &[u8]) -> (&[u8], u64);
```

- `Source` derives `Clone`, `Debug`, `PartialEq`, and `Eq`, as `Snippet`
  does.
- `Source::File(path).label()` gives `path.as_os_str().as_encoded_bytes()`.
  `Source::StandardInput.label()` gives the bytes of
  `STANDARD_INPUT_LABEL`.
- `first_line()` gives the number of the first line, and `last_line()` gives
  the number of the last line. `read_snippets` never gives a `Snippet` with
  no line. For an empty `Snippet`, both functions give 0, so they never
  panic.
- `Window` keeps `path: PathBuf`, because the scan plans windows only for
  files. `read_snippets` sets `source: Source::File(window.path.clone())`.
  FEATURE-14 builds the snippets of standard input.
- The FEATURE-04 helper `first_line` in `src/scorer.rs` goes away.
  `rank` calls `Snippet::first_line()`.

#### The cut of a line

`cut_line` gives the kept bytes and the count of cut characters (decisions
2, 9, and 10). It walks `bytes.utf8_chunks()`:

1. For each chunk, count each `char` of `chunk.valid()` as 1. Then, when
   `chunk.invalid()` is not empty, count it as 1. `String::from_utf8_lossy`
   makes one replacement character from each such part.
2. Keep the byte offset after the 300th character.
3. When the line has 300 characters or fewer, give `(bytes, 0)`.
4. Else, give the bytes before that offset and the count of the other
   characters.

So the cut never splits a UTF-8 character. The kept bytes are the first
bytes of the line, with no change. `String::from_utf8_lossy` of the kept
bytes is the same text as the first 300 characters of
`String::from_utf8_lossy` of the full line. That is the text that the
FEATURE-04 request holds.

`SnippetLine::new` calls `cut_line`. `read_snippets` builds each line of
each window with `SnippetLine::new`, after the split into lines (FEATURE-04,
`read_snippets` step 2). Each `SnippetLine` keeps only the kept bytes. The full file stays in
memory only while `read_snippets` builds its windows, as in FEATURE-04.

Each code that makes a `SnippetLine` from the text of a file or of a pipe
uses `SnippetLine::new`. FEATURE-14 does this for standard input. So each
line in a request has 300 characters or fewer, and the batch limit of
FEATURE-04 decision 22 stays true. Only tests set the fields directly.

Examples, with `MAX_LINE_CHARS = 300`:

| Line | Kept | `cut` |
|---|---|---|
| 300 ASCII characters | all 300 bytes | 0 |
| 301 ASCII characters | the first 300 bytes | 1 |
| 1,000 ASCII characters | the first 300 bytes | 700 |
| 400 CJK characters (3 bytes each) | the first 900 bytes | 100 |
| 400 bytes `0xFF` | the first 300 bytes | 100 |
| 299 `a`, the bytes `0xE2 0x82`, 10 `b` | 299 `a` and `0xE2 0x82` (301 bytes) | 10 |
| 300 ASCII characters and a `\r` at the end | the first 300 bytes | 1 |

In the last example, the `\r` is a character. jag cuts it with the rest.
So a CRLF line of exactly 300 visible characters shows ` [+1 chars]`.
The request counts the `\r` in the same way. This is a known limit.

### The request (C-scorer)

`snippet_json` in `src/scorer/prompt.rs` makes each line text with
`String::from_utf8_lossy(&line.text)`, and it keeps
`.chars().take(MAX_LINE_CHARS)` as a guard. `read_snippets` already cut
the line, so the guard changes nothing for a line from `SnippetLine::new`.
It stops a line that a later producer sets directly. `prompt.rs` uses
`crate::search::snippet::MAX_LINE_CHARS`, and its own constant goes away.

- The request holds no cut mark (decision 11). So the body of each call is
  the same as before, and the prompt version stays `jag-p1`.
- `path` is `String::from_utf8_lossy(snippet.source.label())`. For a file,
  this is the same text as `to_string_lossy` of the path. For standard
  input, it is `(standard input)`.
- The batch estimate of FEATURE-04 decision 22 does not change, because the
  text of the request does not change.

### Text output (C-output)

`write_text` keeps its signature. For each line of a result, it writes:

1. `snippet.source.label()`;
2. `:` for a matching line, `-` for another line;
3. the line number;
4. the same separator again;
5. the bytes of `line.text`;
6. when `line.cut` is more than 0, the ASCII text ` [+<cut> chars]`
   (decisions 3 and 12);
7. `\n`.

A line `--` comes between two results, as before. The no-good-answer label
does not change.

Example (the long line of `app.min.js` is shown short here; jag writes its
first 300 characters):

```
src/net.rs-38-    let wait = Duration::from_secs(1);
src/net.rs:41:    // retry one time after a timeout
src/net.rs-42-    retry(call, wait)
--
dist/app.min.js:1:!function(e){var t={};function n(r){if(t[r])return t[r].exports; [+48211 chars]
```

A result from standard input:

```
(standard input)-6-The client starts a call.
(standard input):7:The client waits one second, then tries again.
```

### The path form (C-search, C-output)

The scan gives `FileMatches.path` as the root, as the user typed it, joined
with the rest of the path (FEATURE-05 spec). `read_snippets` and
`write_text` keep it as it is (decisions 5 and 7). So this part needs no new
code. The tests below prove it.

The forms, with the folder `dir` that holds `sub/a.txt`. The grep column
comes from GNU grep 3.12 on 2026-10-13:

| The user gives | jag writes | `grep -rn` writes |
|---|---|---|
| `dir` | `dir/sub/a.txt` | `dir/sub/a.txt` |
| `dir/` | `dir/sub/a.txt` | `dir/sub/a.txt` |
| `./dir` | `./dir/sub/a.txt` | `./dir/sub/a.txt` |
| `/tmp/x/dir` (absolute) | `/tmp/x/dir/sub/a.txt` | `/tmp/x/dir/sub/a.txt` |
| `dir/sub/a.txt` (one file) | `dir/sub/a.txt` | no path; `grep -Hn` writes `dir/sub/a.txt` |

For one file, jag writes the path, as `grep -Hn` does (decision 6).

### Acceptance criteria and their tests

The end-to-end tests use the setup of the FEATURE-04 spec: `HOME` in a
`tempfile` folder, the key text `SENTINEL-SENTINEL-SENTINEL`, the fake
server, and no proxy variables. A test of a relative path runs `jag` with
`current_dir` set to the `tempfile` folder.

| Criterion | Test |
|---|---|
| CRITERION-31 Each result shows the file, the line, and the snippet | `tests/search.rs`: a folder with `notes/a.txt` of 8 lines, with `needle` on line 5. The server gives 0.8. Standard output is `notes/a.txt-2-<line 2>` to `notes/a.txt-8-<line 8>`, and line 5 is `notes/a.txt:5:<line 5>`. Each line has the path, the number, and the exact text. |
| CRITERION-32 The path has the same form as in grep | `tests/search.rs`, one test for each row of the table in "The path form": `dir`, `dir/`, `./dir`, the absolute path of the `tempfile` folder joined with `dir`, and `dir/sub/a.txt`. Each result line starts with the path in the "jag writes" column. |
| CRITERION-33 A joined result shows its line range | Unit (`src/output.rs`): a `Snippet` of lines 7 to 17 with matches on lines 10 and 14. The first output line starts with `a.txt-7-`, and the last with `a.txt-17-`. Lines 10 and 14 start with `a.txt:10:` and `a.txt:14:`. Each other line has `-`. Unit (`src/search/snippet.rs`): `first_line()` gives 7 and `last_line()` gives 17. End-to-end: a file of 20 lines with `needle` on lines 10 and 14 gives one result from `a.txt-7-` to `a.txt-17-`. |
| CRITERION-34 jag cuts a very long line and marks the cut | Unit (`src/search/snippet.rs`): `cut_line` with each example of "The cut of a line". When the line is valid UTF-8, the kept bytes are valid UTF-8. Unit (`src/output.rs`): a matching line with 300 `x` and `cut` 700 gives `a:1:<300 x> [+700 chars]\n`. A line around the match with `cut` 1 gives `a-2-<text> [+1 chars]\n`. A line with `cut` 0 has no mark. End-to-end: see "The long line test". |
| CRITERION-35 Piped text has the label (standard input) | Unit (`src/output.rs`): a `Snippet` with `Source::StandardInput`, lines 6 and 7, and a match on line 7, gives `(standard input)-6-...` and `(standard input):7:...`. Unit (`src/search/snippet.rs`): `Source::StandardInput.label()` is `b"(standard input)"`. Unit (`src/scorer/prompt.rs`): the request of that `Snippet` has `"path": "(standard input)"`. FEATURE-14 adds the end-to-end test with a pipe (decision 4). |

#### The long line test

The FEATURE-04 test
`tests/search.rs::a_long_line_is_cut_in_the_request_and_whole_in_the_output`
changes, because the output now shows the cut. Rename it to
`a_long_line_is_cut_and_marked`. The line is `needle` and 994 `x` (1,000
characters):

- The request holds 300 characters of the line, as before.
- Standard output holds `a.txt:1:needle<294 x> [+700 chars]\n`.
- The text of the line in the request is the same as the text before the
  mark on standard output. So the reader sees the text that the model scored.

#### More tests

- The request does not change (decision 11). Unit (`src/scorer/prompt.rs`):
  for each example line of "The cut of a line", build the `SnippetLine` with
  `SnippetLine::new`. The text of the line in the request is the same as
  `String::from_utf8_lossy(<full line>).chars().take(300).collect::<String>()`.
  That is the FEATURE-04 rule.
- The `prompt.rs` test
  `a_line_is_cut_at_300_characters_at_a_character_border` builds its line
  with `SnippetLine::new`, and it stays green. The batch tests of
  `prompt.rs` use lines of 300 characters or fewer, so they do not change.
- `read_snippets`: a file with a line of 1,000 characters gives a
  `SnippetLine` with 300 bytes of text and `cut` 700.
- `rank`: two results with equal scores, one from `Source::StandardInput`
  and one from the file `a.txt`. The order follows the label bytes (decision
  15), so `(standard input)` comes first, because `(` is before `a`.
- Each test that builds or compares a `Snippet` (`src/search/snippet.rs`,
  `src/output.rs`, `src/scorer.rs`, `src/scorer/prompt.rs`,
  `tests/score.rs`) sets `source: Source::File(...)` and `cut: 0`.

## Build order

0. The FEATURE-04 code (merge 61a8f22) and the FEATURE-03 code (merge
   a9e886c) are on `main`.
1. `Source`, `Snippet::first_line`, and `Snippet::last_line`. Change each
   use of `Snippet.path` (`rank`, `snippet_json`, `write_text`, the test
   helpers). Add the tests of CRITERION-33 and CRITERION-35 (STORY-22,
   STORY-24).
2. `MAX_LINE_CHARS`, `cut_line`, and `SnippetLine::new` in
   `src/search/snippet.rs`, the cut in `read_snippets`, and the change in
   `snippet_json`. Add the test that the
   request does not change.
3. The cut mark in `write_text`. Change the long line test, and add the
   other tests of CRITERION-34 (STORY-23).
4. The end-to-end tests of CRITERION-31 and CRITERION-32 (STORY-21).

## Assumptions

- With two `/` or more at the end of a root, for example `dir//`, the walk
  keeps them: `dir//sub/a.txt`. GNU grep 3.12 writes `dir/sub/a.txt`. This
  spec does not test or change this case. Source: `PathBuf::push` adds no
  `/` after a path that ends with `/`, and the `ignore` crate joins the
  names to the root in this way.
- A file with a byte order mark of UTF-16 is changed to UTF-8 before the
  split (FEATURE-04, `decode` in `src/search/snippet.rs`). For this file,
  "the bytes of the line" are the UTF-8 bytes. Source:
  `src/search/snippet.rs`.

## Risks

- A long line can hold the answer after its first 300 characters. The
  model does not see that text, and the reader sees only the mark. Signal:
  `jag-bench` finds an answer that is past the cut. Mitigation: the length
  is one constant, and a new length needs a new prompt version
  (ADR-0004).
- The mark ` [+N chars]` can look like source text. Signal: a user reports
  that a line in the source ends with the same text. Mitigation: the
  structured output of FEATURE-11 gives the cut count as its own field.
- `dir//` gives another path than grep (see "Assumptions"). Signal: a user
  reports it. Mitigation: FEATURE-14 owns the path handling, and it can
  remove the extra `/`.

## Out of scope

- Reading standard input, and the end-to-end test of CRITERION-35:
  FEATURE-14. Note for FEATURE-14: with `-r` and no path, GNU grep and `rg`
  write paths with no `./` at the start (for example `dir/sub/a.txt`).
- The size limit, and the cut of a top result that is larger than the limit
  (DECISION-23): FEATURE-09. It counts the kept text and the mark.
- The first line, the last line, and the cut count as fields of structured
  output: FEATURE-11.
- Whether plain grep with no question cuts long lines: FEATURE-12.
- Color, `-h`, `-H`, `-A`, `-B`, and `-C`: FEATURE-13.
