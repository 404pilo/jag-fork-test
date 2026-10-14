---
title: "Text output for a person and structured output for an agent: the --json option, one JSON document, structured errors, and a stable form"
feature: FEATURE-11
stories: [STORY-29, STORY-30, STORY-31, STORY-32]
kind: build
gh_issue: 90
lifecycle_state: draft
inherits: [IF-3, IF-4, IF-5, IF-6, C-cli, C-output, CV-1, CV-2, CV-3, CV-4, CV-6, CV-7]
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Text output for a person and structured output for an agent: the --json option, one JSON document, structured errors, and a stable form

## Problem

A person at the shell reads text. An agent needs output that it can parse
(CAPABILITY-05). jag writes text by default, also in a pipe, as grep does.
Only an explicit option turns on structured output (DECISION-27). In text
mode, the results go to standard output and the notices go to the error
stream, so a pipe gets only the results (DECISION-29). In structured mode,
jag writes everything to standard output, inside the structure. The
structure holds the same results as the text, a named field for each
label, count, mark, and notice, and the rank of each result with no score
(DECISION-28). It is complete also when nothing matches (DECISION-30). An
error has the same form, with the cause, the fix, and the exit code
(DECISION-07). The structure shows the version of its form, and in one
major version jag only adds fields (CONSTRAINT-05).

- STORY-29: a shell power user wants readable text by default, also in a
  pipe, with the results on standard output and the notices on the error
  stream.
  - CRITERION-44: jag writes readable text when the user gives no output
    option, also when the output goes to a pipe.
  - CRITERION-45: in text mode, jag writes the results to standard output
    and each notice to the error stream.
- STORY-30: an agent builder wants an explicit option for structured
  output, with the same results as the text, a named field for each item,
  and the rank of each result with no score.
  - CRITERION-46: jag writes structured output only when the user gives the
    structured output option.
  - CRITERION-47: structured output holds the same results as the text,
    and each label, count, mark, and notice of the text is a named field.
  - CRITERION-48: each result in structured output has its rank, and it
    has no score.
- STORY-31: an agent builder wants a complete structured output with no
  results, and each notice and each error inside the structure.
  - CRITERION-49: with no results, structured output holds an empty result
    list and the summary fields.
  - CRITERION-50: in structured mode, jag writes each notice inside the
    structure.
  - CRITERION-51: in structured mode, an error is structured and gives the
    cause, the fix, and the exit code.
- STORY-32: an agent builder wants the structured output to show its form
  version and to only add fields in one major version.
  - CRITERION-52: the structured output shows its form version, and in one
    major version jag never removes or renames a field.

The FEATURE-08 code (merge 769cffe) writes the text form. Text mode already
does what CRITERION-44 and CRITERION-45 say. This spec adds the tests that
prove it, and it does not change the text output. It adds the `--json`
option, the JSON writer, and the structured errors.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Option | `--json`. It turns on structured output for a search. It takes no value. Rejected: `--format json` and `--output json`, because ADR-0008 has one structured form, so the value has only one choice | 0008-write-structured-output-as-one-json-document.md (the form is JSON); system-design.md IF-3 (an option of the public command line) and IF-5; ripgrep `--json` (0001-find-matches-with-the-ripgrep-libraries.md) and `gh --json` use this name (inferred) |
| 2 | Only the option | Text is the default, also in a pipe. No variable, no setting in the config file, and no test of the terminal turns on structured output. So `--json` is not a setting of CV-7 | DECISION-27 (rejected: structured output when the output is not a terminal; a setting that makes structured output the default); CRITERION-44; CRITERION-46 |
| 3 | Text mode | No change. The text output keeps the form of FEATURE-08, byte for byte. The notices stay on the error stream | DECISION-29; system-design.md CV-3; FEATURE-08 spec decision 1 |
| 4 | Document | One JSON object on standard output, on one line, with `\n` at the end. jag writes it only after the search ends. There is no space between two tokens | 0008-write-structured-output-as-one-json-document.md; system-design.md IF-5; DECISION-22 (each character on standard output counts toward the size limit, so jag spends none on spaces) |
| 5 | Field names and order | Words in lower case, joined with `_`, as `prompt_version` of IF-5. The order is fixed: the summary fields first, `results` last. A new field goes before `results`. An agent must not depend on the order | system-design.md IF-5 (inferred) |
| 6 | `version` | The whole number `1`. It changes only when a field is removed or renamed, and that can occur only in a new major version of jag. A new field does not change it | CONSTRAINT-05 (so that agent code keeps working when jag updates); system-design.md IF-5 |
| 7 | `prompt_version` | The value of `scorer::PROMPT_VERSION` (now `jag-p1`), in each document | 0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md; system-design.md IF-5 |
| 8 | `exit_code` | In each document. It is the exit code of the process (IF-4) | DECISION-07 and CRITERION-51 (an error gives its exit code); this spec: one rule for each document, so an agent reads one field |
| 9 | `error` | In each document: `null` when jag did not fail, else an object with `cause` and `fix`, the two texts of the failure | DECISION-07; system-design.md CV-1; src/failure.rs (`Failure`) |
| 10 | Results | `results` is a list in rank order. Each result has `rank` (1 for the first result), `path`, `first_line`, `last_line`, and `lines`. Each line has `number`, `text`, `is_match`, and `cut`. A result has no score | DECISION-28; CRITERION-48; system-design.md IF-6; FEATURE-08 spec decisions 1, 3, and 14 |
| 11 | `is_match` | The name of the mark on a matching line. It is the name of `SnippetLine.is_match`. The word `match` is a keyword in Rust and in Python, the languages of much agent code | src/search/snippet.rs (inferred) |
| 12 | Text of a line | The kept bytes of the line as UTF-8 text. Each byte sequence that is not UTF-8 becomes U+FFFD. The text has no cut mark: `cut` is the count of cut characters, and 0 means no cut | RFC 8259 (a JSON string is Unicode text); FEATURE-08 spec decision 9 (the model scores this same text); FEATURE-08 spec decision 3 |
| 13 | `path` | The path as the text shows it, as UTF-8 text, with U+FFFD for each byte sequence that is not UTF-8. `null` for standard input, in the place of the label `(standard input)`, because no file has that path | FEATURE-08 spec decisions 4, 5, and 7; DECISION-18 (inferred) |
| 14 | Counts of the cost limit | `results_found` (N), `results_scored` (K), and `results_skipped` (S = N - K): the three numbers of the cost-limit notice. They are in each document. When the notice does not show, K is N and S is 0 | DECISION-28 ("the counts of skipped matches"); docs/superpowers/specs/2026-10-13-STORY-18-cost-limit-on-scoring-design.md (the notice); src/cli.rs (the notice uses `kept.len()`) |
| 15 | Count of skipped files | `files_skipped`: an object with `total`, `binary`, `hidden`, and `ignored`, the numbers of the skip notice | DECISION-28 ("skipped files"); docs/superpowers/specs/2026-10-13-STORY-16-search-a-folder-as-grep-r-design.md decision 4 (the structured field comes with FEATURE-11) |
| 16 | Size-limit fields | `left_out`, `top_result_cut`, `size_limit_fix`, and `warnings_left_out`. This feature writes them with the values for "nothing left out or cut": 0, 0, `""`, and 0. FEATURE-09 sets them, and its spec gives their meaning: the count of results left out, the count of characters of the top result that jag did not write, the fix text of the size-limit notice (no `jag: `), and the count of warnings left out | DECISION-28 ("the notice about results that jag left out"); the FEATURE-09 spec (PR #134, head 87b3a7a) through the orchestrator (INFO, 2026-10-13) |
| 17 | Warnings | `warnings`: a list in the order of the error stream. Each warning is an object with `path` (the path of the warning, or `null`) and `text` (the line of the text mode). An object, not a string, so that a later version can add a field to each warning | DECISION-29; CRITERION-50; src/search/walk.rs (`WalkWarning`); CONSTRAINT-05 |
| 18 | No list of notice texts | Each notice of the text mode is in the document as its named numbers (decisions 14 to 16) or as its warning object (decision 17). The one text of a notice is `size_limit_fix` (decision 16), because no number gives that fix. The document has no list of notice texts | DECISION-28 (it names counts); CONSTRAINT-05 (each field is permanent, so jag adds no text that repeats the numbers); the FEATURE-09 spec (PR #134) |
| 19 | `jag: ` | No text in the document starts with `jag: `. The text mode puts `jag: ` in front of each message. The document does not | src/failure.rs (`render` adds `jag: `, and `cause` does not hold it) (inferred) |
| 20 | No match | A complete document: `results` is `[]`, each summary field has its value, and `exit_code` is 1. Text mode still writes nothing to standard output | DECISION-30; CRITERION-49; FEATURE-04 spec decision 23 |
| 21 | Error document | In structured mode, each failure writes one document on standard output. `error` holds the cause and the fix, `exit_code` holds the code, and `results` is `[]`. The summary fields hold what jag counted before the failure, and 0 for a step that did not run. jag writes nothing on the error stream. Decision 25 is the one exception | DECISION-07; system-design.md CV-1, CV-2 (no partial result), CV-3, and IF-5 ("an error is a JSON document of the same form") |
| 22 | Argument errors | When the command line holds `--json` before the first `--`, each argument error writes the error document with the cause `the command line is not valid.` and the fix `Run jag --help to see the usage.`, and exits with 2. Else jag writes the one usage line, as now. The document holds no value from the command line | DECISION-07; system-design.md CV-1 and CV-6; FEATURE-04 spec decision 26 (an argument error never repeats a value, because the value can be a key). DECISION-07 sets the form and decision 26 sets the text, so both apply |
| 23 | Commands with no structured form | `--json` conflicts with `--setup`, `--forget-key`, and `--check`. A conflict is an argument error, so decision 22 gives the error document and exit 2. The FEATURE-02 code is merged (5288944), so this feature adds the `--check` conflict | DECISION-07 (its `about` list holds CAPABILITY-01: the setup and the check); docs/superpowers/specs/2026-10-13-STORY-06-readiness-check-design.md "Out of scope"; src/cli.rs (an option that has no effect in a mode conflicts with that mode); verifier-2 note and the orchestrator, 2026-10-13 |
| 24 | Help and version | `--help` and `--version` write their text, also with `--json`. They are for a person, and they are not errors | src/cli.rs (`parse` prints them); clap docs (`ArgAction::Help` shows the help when clap meets the flag) (inferred) |
| 25 | A failure to write the document | When jag cannot write the document, it writes the failure as text on the error stream, because standard output does not work. It does not try standard output again. When the reader closes the pipe, jag exits with no message | system-design.md CV-4; src/cli.rs (`write_failure`) (inferred) |
| 26 | JSON code | `serde` with `derive`, and `serde_json`, both already in `Cargo.toml`. No new crate | 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md (serde); 0014-call-the-model-service-with-ureq-and-serde-json-on-std.md (serde_json) |
| 27 | Writer for FEATURE-09 | `output::write_json(out, &summary, &ranked[..k])` writes the full document with the first k results, with the ranks 1 to k. The fields of `Summary` are public. A count has at most 20 digits, so the document with no result, no warning, no error, and the FEATURE-09 fix text has at most 597 characters. Its unit test uses a fix text of 120 characters, which gives 622 characters. Both are less than the smallest size limit of 1,000. Warnings and errors have no upper size | the FEATURE-09 points through the orchestrator (INFO, 2026-10-13); DECISION-22; DECISION-30; 0008-write-structured-output-as-one-json-document.md (the size limit counts the whole document) |
| 28 | Code place | The JSON writer and `Summary` in `src/output.rs` (C-output). The option, the mode, and the error path in `src/cli.rs` (C-cli) | 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md; system-design.md C-output ("writes the ranked results as text or as structured output") and C-cli |
| 29 | Docs | The usage line and the help text name `--json`. The README gets the section "Output for an agent", with an example and the fields | docs/superpowers/specs/2026-10-13-STORY-06-readiness-check-design.md (a README section for each command) (inferred) |
| 30 | Spec grouping | One spec for STORY-29 to STORY-32 | docs/adlc.md (one feature gives one spec) |
| 31 | Size limit of a document | The FEATURE-09 spec gives the rules. The results fit first, with no warning. Then jag adds the warnings, in order, while the document fits, and `warnings_left_out` counts the others. A warning that jag leaves out gives no exit 8. jag never cuts a document. When an error document does not fit, jag makes each path in its texts shorter: it keeps the last characters of the path that fit, after `...`. The code that merges second, FEATURE-09 or FEATURE-11, builds these rules, because they need the size limit of FEATURE-09 and the document of this feature. This feature alone has no size limit | the FEATURE-09 spec (PR #134, "Structured mode") through the orchestrator (INFO, 2026-10-13); DECISION-30; 0008-write-structured-output-as-one-json-document.md |

## Design

### Files

```
src/output.rs        C-output: JSON_VERSION, Summary, is_no_good_answer, write_json,
                     and the private types of the document
src/cli.rs           C-cli: --json, Mode, json_requested, the notices by mode,
                     the document of a search, the error document, the usage line
src/search/walk.rs   C-search: WalkWarning::text (the message with no "jag: ")
tests/output.rs      the end-to-end tests of CRITERION-44 to CRITERION-52
tests/usage.rs       the argument error tests with --json
README.md            the section "Output for an agent"
```

No new crate (decision 26).

### The option (C-cli)

```rust
/// Write one JSON document for an agent, in place of text
#[arg(long, conflicts_with_all = ["setup", "forget_key", "check"])]
json: bool,
```

The usage line becomes:

```
jag [--json] [--timeout SECONDS] [--retry-wait SECONDS] [--max-scored COUNT] TERM "QUESTION" [PATH...]
jag --check [--timeout SECONDS] [--retry-wait SECONDS]
jag --setup
jag --forget-key
```

The code that merges second, FEATURE-09 or FEATURE-11, puts the options of
both features in the first line.

The mode comes only from this flag (decision 2):

```rust
/// The form of the output (DECISION-27).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Text,
    Json,
}
```

### The document (C-output)

The document is one JSON object. jag writes it on one line, with `\n` at
the end (decision 4). This table gives each field, in its order.

| Field | Type | Value |
|---|---|---|
| `version` | whole number | `1` (decision 6) |
| `prompt_version` | text | `scorer::PROMPT_VERSION` (decision 7) |
| `exit_code` | whole number | the exit code of the process (decision 8) |
| `error` | object or `null` | `cause` and `fix`, both text (decision 9) |
| `no_good_answer` | true or false | true when the text shows the no-good-answer label |
| `results_found` | whole number | N of the cost-limit notice (decision 14) |
| `results_scored` | whole number | K of the cost-limit notice |
| `results_skipped` | whole number | S of the cost-limit notice |
| `left_out` | whole number | 0; FEATURE-09 sets it (decision 16) |
| `top_result_cut` | whole number | 0; FEATURE-09 sets it (decision 16) |
| `size_limit_fix` | text | `""`; FEATURE-09 sets it (decision 16) |
| `files_skipped` | object | `total`, `binary`, `hidden`, `ignored`, all whole numbers (decision 15) |
| `warnings` | list of objects | `path` (text or `null`) and `text` (decision 17) |
| `warnings_left_out` | whole number | 0; FEATURE-09 sets it (decision 16) |
| `results` | list of objects | in rank order (decision 10) |

Each object of `results`:

| Field | Type | Value |
|---|---|---|
| `rank` | whole number | 1 for the first result, then 2, 3, and so on |
| `path` | text or `null` | the path as the text shows it; `null` for standard input (decision 13) |
| `first_line` | whole number | `Snippet::first_line()` |
| `last_line` | whole number | `Snippet::last_line()` |
| `lines` | list of objects | each line of the result, in line order |

Each object of `lines`:

| Field | Type | Value |
|---|---|---|
| `number` | whole number | the line number; the first line of a file is 1 |
| `text` | text | the kept bytes as UTF-8, with U+FFFD for bad bytes (decision 12) |
| `is_match` | true or false | true for a line that matches the term (decision 11) |
| `cut` | whole number | the count of cut characters; 0 for no cut |

Each number in the document is a whole number. So no score can be in it
(CRITERION-48).

#### Each item of the text and its field (CRITERION-47)

| Item of the text mode | Stream | Field |
|---|---|---|
| The label `jag: no good answer. ...` | standard output | `no_good_answer: true` |
| The path at the start of each line | standard output | `path` of the result |
| `(standard input)` in the place of a path | standard output | `path: null` |
| `:` after the path and after the number (a matching line) | standard output | `is_match: true` |
| `-` after the path and after the number (another line) | standard output | `is_match: false` |
| The line number | standard output | `number`; the first and the last number of a result are `first_line` and `last_line` |
| The text of the line | standard output | `text` |
| ` [+N chars]` at the end of a line | standard output | `cut: N`; `cut: 0` when the line has no mark |
| `--` between two results | standard output | the next object of `results`; `rank` gives the order |
| `jag: skipped T files and folders (B binary, H hidden, I ignored)` | error stream | `files_skipped`: `total` T, `binary` B, `hidden` H, `ignored` I |
| `jag: cannot read ...`, `jag: a rule in ... is not valid; ...`, `jag: cannot walk ...` | error stream | one object of `warnings`: its `path`, and its `text` (the line with no `jag: `) |
| `jag: cost limit reached: scored K of N results, skipped S.` | error stream | `results_found` N, `results_scored` K, `results_skipped` S |
| A failure: `jag: <cause>`, then `<fix>` | error stream | `error.cause`, `error.fix`, `exit_code` |
| The one usage line | error stream | `error` with the fixed texts of decision 22, `exit_code` 2 |
| The size-limit notice of FEATURE-09 | error stream | `left_out`, `top_result_cut`, and `size_limit_fix` (decision 16). The FEATURE-09 spec gives the map |
| The exit code | process | `exit_code` |

#### Types

```rust
/// The version of the form of the document (CONSTRAINT-05). Change it only
/// when a field is removed or renamed, in a new major version of jag.
pub const JSON_VERSION: u32 = 1;

/// Each part of the document other than the results.
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
pub fn is_no_good_answer(ranked: &[(Snippet, f64)]) -> bool;

/// Write the document, with `ranked` as its results in that order, on one
/// line with `\n` at the end. The score is never written.
pub fn write_json(out: &mut dyn Write, summary: &Summary, ranked: &[(Snippet, f64)]) -> io::Result<()>;
```

`write_text` calls `is_no_good_answer`, so the two modes use the same rule.

`write_json` builds private `serde::Serialize` types that borrow the data.
Their fields have the names and the order of the tables above:
`Document`, `ErrorField { cause, fix }`, `FilesSkipped`,
`WarningField { path, text }`, `ResultField`, and `LineField`. Each text
is a `Cow<str>` from `String::from_utf8_lossy` or `Path::to_string_lossy`.
`Source::StandardInput` gives `path: None`, so `null`. Then
`serde_json::to_writer` writes the compact form, and `write_json` writes
`\n`. `serde_json` escapes only `"`, `\`, and the control characters, so a
character that is not ASCII stays as it is.

`WalkWarning::text` gives the line of the text mode with no `jag: `, and
`WalkWarning::message` becomes `format!("jag: {}", self.text())`. So the
error stream does not change.

### The search flow (C-cli)

`run` gets the mode from `cli.json`, makes a `Summary::default()`, and
gives both to `search` and to `finish`. `search` fills the summary as it
goes, so that a failure finds in it what jag counted before.

```rust
let (snippets, read_warnings) = read_snippets(&kept);
summary.files_skipped = report.skipped;
summary.warnings = report.warnings.iter().chain(&read_warnings).cloned().collect();
summary.results_found = total as u64;
summary.results_scored = kept.len() as u64;
summary.results_skipped = (total - kept.len()) as u64;

if mode == Mode::Text {
    // CRITERION-85: the notice and the warnings go to the error stream.
    if let Some(notice) = report.skipped.notice() {
        let _ = writeln!(stderr, "{notice}");
    }
    for warning in &summary.warnings {
        let _ = writeln!(stderr, "{}", warning.message());
    }
}
```

Then:

1. No result: text mode returns 1 and writes nothing, as now. JSON mode
   sets `exit_code` to 1 and writes the document with no result
   (decision 20).
2. The cost-limit notice goes to the error stream only in text mode.
3. After `score` and `rank`: `no_good_answer = is_no_good_answer(&ranked)`,
   and `search_exit_code` gives the code, as now. Text mode writes
   `write_text` to the buffer, as now. JSON mode sets `no_good_answer` and
   `exit_code` in the summary, then writes `write_json(&mut buffer,
   &summary, &ranked)`.
4. Both modes write the buffer in one `write_all`, as now. All output stays
   in memory until then, so a failure writes no partial result (CV-2).

In JSON mode, `search` writes nothing on the error stream.

### The error path (C-cli)

`finish(result, mode, &mut summary)`:

- `Ok(code)`: exit with `code`.
- `Err(failure)` in text mode: write `failure.render()` on the error
  stream, as now.
- `Err(failure)` in JSON mode: set `summary.exit_code` to
  `failure.exit_code`, set `summary.error` to the failure, and write
  `write_json(stdout, &summary, &[])`. Exit with `failure.exit_code`.

A failure to write the document (decision 25): jag writes
`failure.render()` of `write_failure()` on the error stream, and it does
not write a second document. When the error is `BrokenPipe`, jag writes
nothing and exits with the code of the search (CV-4).

### Argument errors (C-cli)

clap gives no `Cli` when the command line is not valid, so the mode comes
from the raw arguments:

```rust
/// True when the command line holds `--json` before the first `--`. After
/// `--`, `--json` is a term or a path, not the option.
fn json_requested(args: impl IntoIterator<Item = OsString>) -> bool {
    args.into_iter()
        .skip(1)
        .take_while(|arg| arg != "--")
        .any(|arg| arg == "--json")
}
```

In `parse`, the arm for each other error kind becomes:

- `json_requested(std::env::args_os())` is true: write the error document
  with the cause `the command line is not valid.`, the fix
  `Run jag --help to see the usage.`, and `exit_code` 2, on standard
  output. Write nothing on the error stream.
- Else: write the one usage line on the error stream, as now.

Both exit with 2. Neither shows a value from the command line (decision
22). The arm for help and version does not change (decision 24).

Examples:

- `jag --json --setup`, `jag --json --forget-key`, and `jag --json --check`:
  a conflict, so the error document, exit 2 (decision 23).
- `jag --json --timeout`: a missing value, so the error document, exit 2.
- `jag --bogus -- --json`: `--json` comes after `--`, so the usage line,
  exit 2.
- `jag --json` with no term: clap accepts it, and the search gives "a
  search needs a term and a question." as an error document, exit 2.

### Output

A search in text mode (no change):

```
src/net.rs-1-fn call() {
src/net.rs:2:    // retry one time after a timeout
src/net.rs-3-}
--
docs/notes.md:1:The client waits one second, then tries again.
```

The same search with `--json`. jag writes it on one line. It is shown
here on more lines so that it is easy to read:

```json
{"version":1,"prompt_version":"jag-p1","exit_code":0,"error":null,
 "no_good_answer":false,"results_found":2,"results_scored":2,
 "results_skipped":0,"left_out":0,"top_result_cut":0,"size_limit_fix":"",
 "files_skipped":{"total":1,"binary":0,"hidden":1,"ignored":0},
 "warnings":[],"warnings_left_out":0,
 "results":[
  {"rank":1,"path":"src/net.rs","first_line":1,"last_line":3,"lines":[
   {"number":1,"text":"fn call() {","is_match":false,"cut":0},
   {"number":2,"text":"    // retry one time after a timeout","is_match":true,"cut":0},
   {"number":3,"text":"}","is_match":false,"cut":0}]},
  {"rank":2,"path":"docs/notes.md","first_line":1,"last_line":1,"lines":[
   {"number":1,"text":"The client waits one second, then tries again.","is_match":true,"cut":0}]}]}
```

No match, with `--json`, exit 1:

```json
{"version":1,"prompt_version":"jag-p1","exit_code":1,"error":null,"no_good_answer":false,"results_found":0,"results_scored":0,"results_skipped":0,"left_out":0,"top_result_cut":0,"size_limit_fix":"","files_skipped":{"total":0,"binary":0,"hidden":0,"ignored":0},"warnings":[],"warnings_left_out":0,"results":[]}
```

HTTP 402 after a scan that found two results, with `--json`, exit 5:

```json
{"version":1,"prompt_version":"jag-p1","exit_code":5,"error":{"cause":"the TypeSafe account has no credit (HTTP 402).","fix":"Add credit at https://console.typesafe.ai, then try again."},"no_good_answer":false,"results_found":2,"results_scored":2,"results_skipped":0,"left_out":0,"top_result_cut":0,"size_limit_fix":"","files_skipped":{"total":0,"binary":0,"hidden":0,"ignored":0},"warnings":[],"warnings_left_out":0,"results":[]}
```

`jag --json --setup`, exit 2:

```json
{"version":1,"prompt_version":"jag-p1","exit_code":2,"error":{"cause":"the command line is not valid.","fix":"Run jag --help to see the usage."},"no_good_answer":false,"results_found":0,"results_scored":0,"results_skipped":0,"left_out":0,"top_result_cut":0,"size_limit_fix":"","files_skipped":{"total":0,"binary":0,"hidden":0,"ignored":0},"warnings":[],"warnings_left_out":0,"results":[]}
```

### Acceptance criteria and their tests

The end-to-end tests are in `tests/output.rs`. They use `jag` or
`jag_bare` of `tests/common/mod.rs`, a temporary `$HOME`, and the fake
server. They parse standard output with `serde_json`, and they check that
the output does not hold the key.

- CRITERION-44 (text is the default, also in a pipe):
  - End-to-end: a search with no option. `assert_cmd` gives jag a pipe as
    standard output. Standard output is exactly the text form of the two
    results, and it does not parse as JSON.
  - End-to-end: the same search with `JAG_JSON=1`, `JAG_OUTPUT=json`, and
    `JAG_FORMAT=json`. Standard output is the same text (decision 2).
- CRITERION-45 (text mode: results on standard output, each notice on the
  error stream):
  - End-to-end: the "notice folder": a hidden file, a binary file, a file
    with mode `0000`, and two results with `JAG_MAX_SCORED=1`, all with a
    score of 0.8, so that no label shows. Each line of
    standard output starts with the path of the folder, or is `--`. Each
    line of the error stream starts with `jag: `. The error stream holds
    the skip notice, the warning, and the cost-limit notice. A user who can
    read the file with mode `0000` (root) skips the test, as in
    tests/search.rs.
- CRITERION-46 (only the option turns on structured output):
  - End-to-end: with `--json`, standard output is exactly one line that
    ends with `\n`, and the line parses as one JSON object. The CRITERION-44
    tests show text with no option.
- CRITERION-47 (the same results, and a named field for each item):
  - End-to-end: the same folder and the same scores, one time in text mode
    and one time with `--json`. The test builds the text from the JSON
    fields with the table "Each item of the text and its field": for each
    result in order, for each line, the path, `:` or `-`, the number, `:`
    or `-`, the text, and ` [+N chars]` when `cut` is more than 0, with `--`
    between two results. The built text is equal to the text standard
    output. The folder has a line of more than 300 characters and lines
    around a match, so a cut mark and both separators show. The folder
    holds only UTF-8 text, so the text of each line is the same in the two
    forms.
  - End-to-end: a score of 0.2 gives the label line in text mode and
    `no_good_answer: true` with `--json`. A score of 0.8 gives no label and
    `no_good_answer: false`.
  - End-to-end: the notice folder with `--json`. `files_skipped` has the
    four numbers of the skip notice of the text run. `results_found`,
    `results_scored`, and `results_skipped` have the numbers of the
    cost-limit notice. The warning of the text run is one object of
    `warnings`: its `text` is the line with no `jag: `, and its `path` is
    the path of the file.
  - Unit: `write_json` gives `"path":null` for a result from standard
    input, U+FFFD in `text` for a line with the byte `0xff`, `"\r"` for a
    `\r` at the end of a line, and `first_line` and `last_line` equal to
    `Snippet::first_line()` and `Snippet::last_line()`.
- CRITERION-48 (a rank and no score):
  - End-to-end: three results with the scores 0.9, 0.7, and 0.5. `rank` is
    1, 2, and 3 in the list order, and the paths have the order of the text
    output. No object in the document has the key `score`, and each number
    in the document is a whole number.
- CRITERION-49 (a complete document with no result):
  - End-to-end: `--json` and a term that matches nothing. Exit 1. Standard
    output is one document with `results: []`, `exit_code: 1`,
    `error: null`, `results_found: 0`, and each field of the top-level
    list of CRITERION-52. The server gets no request, and the error stream
    is empty.
  - End-to-end: the same search with no `--json` writes nothing to
    standard output (no change).
- CRITERION-50 (each notice inside the structure):
  - End-to-end: the notice folder with `--json`. The error stream is empty.
    The CRITERION-47 test checks the values in the document.
- CRITERION-51 (errors are structured):
  - End-to-end, one test for each code, all with `--json`: 2 (the term
    `(`), 3 (no key: `jag_bare` with `JAG_BASE_URL` set to the fake
    server), 4 (HTTP 401), 5 (HTTP 402), and 6 (HTTP 500 two times). For
    each: standard output is one document, `exit_code` is the exit code of
    the process, `error.cause` and `error.fix` are the texts of the text
    mode for that cause with no `jag: `, `results` is `[]`, and the error
    stream is empty.
  - End-to-end: HTTP 500 two times with `JAG_MAX_SCORED=1` and two results.
    Exit 6. The document has `results_found: 2`, `results_scored: 1`,
    `results_skipped: 1`, and `results: []` (CV-2).
  - End-to-end (tests/usage.rs): `--json --setup`, `--json --forget-key`,
    `--json --check`, `--json --bogus`, `--json --timeout`, and
    `--json --setup` followed by the made-up key. Each exits 2. Standard output is one document with the
    cause `the command line is not valid.` and the fix
    `Run jag --help to see the usage.`. The error stream is empty. The
    output does not hold `bogus` or the made-up key.
  - End-to-end (tests/usage.rs): `--bogus -- --json` gives exit 2, the one
    usage line on the error stream, and nothing on standard output.
  - End-to-end (tests/usage.rs): `--json --help` and `--json --version`
    exit 0 with the help text and the version text, not JSON.
  - End-to-end, only on Linux: `--json` with standard output open on
    `/dev/full`. Exit 2, and the error stream is exactly the text of
    `write_failure()` (decision 25).
  - Unit: `json_requested` is true for `jag --json` and `jag x --json`, and
    false for `jag -- --json`, `jag --json=1`, and `jag`.
- CRITERION-52 (the form version, and no field removed or renamed):
  - Unit: two documents from `write_json`: one with a result, a line, and a
    warning, and one with an error. The test collects the field names at
    each level and compares them with this fixed list. A comment says: add
    the name of each new field to the list; never remove or rename a name
    in one major version of jag (CONSTRAINT-05).

    ```
    top level   version, prompt_version, exit_code, error, no_good_answer,
                results_found, results_scored, results_skipped, left_out,
                top_result_cut, size_limit_fix, files_skipped, warnings,
                warnings_left_out, results
    error       cause, fix
    files_skipped  total, binary, hidden, ignored
    warning     path, text
    result      rank, path, first_line, last_line, lines
    line        number, text, is_match, cut
    ```

  - Unit: `version` is the whole number 1, and `JSON_VERSION` is 1.
  - Unit: the document with no result, no warning, no error, and
    `exit_code` 255 has less than 1,000 characters (decision 27). Each
    count is `u64::MAX`, also `top_result_cut` and `warnings_left_out`.
    The three parts of `files_skipped` are `u64::MAX / 3`, so that their
    total stays a `u64`. `size_limit_fix` is a text of 120 characters,
    longer than the FEATURE-09 fix text with a limit of 20 digits (95
    characters).
  - End-to-end: the document of a search has `version: 1` and
    `prompt_version: "jag-p1"`.

## Build order

1. STORY-29: the end-to-end tests of CRITERION-44 and CRITERION-45. They
   pass on the code of today, and they must pass after each step.
2. C-search: `WalkWarning::text`, and `message` uses it. The unit tests of
   the messages do not change.
3. C-output: `JSON_VERSION`, `Summary`, `is_no_good_answer`, `write_json`,
   the private types, and their unit tests.
4. STORY-30: `--json` with its conflicts, `Mode`, the usage line, and the
   help text. The search flow fills `Summary` and writes the document.
   Then the tests of CRITERION-46, CRITERION-47, and CRITERION-48.
5. STORY-31: the document with no result, the error document in `finish`,
   `json_requested` and the argument error document, and the rule for a
   failure to write. Then the tests of CRITERION-49, CRITERION-50, and
   CRITERION-51.
6. STORY-32: the tests of CRITERION-52. Then the README section "Output
   for an agent": the option, the one-line example, the field tables, the
   rule that `version` changes only on a removed or renamed field, and the
   rule that agent code must accept a field that it does not know.
7. The size limit of the document (decision 31): only when the FEATURE-09
   code is on `main` before this code merges. Else the FEATURE-09 code
   does it, with the tests of its "Structured mode".

`cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` must
pass on the CI runs of `ubuntu-latest` and `macos-latest`.

## Assumptions

- Agent code accepts a field that it does not know. From CONSTRAINT-05: a
  new version can add a field, so agent code that reads the document must
  accept it. The README says so.
- `--json` is the usual name of this option (decision 1). From the
  ripgrep docs (`rg --json`) and the GitHub CLI docs (`gh --json`).
  ripgrep writes JSON Lines, one object for each event. jag writes one
  document (ADR-0008).
- Field names are words in lower case, joined with `_` (decision 5). From
  system-design.md IF-5 (`prompt_version`).
- The mark on a matching line is `is_match` (decision 11). From
  src/search/snippet.rs (`SnippetLine.is_match`).
- `path` is `null` for standard input (decision 13). From
  src/search/snippet.rs (`Source::StandardInput` has no path) and the
  FEATURE-08 spec decision 4.
- No text in the document starts with `jag: ` (decision 19). From
  src/failure.rs (`render` adds `jag: `, and `cause` does not hold it).
- A failure to write the document goes to the error stream as text
  (decision 25). From src/cli.rs (`write_failure`) and system-design.md
  CV-4.
- The README gets a section for the option (decision 29). From the
  FEATURE-02 spec (a README section for each command).
- clap shows the help and the version when it meets the flag, before it
  examines a conflict (decision 24). From src/cli.rs (`parse`) and the
  clap docs (`ArgAction::Help`).
- `results_scored` is K of the cost-limit notice, also when a file cannot
  be read after the scan. Then the document has fewer results than K, and
  a warning names the file. When no kept file can be read, text mode
  writes no cost-limit notice, but the document still has the three
  numbers. From src/cli.rs (the notice uses `kept.len()`).
- A write to `/dev/full` fails on Linux. From the Linux docs of
  `/dev/full`. macOS has no such file, so that test runs only on Linux.

## Risks

- A path that is not UTF-8 shows U+FFFD in `path`, so an agent cannot open
  that file. Signal: an agent reports that a path from jag does not exist.
  Mitigation: a later version adds a field with the bytes of the path.
  Adding a field is allowed (CONSTRAINT-05).
- Many files that jag cannot read make a long `warnings` list. Signal: a
  large document with few results. Mitigation: with the size limit, the
  results fit first, and `warnings_left_out` counts the warnings that do
  not fit (decision 31).
- A user of ripgrep expects one JSON object for each match from `--json`.
  Signal: a question or an issue about the form. Mitigation: the help text
  and the README say "one JSON document". A reader of JSON Lines still
  reads the document, because it is on one line.
- `json_requested` reads a value of a later option that is exactly
  `--json` as the option. Then an argument error gives the document, not
  the usage line. Signal: a test of that later feature. Mitigation: that
  feature makes `json_requested` skip the value of each option that takes
  one.
- An agent depends on the order of the fields. Signal: an agent breaks
  when a later version adds a field. Mitigation: the README says that the
  order can change, and a new field goes before `results` (decision 5).

## Out of scope

- The size limit, the values of the 4 size-limit fields (decision 16),
  exit 8, the size-limit notice, and the cut of the top result:
  FEATURE-09. In structured mode, the limit counts the whole document from
  `write_json` (decision 27). The code that merges second, FEATURE-09 or
  FEATURE-11, connects the two parts: the fit of the results and the
  warnings, and the shorter paths in an error document (decision 31).
- The cost of a search as a field (DECISION-41): FEATURE-20.
- Progress in a terminal (DECISION-37): a later feature. Structured mode
  shows no progress.
- Ctrl-C, its document, and exit 130: FEATURE-15.
- The read of standard input, and the end-to-end test of `path: null`:
  FEATURE-14.
- `-l`, `-c`, `-q`, and the other grep options of DECISION-34 with
  `--json`: the features that add them. Each one gives its structured form
  or refuses `--json` as an argument error (decision 23).
- The document of a search with no question (plain grep): FEATURE-12.
- A structured form of `--check`: a later change. This feature adds the
  conflict of `--json` with `--check` (decision 23).
- An AI gateway: FEATURE-21.
- The bytes of a path that is not UTF-8: a later added field (Risks).
