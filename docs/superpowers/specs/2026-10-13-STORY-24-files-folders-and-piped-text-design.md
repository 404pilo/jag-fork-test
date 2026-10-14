---
title: "Files, folders, and piped text: search with no path, the text of a pipe, a mix of files and folders, and a path that does not exist"
feature: FEATURE-14
stories: [STORY-24, STORY-39, STORY-40]
kind: build
gh_issue: 85
lifecycle_state: draft
inherits: [IF-3, IF-4, IF-5, IF-6, C-cli, C-search, CV-1, CV-3, CV-4]
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Files, folders, and piped text: search with no path, the text of a pipe, a mix of files and folders, and a path that does not exist

## Problem

jag must feel like grep (CAPABILITY-06). It works in a shell pipe, and a
shell user keeps their habits. Today a search needs one path or more
(FEATURE-04 spec decision 19). When the user gives no path, jag must search
the current folder, as rg does. When text comes in through a pipe, jag
must read that text (DECISION-35). A result from piped text has the label
`(standard input)` and the line numbers of the piped text (DECISION-18).
The user can give several paths, and each path can be a file or a folder,
as in grep (DECISION-36). The command line is `jag [OPTIONS] TERM
[PATH...]`, and a question search adds the option `--ask QUESTION`
(`0018-take-the-question-only-from-the-ask-option.md`, system-design.md
IF-3). So each word after the term is a path.

The user answered the two open points of this spec on 2026-10-14. An
empty pipe gives the empty text, as rg does: no match and exit 1
(decision 3). A path that does not exist gives a warning, jag searches the
other paths, and it exits with 2 at the end, as grep does (decisions 7
and 20).

- STORY-24: a shell power user wants a result from piped text to have the
  label `(standard input)` and the line numbers of the piped text, so that
  they can read a result of a pipe as they read a result of a file. The
  FEATURE-08 code writes the label (PR #129). This spec reads standard
  input and adds the end-to-end test (Q-ar).
  - CRITERION-35: a result from piped text has the label
    `(standard input)` and the line numbers of the piped text.
- STORY-39: a shell power user wants jag with no path to search the
  current folder, or to read the pipe when text comes in through one, so
  that they use jag as they use grep.
  - CRITERION-59: when the user gives no path and no text comes in through
    a pipe, jag searches the current folder.
  - CRITERION-60: when the user gives no path and text comes in through a
    pipe, jag searches the piped text.
- STORY-40: a shell power user wants to give several paths, each a file or
  a folder, so that they search exactly the places that they choose.
  - CRITERION-61: jag accepts several paths, and each path can be a file or
    a folder.
  - CRITERION-70: when a path does not exist, jag names the path on the
    error stream and exits with code 2, as grep does.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Paths are optional | A search needs no path, with or without `--ask`. jag removes the failure `a search needs a path.` and its fix `Searching standard input comes later.` The fix of `a search needs a term.` becomes `Use: jag [OPTIONS] TERM [PATH...]` | `0018-take-the-question-only-from-the-ask-option.md`; system-design.md IF-3; FEATURE-04 spec decision 19 ("Until FEATURE-14, a search needs one PATH or more"); FEATURE-12 spec decision 20 (the two failures of today); `src/cli.rs` (`search_args`; `override_usage` shows `[PATH...]` today) |
| 2 | When jag reads standard input | Only when the user gives no path, and only when standard input can give text: it is not a terminal, and it is a regular file, a pipe (FIFO), or a socket. This is the test of ripgrep. In all other cases jag searches the current folder: for a terminal, for `/dev/null` and each other device, and when the test gets an error | DECISION-35 ("as rg does"); CRITERION-59; CRITERION-60; ripgrep `crates/cli/src/lib.rs` (`is_readable_stdin`) (inferred) |
| 3 | No text comes in | When standard input can give text, jag reads all of it and searches it. Zero bytes: jag searches the empty text, as rg does, and exits 1. Only a terminal, /dev/null, or a source that is not a file, a pipe, or a socket gives the current folder. The empty text has no line, so it gives no match, no model call, and the no-result path of today (exit 1). Text with a NUL byte gives the binary rule (decision 11) | human:404pilo, 2026-10-14 (hitl answer FEATURE-14 "Pipe sends no text": search the empty text, as rg does); CRITERION-59; CRITERION-60; rg 15.1.0 and GNU grep 3.12 (checked on 2026-10-14: `printf '' \| rg needle` and `printf '' \| grep needle` exit with 1) |
| 4 | Paths and a pipe | When the user gives a path, jag never reads standard input, also when text comes in through a pipe | CRITERION-60 ("When the user gives no path"); grep and rg (a path takes the place of standard input) (inferred) |
| 5 | Path form with no path | jag walks `.`, and removes the `./` at the start of the path of each result and of each warning. A path that is only `.` stays `.`. The user typed no root, so the path is the rest of the path. With the path `.`, each path keeps `./`, as today | DECISION-15; FEATURE-08 spec decision 5 and its "Out of scope" (with no path, GNU grep and rg write no `./`) |
| 6 | Several paths | Each path is a file or a folder, in any mix and in any order. jag searches a file or a folder that the user names, also when it is hidden or ignored, as rg does. A file that the user names two times gives its results two times, as in grep. The order of the scan is the order of the paths on the command line. Inside one path, it is the byte order of the path, then the line number. So with the paths `b.txt a.txt`, the scan gives `b.txt` first, as grep does. `scan` keeps one walk for all paths, as today, and gives each file and each warning the place of its path on the command line (Design). It does not walk each path alone: each walk of the `ignore` crate starts its threads again. The FEATURE-12 code (PR #160) put this order on `main`. The output of a question search stays in rank order | DECISION-36; DECISION-14; CAPABILITY-06 ("a shell user keeps their habits"); grep (it searches the paths in the order of the command line); FEATURE-05 spec decisions 3 and 11 (decision 11 asks only for a fixed order); `src/search/walk.rs` on `main` (`order_files`, `warning_place`, and `finish(shared, roots)`); the `ignore` crate (it never skips a root); the review of PR #151 (verifier-1, 2026-10-14: plain grep must keep the order of the command line); FEATURE-16 spec decision 12 (PR #155; the probe of 2026-10-14 on Linux x86_64 with 12 CPUs: each walk costs about 1.4 to 2.3 ms; 1,000 file paths took 4.5 ms in one walk and 1.44 to 2.33 s with a walk for each path; 5,000 file paths took 20 ms against 11.8 s) (inferred) |
| 7 | A path that does not exist | jag writes a warning that names the path, searches the other paths, and exits with 2 at the end, as grep does (decision 20). The one walk of decision 6 finds such a path, with no check before it: the `ignore` crate reads the metadata of each root with `fs::metadata`, which follows a symbolic link, and gives an error at depth 0 when it cannot. An error at depth 0 with the kind `NotFound` or `NotADirectory` gives the warning `the path PATH does not exist`, with `jag: ` in front on the error stream, as each warning of the walk. PATH is the path as typed (`Path::display`). Another error at depth 0, for example `PermissionDenied`, stays a read warning (decision 10). The warning takes the place of its path on the command line by the rule of decision 6, so `nofile2 a.txt nofile1` gives the warning of `nofile2` first. With `--ask`, jag scores the results of the other paths, and those model calls cost credit. When no path exists, jag finds no result, makes no model call, and exits with 2. This is a warning and not a failure of CV-1: jag does not stop, so it writes no fix, and in structured mode `error` stays `null` (decision 16). The code 2 keeps its meaning of IF-4: the command line has an error | system-design.md CV-1 and IF-4 (2 is a usage error); human:404pilo, 2026-10-14 (hitl answer FEATURE-14 "One path does not exist": search the other paths, as grep does; not the recommended option); CRITERION-70; DECISION-05 (jag keeps the grep exit codes 0, 1, and 2); GNU grep 3.12 and rg 15.1.0 (checked on 2026-10-14: `grep -H needle nofile a.txt` writes `grep: nofile: No such file or directory`, then `a.txt:needle`, and exits with 2; `a.txt/x` (`Not a directory`) and a broken symbolic link give the same; `rg needle nofile a.txt` gives the same); ignore 0.4.33 `src/walk.rs` (`WalkParallel::visit` gives each root to `DirEntryRaw::from_path(0, path, false)`, which calls `fs::metadata` and gives `Error::Io(err).with_depth(depth).with_path(&pb)`) |
| 8 | The path `-` and the empty path | Two paths are usage errors of step 1, with exit 2. The path `-` gives the cause `the path - is not valid.` and the fix `To search the text of a pipe, give no path. To search a file with the name -, give ./-.` grep and rg read standard input for `-`, but no story asks for it. Today the walk takes `-` as standard input and searches nothing, with no warning. The empty path (`""`) gives the cause `a path is empty.` and the fix `Give the path of a file or a folder, or give no path.` With no check, the walk gives the warning of decision 7 with two spaces and no name: `the path  does not exist`. grep takes `""` as a path that does not exist and searches the other paths. jag stops at step 1, because a warning with no name does not show the mistake | the `ignore` crate (a root `-` gives an entry of standard input with no file type), and `visit_entry` in `src/search/walk.rs` (it skips an entry with no file type); system-design.md CV-1; the review of PR #142 (verifier-3: the empty path gives two spaces); GNU grep 3.12 (checked on 2026-10-14: `grep -H needle '' a.txt` writes `grep: : No such file or directory`, then `a.txt:needle`, and exits with 2) (inferred) |
| 9 | Order of the checks | Step 1 (the arguments, with decision 8). With `--ask`, the key (exit 3) and the settings come next. Then the term (exit 2). Then the scan, which finds each path that does not exist (decision 7), or the read of standard input. So with `--ask`, a missing key and a path that does not exist give exit 3 and no warning, because jag stops before the scan, as a missing key and a bad term do today. A bad term and a path that does not exist give the failure of the term. With no `--ask`, jag reads no key | FEATURE-04 spec decision 18 ("jag finds the key before it reads a file. ... Only an argument error of step 1 (exit 2) comes before it"); FEATURE-12 spec decision 4 (no key with no question); `src/cli.rs` (`search`: the key, then `build_matcher`, then `scan`; `plain`: `build_matcher`, then `scan`) |
| 10 | Read errors | A file or a folder that jag cannot read gives a warning, as today. The warning does not change the exit code: 0 with a result, and 1 with no result. `grep -r` gives exit 2 after a read error. jag keeps 2 for a usage error and for a path that does not exist (decision 7). A path that exists and that jag cannot read, for example a folder with mode `0000`, is a read error | system-design.md IF-4; CRITERION-58; FEATURE-05 spec decision 6 (Q-g); FEATURE-04 spec, "The search flow" ("FEATURE-14 decides that rule for jag"); FEATURE-11 spec decisions 17 and 21, and the FEATURE-11 code on `main` (7150e83: the document holds `warnings` with exit code 0 or 1) |
| 11 | Piped text | jag searches the piped text with the searcher of a file, on the same path of `grep-searcher` (`search_reader`). So the binary rule and the byte order mark rule are the same as for a file. Text with a NUL byte gives no result, and the skip notice counts it in `binary`. The snippets come from the same bytes, with `decode`, `split_lines`, and `SnippetLine::new`. The first line is 1 | DECISION-18; CRITERION-35; FEATURE-08 spec decision 4 (Q-ar) and its Design ("FEATURE-14 does this for standard input"); `grep-searcher` (`search_path` uses `search_reader` when memory maps are off, and they are off by default) (inferred) |
| 12 | A read error of standard input | The warning `jag: cannot read standard input: KIND`. jag searches no text, and it does not search the current folder. Decision 10 gives the exit code. In structured mode, the warning has `path: null` | decision 10; FEATURE-11 spec decision 17; `src/search/walk.rs` (`WalkWarning` with `path: None`) (inferred) |
| 13 | Piped text in memory | jag keeps all piped text in memory until the search ends. It needs the bytes two times (for the scan and for the snippets), and it cannot read a pipe two times | `src/search/snippet.rs` (`read_snippets` reads each file again after the scan) (inferred) |
| 14 | No new crate | The test of decision 2 uses only `std`: `IsTerminal`, `AsFd::try_clone_to_owned`, `File::metadata`, and `std::os::unix::fs::FileTypeExt`. It has no `unsafe` code. jag does not add `grep-cli` for one function | 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md ("Each new outside crate needs a reason"); 0001-find-matches-with-the-ripgrep-libraries.md; `Cargo.toml` (`unsafe_code = "forbid"`); `src/config/fs.rs` (jag runs only on Unix) |
| 15 | Code place | C-search: `Source` moves to `src/search.rs`, the new `src/search/input.rs` holds the input of a search with no path, `src/search/walk.rs` gets `scan_input`, `scan_current_folder`, and the warning of decision 7, and `src/search/snippet.rs` makes the snippets of piped text. C-cli (`src/cli.rs`): the input of both flows, the checks of decision 8, the exit code of decision 20, and the usage texts | system-design.md C-search ("the files or the piped text") and C-cli; 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md |
| 16 | Structured mode | The document gets no new field. A path that does not exist (decision 7) gives `exit_code` 2, `error` `null`, the warning `{"path":"nofile","text":"the path nofile does not exist"}` in `warnings`, and the results of the other paths in `results`. With no other result, `results` is `[]`. An empty pipe gives the document of no match: `results` `[]` and `exit_code` 1. The failures of decision 8 give the error document of FEATURE-11. This spec adds the end-to-end test of `path: null` for a result from standard input | FEATURE-11 spec decisions 8 (`exit_code` is the exit code of the process), 9 (`error` is `null` when jag did not fail), 13, 17 (each warning is an object with `path` and `text`), 19 (no `jag: ` in the document), 20, and 21, and its "Out of scope" ("the end-to-end test of `path: null`: FEATURE-14"); the FEATURE-11 code on `main` (7150e83) |
| 17 | FEATURE-09 | The FEATURE-09 code also changes the search flow in `src/cli.rs`. The code that merges second joins the two parts. In text mode, the warning of decision 7 goes to the error stream, and the size limit does not count it. In structured mode, the results fit first, then the warnings. So a document at the size limit can leave out the warning of decision 7 and count it in `warnings_left_out`, and `exit_code` stays 2 (decision 20, Risk 3). The error documents of decision 8 hold no path | FEATURE-09 spec on `main` (PR #134, 083e17e: decisions 3, 17, and 18) |
| 18 | Docs | The help of PATH says: "The files and folders to search. With no path, jag searches the text of a pipe or the current folder". The README gets the section "Where jag searches". The example of "Plain grep with no question" becomes `jag TERM [PATH...]`. The example of "Output for an agent" keeps `PATH...`, because the README tells an agent to give a path (Risk 1). That section gets one sentence for a path that does not exist (decision 16) | docs/superpowers/specs/2026-10-13-STORY-06-readiness-check-design.md (a README section for each command); FEATURE-11 spec decision 29; FEATURE-12 spec decision 22 (the README section "Plain grep with no question") (inferred) |
| 19 | Spec grouping | One spec for STORY-24, STORY-39, and STORY-40. STORY-24 moves from the FEATURE-08 spec to this spec: its `spec` field names this file, and the `stories` list of the FEATURE-08 spec loses STORY-24 | docs/adlc.md (one feature gives one spec); human:404pilo, 2026-10-13 (Q-ar: STORY-24 moves to FEATURE-14); the orchestrator TASK of 2026-10-13 |
| 20 | Exit code of a search | A search that ends gives one code, in this order: 2 when a path does not exist (decision 7), then 1 (no match or no good answer), then 7 (cost limit reached), then 8 (results left out by the size limit), then 0. The notices of 7 and 8 and the label of no good answer still show. The mistake in the command comes first, as in grep, where an error gives 2 also when lines match. A failure stops jag before the end, and its code is the code of the process, as today: a usage error of step 1 or a bad term (2), the key (3 and 4), no credit (5), the service (6), and Ctrl-C (130). The one exception: `-q` with a match gives 0 also when a path does not exist, as `grep -q` (FEATURE-13 spec decision 21) | human:404pilo, 2026-10-14 (hitl answer FEATURE-14 "One path does not exist": exit 2 at the end); GNU grep 3.12 (`man grep`: "the exit status is 0 if a line is selected, 1 if no lines were selected, and 2 if an error occurred. However, if the -q or --quiet or --silent is used and a line is selected, the exit status is 0 even if an error occurred"; checked on 2026-10-14: `grep -H zzz nofile a.txt` exits with 2); DECISION-13 and DECISION-47 (1, then 7, then 8); system-design.md IF-4 ("DECISION-13 and DECISION-47 give the order when two apply"); `src/cli.rs` (`search_exit_code`: "IF-4 order: 1 wins, then 7, then 8, then 0"); FEATURE-09 spec decision 17 (on `main`, PR #134: "The order 1, 7, 8, 0 does not change"; this decision puts 2 in front of it); FEATURE-13 spec decisions 21 and 29 (inferred: the place of 2 before 7 and 8) |
| 21 | ADR | No ADR. IF-4 and DECISION-05 hold the codes and their meanings, and the two user answers of 2026-10-14 follow rg and grep, as DECISION-35 and CRITERION-70 ask. The place of 2 in the order (decision 20) is new: IF-4 gives the order from DECISION-13 and DECISION-47, and they do not name 2. Before the first release, a change of that place is cheap. The architect can add it to IF-4 | docs/adr/CLAUDE.md ("If it is cheap to change your mind, do not write an ADR"); system-design.md IF-4; FEATURE-13 spec decision 33 |
| 22 | Story state | STORY-24, STORY-39, and STORY-40 go to `ready` in this PR. The user answers of 2026-10-14 close the ASKs of decisions 3 and 7, so no open ASK blocks a story. FEATURE-14 has only FEATURE-05 in `blocked_by`, and FEATURE-05 is `done`. The spec merges with this PR | docs/adlc.md rule 11 (a spec PR can set its own stories to `ready`); the orchestrator ANSWER of 2026-10-14 ("STORY-39/40 → ready if rule 11 allows") |

## Design

### Files

```
src/search.rs          C-search: Source (it moves here from snippet.rs)
src/search/input.rs    C-search (new): Input, stdin_can_give_text, read_input,
                       no_path_input
src/search/walk.rs     C-search: FileMatches.source, scan_input,
                       scan_current_folder, WarningKind::StandardInput,
                       WarningKind::Missing, ScanReport::path_missing
src/search/snippet.rs  C-search: Window.source, read_snippets with the piped text
src/cli.rs             C-cli: no path, the paths - and "", the input of both
                       flows, the exit code of decision 20, the usage texts
tests/input.rs         the end-to-end tests of CRITERION-35, 59, 60, 61, and 70,
                       and of decisions 3, 8, 9, 10, and 20
tests/usage.rs         the two tests of "needs a path" change (decision 3)
tests/plain_grep.rs    the test of the old form gets the warning of decision 7
tests/walk.rs          the field source in place of path, and the order of
                       several roots with 8 threads
README.md              the section "Where jag searches", and one sentence in
                       "Output for an agent"
```

No new crate (decision 14). `jag_std` of `tests/common/mod.rs` (a
`std::process::Command` with the variables of jag) is on `main` already.
The order of decision 6 is on `main` too (PR #160): `visit_entry` keeps the
root of each file, and `finish` calls `order_files` and `warning_place`.

### Source and the plan of a result (C-search)

`Source`, with `STANDARD_INPUT_LABEL` and `label`, moves from
`src/search/snippet.rs` to `src/search.rs`. Then `walk.rs` and
`snippet.rs` both use it, and the two modules do not use each other in a
loop. `snippet.rs` keeps `pub use super::Source;`, so the imports of
`src/output.rs` and of the tests do not change.

`FileMatches.path: PathBuf` and `Window.path: PathBuf` become
`source: Source`:

- The walk gives `Source::File(path)`. `order_files` sorts the files by the
  place of their root, then by the bytes of the path, as today
  (decision 6).
- `plan_windows` copies the source to each window. `match_texts` does not
  change.
- `read_snippets(windows, input: Option<&[u8]>)` groups the windows by
  source, as it groups them by path today. For `Source::File`, it reads the
  file, as today. For `Source::StandardInput`, it uses `input`. Then both
  use `decode`, `split_lines`, and `snippet_of`, and `snippet_of` copies
  `window.source` to the snippet.

The other uses of the field are in `src/search/snippet.rs`, its unit tests,
and 3 lines of `tests/walk.rs`.

### The input of a search with no path (C-search)

`src/search/input.rs`:

```rust
/// What a search with no path searches (decisions 2, 3, and 12).
pub enum Input {
    /// The text of standard input, also when it has no byte.
    PipedText(Vec<u8>),
    /// Standard input gave an error. jag searches nothing.
    ReadError(WalkWarning),
    /// Standard input cannot give text: jag searches the current folder.
    CurrentFolder,
}

/// True when standard input can give text (decision 2).
pub fn stdin_can_give_text() -> bool;

/// Read all of `reader`. Zero bytes give `PipedText` with no byte
/// (decision 3).
pub fn read_input(reader: impl io::Read) -> Input;

/// `read_input(io::stdin().lock())` when `stdin_can_give_text()`, else
/// `CurrentFolder`.
pub fn no_path_input() -> Input;
```

`stdin_can_give_text` is `!io::stdin().is_terminal()` and
`is_text_source(&file)`. The `file` is a copy of the descriptor of
standard input: `io::stdin().as_fd().try_clone_to_owned()`, then
`File::from`. `is_text_source` reads the metadata of the file and gives
`is_file() || is_fifo() || is_socket()`. Each error gives false. Keep the
type test: a device that is not a terminal, for example `/dev/zero`, never
ends. Do not change the test to `is_terminal()` alone.

`read_input` uses `read_to_end`. An error gives `ReadError` with
`WalkWarning { path: None, kind: WarningKind::StandardInput(kind) }`, and
jag keeps no byte of the text.

### The scan of several paths, of piped text, and of the current folder (C-search)

`src/search/walk.rs`:

- `scan(roots, matcher, options)` keeps its signature, its one walk (one
  `WalkBuilder` with all roots), and the order of decision 6, which the
  FEATURE-12 code put on `main` (PR #160). This spec keeps that code and
  adds the test "20 times with 8 threads" (CRITERION-61). The code gives
  each file and each warning the place of its root on the command line:
  - The root of a file at depth `d` is `entry.path().ancestors().nth(d)`,
    because the walk joins the name of each child to the path of its
    folder. `visit_entry` keeps this root with the file.
  - `order_files` maps each root path to its places, the indices in `roots`.
    `Path` compares components, so `dir` and `dir/` are one key. A path
    that the user gives two times has two places. The walk gives its files
    two times, and the copies of one file under one root take the places
    of that root, one each, in order. The copies are equal, so the order
    of the threads does not change the report.
  - A warning takes the place of the first root, in the order of `roots`,
    that its path starts with (`Path::starts_with`). The depth of an error
    does not give its root: an error in the list of a folder has the path
    of the folder and the depth of its child. A warning with no path, or
    with a path under no root (for example an error of an ignore file above
    a root, which has an absolute path), takes place 0. So with one root,
    each warning has place 0 and keeps the order of today. With roots that
    overlap, for example `dir dir/sub`, a warning under `dir/sub` takes the
    place of `dir`. This changes only the order of the warnings.
  - `order_files` sorts the files by place, then by the bytes of the path.
    `finish` sorts the warnings by place, then by path, then by message.
    The skip counts do not change. One root gives the same report as
    before PR #160. The doc comments of `ScanReport.files` and
    `ScanReport.warnings` give this order.
- The warning of decision 7. `make_warning` gives the kind
  `WarningKind::Missing` to an error at depth 0 (`error.depth()` is
  `Some(0)`) whose `io_error()` has the kind `NotFound` or `NotADirectory`.
  The walk gives such an error only for a root: `WalkParallel::visit`
  reads each root with `DirEntryRaw::from_path(0, ...)`. A root folder
  that exists and that jag cannot open gives an error at depth 0 with
  another kind, for example `PermissionDenied`, so it stays
  `WarningKind::Read` (decision 10). `text()` gives
  `the path PATH does not exist`, and `message()` adds `jag: ` in front,
  as for the other kinds. `visit_error` keeps no error path for an error
  at depth 0, as today, so the skip counts do not change. `warning_place`
  gives the warning the place of its root, so the warnings come in the
  order of the command line. `ScanReport::path_missing()` is true when a
  warning has the kind `Missing`. The flows of `src/cli.rs` read it for the
  exit code (decision 20), and the flows of `-q`, `-l`, and `-c` of
  FEATURE-13 read it too.
- The pre-rank of FEATURE-06 keeps the order of the scan for equal BM25
  scores (a stable sort in `src/prerank.rs`), so the snippets go to the
  model in the order of decision 6. The comment of that sort in `best`
  says so on `main` (PR #160). `rank` in `src/scorer.rs` does not
  change: equal model scores still come in the byte order of the path
  (FEATURE-04 spec decision 28).
- `scan_input(bytes: &[u8], matcher) -> ScanReport`. It uses the searcher
  of `scan` (one function builds the searcher for both), and it calls
  `search_reader(matcher, bytes, &mut sink)`. That is the path that
  `search_path` takes for a file, because memory maps are off. A match
  gives one `FileMatches` with `Source::StandardInput`. Binary text gives
  `skipped.binary = 1` and no file. An error gives a warning with
  `WarningKind::StandardInput`. Text with no byte gives no file and no
  warning (decision 3).
- `scan_current_folder(matcher, options) -> ScanReport`. It calls
  `scan(&[PathBuf::from(".")], matcher, options)`. Then it removes the
  first component `.` from the path of each file and of each warning:
  `./a.txt` becomes `a.txt`, and `./sub/b.txt` becomes `sub/b.txt`. A path
  that is only `.` stays `.`. Then it sorts the two lists again with the
  keys of `order_files` and `finish`.
- `WarningKind::StandardInput(io::ErrorKind)`. `text()` gives
  `cannot read standard input: KIND`, and `message()` adds `jag: ` in
  front, as for the other kinds.

### The search flow (C-cli)

Both flows get their input from one new function:

```
search_args                      step 1, with - and "" (decision 8)
with --ask:
    find_key                     exit 3
    model_settings, cost_limit
build_matcher                    exit 2 for a bad term
scan_search(paths, &matcher):
    if the user gave paths:
        scan(paths)              a warning for each path that does not
                                 exist (decision 7)
    else:
        match no_path_input():
            PipedText(bytes) -> scan_input(&bytes)
            CurrentFolder    -> scan_current_folder()
            ReadError(w)     -> a ScanReport with only the warning w
with --ask:
    plan_windows, the pre-rank, read_snippets(&kept, the piped text),
    the score, and the rank
with no --ask:
    line_windows, read_snippets(&windows, the piped text)
the exit code (decision 20)
```

`fn scan_search(paths: &[PathBuf], matcher: &RegexMatcher) -> (ScanReport,
Option<Vec<u8>>)` gives the report and the piped text. `search` and `plain`
call it in the place of `scan(args.paths, ...)`, and give the piped text to
`read_snippets`. The rest of each flow does not change. `record` writes the
notices and the warnings, so in text mode the warning of decision 7 goes to
the error stream before the first model call. Then come the summary of
FEATURE-11, the score, the rank, and the output.

The exit code of decision 20:

- `SearchOutcome` gets `path_missing: bool`. `search_exit_code` gives 2
  when it is true, and else 1, 7, 8, or 0, as today. Its doc comment
  becomes "IF-4 order with FEATURE-14 decision 20: 2 wins, then 1, then 7,
  then 8, then 0".
- `search` sets `path_missing: report.path_missing()`.
- `no_result` gets the code: 2 when `report.path_missing()`, and else 1. In
  structured mode, `exit_code` is that code. So a search where no path
  exists makes no model call and exits with 2.
- `plain` gives 2 when `report.path_missing()`, and else 0, for the text and
  for the document.

The warning of decision 7, for `jag needle nofile a.txt`, where `a.txt`
holds `needle` on line 1:

```
$ jag needle nofile a.txt
jag: the path nofile does not exist
a.txt:1:needle
$ echo $?
2
```

The first line is on the error stream, and the second line is on standard
output. With `--json`, the document has `"exit_code":2`, `"error":null`,
`"warnings":[{"path":"nofile","text":"the path nofile does not exist"}]`,
and the result of `a.txt` in `results`.

The failures of decision 8, for `-` and for `""`:

```
jag: the path - is not valid.
To search the text of a pipe, give no path. To search a file with the name -, give ./-.
```

```
jag: a path is empty.
Give the path of a file or a folder, or give no path.
```

`search_args`: replace the check of an empty `paths` with the checks of
decision 8, in the same place: after the term and before the checks of the
question, the order of FEATURE-12 spec decision 20 and FEATURE-13 spec
decision 30. They examine the paths in the order of the command line, and
the first path that is `-` or `""` gives its failure. The fix of
`a search needs a term.` becomes `Use: jag [OPTIONS] TERM [PATH...]`.

The help of `paths`:
`/// The files and folders to search. With no path, jag searches the text of a pipe or the current folder`.

### Output

`src/output.rs` does not change. Examples, from a folder that holds
`a.txt` and `sub/b.txt`, each with `needle` on line 1, with no pipe, and
with scores that show no label:

```
$ jag needle
a.txt:1:needle
sub/b.txt:1:needle

$ jag --ask "QUESTION" needle
a.txt:1:needle
--
sub/b.txt:1:needle

$ jag --ask "QUESTION" needle .
./a.txt:1:needle
--
./sub/b.txt:1:needle

$ printf 'a\nb\nc\nd\ne\nf\nneedle\n' | jag --ask "QUESTION" needle
(standard input)-4-d
(standard input)-5-e
(standard input)-6-f
(standard input):7:needle

$ printf '' | jag needle
$ echo $?
1
```

### The README section "Where jag searches"

The section comes before "Output for an agent". It says:

- jag searches each file and each folder that you give. It searches a
  folder as `grep -r` does. You can give any mix of files and folders.
- With no path, jag searches the text of a pipe, and each result shows
  `(standard input)` in the place of a path. An empty pipe gives no match
  and exit 1, as in rg.
- With no path and no pipe, for example in a terminal or with `/dev/null`
  as standard input, jag searches the current folder, and the paths have
  no `./` at the start.
- jag reads all piped text before it ranks the results, so it waits until
  the pipe closes.
- An agent gives a path, for example `.`. Some tools keep standard input
  open, and then jag waits for text that does not come. Other tools give
  an empty pipe, and then jag finds no match.
- A path that does not exist gives a warning that names the path. jag
  searches the other paths, and it exits with 2 at the end, as grep does.
  With `--ask`, the results of the other paths cost model calls.
- `-` is not standard input in jag. To search a pipe, give no path.

The section "Output for an agent" gets one sentence after the sentence of
no match: "When a path does not exist, `exit_code` is 2, `error` is `null`,
`warnings` names the path, and `results` holds the results of the other
paths."

### Other features

- FEATURE-12 (on `main`, PR #160): the question comes only from `--ask`
  (ADR-0018), so each word after the term is a path. A path with a typo,
  and a command in the old form `jag TERM "QUESTION" PATH`, name a path
  that does not exist. jag writes the warning of decision 7, searches the
  other paths, and exits with 2. In plain grep this costs nothing. With
  `--ask`, jag scores the results of the other paths, and those model calls
  cost credit (Risk 10). FEATURE-12 spec decision 21 gives this rule to
  this spec.
- FEATURE-13 (spec on `main`, PR #153): `-q` with a match exits with 0 also
  when a path does not exist, as `grep -q` (FEATURE-13 spec decision 21).
  This is the one exception to decision 20. `-q` with no match and a
  missing path exits with 2. `-l` and `-c` write the lines of the other
  paths and exit with 2 (FEATURE-13 spec decision 29). An empty pipe with
  `-c` writes `(standard input):0` and exits with 1 (FEATURE-13 spec
  decision 20). The flows of `-q`, `-l`, and `-c` read
  `ScanReport::path_missing`. When the FEATURE-13 code merges first, the
  FEATURE-14 code adds the tests of FEATURE-13 that say "after the
  FEATURE-12 and FEATURE-14 code", with the tests of `-q` (FEATURE-13 spec,
  Build order step 6).
- FEATURE-09 (spec on `main`, PR #134): the size limit counts only
  standard output.
  In structured mode, a document at the limit can leave out the warning of
  decision 7, and exit 2 wins over 8 (decisions 17 and 20, Risk 3).
- FEATURE-11 (on `main`): the document gets no new field (decision 16).
- FEATURE-06 (on `main`): the cost limit gives 7, and 2 wins over 7
  (decision 20). The pre-rank keeps the order of the scan for equal
  scores.
- FEATURE-16 (spec PR #155): one walk for all paths (decision 6, Risk 9).
  The warning of decision 7 comes from the same walk, so jag examines no
  path before the walk.

### Acceptance criteria and their tests

The end-to-end tests are in the new file `tests/input.rs`. They use `jag`
and `jag_std` of `tests/common/mod.rs`, a temporary `$HOME`, and the fake
server. Each test with no path sets `current_dir` to a temporary folder,
so that the walk never reads the files of the crate. `assert_cmd` gives jag
an empty pipe that closes at once, and `write_stdin` gives it piped text.
`jag_std` with `stdin(Stdio::null())` gives jag no pipe. A test with no
`--ask` runs plain grep, and a test with `--ask "QUESTION"` runs the
question search.

- CRITERION-59 (no path and no piped text: the current folder):
  - End-to-end (no pipe): no path, with `jag_std` and `Stdio::null()`, from
    a folder with `a.txt` and `sub/b.txt`, each with `needle`. Exit 0.
    Standard output is exactly `a.txt:1:needle\nsub/b.txt:1:needle\n`. With
    `--ask "QUESTION"`: exit 0, the two results, and no line starts with
    `./`.
  - End-to-end (the same as `.`): a folder with a hidden file, a file that
    `.ignore` names, and a binary file. With `jag_std` and `Stdio::null()`,
    the search with no path and the search of `.` give the same exit code
    and the same skip notice. Their standard outputs are the same after the
    test removes `./` from each path of the search of `.`.
  - Unit (`walk.rs`): `scan_current_folder` gives `a.txt` and `sub/b.txt`.
    A warning path `./x` becomes `x`, and the path `.` stays `.`.
  - Unit (`input.rs`): `is_text_source` gives false for `/dev/null` and for
    a folder. It gives true for a regular file, for the read end of
    `std::io::pipe()`, and for one end of `UnixStream::pair()`.
- Decision 3 (an empty pipe):
  - End-to-end: no path and the empty pipe of `assert_cmd`, from a folder
    whose `a.txt` holds `needle`. Exit 1, no standard output, an empty
    error stream, and no request. With `--ask "QUESTION"`: exit 1, no
    standard output, and no request. With `--json`: one document with
    `results` `[]`, `exit_code` 1, and `error` `null`. These tests take the
    place of `a_term_with_no_path_exits_2_and_makes_no_call` and
    `a_term_and_a_question_with_no_path_exit_2_and_make_no_call` in
    `tests/usage.rs`.
  - Unit (`input.rs`): `read_input` of zero bytes gives `PipedText` with no
    byte. Unit (`walk.rs`): `scan_input` of no byte gives no file, no
    warning, and no skip.
- CRITERION-60 (no path and piped text: the piped text):
  - End-to-end: no path and the piped text `x\nneedle here\n`, from a
    folder whose `a.txt` also holds `needle`. Plain grep: standard output
    is exactly `(standard input):2:needle here\n`. With
    `--ask "QUESTION"`: exactly
    `(standard input)-1-x\n(standard input):2:needle here\n`.
  - End-to-end (a path wins): the same piped text with the path `a.txt`.
    Standard output names only `a.txt`.
  - Unit (`input.rs`): `read_input` of `b"x"` gives `PipedText`. A reader
    that fails gives `ReadError` with `path: None` and the kind of the
    error.
- CRITERION-35 (the label and the line numbers):
  - End-to-end: `--ask "QUESTION"` and the piped lines `a` to `f`, then
    `needle` on line 7. Standard output is exactly
    `(standard input)-4-d\n(standard input)-5-e\n(standard input)-6-f\n(standard input):7:needle\n`.
    Plain grep gives exactly `(standard input):7:needle\n`.
  - End-to-end (structured mode): the same with `--json --ask "QUESTION"`.
    `results[0].path` is `null`, `first_line` is 4, `last_line` is 7, and
    only line 7 has `is_match: true`.
  - End-to-end (binary): `--ask "QUESTION"` and the piped text
    `needle\0\n`. Exit 1, no standard output, the notice
    `jag: skipped 1 files and folders (1 binary, 0 hidden, 0 ignored)`, and
    no request to the server.
  - Unit (`walk.rs`): `scan_input` gives the same line numbers and texts as
    the scan of a file with the same bytes, also with a byte order mark of
    UTF-8 and of UTF-16.
  - Unit (`snippet.rs`): `read_snippets` with windows of
    `Source::StandardInput` and the piped text gives the lines and the
    label.
- CRITERION-61 (several paths, each a file or a folder):
  - End-to-end: from a folder with `one.txt`, `dir/two.txt`,
    `dir/sub/three.txt`, and `other/four.txt`, the paths
    `one.txt dir other/four.txt`. Exit 0, and the four results with these
    paths.
  - End-to-end: the hidden file `.hidden.txt` as a path gives its result.
    In a search of its folder, the walk skips it, and the notice counts it.
  - End-to-end (decision 6): `--ask "QUESTION"` and the paths `a.txt a.txt`,
    where `a.txt` holds `needle` on line 1. Exit 0. Standard output is
    exactly `a.txt:1:needle\n--\na.txt:1:needle\n`: `read_snippets` groups
    the two windows by source, and the two results stay two results.
  - Unit (`tests/walk.rs`, the order of decision 6): the roots `z.txt`,
    `dir`, and `a.txt`, where `dir` holds `b.txt` and `a.txt`, and each file
    holds `needle`. `scan` gives the files `z.txt`, `dir/a.txt`,
    `dir/b.txt`, and `a.txt`, in that order, 20 times with 8 threads. The
    roots `nofile2` and `nofile1` give their two warnings of the kind
    `Missing` in that order.
  - Unit (`tests/walk.rs`, a path two times and paths that overlap): the
    roots `dir`, `dir/sub`, and `dir`, where `dir` holds `a.txt` and
    `sub/c.txt`, and each file holds `needle`. `scan` gives the files
    `dir/a.txt`, `dir/sub/c.txt`, `dir/sub/c.txt`, `dir/a.txt`, and
    `dir/sub/c.txt`, in that order, 20 times with 8 threads: the order of
    the paths on the command line, as `grep -r needle dir dir/sub dir`,
    and the byte order inside each path.
  - On `main` already (PR #160), and they stay: the unit tests of
    `order_files` and `warning_place` in `src/search/walk.rs`,
    `files_come_in_the_order_of_the_paths_on_the_command_line` in
    `tests/walk.rs`, and the tests of the order in `tests/plain_grep.rs`.
- CRITERION-70 (a path that does not exist):
  - End-to-end: the path `nofile`. Exit 2, no standard output, the error
    stream is exactly `jag: the path nofile does not exist\n`, and no
    request. The same with `--ask "QUESTION"`.
  - End-to-end: `a.txt nofile`, where `a.txt` holds `needle` on line 1.
    Exit 2, standard output is exactly `a.txt:1:needle\n`, and the error
    stream is exactly `jag: the path nofile does not exist\n`. With
    `--ask "QUESTION"`: exit 2, the result of `a.txt`, and one request.
  - End-to-end (the order of the warnings): `nofile2 a.txt nofile1`. The
    error stream is exactly
    `jag: the path nofile2 does not exist\njag: the path nofile1 does not exist\n`,
    and the exit code is 2.
  - End-to-end: a symbolic link to a file that does not exist, and
    `a.txt/x`, each give the warning with the path as typed, and exit 2.
  - End-to-end (structured mode): `--json` and `a.txt nofile`. Exit 2, an
    empty error stream, and one document with `exit_code` 2, `error`
    `null`, `warnings`
    `[{"path":"nofile","text":"the path nofile does not exist"}]`, and the
    result of `a.txt`. `--json` and `nofile` alone: `results` `[]` and
    `exit_code` 2.
  - End-to-end (decision 20): `--ask "QUESTION"`, `a.txt nofile`, and a
    fake server whose score gives no good answer: exit 2, and the label of
    no good answer still shows. `--ask "QUESTION" --max-scored 1` with two
    results and `nofile`: exit 2, and the cost-limit notice still shows.
  - End-to-end (the order of decision 9): `--ask "QUESTION"` with no key
    and `nofile` gives exit 3 and no warning. A bad term and `nofile` give
    the failure of the term and no warning.
  - End-to-end (decision 8): the path `-` gives exit 2 and its two lines,
    also with no key, and also when a file with the name `-` exists. The
    path `./-` searches that file. The empty path `""` gives exit 2 and its
    two lines, also after the path `a.txt`, and the error stream has no two
    spaces together.
  - `tests/plain_grep.rs`: `decision_1_the_old_form_makes_no_call_and_names_the_path`
    keeps no request and the three lines of `dir` on standard output. It
    gets exit 2, and the error stream holds
    `jag: the path which line says retry does not exist`.
  - Unit (`src/cli.rs`): `the_exit_code_follows_the_order_1_7_8_0` becomes
    `the_exit_code_follows_the_order_2_1_7_8_0`. 2 wins over each other
    code, and the cases of today give the same codes.
  - Unit (`walk.rs`): `make_warning` of an error built from the public
    variants `ignore::Error::WithPath`, `ignore::Error::WithDepth`, and
    `ignore::Error::Io` (the form that `DirEntryRaw::from_path` gives): the
    kind `NotFound`, depth 0, and the path `nofile` give
    `WarningKind::Missing` and the text `the path nofile does not exist`.
    `NotADirectory` at depth 0 gives `Missing`. `NotFound` at depth 1 and
    `PermissionDenied` at depth 0 give `WarningKind::Read`.
- Decision 10 (read errors):
  - End-to-end: a file with mode `0000` as the only path. Exit 1, the
    warning, and no request. A folder with mode `0000` as the only path:
    exit 1, the read warning, and not the warning of decision 7. A user who
    can read the file (root) skips the test, as in `tests/search.rs`.

## Build order

1. C-search: `Source` in `src/search.rs`, the field `source`, and
   `read_snippets` with the piped text. No behavior changes, and all tests
   pass.
2. STORY-40: the order of decision 6 is on `main` (PR #160), so add only
   its tests "20 times with 8 threads". Then decision 7:
   `WarningKind::Missing`, `ScanReport::path_missing`, and the exit code of
   decision 20 in `search_exit_code`, `search`, `plain`, and `no_result`.
   Then the checks of decision 8 and their failures. Then the tests of
   CRITERION-61, CRITERION-70, decisions 8, 9, 10, and 20, and the change
   of `decision_1_the_old_form_makes_no_call_and_names_the_path`.
3. STORY-39: `src/search/input.rs`, `scan_input`, `scan_current_folder`,
   `scan_search` in both flows, and the usage texts. Change the two tests
   of "needs a path" in `tests/usage.rs`. Then the tests of CRITERION-59,
   CRITERION-60, and decision 3.
4. STORY-24: the tests of CRITERION-35, with the test of `path: null`.
5. The help of PATH, the README section "Where jag searches", and the
   sentence of "Output for an agent".
6. The FEATURE-09 join (decision 17): only when the FEATURE-09 code is on
   `main` before this code merges.
7. The FEATURE-13 join ("Other features"): only when the FEATURE-13 code is
   on `main` before this code merges. Add the tests of FEATURE-13 that say
   "after the FEATURE-12 and FEATURE-14 code".

`cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` must
pass on the CI runs of `ubuntu-latest` and `macos-latest`.

## Assumptions

- The test of decision 2 is the test of ripgrep. Source: ripgrep
  `crates/cli/src/lib.rs` (`is_readable_stdin`: not a terminal, and a
  regular file, a FIFO, or a socket; false on each error).
- `assert_cmd` gives each child an empty pipe as standard input when the
  test gives no input. Source: assert_cmd 2.2.2 `src/cmd.rs` (`spawn` sets
  `Stdio::piped()`, and the pipe closes when the test gives no input). So
  a test of `assert_cmd` with no path gets the empty text (decision 3),
  and a test of the current folder uses `jag_std` with `Stdio::null()`.
- The Bash tool of Claude Code gives each command `/dev/null` as standard
  input. Source: checked on 2026-10-13. So an agent in that tool gets the
  current folder when it gives no path.
- GNU grep 3.12 and rg 15.1.0 write the error of a path that does not
  exist, search the other paths, and exit with 2. An empty pipe gives exit
  1. Source: checked on 2026-10-14 (decisions 3, 7, and 20).
- `grep-searcher` searches a file with `search_reader` when memory maps are
  off, and they are off by default. Source: ripgrep
  `crates/searcher/src/searcher/mod.rs` (`search_file_maybe_path`) and
  `crates/searcher/src/searcher/mmap.rs` (`MmapChoice::default` is
  `Never`). So `search_reader` on the piped text gives the binary rule and
  the byte order mark rule of a file.
- The `ignore` crate never skips a root, and each walk starts its own
  threads. Source: ignore 0.4.33 `src/walk.rs`: `skip_entry` gives false
  at depth 0. `WalkParallel::visit` puts each root on the stack of the
  workers (its comment "For files, we send to them the callback directly"
  is old), then starts `threads()` workers: the CPU count, at most 12, for
  0. A worker gives a root that is not a folder to the visitor with no
  filter (`Worker::run_one`).
- The `ignore` crate gives an error at depth 0 for a root that it cannot
  read, and only a root gets such an error from `fs::metadata`. Source:
  ignore 0.4.33 `src/walk.rs` (`WalkParallel::visit` calls
  `DirEntryRaw::from_path(0, path, false)`; on Unix, `from_path` calls
  `fs::metadata`, which follows a symbolic link, and gives
  `Error::Io(err).with_depth(depth).with_path(&pb)`; then `visit` gives the
  error to the visitor and goes on with the next root) and `src/lib.rs`
  (`Error::depth` is public, and `with_depth` and `with_path` are not). An
  error to open a root folder (`Work::read_dir`) also has depth 0, with
  the kind of that error.
- The one walk gives the skip counts of today, because `finish` counts the
  children of each folder that the walk gives. Source:
  `src/search/walk.rs` (`count_children` and `finish`).
- The walk joins the name of each child to the path of its folder, and a
  root has depth 0, so `entry.path().ancestors().nth(entry.depth())` is the
  root of a file. An error in the list of a folder has the path of the
  folder and the depth of its child, so this rule does not find the root
  of that error. Source: ignore 0.4.33 `src/walk.rs` (`DirEntryRaw::from_entry`
  takes the path of the `std::fs::DirEntry`, which is the folder joined
  with the name; `Work::read_dir` gives such an error
  `with_path(self.dent.path())` and `with_depth(self.dent.depth() + 1)`).
- `read_snippets` and the pre-rank keep the order of the windows.
  Source: `src/search/snippet.rs` (`read_snippets` groups only windows
  that are next to each other) and `src/prerank.rs` (the kept indices are
  sorted again).
- The `ignore` crate takes the root `-` as standard input. Source: ripgrep
  `crates/ignore/src/walk.rs` (`WalkParallel::visit` makes
  `DirEntry::new_stdin()` for `-`, and its `file_type` is `None`).
- Under the root `.`, the walk gives `./a.txt` for the file `a.txt`.
  Source: `src/search/walk.rs` (`FileMatches.path` is "The root joined
  with the rest of the path, as the walk gives it").
- jag runs only on Unix, so the test of decision 2 uses `std::os::unix`
  with no `cfg`. Source: `src/config/fs.rs` (it uses `std::os::unix` with
  no `cfg`).
- `io::ErrorKind::NotADirectory` and `std::io::pipe` are in Rust 1.99.
  Source: `Cargo.toml` (`rust-version = "1.99"`); the Rust release notes
  (1.83 and 1.87).
- `Read::read_to_end` tries again after `ErrorKind::Interrupted`. Source:
  the `std::io::Read` docs.

## Risks

1. A pipe that stays open and gives no text makes jag wait, as rg does. A
   tool that gives an empty pipe gets no match and exit 1, as rg gives
   (decision 3). Signal: a call of jag by an agent does not end, or it
   exits with 1 and searched nothing. Mitigation: the README tells an
   agent to give a path, for example `.`. A tool that gives `/dev/null` is
   safe.
2. A command that does not end, for example `tail -f`, makes jag wait,
   because jag ranks all results before it writes. Signal: jag shows
   nothing. Mitigation: the README says that jag reads all piped text
   first.
3. In structured mode, a document at the size limit of FEATURE-09 can
   leave out the warning of decision 7, because the results fit first
   (decision 17). Signal: a document with `exit_code` 2, `error` `null`,
   no warning of the kind of decision 7, and `warnings_left_out` above 0.
   Mitigation: `warnings_left_out` counts the warning, and the text mode
   writes each warning on the error stream, which the limit does not
   count.
4. Large piped text uses much memory (decision 13). Signal: the memory of
   jag grows with the size of the pipe. Mitigation: give the path of the
   file.
5. A key typed as a path shows on the error stream (decision 7). FEATURE-04
   spec decision 26 hides each value of an argument error, but CRITERION-70
   asks for the path, and the warnings of the walk name paths today.
   Signal: a key in a log of the error stream. Mitigation: no option takes
   a key (CV-6), and `--setup` reads the key with no echo.
6. A script expects exit 2 after a read error, as with `grep -r`
   (decision 10). Signal: the script reads exit 0 or 1 when a warning
   shows. Mitigation: the warning on the error stream, and the `warnings`
   field in structured mode.
7. A file that the user names two times gives the same result two times,
   and it costs two scores (decision 6). Signal: two equal results.
   Mitigation: give each path one time.
8. A merge conflict with the FEATURE-09 code or the FEATURE-13 code in
   `src/cli.rs` (decision 17, "Other features"). Signal: a conflict, or a
   test of one of the features that fails. Mitigation: the code that
   merges second joins the parts, and the tests of each feature stay.
9. A later change walks each root alone, for example to get the order of
   decision 6 in a simpler way. Each walk of the `ignore` crate starts its
   threads again, about 1.4 to 2.3 ms on Linux x86_64 with 12 CPUs. So a
   shell glob of 1,000 files takes 1.4 to 2.3 s, and one of 5,000 files
   takes 11.8 s, against 4.5 ms and 20 ms in one walk (FEATURE-16 spec,
   decision 12). Signal: check (c) of the FEATURE-16 speed test fails.
   Mitigation: keep one walk (decision 6).
10. A path with a typo costs the model calls of the other paths
    (decision 7). With `--ask`, jag scores the results of the other paths,
    and then it exits with 2. Signal: a run with exit 2 and a cost. Mitigation:
    in text mode, jag writes the warning before the first model call, so
    the user can stop jag with Ctrl-C. The cost limit of FEATURE-06 stays.

## Out of scope

- `-` as standard input: no story asks for it. It is a usage error
  (decision 8), so a later feature can add it and break no command that
  works.
- Ctrl-C while jag waits for a pipe: FEATURE-15 (STORY-43, CV-4). Today
  the default action of SIGINT stops jag.
- The plain flow and the option `--ask`: FEATURE-12, on `main` (PR #160).
  This spec changes only their input and their exit code (decisions 3, 7,
  and 20).
- The grep options of DECISION-34, and `-q`, `-l`, and `-c`: FEATURE-13
  (PR #153). See "Other features".
- The size limit, and the shorter path in an error document: FEATURE-09
  (decision 17).
- A root with two `/` or more at the end, for example `dir//`: no change
  (FEATURE-08 spec, "Assumptions").
- An order of the results of a question search by the command line: the
  output stays in rank order. Decision 6 gives only the order of the scan.
- Windows: jag runs only on Unix today.
