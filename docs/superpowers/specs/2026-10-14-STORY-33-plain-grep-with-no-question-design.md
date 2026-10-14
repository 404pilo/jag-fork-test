---
title: "Plain grep with no question: the matching lines as grep gives them, with no model call, no cost, and no key"
feature: FEATURE-12
stories: [STORY-33, STORY-34]
kind: build
gh_issue: 94
lifecycle_state: draft
inherits: [IF-3, IF-4, IF-5, IF-6, C-cli, C-search, C-output, CV-1, CV-2, CV-3, CV-4, CV-6, CV-7]
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Plain grep with no question: the matching lines as grep gives them, with no model call, no cost, and no key

## Problem

jag must feel like grep (CAPABILITY-06). A user who types a term and no
question wants the matching lines, as grep gives them. A model call costs
money and time, and it needs a key. A plain search must need neither.

- STORY-33: a shell power user wants jag with a term and no question to
  return the matching lines as grep does, with no model call, so that a
  plain search costs nothing (DECISION-32).
  - CRITERION-53: with a term and no question, jag returns the matching
    lines as grep does.
  - CRITERION-54: with no question, jag makes no model call and the search
    costs nothing.
- STORY-34: a shell power user wants plain grep mode to work with no stored
  key, so that they can use jag before they set it up (DECISION-49).
  - CRITERION-68: with no stored key, `jag TERM PATH`, with no `--ask`,
    returns the grep matches and does not ask for a key.

The user answered on 2026-10-14: the question comes only from the option
`--ask QUESTION` (DECISION-31, ADR-0018). The command form is `jag
[OPTIONS] TERM [PATH...]`, and each word after the term is a path. With
`--ask`, the model ranks the matches. With no `--ask`, jag works as plain
grep (DECISION-32). In this spec, "no question" means no `--ask`, and "a
question search" means a search with `--ask`.

Today each search needs a question, and the question is the word after the
term (FEATURE-04 spec decision 19). `jag TERM` gives the usage error `a
search needs a question.` and exit 2. `jag TERM PATH` takes PATH as the
question, and then gives `a search needs a path.` and exit 2.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | The question and the paths | The question comes only from the option `--ask QUESTION`. In `jag [OPTIONS] TERM [PATH...]`, the first word that is not an option is the term, and each word after it is a path. With `--ask`, the search is a question search. With no `--ask`, it is plain grep. Only the value of `--ask` is a question, so no file or folder on the disk changes what a command does | human:404pilo, 2026-10-14 (hitl answer FEATURE-12 "A question or a path after the term": the user replaced both options); 0018-take-the-question-only-from-the-ask-option.md (accepted); DECISION-31; DECISION-32; CRITERION-68 (`jag TERM PATH`) |
| 2 | The value of `--ask` | `--ask QUESTION` or `--ask=QUESTION`. No short form. The option can come before the term, after the term, or after the paths, as each option of jag. A value that starts with `-` needs `=`, for example `--ask=-v`. With a space, clap reads that value as an option and gives an argument error. Only `-` alone works in both forms. So `jag --ask --json TERM PATH` is an argument error, never a question search. `--ask` two times, and `--ask` with no value, are argument errors. After `--`, `--ask` is a term or a path. `--ask` does not work with `--check`, `--setup`, or `--forget-key`. Each argument error gives the failure of `argument_failure`, `the command line is not valid.` and `Run jag --help to see the usage.`, exit 2, or its document when `--json` is on the command line, as today | 0018-take-the-question-only-from-the-ask-option.md (accepted: `--ask=QUESTION`, no short form); clap 4.6.7, checked with a copy of the `Cli` of this spec in `rust:1.99` on 2026-10-14; `src/cli.rs` (`parse`, `json_requested`, `argument_failure`) (inferred) |
| 3 | A question that is not UTF-8 | `Cli.ask` is `Option<String>`, as `question` is today. Each value of `--ask` is a question, and the model gets UTF-8 text. clap refuses a value that is not UTF-8 (`ErrorKind::InvalidUtf8`), so `parse` gives the failure of `argument_failure`, exit 2, as for each argument error today. This needs no new code. A path that is not UTF-8 still works, because `paths` stays `Vec<PathBuf>` | `src/cli.rs` (`question: Option<String>`, `parse`); clap 4.6.7, checked in `rust:1.99` on 2026-10-14 (inferred) |
| 4 | What plain grep does not do | No model call. No key lookup: jag does not call `find_key`, so it reads no key variable and does not open the key file. No model setting: jag does not read `JAG_BASE_URL`, `JAG_MODEL`, `JAG_TIMEOUT`, `JAG_RETRY_WAIT`, or `JAG_MAX_SCORED`. So a key file that other users can read does not stop plain grep. A question search and `--check` still refuse that file | DECISION-32; DECISION-49; CRITERION-54; CRITERION-68; system-design.md CV-6 (the refusal is a rule of the key read) and CV-7 (inferred) |
| 5 | The options of the model | `--timeout`, `--retry-wait`, and `--max-scored` keep their checks in step 1. A bad value gives exit 2 with the message of today. A good value has no effect | FEATURE-03 spec decision 11 and FEATURE-06 spec decision 6 (the options are checked in step 1, before the key); `src/cli.rs` (`search_args`) (inferred) |
| 6 | A result | One matching line, with no line around it. Each matching line is one result | CRITERION-53 ("the matching lines as grep does"); DECISION-32; CRITERION-69 (the lines around a match are for "a question search"); CRITERION-55 (`-A`, `-B`, and `-C` "with their grep meaning") (inferred) |
| 7 | Text form | `path:N:text` for each line. This is the form of a matching line of FEATURE-08: the path on each line, as `grep -H`, and the line number, as `grep -n`. No `--` between lines, and no label line. `path` is the path as the walk gives it. `text` is the bytes of the line as the file holds them, also a `\r` at the end | FEATURE-08 spec decisions 1 and 5 to 8; GNU grep 3.12 (`/usr/bin/grep`), checked on 2026-10-14: with no context option, grep writes no `--`, and it keeps a `\r` |
| 8 | Long lines | jag cuts a line of more than 300 characters and adds ` [+N chars]`, as in a question search. This is not the same as grep, which does not cut. DECISION-17 wins, because it is the rule for each result. No option turns off the cut (Risk 10) | DECISION-17 ("A result shows the exact source text. When a line is longer than a fixed length, jag cuts it and marks the cut"); DECISION-20 (a default size limit always applies); FEATURE-08 spec decisions 2 and 3, and its "Out of scope" ("Whether plain grep with no question cuts long lines: FEATURE-12") (inferred) |
| 9 | How jag gets the lines | The scan of FEATURE-05, then `read_snippets` with one window for each matching line (`line_windows`, new in C-search). So a line has the same bytes, the same cut, and the same text of a file with a byte order mark as in a question search. The scan does not change: its line text keeps no `\n` and no `\r`. jag reads each file with a match two times, as a question search does | `src/search/snippet.rs` (`read_snippets`, `decode`, `SnippetLine::new`); `src/search/walk.rs` (`matched` removes `\n` and `\r`); FEATURE-08 spec decision 8 (inferred) |
| 10 | Order | The order of the scan of FEATURE-14 spec decision 6: the paths in the order of the command line, and inside one path, the path in byte order, then the line number. So `jag needle b.txt a.txt` writes the lines of `b.txt` first, as grep does. `scan` keeps one walk for all paths and gives each file and each warning the place of its path on the command line, as FEATURE-14 spec decision 6 and its Design say. The code that merges first, of FEATURE-12 or of FEATURE-14, gives `scan` this order and adds the unit tests of FEATURE-14 decision 6 | CRITERION-53 ("the matching lines as grep does"); CAPABILITY-06 and EPIC-04 ("a shell user keeps their habits"); FEATURE-05 spec decision 11 and its reason (it asks only for a fixed order: "must sort them or say that the order is not fixed"); FEATURE-14 spec decision 6 (PR #142, head 4b1ca3f: one walk for all paths); FEATURE-16 spec decision 12 (PR #155: a walk for each path makes 1,000 file paths take 1.44 to 2.33 s, against 4.5 ms in one walk); the review of PR #151 (verifier-1, 2026-10-14) |
| 11 | Output in memory | jag keeps all output in memory and writes it in one call at the end, as a question search does. It writes no line before the search ends | system-design.md CV-2 and CV-4; `src/cli.rs` ("All output stays in memory until here", `emit`) (inferred) |
| 12 | Ctrl-C and a closed pipe | The same as in a question search. Plain grep is a search, so the stop thread and the watch thread of FEATURE-15 run. Ctrl-C writes `jag: the search was stopped.` and its fix in text mode, or the stop document with `exit_code` 130 with `--json`. Then it ends jag by SIGINT. A closed pipe ends jag by SIGPIPE with no message | FEATURE-15 spec decisions 11 to 15 (decision 11: "Only in the search flow"); `src/cli.rs` (`run` starts both threads before `search`) |
| 13 | Exit codes | 0 when jag writes one line or more, 1 when no line matches, and 2 for a usage error, also for a term that is not a valid regular expression. No code from 3 to 7. A warning, for example a file that jag cannot read, does not change the code, as in a question search | system-design.md IF-4; CRITERION-58; FEATURE-05 spec decision 6; FEATURE-04 spec (the error of the term) |
| 14 | The document with `--json` | The document of FEATURE-11, with the same fields. `prompt_version` stays. `no_good_answer` is `false`. `results_found` is the count of matching lines. `results_scored` is 0, because the model scores no result. `results_skipped` is 0, because the cost limit leaves out no result. `results` holds the lines in the order of decision 10, with one line in each result. `rank` is the place in that order. jag adds no field | FEATURE-11 spec decisions 5 to 7 and 14 ("When the notice does not show, K is N and S is 0": with no question, the model scores nothing, so K is 0); system-design.md IF-5; CONSTRAINT-05 (inferred) |
| 15 | Code of the document | `write_json(out, summary, results: &[&Snippet])` in place of `ranked: &[(Snippet, f64)]`. A question search gives the snippets of `ranked`, in rank order. Plain grep gives its lines. No score goes in, because the document has no score | `src/output.rs` (`write_json` never writes the score) (inferred) |
| 16 | Code of the text | C-output gets `write_lines(out, results: &[Snippet], paint: Option<Paint>)`. It writes each line with the private function `write_line`, which `write_text` also uses for each line. The bytes of `write_text` do not change. Plain grep does not use `write_text`, because `write_text` writes `--` between results and can write the label line | `src/output.rs` (`write_text`); FEATURE-15 spec "Other features" ("it gets the same color") (inferred) |
| 17 | Color | The same rule and the same colors as a matching line of a question search: `--color`, `color_on`, and the `paint` function of C-cli | FEATURE-15 spec decisions 5 to 8 and 10 |
| 18 | Skips, notices, and warnings | The same scan, skip rules, skip notice, and warnings as a question search. In text mode, they go to the error stream through `Stop::notice`. With `--json`, they are fields of the document | DECISION-14; FEATURE-05 spec decisions 2 to 7; CRITERION-85; system-design.md CV-3; FEATURE-15 spec decision 14 |
| 19 | Progress and the cost-limit notice | Plain grep shows no progress line and no cost-limit notice, because the model scores nothing | DECISION-37; FEATURE-15 spec "Other features" ("It makes no model call, so it shows no progress line"); FEATURE-06 spec (the notice shows only when the model scores fewer results than jag found) |
| 20 | Usage errors of step 1 | No term: `a search needs a term.`, with the fix `Use: jag [OPTIONS] TERM PATH...`. No path: `a search needs a path.` and `Searching standard input comes later.`, as today. A question keeps its checks: `the question is blank.` and `the question is longer than 12,000 bytes.`. The order: the term, the paths, the checks of the question, and then the options of the model. No message shows a value | `src/cli.rs` (`search_args`); FEATURE-04 spec decisions 26 and 30; system-design.md CV-1 and CV-6 |
| 21 | FEATURE-14 | FEATURE-14 (PR #142, not merged) makes the paths optional and reads a pipe. Its two points, an empty pipe and a path that does not exist, apply to plain grep as the FEATURE-14 spec decides them. This spec decides neither. A command in the old form, `jag TERM "QUESTION" PATH`, names a path that does not exist (Risk 1). So the rule of FEATURE-14 for that path applies to it | Orchestrator TASK and ACK (2026-10-14); FEATURE-14 spec (PR #142) decisions 1, 3, 7, and 9 |
| 22 | Help and usage | The usage lines `jag [OPTIONS] TERM [PATH...]` and `jag [OPTIONS] --ask QUESTION TERM [PATH...]`, then the lines of `--check`, `--setup`, and `--forget-key` of today. The help of `--ask`: `What you look for: the model ranks the matches by this question. With no --ask, jag works as grep: no model call, no cost, and no key`. The README gets the section "Plain grep with no question", and its example for an agent gets `--ask`. The section "Output for an agent" gets the values of decision 14 | 0018-take-the-question-only-from-the-ask-option.md (its Consequences: the help, the README, and the usage line show `jag [OPTIONS] TERM [PATH...]` and `--ask QUESTION`); `src/cli.rs` (`override_usage`, the help of each argument); FEATURE-11 and FEATURE-15 specs (a README section for each feature) (inferred) |
| 23 | Code place and crates | C-cli: the option `--ask`, the flow with no question, and the help. C-search: `line_windows`. C-output: `write_lines`, `write_line`, and the new parameter of `write_json`. No new crate | 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md; 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md |
| 24 | ADR | ADR-0018 (`0018-take-the-question-only-from-the-ask-option.md`, `status: accepted`, user answer 2026-10-14) records decision 1, and the two forms and the missing short form of decision 2. The command form is a contract of the command line (IF-3): after the first release, a change of it needs a new major version and a new ADR | The architect, 2026-10-14 (commit a2684f9); human:404pilo, 2026-10-14 (hitl answer FEATURE-12); docs/adr/CLAUDE.md; system-design.md IF-3 |
| 25 | Spec grouping | One spec for STORY-33 and STORY-34. STORY-34 is the path with no key of the same flow, and its tests need the flow of STORY-33 | docs/adlc.md (one feature gives one spec) |
| 26 | Tests | The new file `tests/plain_grep.rs` holds the end-to-end tests of the three criteria and of decisions 1 and 2. Unit tests go in `src/search/snippet.rs` and `src/output.rs`. Each test that gives the question as the word after the term changes to `--ask` (see "Tests that change"). One test compares the lines of jag with the lines of the `grep` program of the system | FEATURE-11 and FEATURE-15 specs (a temporary `$HOME`, the fake server, one test file for each feature); `tests/common/mod.rs` (`jag_bare`, `jag`, `write`) (inferred) |

## Design

### Files

- `src/cli.rs` (C-cli): the option `--ask` (`Cli.ask`) takes the place of
  the argument QUESTION. `search_args` gives `question: Option<&str>`. The
  new function `plain` runs the flow with no question. The usage lines, the
  help of `--ask`, and the fix of a search with no term change.
- `src/search/snippet.rs` (C-search): the new function `line_windows`.
- `src/output.rs` (C-output): the private function `write_line`, the new
  function `write_lines`, and `write_json` with `results: &[&Snippet]`.
- `tests/plain_grep.rs` (new): the end-to-end tests.
- The tests that give the question as the word after the term: see "Tests
  that change".
- `README.md`: the new section "Plain grep with no question", and the
  section "Output for an agent". Its example on line 96, `$ jag --json
  TERM "QUESTION" PATH...`, becomes `$ jag --json --ask "QUESTION" TERM
  PATH...`.

### The option --ask (C-cli)

The field takes the place of `question`:

```rust
/// What you look for: the model ranks the matches by this question. With no
/// --ask, jag works as grep: no model call, no cost, and no key
#[arg(long, value_name = "QUESTION")]
ask: Option<String>,
```

The id of the argument changes from `question` to `ask`. The
`conflicts_with_all` lists of `--check`, `--setup`, and `--forget-key` name
`ask` in place of `question`. So clap refuses `--ask` with each of these
three options (decision 2).

The usage (decision 22):

```rust
override_usage = "jag [OPTIONS] TERM [PATH...]\n       jag [OPTIONS] --ask QUESTION TERM [PATH...]\n       jag --check [--timeout SECONDS] [--retry-wait SECONDS]\n       jag --setup\n       jag --forget-key"
```

The help lists each option with its text, as today.

What clap 4.6.7 does with this `Cli` (a copy of the struct, run in
`rust:1.99` on 2026-10-14). An argument error gives the fixed line, or the
error document when `--json` is on the command line before `--` (decision
2):

- `--ask "how long" needle dir` and `--ask="how long" needle dir`: the
  question `how long`, the term `needle`, and the path `dir`. `needle
  --ask "how long" dir` and `needle dir --ask "how long"` give the same.
- `--ask=-x needle dir`: the question `-x`. `--ask - needle dir`: the
  question `-`.
- `--ask -x needle dir`, `--ask "-why is it slow" needle dir`, `--ask
  --json needle dir`, `--ask -- needle dir`, and `needle dir --ask`: an
  argument error. clap does not take the next word as the value, because
  it starts with `-`, or because there is no next word.
- `--ask a --ask b needle dir`: an argument error.
- `--ask "" needle dir` and `--ask= needle dir`: an empty question.
  `search_args` gives `the question is blank.`.
- `-- --ask dir`: the term `--ask`, the path `dir`, and no question.
  `needle dir -- --ask`: the paths `dir` and `--ask`.
- `--check --ask q`, `--setup --ask q`, and `--forget-key --ask q`: an
  argument error.
- `--as q needle dir`: an argument error. clap does not take a part of the
  name of a long option.
- A value that is not UTF-8, in both forms: an argument error
  (`InvalidUtf8`).
- `--ask q` with no term: no argument error. `search_args` gives `a search
  needs a term.`. `--ask q needle` with no path: `search_args` gives `a
  search needs a path.`.

`SearchArgs` gets `question: Option<&'a str>`. The other fields do not
change:

```rust
struct SearchArgs<'a> {
    term: &'a str,
    /// `None` for a search with no question (FEATURE-12).
    question: Option<&'a str>,
    paths: &'a [PathBuf],
    times: (Option<Duration>, Option<Duration>),
    cost_limit: Option<usize>,
}
```

`search_args` does these steps, in this order (decision 20):

1. No term: the failure `a search needs a term.`, with the fix `Use: jag
   [OPTIONS] TERM PATH...`, exit 2.
2. No path: `a search needs a path.` and `Searching standard input comes
   later.`, exit 2, as today.
3. A question (`cli.ask`): the checks of today, blank and longer than
   12,000 bytes.
4. The options of the model, as today: `--timeout`, then `--retry-wait`,
   then `--max-scored`.

`search` then chooses the flow:

```rust
let args = search_args(cli)?;
let Some(question) = args.question else {
    return plain(cli, &args, env, context, summary, stdout, stderr);
};
// The key comes before any file read (decision 18).
let (key, _source) = find_key(env, &RealFs).map_err(Failure::from)?;
```

The question flow does not change. It uses `question` and `args.paths`.

### The flow with no question (C-cli)

`plain` does these steps:

1. It makes the matcher, as the question flow does. A term that is not a
   valid regular expression gives `the term is not a valid regular
   expression.`, exit 2.
2. `scan(args.paths, &matcher, &ScanOptions::default())`, with the order
   of decision 10.
3. `line_windows(&report.files)`, then `read_snippets` of those windows.
   Each snippet holds one line.
4. The counts: `files_skipped`, `warnings` (the warnings of the scan, then
   the warnings of the read), and `results_found`, the count of windows.
   `results_scored` and `results_skipped` stay 0 (decision 14). Then
   `stop.count(summary)`.
5. In text mode: the skip notice and each warning, through `stop.notice`.
6. No line: in text mode, exit 1. With `--json`, the document with
   `exit_code` 1 and no result, through `emit_claimed`.
7. In text mode: `write_lines(&mut buffer, &lines, paint)`, with the `paint`
   function of FEATURE-15 when `color_on` gives true. With `--json`:
   `summary.exit_code = 0`, then `write_json(&mut buffer, summary, &refs)`,
   with `refs` a `Vec<&Snippet>` of the lines.
8. `emit_claimed(stop, stdout, stderr, &buffer, 0)`.

`plain` does not call `find_key`, `model_settings`, or `cost_limit`, and it
does not call `progress.start` (decisions 4 and 19).

The two flows share steps 1, 4, and 5 and the `paint` function. The code
moves these parts out of `search` into private functions of `src/cli.rs`,
with no change of behavior. So the code of the two flows has no copy.

### One window for each matching line (C-search)

```rust
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
```

`read_snippets` reads each file of these windows one time, because the
windows of one file are next to each other. `snippet_of` gives one line for
each window, with `is_match` true.

### The text (C-output)

The body of the inner loop of `write_text` moves into a private function:

```rust
/// Write one line of a result: `path:N:text` for a matching line, and
/// `path-N-text` for another line. A cut line ends with ` [+N chars]`.
fn write_line(
    out: &mut dyn Write,
    label: &[u8],
    line: &SnippetLine,
    paint: Option<Paint>,
) -> io::Result<()>
```

`write_text` calls it for each line. Its bytes do not change, and its unit
tests do not change.

```rust
/// Write the lines of a search with no question (FEATURE-12): one
/// `path:N:text` line for each line of `results`, in that order. No `--` and
/// no label line, as grep with no context option.
pub fn write_lines(
    out: &mut dyn Write,
    results: &[Snippet],
    paint: Option<Paint>,
) -> io::Result<()> {
    for snippet in results {
        let label = snippet.source.label();
        for line in &snippet.lines {
            write_line(out, label, line, paint)?;
        }
    }
    Ok(())
}
```

Example: `jag needle dir` on `dir/a.txt` (`needle one`, `other`, `needle
two`) and `dir/sub/b.txt` (`x needle`) writes:

```
dir/a.txt:1:needle one
dir/a.txt:3:needle two
dir/sub/b.txt:1:x needle
```

### The document (C-output)

`write_json` gets `results: &[&Snippet]`:

```rust
pub fn write_json(
    out: &mut dyn Write,
    summary: &Summary,
    results: &[&Snippet],
) -> io::Result<()>
```

`rank` is the index in `results` + 1, as today. The question flow gives
`ranked.iter().map(|(snippet, _)| snippet).collect::<Vec<_>>()`. The error
document and the stop document give `&[]`, as today.

The document of `jag --json needle dir` for the example above, on one line
(here on more lines, so that it is easy to read):

```json
{"version":1,"prompt_version":"jag-p1","exit_code":0,"error":null,
 "no_good_answer":false,"results_found":3,"results_scored":0,
 "results_skipped":0,"left_out":0,"top_result_cut":0,"size_limit_fix":"",
 "files_skipped":{"total":0,"binary":0,"hidden":0,"ignored":0},
 "warnings":[],"warnings_left_out":0,
 "results":[
  {"rank":1,"path":"dir/a.txt","first_line":1,"last_line":1,
   "lines":[{"number":1,"text":"needle one","is_match":true,"cut":0}]},
  {"rank":2,"path":"dir/a.txt","first_line":3,"last_line":3,
   "lines":[{"number":3,"text":"needle two","is_match":true,"cut":0}]},
  {"rank":3,"path":"dir/sub/b.txt","first_line":1,"last_line":1,
   "lines":[{"number":1,"text":"x needle","is_match":true,"cut":0}]}]}
```

### Other features

- FEATURE-04 (done): this PR changes its criterion CRITERION-18 to the form
  of ADR-0018, and the pages DECISION-31, DECISION-32, CRITERION-68, and
  FEATURE-04 to the user answer of 2026-10-14. The FEATURE-12 code keeps
  CRITERION-18 true: its test changes to `--ask` (see "Tests that change").
- FEATURE-14 (spec PR #142, not merged): it makes the paths optional, reads
  a pipe, and gives `Source::StandardInput`. Plain grep uses its input
  rules. Its two points, an empty pipe and a path that does not exist,
  apply to plain grep as FEATURE-14 decides them (decision 21). The code
  that merges second joins the usage texts: `Use: jag [OPTIONS] TERM
  [PATH...]`, and the flow with no question takes the check order of
  FEATURE-14 decision 9 with no key and no settings. The code that merges
  second also changes the test `a_term_with_no_path_exits_2_and_makes_no_call`
  and the test of the old form (see the tests below). With `--ask`, each
  word after the term is a path, so a mistyped path, or a question in the
  old form, is a path that does not exist. The rule of FEATURE-14 for that
  path applies to it.
  The lines come in the order of the scan of FEATURE-14 decision 6
  (decision 10), with one walk for all paths. The code that merges first
  changes `scan`. The code that merges second keeps that change and its
  tests.
- FEATURE-09 (spec PR #134, not merged): the default size limit applies to
  plain grep too (DECISION-20). The FEATURE-09 spec says that plain grep
  uses `write_text`. With this spec, it uses `write_lines` (decision 16).
  The code that merges second gives `write_lines` the limit: each line counts
  as one result, with no `--`, and the exit code 8 comes when the limit
  leaves out lines (`search_exit_code` with `left_out`). The call
  `write_json(out, summary, &ranked[..k])` of FEATURE-09 becomes a call
  with the first k items of `results`.
- FEATURE-13 (the grep options of DECISION-34): `-i`, `-w`, `-F`, and `-e`
  change the matcher of both flows. `-A`, `-B`, and `-C` give lines around
  each match in plain grep, with their grep meaning (CRITERION-55), and
  then `--` between groups that are not next to each other, as GNU grep
  does, also with `-C0`. With `-e PATTERN`, the term is not a word of the
  command line, so each word is a path. `-l`, `-c`, and `-q` give their
  grep output. The exit code of a search with a file
  that jag cannot read (grep gives 2) belongs to CRITERION-58. FEATURE-13
  decides the details of each option.
- FEATURE-20 (the cost): a search with no model call reports a cost of zero
  and the reason "no question" (DECISION-43, CRITERION-91). FEATURE-20 adds
  the cost line and the cost field. This spec writes neither.
- FEATURE-21 (spec PR #143, not merged): it changes the settings of the
  model service. Plain grep reads none of them (decision 4).
- FEATURE-16 (spec PR #155, not merged): plain grep reads each file with
  a match two times (decision 9). FEATURE-16 measured that second read and
  keeps it (its decision 14): it took 20 to 43 ms on Django 5.2 and on
  Go 1.25.0.
- The system design: IF-3 and the row contracts-1 show `jag TERM "QUESTION"
  [PATH...]`. The architect changes both to the form of ADR-0018 in the
  draft PR #157, which merges after this spec. This spec does not change
  `system-design.md`.

### The README section "Plain grep with no question"

The section comes before "Output for an agent":

- With no `--ask`, `jag TERM PATH...` gives the matching lines, as
  `grep -rnH TERM PATH...` does: one `path:N:text` line for each match.
  jag makes no model call, so the search costs nothing, and it needs no
  key.
- With `--ask "QUESTION"`, the model ranks the matches by the question, for
  example `jag --ask "how long is the wait" retry src`. Each word after the
  term is a path. A question that starts with `-` needs `=`, for example
  `jag --ask="-v or --verbose?" verbose src`.
- jag cuts a line of more than 300 characters and adds ` [+N chars]`.
  grep does not cut.
- The lines come in the order of the paths on the command line. Inside a
  folder, they come in the byte order of the path, then of the line number.
- jag skips binary, hidden, and ignored files, as in a question search.
- `--timeout`, `--retry-wait`, and `--max-scored` have no effect with no
  question. jag still checks their values.

In "Output for an agent", the example on line 96 becomes `$ jag --json
--ask "QUESTION" TERM PATH...`. The row of `results_found`, `results_scored`,
`results_skipped` says: "With no question, `results_found` is the count of
matching lines, and the other two are `0`". The text about `results` says:
"With no question, each result has one line, and the results come in the
order of the text output".

### Acceptance criteria and their tests

The end-to-end tests are in `tests/plain_grep.rs`. They use a temporary
`$HOME`, and `write` of `tests/common/mod.rs` makes the files. A test that
proves "no model call" uses `jag` with the fake server and checks that
`server.requests()` is empty. A test that proves "no key" uses `jag_bare`.

- CRITERION-53 (the matching lines as grep does):
  - End-to-end: the folder `dir` of the example of "The text". `jag
    needle dir`, run in the parent of `dir`, writes exactly the three lines
    of the example. The error stream is empty. Exit 0.
  - End-to-end, the same lines as grep: a folder with three text files.
    Each file has lines with and with no `needle`. No file is hidden,
    ignored, or binary, no line has more than 300 characters, and no line
    has a `\r`. In that folder, `jag needle .` and `grep -rnH needle .`
    (the `grep` program of the system) give the same set of lines. The
    test sorts both lists, because the walk of grep has no fixed order.
  - End-to-end: `jag needle dir/a.txt` writes `dir/a.txt:1:needle one` and
    `dir/a.txt:3:needle two`: the path shows also for one file.
  - End-to-end: a file `crlf.txt` with the bytes `needle\r\n`. `jag needle
    crlf.txt`, run in the folder of the file, gives the bytes
    `crlf.txt:1:needle\r\n` on standard output.
  - End-to-end: a line of `needle` and 394 `x` characters (400 characters)
    gives its first 300 characters and ` [+100 chars]`.
  - End-to-end: `jag absent dir` gives exit 1, an empty standard output,
    and an empty error stream.
  - End-to-end: `jag "(" dir` gives exit 2 and `not a valid regular
    expression` on the error stream.
  - End-to-end: a hidden file `.h.txt` with `needle` does not show. The
    error stream holds `jag: skipped 1 files and folders (0 binary, 1
    hidden, 0 ignored)`. Exit 0.
  - End-to-end: `jag needle b.txt a.txt` writes the lines of `b.txt` first,
    then the lines of `a.txt` (decision 10).
  - End-to-end: `jag --color=always needle a.txt` on `a.txt` with
    `needle one` writes the bytes
    `\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K\x1b[32m\x1b[K1\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K\x1b[01;31m\x1b[Kneedle\x1b[m\x1b[K one\n`.
  - Unit (`output.rs`): `write_lines` of three one-line snippets of
    `a.txt` (lines 1 and 3) and `b.txt` (line 2) gives `a.txt:1:x\n`,
    `a.txt:3:y\n`, and `b.txt:2:z\n`, with no `--` and no label line.
  - Unit (`output.rs`): `write_lines` of a snippet with
    `Source::StandardInput` gives `(standard input):1:x\n`. A cut line ends
    with ` [+N chars]`. With a `paint` that finds `x`, the bytes are the
    bytes of `write_text` for the same one-line result.
  - Unit (`output.rs`): the tests of `write_text` pass with no change.
  - Unit (`snippet.rs`): `line_windows` of `a` (lines 1 and 3) and `b`
    (line 2) gives the windows (`a`, 1, 1, [1]), (`a`, 3, 3, [3]), and
    (`b`, 2, 2, [2]). `read_snippets` of them on real files gives three
    snippets of one line, with `is_match` true, and a `\r` at the end of a
    line stays.
- CRITERION-54 (no model call, no cost):
  - End-to-end: the key in its variable and the fake server. `jag needle
    dir` gives the three lines and exit 0. `server.requests()` is empty.
  - End-to-end: the same with `--json`. The document has `exit_code` 0,
    `error` null, `no_good_answer` false, `results_found` 3,
    `results_scored` 0, `results_skipped` 0, and `prompt_version`
    `jag-p1`. `results` has three items with `rank` 1, 2, and 3, in the
    order of the example. Each item has `first_line` equal to `last_line`
    and one line with `is_match` true. The error stream is empty, and
    `server.requests()` is empty.
  - End-to-end: `--max-scored 1` with three matching lines gives the three
    lines, no cost-limit notice, and exit 0, not 7. `server.requests()` is
    empty.
  - End-to-end: `--timeout 60 --retry-wait 60` gives the three lines and
    exit 0. `--timeout abc` gives exit 2 and `the value of --timeout` on the
    error stream, with no key in the variables (decision 5).
  - Unit (`output.rs`): `write_json` of three one-line snippets and a
    `Summary` with `results_found` 3 gives the `results` of the example
    document.
  - The cost line and the cost field: FEATURE-20 (CRITERION-91). With no
    request, no call is billed.
- CRITERION-68 (no key, no question for a key):
  - End-to-end: `jag_bare`, so no key variable, and an empty `$HOME`. `jag
    needle dir` gives the three lines and exit 0. The error stream is
    empty. jag reads no input and does not ask for a key.
  - End-to-end: `$HOME/.config/jag/config.yml` with mode 644. `jag needle
    dir` gives the three lines and exit 0. The same `$HOME` with a question,
    `jag --ask "which line?" needle dir`, gives exit 3 and `can be read by
    other users` (CV-6).
  - End-to-end: with `jag_bare` and each of `JAG_BASE_URL=http://example.com/v1`,
    `JAG_TIMEOUT=abc`, `JAG_RETRY_WAIT=abc`, `JAG_MAX_SCORED=abc`, and
    `JAG_MODEL=` (empty), `jag needle dir` gives the three lines and exit 0
    (decision 4).
  - End-to-end: `jag_bare` and `--json`: the document has `exit_code` 0 and
    the three results.
- Decisions 1 and 2 (the question and the paths, and the value of
  `--ask`). The fixed line is `jag: the command line is not valid. Run jag
  --help to see the usage.`:
  - End-to-end: `jag --ask "which line?" needle dir`, with the key and the
    fake server, is a question search: one request with the question
    `which line?`, exit 0. `jag --ask="which line?" needle dir` and `jag
    needle dir --ask "which line?"` give the same.
  - End-to-end: the folder `my notes` with `a.txt` (`needle`). `jag needle
    "my notes"`, run in its parent, with the key and the fake server,
    writes `my notes/a.txt:1:needle`, exit 0. `server.requests()` is empty.
  - End-to-end, the old form: `jag needle "which line says retry" dir`, run
    in a folder with no file or folder of that name, with the key and the
    fake server. `server.requests()` is empty, and standard output has the
    three lines of `dir`. Until FEATURE-14 merges: exit 0, and `cannot read
    which line says retry` on the error stream (the warning of the walk).
    The code that merges second changes the test to the rule of FEATURE-14
    for a path that does not exist (decision 21).
  - End-to-end: `jag -- --ask dir` on `dir/a.txt` with the line `use --ask
    here` writes `dir/a.txt:1:use --ask here`, exit 0. `server.requests()`
    is empty.
  - End-to-end: `jag --ask=-v needle dir` is a question search with the
    question `-v`: one request. `jag --ask -v needle dir` gives exit 2, the
    fixed line on the error stream, and no request.
  - End-to-end: `jag --ask --json needle dir` gives exit 2 and no request.
    Standard output holds the error document with the cause `the command
    line is not valid.`, and the error stream is empty, because `--json` is
    on the command line.
  - End-to-end: `jag --ask a --ask b needle dir` and `jag needle dir --ask`
    give exit 2, the fixed line, and no request.
  - End-to-end: `jag --check --ask q`, `jag --setup --ask q`, and `jag
    --forget-key --ask q` give exit 2 and the fixed line.
  - End-to-end: `--ask`, then the bytes `how \xff` (not UTF-8), `needle`,
    and `dir`, give exit 2, the fixed line, and no request.
  - End-to-end: `jag --ask q` with no term gives exit 2 and `a search needs
    a term.`, and no request.

### Tests that change

The FEATURE-12 code changes each test that gives the question as the word
after the term. The call gets `--ask` before the question, then the term
and the paths: `["needle", QUESTION, "dir"]` becomes `["--ask", QUESTION,
"needle", "dir"]`. Each test then checks the same things as today. The list
comes from a search of `tests/` on main at 7a7a89e:

- `tests/cost_limit.rs`: the helper `search`.
- `tests/failure.rs`: the helper `search_at`,
  `no_key_gives_exit_3_and_the_setup_fix`, and
  `a_search_with_results_exits_0_no_match_exits_1_and_a_bad_term_exits_2`.
- `tests/output.rs`: the helper `run`,
  `criterion_51_an_invalid_term_gives_an_error_document_with_exit_2`,
  `criterion_51_no_key_gives_an_error_document_with_exit_3`, and
  `criterion_51_a_document_that_cannot_be_written_gives_the_text_failure_and_exit_2`.
- `tests/search.rs`: the helpers `search` and `search_from`,
  `criterion_71_with_no_key_and_no_match_the_exit_is_3`,
  `a_search_in_several_paths_scores_the_results_of_each`, and
  `an_https_address_that_closes_the_connection_gives_exit_6_and_no_panic`.
  The test `criterion_18_the_command_takes_a_term_a_question_and_a_path`
  uses the helper `search`. So it proves CRITERION-18 in its new form,
  `jag --ask "QUESTION" TERM PATH`.
- `tests/stop.rs`: the helper `start`.
- `tests/terminal.rs`: the helper `run`,
  `criterion_62_a_pipe_gets_no_progress_line`, and
  `criterion_62_structured_mode_gets_no_progress_line`.
- `tests/usage.rs`:
  `a_bad_regular_expression_exits_2_and_the_term_is_not_repeated`,
  `a_base_url_that_is_not_safe_exits_2_and_is_not_repeated`,
  `a_slash_at_the_end_of_the_base_url_is_removed`,
  `localhost_is_an_allowed_plain_http_host`, the helper `time_search`,
  `a_bad_time_option_comes_before_the_key_and_a_bad_variable_comes_after`,
  and `a_bad_limit_option_comes_before_the_key_and_a_bad_variable_after`.

These tests of `tests/usage.rs` change more:

- `a_term_with_no_question_exits_2_and_makes_no_call` becomes
  `a_term_with_no_path_exits_2_and_makes_no_call`: `jag needle` gives exit
  2 and `needs a path`, and makes no call. FEATURE-14 changes it again.
- `a_term_and_a_question_with_no_path_exit_2_and_make_no_call`: `jag --ask
  "which line?" needle` gives exit 2 and `needs a path`, and makes no call.
  With no `--ask`, `which line?` is a path. FEATURE-14 changes it again.
- `a_usage_error_comes_before_the_key_lookup`: `jag_bare` with `--ask`,
  `"   "`, `needle`, and a folder gives exit 2. A blank question stays a
  usage error with FEATURE-14.
- `a_blank_question_exits_2`: `--ask "   "`, then `needle` and the folder.
- `the_question_limit_is_12000_bytes`: only the form changes, with `--ask`
  before each question. `"a"` 12,000 times gives exit 0 and one request.
  `"a"` 12,001 times and `"漢"` 4,001 times give exit 2 and no request.
- `json_with_no_term_gives_an_error_document`: the cause is `a search
  needs a term.`.
- `help_and_version_exit_0`: the usage lines of decision 22.

The unit tests of `src/cli.rs` do not change. In `color_of` and in
`a_value_of_color_needs_an_equals_sign`, the word `question` becomes a
path, and the tests check only the color and the term.

## Build order

1. C-output: `write_line`, moved out of `write_text` with no change of
   bytes. Then `write_lines`, and `write_json` with `results: &[&Snippet]`.
   The question flow, the error document, and the stop document give their
   new arguments. All tests pass. Then the unit tests of `write_lines` and
   `write_json`.
2. C-search: `line_windows` and its unit tests. When the FEATURE-14 code
   is not on `main`: the order of decision 10 in `scan`, with the unit test
   of FEATURE-14 decision 6, with one walk for all paths.
3. STORY-33: the option `--ask` (`Cli.ask`), `search_args`, the private
   functions that the two flows share, and `plain`. Then the usage lines
   and the help. Then the tests of "Tests that change", and the tests of
   CRITERION-53, CRITERION-54, and decisions 1 and 2.
4. STORY-34: the tests of CRITERION-68. The flow of step 3 already reads no
   key, so this step adds only tests.
5. The README.

`cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` must
pass on the CI runs of `ubuntu-latest` and `macos-latest`.

## Assumptions

The inferred rows, with the file that each comes from:

- Decision 2 (the value of `--ask`): clap 4.6.7, and `src/cli.rs`
  (`parse`, `json_requested`, `argument_failure`).
- Decision 3 (a question that is not UTF-8): clap 4.6.7, and `src/cli.rs`
  (`question: Option<String>`, `parse`).
- Decision 4 (no key, no model setting): DECISION-32, DECISION-49, and
  `system-design.md` (CV-6, CV-7).
- Decision 5 (the options of the model): the FEATURE-03 and FEATURE-06
  specs, and `src/cli.rs` (`search_args`).
- Decision 6 (one line for each result): CRITERION-53, CRITERION-55, and
  CRITERION-69.
- Decision 8 (long lines): DECISION-17, DECISION-20, and the FEATURE-08
  spec.
- Decision 9 (how jag gets the lines): `src/search/snippet.rs` and
  `src/search/walk.rs`.
- Decision 11 (output in memory): `src/cli.rs` and `system-design.md`
  (CV-2).
- Decision 14 (the document): the FEATURE-11 spec (decision 14).
- Decision 15 (code of the document): `src/output.rs` (`write_json`).
- Decision 16 (code of the text): `src/output.rs` (`write_text`).
- Decision 22 (help and usage): `src/cli.rs` (`override_usage`).
- Decision 26 (tests): `tests/common/mod.rs` and the FEATURE-11 and
  FEATURE-15 specs.

The facts that the design uses:

- clap 4.6.7 parses the command lines of "The option --ask" as that list
  says. Source: a copy of the `Cli` of this spec, with `ask:
  Option<String>`, run with `Cli::try_parse_from` in `rust:1.99` on
  2026-10-14. The clap documentation (`/websites/rs_clap` in context7)
  agrees: an option that takes one value gives an error when it comes two
  times. Signal: an end-to-end test of decisions 1 and 2 fails.
- GNU grep 3.12 writes no `--` with no context option, writes `--` with
  `-C0` between groups that are not next to each other, and keeps a `\r`.
  Source: `/usr/bin/grep`, checked on 2026-10-14.
- The `grep` program of macOS writes `path:N:text` for `grep -rnH`, as GNU
  grep does, for a folder of text files. Not checked here. Signal: the
  test that compares with grep fails on the CI run of `macos-latest`
  (Risk 7).
- The scan finds the same matching lines as `grep -rn` for a folder of
  text files and a term with no special character. Source: the FEATURE-05
  spec (the libraries of ripgrep) and `tests/walk.rs`.
- A path that does not exist gives the warning `cannot read <path>: entity
  not found` of the walk. Source: `src/search/walk.rs` (`make_warning`
  takes the `io::ErrorKind`, and `WalkWarning::text`).
- The fake server counts each request. Source:
  `tests/common/fake_server.rs` (`requests`).

## Risks

1. A user types the old form, `jag TERM "QUESTION" PATH`, from habit. The
   question is then a path that does not exist (decision 1). Signal: a user
   report about a question that jag took as a path. Mitigation: jag makes
   no model call for it, so it costs nothing. The walk names the path in
   its warning, and FEATURE-14 names it in its message. The help and the
   README show `--ask`.
2. A question that starts with `-`, given with a space, for example `--ask
   -v`, gives an argument error (decision 2). The fixed line does not say
   why, because it shows no value. Signal: a user report about this error.
   Mitigation: the README says that such a question needs `=`.
3. Inside a folder, the lines come in the byte order of the path
   (decision 10). grep uses the order of the folder read, which is not
   fixed. Signal: a script or a test that compares the lines of jag with
   the lines of grep in their order. Mitigation: the README gives the
   order. The order is fixed, so two runs give the same output, and the
   test that compares with grep sorts both lists.
4. jag reads each file with a match two times (decision 9). Signal: on a
   large folder, plain grep takes more time than `rg` by more than the time
   of one read (METRIC-03). Mitigation: the system keeps the file in its
   cache after the first read. FEATURE-16 measured the second read (20
   to 43 ms on Django 5.2 and on Go 1.25.0) and keeps it (FEATURE-16 spec
   decision 14, PR #155).
5. All output stays in memory until the end (decision 11). A plain grep
   with a very large count of matching lines uses much memory, and the
   first line comes only at the end. Signal: a user report about memory or
   about a slow first line. Mitigation: the default size limit of FEATURE-09
   cuts the output.
6. A file that jag cannot read gives a warning and the exit code of the
   lines (decision 13). grep gives 2. Signal: a script that checks for the
   code 2. Mitigation: FEATURE-13 (CRITERION-58) owns the grep exit codes.
7. The test that compares with the `grep` program fails on macOS if its
   grep writes other text. Signal: the test fails on the CI run of
   `macos-latest`. Mitigation: the test compares only sorted `path:N:text`
   lines of text files with no special case.
8. The specs of other features that are not merged give their examples and
   tests in the old form: FEATURE-13 (PR #153), FEATURE-14 (PR #142), and
   FEATURE-16 (PR #155). Signal: a test of that code gives the question as
   the word after the term and fails, because the question is a path.
   Mitigation: those specs change to `--ask` after this spec (orchestrator
   ANSWER, 2026-10-14). The code that merges second uses `--ask`.
9. A merge conflict with the code of FEATURE-09, FEATURE-14, or FEATURE-21
   in `src/cli.rs`, `src/output.rs`, or `tests/usage.rs`. Signal: a
   conflict, or a test of the other feature that fails. Mitigation: the
   code that merges second joins the parts, as "Other features" says.
10. A script that reads the full line gets the first 300 characters and
    ` [+N chars]` (decision 8), and jag has no option that turns off the
    cut. Signal: a user report about a cut line in plain grep. Mitigation:
    the README says that jag cuts a long line and that grep does not.

## Out of scope

- The grep options `-i`, `-w`, `-F`, `-e`, `-A`, `-B`, `-C`, `-l`, `-c`,
  `-q`, `--include`, and `--exclude`: FEATURE-13.
- A search with no path, and the text of a pipe: FEATURE-14.
- The size limit and the exit code 8: FEATURE-09.
- The cost line and the cost field with the reason "no question":
  FEATURE-20.
- Lines that jag writes before the search ends, in the order of the walk:
  no story asks for it, and CV-2 keeps all output in memory.
- The forms of grep with no line number or with no path (`-h`): DECISION-34
  does not keep them.
- The text of IF-3 and contracts-1 in `system-design.md`: the architect
  (draft PR #157).
- A short form of `--ask`: ADR-0018 gives none.
