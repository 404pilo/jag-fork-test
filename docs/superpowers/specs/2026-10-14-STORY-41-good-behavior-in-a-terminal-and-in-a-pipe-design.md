---
title: "Good behavior in a terminal and in a pipe: progress only in a terminal, color as grep --color=auto, and a stop at once with no partial results"
feature: FEATURE-15
stories: [STORY-41, STORY-42, STORY-43]
kind: build
gh_issue: 102
lifecycle_state: draft
inherits: [IF-3, IF-4, IF-5, IF-6, C-cli, C-output, C-scorer, CV-1, CV-2, CV-3, CV-4]
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Good behavior in a terminal and in a pipe: progress only in a terminal, color as grep --color=auto, and a stop at once with no partial results

## Problem

jag must feel like grep (CAPABILITY-06). A person uses jag in a terminal.
A script or an agent uses it in a pipe. In a terminal, a person wants to
see that jag works while the model scores, and wants to see the matches in
color. In a pipe, the output must stay clean. When the user stops jag, or
the reader of the pipe stops, jag must stop and spend no more credit.

- STORY-41: a shell power user wants a progress line on the error stream
  only when that stream is a terminal, so that a pipe or a log holds no
  progress noise (DECISION-37).
  - CRITERION-62: while jag ranks, it shows a progress line on the error
    stream only when that stream is a terminal.
- STORY-42: a shell power user wants jag to color the matching text only
  in a terminal, with `--color=never` and `--color=always` to change that,
  so that color works as `grep --color=auto` (DECISION-38).
  - CRITERION-63: jag colors the matching text only in a terminal, and
    `--color=never` and `--color=always` change that.
- STORY-43: a shell power user wants jag to stop at once, start no new
  model call, and show no partial results when they press Ctrl-C or the
  pipe closes, so that a stopped search wastes no more credit
  (DECISION-39).
  - CRITERION-64: on Ctrl-C or a closed pipe, jag stops at once, starts no
    new model call, and shows no partial results.

The system design gives the rules of the two streams and of the stop. In
text mode, progress goes to the error stream. In structured mode,
everything goes to standard output, inside the document (CV-3). On Ctrl-C,
jag starts no new model call, writes no partial result, and exits with
130. When the reader closes the pipe, jag exits with no message (CV-4).

Today jag shows no progress and no color. It has no handler for SIGINT, so
the default action of the system stops it. jag keeps all output in memory
until the search ends (CV-2), so that stop writes no partial result. jag
finds a closed pipe only when it writes, after the last model call. Then
`emit` in `src/cli.rs` writes no message (FEATURE-11). So a reader that
closes the pipe while the model scores does not stop the paid calls.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | When the progress line shows | Only in text mode, only when the error stream is a terminal (`io::stderr().is_terminal()`), and only while the model scores: from the time before the first call of `scorer::score` to the end of its last call. Structured mode shows no progress. A search with no match makes no call, so it shows no line. `--check`, `--setup`, and `--forget-key` show no line, because they do not rank | DECISION-37 ("While jag ranks"; "In a pipe, jag shows no progress"); CRITERION-62; system-design.md CV-3; FEATURE-11 spec "Out of scope" ("Structured mode shows no progress") |
| 2 | Text of the progress line | `jag: the model scored K of N results`. N is the count of results that jag sends to the model. K is the count of results of the calls that ended with scores. jag writes the line with K = 0 before the first call. After each call that ends with scores, it writes the line again, after a `\r`. The line has no `\n` at the end. jag uses no timer | DECISION-37 ("a progress line"); `src/scorer.rs` (`score` sends calls of 32 results, 6 at the same time, so K grows 32 at a time); `src/cli.rs` (`read_snippets` drops the results of a file that jag cannot read, so N is the count of snippets and not `results_scored`) (inferred) |
| 3 | Removal of the progress line | When the scoring step ends, with scores or with a failure, jag removes the line before it writes more text to a stream. It writes `\r`, one space for each character of the line, and `\r`. It uses no escape code, so a terminal with `TERM=dumb` shows no stray text. A guard removes the line on each path out of the scoring step (`Drop`) | DECISION-37 ("jag removes the line when the results come"); system-design.md CV-1 (a failure message must not start at the end of the progress line) (inferred) |
| 4 | Code of the progress line | C-scorer writes no text. `score` gets the callback `on_call_end: &(dyn Fn(usize) + Sync)` with the count of results of each call that ends with scores. C-output gets the new module `src/output/progress.rs` with `Progress`: a `Mutex` around its writer and its state, so that the 6 threads of `score` and a stop can use it. C-cli makes the `Progress` and connects the callback | 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md; system-design.md C-output ("writes the notices and the cost line"; `depends_on: []`) and C-scorer (its `depends_on` holds no C-output) (inferred) |
| 5 | The option `--color` | `--color[=WHEN]`. WHEN is `never`, `always`, or `auto`. `--color` with no value means `auto`. A value needs `=`: in `--color never`, `never` is the TERM, as in grep. With no `--color`, the choice is `auto`. Here jag is not the same as grep with no option, which never colors: DECISION-38 makes `auto` the default. `--colour` is an alias, as in grep. When the option shows two times or more, the last one wins, as in grep. Another WHEN is an argument error: exit 2 and the one usage line, or the argument error document with `--json`. grep 3.12 shows its help and exits with 0 for a bad WHEN. jag does not, because 2 is a usage error. `--color` conflicts with `--setup`, `--forget-key`, and `--check`. `--color` is not in the list of DECISION-34: DECISION-38 adds it, as DECISION-27 adds `--json` | DECISION-38; CRITERION-63; system-design.md IF-3 and IF-4; FEATURE-04 spec decision 26 and FEATURE-11 spec decision 22 (argument errors); `src/cli.rs` (`--json` conflicts with the same three options); GNU grep 3.12 (`grep --help`: "--color[=WHEN], --colour[=WHEN] ... WHEN is 'always', 'never', or 'auto'"; checked on 2026-10-14: `--color` alone is `auto`, the last option wins, `--color=bad` shows the help with exit 0); clap docs (`default_missing_value` with `num_args(0..=1)` and `require_equals(true)`; `overrides_with` its own name) |
| 6 | The rule of `auto` | jag colors when standard output is a terminal (`io::stdout().is_terminal()`) and the variable `TERM` is set and is not `dumb`. An empty `TERM` colors. jag does not read `NO_COLOR`. This is the rule of GNU grep | DECISION-38 ("as `grep --color=auto` does"); GNU grep 3.12, checked in a terminal (`script`) on 2026-10-14: `TERM=xterm` and an empty `TERM` color, `TERM=dumb` and no `TERM` do not color, and `NO_COLOR=1` colors (inferred) |
| 7 | What jag colors | The items that GNU grep colors with its default `GREP_COLORS` (`ms=01;31:mc=01;31:sl=:cx=:fn=35:ln=32:bn=32:se=36`): each match on a matching line in bold red (`01;31`), the path or `(standard input)` in magenta (`35`), the line number in green (`32`), and the separators `:` and `-` and the line `--` in cyan (`36`). Each colored item starts with `ESC[<code>m` and `ESC[K`, and it ends with `ESC[m` and `ESC[K`, byte for byte as grep. An empty match gets no color. A line around the match gets no match color. The label line `jag: no good answer. ...` and the cut mark ` [+N chars]` get no color, because grep has no such text. jag does not read `GREP_COLORS`. Rejected: color only the matching text, the words of the body of DECISION-38. The title of DECISION-38 and CRITERION-63 say "Color works as grep --color=auto", and grep colors all these items | DECISION-38 (title); CRITERION-63 (title); CAPABILITY-06 ("Feels like grep"); GNU grep manual (the default of `GREP_COLORS`); the bytes of GNU grep 3.12 with `--color=always -n -H`, checked with `od -c` on 2026-10-14 (inferred) |
| 8 | How jag finds the matches to color | When it writes the text, C-output gets the byte ranges of the matches of each matching line from a function that C-cli gives. That function runs the same `RegexMatcher` that found the matches (`grep_matcher::Matcher::find_iter`) on the kept bytes of the line. IF-6 does not change. On a cut line, a match that ends at the last kept byte gets no color, because the end of the kept bytes is not the end of the line (`$` or `\b` can match there) | `src/search/snippet.rs` (`SnippetLine` holds `is_match` and no position of a match; `MAX_LINE_CHARS` is 300); system-design.md IF-6 (inferred) |
| 9 | Color in structured mode | No color. `--color` with `--json` is not an error, and it has no effect: the document never holds an escape code | FEATURE-11 spec decision 23 (each option gives its structured form or refuses `--json`); system-design.md IF-5; ripgrep 15.1.0 `rg --help` ("Other flags that control aspects of the standard output ... have no effect when --json is set"), and `rg --json --color=always` writes no escape code (checked on 2026-10-14) (inferred) |
| 10 | Code of color | C-cli parses `--color` into `ColorWhen`, decides with `color_on(when, stdout_is_terminal, term)`, and makes the function of decision 8. C-output: `write_text` gets `paint: Option<&dyn Fn(&[u8]) -> Vec<Range<usize>>>`, and `None` gives the text of today, byte for byte. So C-output does not use the matcher | 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md; system-design.md C-cli ("Reads the command and the grep options") and C-output (`depends_on: []`) (inferred) |
| 11 | Where jag handles SIGINT | Only in the search flow. Before the search flow starts, `run` makes a `signal_hook::iterator::Signals` for SIGINT and starts one thread that waits for it (the stop thread). `--setup` keeps its own handler. `--check` (decision 16) and `--forget-key` keep the default action. Before the handler is in place, the default action applies, with the same result for the user: no output, and the end by SIGINT. When the handler cannot be made, the search goes on with the default action | system-design.md CV-4 (C-cli); `src/config/setup.rs` (`TtyPrompter::secret` owns SIGINT while `rpassword` reads the key); `Cargo.toml` (signal-hook 0.4.5, with its default feature `iterator`) (inferred) |
| 12 | What a Ctrl-C does | In this order: (a) the stop thread sets the stop flag, so no model call and no retry starts after it; (b) it takes the output (decision 14); (c) in text mode, when the error stream is a terminal, it removes the progress line and the `^C` that the terminal shows. Then it writes the failure `jag: the search was stopped.` and `To get the results, run the search again.` to the error stream. In structured mode, it writes the error document with `exit_code` 130 to standard output, and nothing to the error stream. The summary fields hold what jag counted before the stop; (d) it ends jag (decision 13). jag does not wait for the calls that run: the end of the process closes their connections. So jag stops at once, and no call starts after the stop | DECISION-39; CRITERION-64; system-design.md CV-1, CV-2, and CV-4; DECISION-44 ("When a search fails or the user stops it, the error message gives the cost"; FEATURE-20 adds the cost); FEATURE-11 spec decision 21 (the error document) and its "Out of scope" ("Ctrl-C, its document, and exit 130: FEATURE-15") |
| 13 | How jag ends after a Ctrl-C | With the default action of SIGINT: `signal_hook::low_level::emulate_default_handler(SIGINT)`, as grep ends. A shell shows 130 (128 + 2). A shell loop or a script that runs jag also stops, as with grep. A program that starts jag with no shell gets the signal SIGINT, not the code 130, as with grep. The document of decision 12 holds `exit_code` 130. Rejected: `exit(130)`. Then a shell thinks that jag handled the Ctrl-C, and a loop or a script goes on to the next command, which can start a new search that costs credit. Checked on 2026-10-14: a `bash -c` loop of 3 runs, stopped with SIGINT, ran all 3 runs when the child exited with 130, and stopped at once when the child ended by SIGINT | DECISION-39 ("It exits as grep does: with code 130 on Ctrl-C"); system-design.md IF-4 and CV-4, and 0007-number-the-exit-codes-above-2-from-3-to-8.md ("Ctrl-C gives 130"): a shell shows 130 for an end by SIGINT; STORY-43 ("so that a stopped search wastes no more credit"); FEATURE-04 spec "Risks" (a program sees the signal, not the code 130: now by design); signal-hook 0.4.5 source (`emulate_default_handler` puts back the default action and raises the signal again) |
| 14 | One output | The output has one owner: the search flow or a stop (decision 12 or 15). The first one that takes it keeps it. A stop takes it before it writes. The search flow takes it at the place where it ends: before it writes the results or a failure, and before it ends with no output. When a stop owns the output, the search flow writes nothing, also no failure, and it waits for the end of the process. When `score` gives `ScoreError::Stopped`, the search flow also waits, because only a stop sets the flag. When the search flow owns the output, a later Ctrl-C ends jag at once, with no text. The output can then be cut, as with grep, and a cut document does not parse. In text mode, the search flow writes a notice to the error stream only when no stop owns the output, and a stop waits while a notice is written. So a notice never comes after the stop failure | system-design.md CV-2 and CV-4 (no partial result); DECISION-30 and 0008-write-structured-output-as-one-json-document.md ("A cut document does not parse, so an agent sees that jag stopped"); `src/cli.rs` (`search` writes its notices before the model calls, and `search` and `finish` write the output one time, at the end) (inferred) |
| 15 | A closed pipe | While the search flow runs, jag watches standard output when it is a pipe or a socket. A thread (the watch thread) asks the system every 100 ms if the reader closed it (`poll`). When the reader closed it, the watch thread: (a) sets the stop flag, so no model call and no retry starts after it; (b) takes the output (decision 14); (c) removes the progress line when it shows, and writes no message; (d) ends jag with the default action of SIGPIPE (`emulate_default_handler(SIGPIPE)`), as grep ends when it writes to a closed pipe. A shell shows 141 (128 + 13). In structured mode, no document comes, because no reader is there (IF-5: an output that does not parse means that jag stopped). When the search flow already owns the output, the watch thread does nothing more: `emit` finds the closed pipe when it writes, and it gives the code of the search with no message, as FEATURE-11 built. jag does not watch a terminal, a regular file, or `/dev/null`. Rejected: find the closed pipe only when jag writes, as grep. grep writes each line when it finds it, so it finds a closed pipe soon. jag keeps all output until the end (CV-2), so it would pay for all model calls after the reader closed the pipe | DECISION-39 ("When the user stops jag, or the pipe closes, jag stops at once. It starts no new model call"; "It exits as grep does: ... with no message when the pipe closes"); CRITERION-64; STORY-43 ("so that a stopped search wastes no more credit"); system-design.md CV-2, CV-4, and IF-5; FEATURE-11 spec decision 25 and `src/cli.rs` (`emit`); signal-hook 0.4.5 source (`emulate_default_handler` ends the process for SIGPIPE as for SIGINT); `poll` on Linux, checked on 2026-10-14 (inferred) |
| 16 | Ctrl-C and a closed pipe during `--check` | The default action of SIGINT, with no handler, and no watch of the pipe. The check writes all its lines after its one call. So a Ctrl-C during the call gives no output, starts no new call, and ends jag by SIGINT (a shell shows 130). A closed pipe stops no call, because the check makes only one, and the write after the call gives no message and keeps the exit code, as FEATURE-02 built. The check has no progress line and no document, and DECISION-44 is about a search. This spec adds the end-to-end test of a Ctrl-C | FEATURE-02 spec decision 21 ("A reader that closes the pipe gives no message and keeps the exit code") and its "Out of scope" ("Ctrl-C during the call, and exit 130: FEATURE-15"); DECISION-39; system-design.md CV-4; `src/cli.rs` (`check` writes after `check::run`) |
| 17 | Code of the stop and of the watch | C-cli: the new module `src/cli/stop.rs` with `Stop` (the flag, the owner of the output, a copy of the counts for the document, the mode, and the `Progress`), `stop::install` (the stop thread), `stop::watch_pipe` (the watch thread), `Stop::on_signal`, and `Stop::on_closed_pipe`. C-scorer: `score` gets `stop: &AtomicBool`. It checks the flag with its own stop flag, before each call and before each retry. When the flag is set at the end of the threads, `score` gives `ScoreError::Stopped`. `probe` gives a flag that is never set. The test of a pipe or a socket uses only `std` (`BorrowedFd::try_clone_to_owned`, `File::metadata`, and `std::os::unix::fs::FileTypeExt`), as FEATURE-14 tests standard input. `poll` comes from one new crate: `rustix` 1.1.5 with the feature `event`. Its `poll` is a safe function, so jag keeps `unsafe_code = "forbid"`. `rustix` 1.1.5 is already in `Cargo.lock`, through `tempfile`. `tempfile` is a dev-dependency, so `rustix` is new in the release binary. The pull request of the code gives the reason for the crate. No ADR: 0012 asks only for a reason | system-design.md C-cli (CV-4 governs it) and C-scorer; 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md; 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md ("Each new outside crate needs a reason in the pull request"); `Cargo.toml` (signal-hook 0.4.5; `unsafe_code = "forbid"`); `Cargo.lock` (rustix 1.1.5); rustix 1.1.5 docs (`event::poll` and `PollFd::new` are safe functions of the feature `event`); FEATURE-14 spec decision 14 (PR #142, not merged) (inferred) |
| 18 | Other features | FEATURE-14 (Ctrl-C while jag waits for a pipe, the color of `(standard input)`, and the helper `jag_std`), FEATURE-11 (structured mode, and the two ends after a closed pipe), FEATURE-09 (color codes count toward the size limit), FEATURE-13 (the matcher and the grep options), FEATURE-12, and FEATURE-20 (the cost of a stop): see "Other features". This spec decides none of their points again | FEATURE-14 spec (PR #142, not merged); FEATURE-11 spec and its code on `main` (7150e83); FEATURE-09 spec (PR #134, not merged); DECISION-22; DECISION-34; DECISION-41; DECISION-44 |
| 19 | Tests in a terminal | The terminal path of decisions 1, 3, 6, and 12 has unit tests, with the terminal tests and `TERM` as parameters. The end-to-end tests run jag in pipes only. jag adds no crate for a pseudo-terminal | FEATURE-01 spec (the terminal path of `--setup` has unit tests with a fake prompter); `Cargo.toml` (the test crates of 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md) (inferred) |
| 20 | Docs | The usage line gets `[--color[=WHEN]]`. The help of `--color` says: "When to color the matches: never, always, or auto (the default: only in a terminal)". The README gets the section "In a terminal and in a pipe" | docs/superpowers/specs/2026-10-13-STORY-06-readiness-check-design.md (a README section for each command); FEATURE-11 spec decision 29 (inferred) |
| 21 | Spec grouping | One spec for STORY-41, STORY-42, and STORY-43 | docs/adlc.md (one feature gives one spec); the orchestrator TASK of 2026-10-14 |

## Design

### Files

- `Cargo.toml`: `rustix = { version = "1.1.5", features = ["event"] }`.
- `src/scorer.rs` (C-scorer): `score` gets the stop flag and the
  callback. `ScoreError::Stopped`.
- `src/output.rs` (C-output): `write_text` gets `paint`. The new module
  `src/output/progress.rs` holds `Progress`.
- `src/cli.rs` (C-cli): the option `--color`, `color_on`, the progress
  line, and the one output. The new module `src/cli/stop.rs` holds `Stop`,
  the stop thread, and the watch thread.
- `tests/common/mod.rs`: `jag_std`, a `std::process::Command` with the
  variables of `jag`. FEATURE-14 (PR #142) names the same helper. The code
  that merges first adds it.
- `tests/terminal.rs` (new): the progress line and color, end to end.
- `tests/stop.rs` (new): Ctrl-C and the closed pipe, end to end.
- `tests/score.rs`: the stop flag and the callback of `score`.
- `README.md`: the section "In a terminal and in a pipe".

### The progress line (C-output, C-scorer, C-cli)

`src/output/progress.rs`:

```rust
/// The progress line of the error stream (DECISION-37). It writes only
/// when it is on: in text mode, when the error stream is a terminal.
pub struct Progress {
    state: Mutex<State>,
}

struct State {
    out: Box<dyn Write + Send>,
    on: bool,
    total: usize,
    done: usize,
    /// The count of characters of the line on the screen. 0 for no line.
    width: usize,
    /// The count of characters of the longest line since `start`. `clear`
    /// does not change it.
    widest: usize,
}

impl Progress {
    pub fn new(out: Box<dyn Write + Send>, on: bool) -> Progress;
    /// Write `jag: the model scored 0 of N results`.
    pub fn start(&self, total: usize);
    /// Add the results of one call, and write the line again after `\r`.
    pub fn advance(&self, count: usize);
    /// Remove the line: `\r`, `width` spaces, `\r`. No text when no line
    /// shows.
    pub fn clear(&self);
    /// For a Ctrl-C: `\r`, `widest + 2` spaces, `\r`, also when no line
    /// shows, so that the `^C` of the terminal goes too.
    pub fn clear_for_stop(&self);
    /// For a closed pipe: `clear`, then `on` is false.
    pub fn stop(&self);
}
```

When `on` is false, each method writes nothing. `clear_for_stop`, and
`clear` when a stop calls it, also set `on` to false, so that a call that
ends after a stop writes no line. `Progress` gets `pub fn stop(&self)`: it
does `clear` and sets `on` to false. `on_closed_pipe` calls `stop` in
place of `clear`. jag ignores a write error
of the progress line, as it ignores a write error of a notice today
(`let _ = writeln!(stderr, ...)` in `src/cli.rs`). K only grows, so the
new text is never shorter than the old text. `clear_for_stop` uses
`widest`, because the terminal shows the `^C` after the longest line, also
when `clear` removed the line first.

`run` makes the `Progress` for the search flow:

```rust
let progress = Arc::new(Progress::new(
    Box::new(io::stderr()),
    mode == Mode::Text && io::stderr().is_terminal(),
));
```

The scoring step in `search`:

```rust
progress.start(snippets.len());
let scores = {
    let _clear = ClearOnDrop(&progress);
    score(&snippets, args.question, &key, &score_settings, stop.flag(), &|count| {
        progress.advance(count)
    })
};
```

`ClearOnDrop` calls `Progress::clear` in `drop`. So each path out of the
scoring step removes the line before the next text: the results, a
failure, and the later notices of FEATURE-09 and FEATURE-20.

In `src/scorer.rs`, after a call ends with scores, the thread calls
`on_call_end(batch.range.len())`. A failed call and a call that never
started do not call it.

### Color (C-cli, C-output)

The option, in `Cli`:

```rust
/// When to color the matches: never, always, or auto (the default: only in a terminal)
#[arg(
    long,
    value_name = "WHEN",
    alias = "colour",
    num_args = 0..=1,
    require_equals = true,
    default_missing_value = "auto",
    value_parser = ["never", "always", "auto"],
    overrides_with = "color",
    conflicts_with_all = ["setup", "forget_key", "check"]
)]
color: Option<String>,
```

`None` (no option) is `auto`. The clap error of a bad WHEN goes through
`parse`, as each argument error does today, so it shows no value.

The choice:

```rust
/// True when the text output gets color (decisions 5 and 6).
fn color_on(when: ColorWhen, stdout_is_terminal: bool, term: Option<&OsStr>) -> bool {
    match when {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => stdout_is_terminal && term.is_some_and(|term| term != "dumb"),
    }
}
```

`run` reads `io::stdout().is_terminal()` one time, and `search` reads
`TERM` through `Env.vars`. In structured mode, jag never calls `color_on`.
`write_json` does not change.

The function of decision 8, in `search`:

```rust
let find = |line: &[u8]| {
    let mut ranges = Vec::new();
    // A `RegexMatcher` gives no error (`NoError`).
    let _ = matcher.find_iter(line, |found| {
        ranges.push(found.start()..found.end());
        true
    });
    ranges
};
```

`write_text(out, ranked, paint: Option<&dyn Fn(&[u8]) -> Vec<Range<usize>>>)`.
The search flow gives `Some(&find)` when `mode == Mode::Text` and
`color_on(...)` is true, and `None` in all other cases. With `Some`, each
line is:

- The path or `(standard input)` in `35`, the separator in `36`, the line
  number in `32`, the separator in `36`, and then the text.
- On a matching line, each non-empty range of `paint` is in `01;31`. On a
  cut line (`cut > 0`), a range that ends at the end of the kept bytes gets
  no color.
- The cut mark ` [+N chars]` and the label line come with no color.
- The line `--` between two results is in `36`.

Each colored item is `ESC[<code>m ESC[K <item> ESC[m ESC[K` (no spaces).
Example for the file `a.txt` with the line `x needle y` and the term
`needle` (the bytes of GNU grep 3.12 for `grep --color=always -n -H needle
a.txt`):

```
\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K\x1b[32m\x1b[K1\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[Kx \x1b[01;31m\x1b[Kneedle\x1b[m\x1b[K y\n
```

### The stop and the watch (C-cli, C-scorer)

`src/cli/stop.rs`:

```rust
/// Who owns the output (decision 14).
enum Owner {
    Nobody,
    Search,
    Stop,
}

/// The stop of a search on SIGINT or on a closed pipe (CV-4, DECISION-39).
pub struct Stop {
    /// Set first by a stop: no model call and no retry starts after it.
    flag: AtomicBool,
    owner: Mutex<Owner>,
    /// What jag counted, for the document of a stop in structured mode.
    counted: Mutex<Summary>,
    mode: Mode,
    progress: Arc<Progress>,
}

impl Stop {
    pub fn new(mode: Mode, progress: Arc<Progress>) -> Stop;
    pub fn flag(&self) -> &AtomicBool;
    /// True when the search flow owns the output. It takes the output when
    /// nobody owns it, and a second call gives true again. False when a
    /// stop owns the output.
    pub fn claim_for_search(&self) -> bool;
    /// Write one notice of the search flow to the error stream, only when
    /// no stop owns the output. It holds the lock of `owner` while it
    /// writes.
    pub fn notice(&self, stderr: &mut dyn Write, text: &str);
    /// The search flow calls it after it counts (the place where it sets
    /// `files_skipped` and the `results_*` fields today).
    pub fn count(&self, summary: &Summary);
    /// Steps (a) to (c) of decision 12.
    pub fn on_signal(&self, stdout: &mut dyn Write, stderr: &mut dyn Write);
    /// Steps (a) to (c) of decision 15. True when this stop took the
    /// output, so that the caller ends jag. False when the search flow or
    /// a Ctrl-C owns the output.
    pub fn on_closed_pipe(&self) -> bool;
}

/// Register SIGINT and start the stop thread. The thread waits for the
/// signal, calls `on_signal` with the real streams, and then calls
/// `emulate_default_handler(SIGINT)`.
pub fn install(stop: &Arc<Stop>) -> io::Result<()>;

/// When standard output is a pipe or a socket, start the watch thread. It
/// calls `wait_for_closed_reader` with 100 ms. When that gives true and
/// `on_closed_pipe` gives true, it calls `emulate_default_handler(SIGPIPE)`.
pub fn watch_pipe(stop: &Arc<Stop>);

/// True when the stream is a pipe or a socket (`is_fifo` or `is_socket` of
/// the metadata of a clone of the descriptor). Each error gives false.
fn is_pipe_or_socket(fd: BorrowedFd<'_>) -> bool;

/// Every `every`, call `rustix::event::poll` on `fd` with `PollFlags::OUT`
/// and a zero timeout. True when `revents` holds `ERR` or `HUP`: the reader
/// closed the pipe or the socket. False when `poll` fails with an error
/// other than `INTR`: then jag watches no more.
fn wait_for_closed_reader(fd: BorrowedFd<'_>, every: Duration) -> bool;
```

The stop failure is a `Failure` with the cause `the search was stopped.`,
the fix `To get the results, run the search again.`, and the exit code
130. `on_signal` writes it with `Failure::render` in text mode, and with
the error document of `src/cli.rs` (`error_document`) in structured mode.
In text mode, `on_signal` calls `Progress::clear_for_stop` first. That
call writes nothing when the error stream is not a terminal, because the
`Progress` is then off. `on_closed_pipe` calls `Progress::stop` and
writes nothing else.

`run`, for the search flow:

```rust
let stop = Arc::new(Stop::new(mode, Arc::clone(&progress)));
// With no handler, the default action of SIGINT stays (decision 11).
let _ = stop::install(&stop);
stop::watch_pipe(&stop);
let result = search(&cli, &env, mode, &stop, &progress, &mut summary, &mut stdout, &mut stderr);
if !stop.claim_for_search() {
    wait_for_the_end();
}
finish(result, mode, &mut summary, &mut stdout, &mut stderr)
```

`wait_for_the_end` calls `std::thread::park` in a loop and never returns:
the stop thread or the watch thread ends the process. In `search`, each
notice goes through `stop.notice`, and `search` calls
`stop.claim_for_search()` before each `emit`. When `score` gives
`ScoreError::Stopped`, `search` calls `wait_for_the_end`: it never writes
that failure.

In `src/scorer.rs`, the loop of each thread runs while neither the stop
flag of the run nor the flag of `Stop` is set. `Run::call` checks both
flags before the retry and after the retry wait. When the flag of `Stop`
is set after the threads end, `score` gives `ScoreError::Stopped`, whose
failure is the stop failure.

The order of a Ctrl-C while the model scores, in text mode, with the error
stream in a terminal:

1. The stop thread sets the flag. No thread starts a call or a retry.
2. It takes the output.
3. It removes the progress line and the `^C`.
4. It writes `jag: the search was stopped.` and the fix.
5. It ends jag by SIGINT. The calls that run end with the process.

The order when the reader closes the pipe while the model scores:

1. Within 100 ms, `poll` gives `ERR` or `HUP` to the watch thread.
2. The watch thread sets the flag. No thread starts a call or a retry.
3. It takes the output, and removes the progress line when it shows.
4. It ends jag by SIGPIPE, with no message. The calls that run end with
   the process.

A Ctrl-C or a closed pipe at another time of the search, for example
during the walk, does the same steps. The flag then stops nothing, because
no call runs.

### Other features

- FEATURE-14 (spec PR #142, not merged): the stop thread also stops jag
  while it waits for piped text. A Ctrl-C then gives the stop of decision
  12. The label `(standard input)` gets the path color. FEATURE-14 tests
  standard input. This spec tests the error stream and standard output.
  `is_pipe_or_socket` reads the file type of standard output as
  `stdin_can_give_text` of FEATURE-14 reads the file type of standard
  input. Both specs name `jag_std` in `tests/common/mod.rs`. The code that
  merges second shares these parts, and it adds the end-to-end test of a
  Ctrl-C while jag waits for piped text: the end by SIGINT, the stop
  failure, and no output.
- FEATURE-11 (code on `main`, 7150e83): structured mode has no progress
  line and no color. The stop document of a Ctrl-C is the error document
  of FEATURE-11 decision 21. A closed pipe now has two ends: before the
  search flow writes, the end by SIGPIPE with no document (decision 15);
  during the write, the code of the search with no message, as FEATURE-11
  built.
- FEATURE-09 (spec PR #134, not merged): color codes are characters on
  standard output, so they count toward the size limit (DECISION-22). The
  FEATURE-09 spec and the FEATURE-08 spec name FEATURE-13 for color. The
  roadmap gives color to STORY-42 of this feature. The code that merges
  second makes the size limit count the colored text that `write_text`
  gives. The notices of FEATURE-09 come after the progress line is removed
  (decision 3), and they go through `Stop::notice` (decision 14).
- FEATURE-13 (the grep options of DECISION-34): `--color` is not one of
  them. The usage error for an unknown option (STORY-37) must accept
  `--color` and `--colour`. `-i`, `-w`, `-F`, and `-e` change the matcher,
  and decision 8 colors with that same matcher. The feature that adds `-l`
  and `-c` decides the color of their output.
- FEATURE-12 (plain grep with no question): it writes its results with
  `write_text`, so it gets the same color. It makes no model call, so it
  shows no progress line.
- FEATURE-20 (the cost): the cost line comes after the progress line is
  removed (decision 3). DECISION-44 adds the cost that jag paid to the stop
  failure and to the stop document. The cost count must then be where the
  stop thread can read it, for example in `Stop`.
- FEATURE-21 (spec PR #143, not merged): it changes the settings of the
  model service, and `score` keeps its new parameters.

### The README section "In a terminal and in a pipe"

- In a terminal, jag shows `jag: the model scored K of N results` on the
  error stream while the model scores, and removes it when the results
  come. In a pipe or a file, it shows no progress.
- jag colors the matches, the paths, the line numbers, and the separators
  as `grep --color=auto` does: only when the output is a terminal and
  `TERM` is not `dumb`. `--color=always` colors also in a pipe, for
  example for `less -R`. `--color=never` never colors.
- Ctrl-C stops jag at once. jag starts no new model call and shows no
  results. A shell shows the exit code 130. With `--json`, the output is an
  error document with `exit_code` 130.
- When the reader of the output closes the pipe, jag stops at once, starts
  no new model call, and exits with no message.

### Acceptance criteria and their tests

The end-to-end tests use a temporary `$HOME` and the fake server. A test
that acts while jag runs uses `jag_std` of `tests/common/mod.rs`, because
`assert_cmd` cannot send a signal to a running child. It sends SIGINT with
the `kill` program, and it reads the signal with
`std::os::unix::process::ExitStatusExt`. Each stop test runs jag with
`--timeout 60 --retry-wait 60`, so that no call times out and no retry
starts while the server holds the calls.

- CRITERION-62 (progress only in a terminal):
  - Unit (`progress.rs`): on, N = 70. `start(70)` writes
    `jag: the model scored 0 of 70 results`. `advance(32)` writes
    `\rjag: the model scored 32 of 70 results`. After a second `advance(32)` and then
    `advance(6)`, the last text is `\rjag: the model scored 70 of 70
    results`. `clear()` then writes `\r`, 38 spaces, and `\r`. A second
    `clear()` writes nothing.
  - Unit (`progress.rs`): off. Each method writes nothing.
  - Unit (`progress.rs`): 6 threads call `advance(1)` 100 times each. The
    last text has K = 600.
  - Unit (`progress.rs`): on, after a line of 38 characters and `clear()`,
    `clear_for_stop()` writes `\r`, 40 spaces, and `\r`. With no line
    since `start`, it writes `\r`, 2 spaces, and `\r`. Off, it writes
    nothing.
  - Unit (`progress.rs`): on, N = 70. After `start(70)`, `clear_for_stop()`, and `advance(32)`, the text after `clear_for_stop` holds no `jag: the model scored`. The same after `stop()`.
  - Integration (`tests/score.rs`): `score` of 200 snippets with the fake
    server calls `on_call_end` 7 times. The counts are 32 six times and 8
    one time, in any order. A call that fails does not call it.
  - Unit (`src/cli.rs`): `ClearOnDrop` removes the line when the scoring
    step gives an error, before the failure text.
  - End-to-end (`tests/terminal.rs`): a search with 70 results. The error
    stream is a pipe. It holds no `\r` and no `the model scored`. Exit 0.
  - End-to-end: the same search with `--json`. The error stream is empty.
- CRITERION-63 (color as `grep --color=auto`):
  - Unit (`src/cli.rs`): `color_on` gives true for (`Always`, false,
    no `TERM`), (`Auto`, true, `xterm`), and (`Auto`, true, empty). It
    gives false for (`Never`, true, `xterm`), (`Auto`, true, `dumb`),
    (`Auto`, true, no `TERM`), and (`Auto`, false, `xterm`).
  - Unit (`output.rs`): with a `paint` that finds `needle`, a result of
    `a.txt` with line 1 `x needle y` (a match) and line 2 `ctx` gives
    exactly the line of the example above, and then
    `\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\x1b[36m\x1b[K-\x1b[m\x1b[K\x1b[32m\x1b[K2\x1b[m\x1b[K\x1b[36m\x1b[K-\x1b[m\x1b[Kctx\n`.
    Two results have `\x1b[36m\x1b[K--\x1b[m\x1b[K\n` between them.
  - Unit (`output.rs`): two matches on one line give two colored items.
    An empty range gives no colored text. `(standard input)` is in `35`.
    On a cut line, a range that ends at the last kept byte has no color,
    and the cut mark has no color. The label line has no color.
  - Unit (`output.rs`): with `None`, each test of today passes with no
    change.
  - Unit (`src/cli.rs`): the parse of `--color` gives `auto` for no
    option, for `--color`, and for `--color=auto`. It gives `never` for
    `--color=always --color=never`. `--colour=always` gives `always`.
    `--color=sometimes` is an error.
  - End-to-end (`tests/terminal.rs`): `--color=always` in a pipe, run in
    the folder of `a.txt` with the path `a.txt` and a server that gives
    the score 0.9, gives exactly the bytes of the example above on
    standard output. Exit 0.
  - End-to-end: no option, `--color`, `--color=auto`, `--color=never`, and
    `--color=always --color=never`, each in a pipe, give exactly
    `a.txt:1:x needle y\n`.
  - End-to-end: `--colour=always` gives the same bytes as `--color=always`.
  - End-to-end: `--color=sometimes` gives exit 2, the one usage line, and
    no call. With `--json`, it gives the argument error document.
  - End-to-end: `--color=always` with `--setup`, with `--check`, and with
    `--forget-key` gives exit 2.
  - End-to-end: `--json --color=always` gives a document with no byte
    0x1B, and its first line text is `x needle y`.
- CRITERION-64 (a stop at once, with no new call and no partial result):
  - End-to-end (`tests/stop.rs`, Ctrl-C, text mode): 200 files with one
    match each. The fake server holds each call for 30 s, so 7 calls are
    needed and 6 run at the same time. The test waits until the server has
    6 calls (at most 10 s), then sends SIGINT. jag ends in less than 5 s.
    `signal()` is `Some(2)` and `code()` is `None`. Standard output is
    empty. The error stream is exactly `jag: the search was
    stopped.\nTo get the results, run the search again.\n`. One second
    after the end, the server still has 6 calls.
  - End-to-end (Ctrl-C, structured mode): the same with `--json`. Standard
    output is one document with `exit_code` 130, the `error` of the stop
    failure, `results` `[]`, `results_found` 200, and `results_scored`
    200. The error stream is empty. `signal()` is `Some(2)`. The server
    has 6 calls.
  - End-to-end (Ctrl-C, `--check`): the server holds its one call for
    30 s. After the call comes, the test sends SIGINT. jag ends in less
    than 5 s, with `signal()` `Some(2)`, an empty standard output, an
    empty error stream, and 1 call.
  - End-to-end (closed pipe, text mode): the same 200 files and server.
    Standard output is a pipe. The test waits until the server has 6
    calls, then closes its end of the pipe. jag ends in less than 5 s.
    `signal()` is `Some(13)`. The error stream is empty. One second after
    the end, the server still has 6 calls.
  - End-to-end (closed pipe, structured mode): the same with `--json`.
    `signal()` is `Some(13)`, the error stream is empty, and the server
    has 6 calls.
  - Unit (`stop.rs`): `claim_for_search` gives true, and true again.
    After it, `on_signal` sets the flag and writes nothing, and
    `on_closed_pipe` gives false.
  - Unit (`stop.rs`): after `on_signal` (when nobody owned the output),
    `claim_for_search` gives false, and `notice` writes nothing.
  - Unit (`stop.rs`): `on_signal` in text mode, with the `Progress` off,
    writes exactly the two lines of the stop failure to the error stream
    and nothing to standard output. With the `Progress` on and a line of
    38 characters, it first writes `\r`, 40 spaces, and `\r`.
  - Unit (`stop.rs`): `on_signal` in structured mode writes one document
    with `exit_code` 130 and the counts of `count` to standard output, and
    nothing to the error stream.
  - Unit (`stop.rs`): `on_signal` sets the flag before its first write. A
    test writer reads the flag when it gets its first byte.
  - Unit (`stop.rs`): `on_closed_pipe`, when nobody owns the output, sets
    the flag, removes the progress line, writes nothing else, and gives
    true.
  - Unit (`stop.rs`): `is_pipe_or_socket` gives true for the writer of
    `std::io::pipe()` and for one end of `UnixStream::pair()`. It gives
    false for a file of `tempfile` and for `/dev/null`.
  - Unit (`stop.rs`): `wait_for_closed_reader` on the writer of
    `std::io::pipe()` gives true less than 1 s after the test drops the
    reader. While the reader is open, it has not returned after 500 ms.
    Then the test drops the reader, and the call gives true, so that no
    thread of the test runs on.
  - Integration (`tests/score.rs`): with the flag set before `score`,
    `score` makes no call and gives `ScoreError::Stopped`. With the flag
    set while the first 6 calls run (the server holds each call for 2 s),
    no 7th call comes, and `score` gives `ScoreError::Stopped`.

## Build order

1. C-scorer: the parameters `stop` and `on_call_end` of `score`, and
   `ScoreError::Stopped`. `probe` and the search flow give a flag that is
   never set and a callback that does nothing. No behavior changes, and
   all tests pass. Then the `tests/score.rs` tests.
2. STORY-41: `src/output/progress.rs`, `ClearOnDrop`, and the connection
   in the search flow. Then the tests of CRITERION-62.
3. STORY-42: the option `--color`, `color_on`, and `paint` in
   `write_text`. Then the tests of CRITERION-63.
4. STORY-43: `src/cli/stop.rs` with `Stop`, the owner of the output, the
   notices, and the stop thread. Then `rustix`, the watch thread, and
   `on_closed_pipe`. Then the tests of CRITERION-64, with the test of
   `--check`.
5. The usage line, the help of `--color`, and the README section.

`cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` must
pass on the CI runs of `ubuntu-latest` and `macos-latest`.

## Assumptions

- signal-hook 0.4.5 has the module `iterator` with its default features.
  Source: docs.rs, the feature page of signal-hook 0.4.5 (`channel` and
  `iterator` are the default features); `Cargo.toml` (`signal-hook =
  "0.4.5"` with its default features).
- `emulate_default_handler` puts back the default action of the signal,
  unblocks the signal, and raises it again. For SIGINT and for SIGPIPE,
  the default action ends the process. When the process does not end, the
  function calls `abort`. It is a safe function. Source: docs.rs,
  signal-hook 0.4.5, `src/low_level/signal_details.rs`.
- rustix 1.1.5 has `event::poll(fds, Option<&Timespec>)`,
  `PollFd::new(&fd, flags)` for each `AsFd`, `PollFd::revents`, and the
  flags `OUT`, `ERR`, and `HUP`. They are safe functions of the feature
  `event`. Source: docs.rs, rustix 1.1.5; `Cargo.lock` (rustix 1.1.5).
- On Linux, `poll` on the writer of a pipe gives `OUT` while the reader is
  open, and `OUT` and `ERR` after the reader closes. On a socket whose
  other end closed, it gives `HUP`. Source: a probe with Python
  `select.poll` on 2026-10-14.
- On macOS, `poll` with `OUT` on the writer of a pipe whose reader closed
  gives `HUP`, because `poll` uses the kqueue filter `EVFILT_WRITE`, which
  sets `EV_EOF` when the reader closes. Not checked here. Signal: a unit
  test of `wait_for_closed_reader` or a closed-pipe test fails on the CI
  run of `macos-latest` (Risk 3).
- `std::io::pipe` (Rust 1.87) and `std::os::unix::net::UnixStream::pair`
  are there for the unit tests. Source: `Cargo.toml` (`rust-version =
  "1.99"`).
- The bytes, the `auto` rule, and the option forms of GNU grep in decisions
  5 to 7. Source: GNU grep 3.12 (`/usr/bin/grep`), checked on 2026-10-14
  with `od -c`, and in a terminal made by `script`.
- A shell loop or a script stops after a Ctrl-C only when the child ends
  by SIGINT. Source: checked on 2026-10-14 with a `bash -c` loop and two
  Python children, one that exits with 130 and one that ends by SIGINT.
- `rg --json` writes no escape code with `--color=always`. Source: ripgrep
  15.1.0, checked on 2026-10-14, and its help text for `--json`.
- The Rust runtime ignores SIGPIPE before `main`, so a write to a closed
  pipe gives `ErrorKind::BrokenPipe`. Source: `src/cli.rs` (`emit` matches
  `BrokenPipe`).
- The fake server counts a call when the request comes, before its delay,
  and it does not wait for its threads when it stops. Source:
  `tests/common/fake_server.rs` (`serve` pushes the request, then sleeps
  for `reply.delay`; `Drop` only wakes the accept loop).
- The `kill` program is on Linux and on macOS. Source: the CI runners of
  `ubuntu-latest` and `macos-latest`.
- A terminal shows `^C` after the text on the line when the user presses
  Ctrl-C (the `echoctl` setting of the terminal). Source: the default
  terminal settings of Linux and macOS.

## Risks

1. A user with `NO_COLOR` sees color (decision 6). Signal: an issue about
   `NO_COLOR`. Mitigation: `--color=never`, or a shell alias.
2. A program that starts jag with no shell expects the code 130 and gets
   the signal SIGINT (decision 13), as with grep. Signal: a caller reports
   it. Mitigation: the stop document holds `exit_code` 130, and a caller
   can take a signal N as the code 128 + N.
3. On macOS, `poll` gives other flags for a pipe whose reader closed, so
   the watch does not see it (decision 15). Signal: a unit test of
   `wait_for_closed_reader` or a closed-pipe test fails on the CI run of
   `macos-latest`. Mitigation: the CI run shows it before the merge, and
   the code then changes the flags for macOS.
4. A closed pipe has two ends (decision 15 and FEATURE-11): the end by
   SIGPIPE before jag writes, and the code of the search while jag
   writes. Signal: a script that reads the exit code after a closed pipe
   reports two values. Mitigation: neither end writes a message or a
   partial result, so the reader that closed the pipe gets the same
   output (none) from both. The write is short, so the second end is
   rare.
5. A terminal that is narrower than the progress line wraps it, and `\r`
   goes back only to the start of the last row. Signal: stray text above
   the results. Mitigation: the line has at most 45 characters with counts
   of 4 digits.
6. `emulate_default_handler` can race with a signal handler that another
   thread sets at the same time (signal-hook docs). No other code of jag
   sets a SIGINT or SIGPIPE handler during a search. Signal: an end by
   SIGABRT.
7. The end-to-end stop tests depend on time. Signal: a CI run that fails
   only some of the time. Mitigation: the server holds each call for 30 s,
   the test acts only after the server has 6 calls, and it waits up to
   5 s for the end of jag.
8. A Ctrl-C while jag writes the results can cut the output (decision
   14), as with grep. Signal: a cut last line, or a document that does not
   parse. Mitigation: the output is small, so the write is short.
9. A match that ends at the cut of a line gets no color (decision 8).
   Signal: a match with no color at the end of a cut line. Mitigation:
   jag cuts only lines of more than 300 characters.
10. A merge conflict with the code of FEATURE-09, FEATURE-14, or
    FEATURE-21 in `src/cli.rs`, `src/output.rs`, `src/scorer.rs`, or
    `tests/common/mod.rs`. Signal: a conflict, or a test of the other
    feature that fails. Mitigation: the code that merges second joins the
    parts.

## Out of scope

- `GREP_COLORS` and `GREP_COLOR`: jag uses the default colors of grep. No
  story asks for other colors.
- `NO_COLOR` (decision 6).
- The WHEN values that grep also takes, such as `yes`, `force`, `no`,
  `none`, `tty`, and `if-tty`, and values in capital letters. jag takes the
  three words of the help of grep.
- Other signals, for example SIGTERM, SIGHUP, and SIGQUIT: the default
  action of the system.
- The cost of a stopped search (DECISION-44) and the cost line
  (DECISION-41): FEATURE-20.
- A progress line during the walk or the read of the files: jag shows
  progress only while the model scores (decision 1).
- The color of the output of `-l` and `-c`: the feature that adds them
  (FEATURE-13).
- A structured form of `--check`: a later change.
- Windows: jag runs only on Unix today.
