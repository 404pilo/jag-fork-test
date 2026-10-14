---
title: "Readiness check: five items, one small call, and the exit code of the first cause"
feature: FEATURE-02
stories: [STORY-06, STORY-07, STORY-08]
kind: build
gh_issue: 67
lifecycle_state: draft
inherits: [IF-3, IF-4, C-cli, C-check, C-config, C-scorer, CV-1, CV-3, CV-4, CV-5, CV-6, CV-7, DF-1]
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Readiness check: five items, one small call, and the exit code of the first cause

## Problem

The user stores a TypeSafe key one time. One command confirms that jag is
ready, or names each problem and its fix (CAPABILITY-01). The check
examines five items and shows pass or fail for each one: a key is present,
the service accepts the key, the account has credit, the service is
reachable, and other users cannot read the key file. It makes one live
call that costs nothing or almost nothing (DECISION-04). It exits with 0
when all five items pass. Else it exits with the code of the first cause
that fails: key missing, key not accepted, no credit, or service down
(DECISION-48, DECISION-05). The check names the source of the key that jag
uses (DECISION-03). jag never shows the key, and it refuses a key file
that other users can read (CONSTRAINT-02).

- STORY-06: an agent builder wants one command that shows pass or fail for
  five items and names the fix for each failure.
  - CRITERION-09: the check shows pass or fail for each item: a key is
    present, the service accepts the key, the account has credit, the
    service is reachable, and other users cannot read the key file.
  - CRITERION-10: for each item that fails, the check names the problem and
    the step that fixes it.
- STORY-07: an agent builder wants the check to exit with 0 when all items
  pass, and with the code of the first cause when one fails.
  - CRITERION-11: when all five items pass, the check says that jag is
    ready and exits with code 0.
  - CRITERION-67: when an item fails, the check exits with the code of the
    first cause that fails.
- STORY-08: an agent builder wants the check to make one call that costs
  nothing or almost nothing.
  - CRITERION-12: the check makes one live call to the model service, and
    that call costs nothing or almost nothing.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Command | `jag --check`. It is part of the public command line | 0006-run-setup-the-readiness-check-and-key-removal-as-options.md; system-design.md IF-3 |
| 2 | Code place | A new module `src/check.rs` for C-check. It uses C-config for the key and the key file, and C-scorer for the call | 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md; system-design.md C-check (depends on C-config and C-scorer) |
| 3 | Items and their order | The five items of DECISION-04, in its order: 1 a key is present, 2 the service accepts the key, 3 the account has credit, 4 the service is reachable, 5 other users cannot read the key file | DECISION-04; CRITERION-09 |
| 4 | The call | One `POST <base>/systemone` call through a new `scorer::probe`. `probe` runs `score` on one fixed result of one short line, with one fixed question. So the call has the request form of a search, the model setting, the shared `ureq` agent, and the status map of a search. jag does not use `GET /v1/models`: that call evaluates nothing, so it cannot show the credit (item 3) | DECISION-04 (one call; the alternatives "only local items" and "a full sample search" are rejected); system-design.md C-check ("one free call"); 0014-call-the-model-service-with-ureq-and-serde-json-on-std.md (C-check uses the one shared agent); TypeSafe docs: no health endpoint, and their health-check example probes System One with one `noul` question |
| 5 | Cost of the call | About 210 bytes of request, so about 70 input tokens by the estimate of FEATURE-04 decision 22. Output tokens are free. The call costs less than $0.00001 | TypeSafe docs, pricing ($0.042 for 1M input tokens); CRITERION-12 |
| 6 | Retry and deadline | CV-5, as for a search: one retry after a timeout, a failed connection, HTTP 429, or HTTP 5xx, after the retry wait. One deadline of `2 x timeout + retry wait` (9 seconds with the defaults). `--timeout`, `--retry-wait`, `JAG_TIMEOUT`, and `JAG_RETRY_WAIT` apply. The TypeSafe health-check example has no retry, but CV-5 governs C-check | system-design.md CV-5 and C-check (governed by CV-5); docs/superpowers/specs/2026-10-13-STORY-09-clear-failure-when-the-model-is-not-available-design.md decisions 1, 5, and 14; CONSTRAINT-07; DECISION-52 |
| 7 | "One live call" | When the call passes, the service gets exactly one request. A failure that CV-5 tries again adds the one retry of that call. With no key that jag can use, the check makes no call | CRITERION-12; system-design.md CV-5 |
| 8 | Item 1 | `find_key` of FEATURE-01. The pass line names the source: `JAG_TYPESAFE_API_KEY`, `TYPESAFE_API_KEY`, or the path of the key file. It never shows the key | DECISION-03 ("The health check names the source that jag uses"); docs/superpowers/specs/2026-10-13-STORY-01-store-a-key-one-time-design.md decision 7; CV-6 |
| 9 | Item 5 | jag examines `~/.config/jag/config.yml` each time it exists, also when the key comes from a variable. It reads the mode of the open file (`fstat`) and never reads its text. Any group or other permission bit fails the item | DECISION-04 item 5 (it has no condition on the source); CONSTRAINT-02; system-design.md CV-6; FEATURE-01 spec decision 10 |
| 10 | Key file problem and item 1 | When item 5 fails and no key variable is set, the key file is the source. Then item 1 shows "not checked", because jag does not read that file. `find_key` would stop with the same error, so the problem shows one time, at item 5 | This spec; FEATURE-01 spec decisions 7 and 15 |
| 11 | A third state | An item that jag cannot examine, because an item before it failed, shows "not checked" and the reason. It has no fix of its own. Rejected: "fail" with the fix "fix item N first", because CRITERION-10 needs a real problem for each item that fails, and one call cannot examine the credit with a key that the service refuses. Rejected: stop at the first failure, because CRITERION-09 needs each item | This spec; forced by DECISION-04 (one call) and CRITERION-10. It adds a third state to the "pass or fail" of DECISION-04 and CRITERION-09, only for an item that jag cannot examine. Each item that jag examines still shows pass or fail |
| 12 | Items 2, 3, and 4 from the call | See "The call results". Each cause of exit 4 fails item 2, the cause of exit 5 fails item 3, and each cause of exit 6 fails item 4 | FEATURE-03 spec decision 3 and its failure texts; 0007-number-the-exit-codes-above-2-from-3-to-8.md |
| 13 | Exit code | 0 when all five items pass. Else the code of the first item that fails, in the order of DECISION-04: item 1 gives 3, item 2 gives 4, item 3 gives 5, item 4 gives 6, item 5 gives 3. DECISION-48 names the causes of items 1 to 4. Exit 3 for item 5 comes from CV-6 and FEATURE-01 decision 9 (3 is each local key problem) | DECISION-48; DECISION-04; system-design.md CV-6 and IF-4; FEATURE-01 spec decision 9 |
| 14 | Streams | Standard output: the five item lines, and `jag is ready.` when all pass. Error stream: for each item that fails, its cause and its fix in the CV-1 form (`jag: <cause>` and then `<fix>`), in item order. So the first block on the error stream is the cause of the exit code | system-design.md CV-1, CV-3; FEATURE-01 spec decision 11; FEATURE-03 spec decision 8 |
| 15 | Failure texts | The texts of FEATURE-01 (`ConfigError`) and FEATURE-03 (`ScoreError`), with no change. One fix for each cause | FEATURE-03 spec decision 12; DECISION-09 |
| 16 | Line text | The exact texts of "Output". Scripts and agents act on the exit code (IF-4), not on the lines. So the line form is not a contract, and it needs no ADR | This spec; system-design.md IF-4 |
| 17 | Usage errors first | A bad `--timeout` or `--retry-wait`, or a bad `JAG_BASE_URL`, `JAG_MODEL`, `JAG_TIMEOUT`, or `JAG_RETRY_WAIT`, stops the check before it examines an item: exit 2, the text of FEATURE-03 or FEATURE-04, and no item line | system-design.md IF-4 (2 is a usage error); FEATURE-03 spec decisions 2 and 11; FEATURE-04 spec decisions 26 and 29 |
| 18 | Conflicts | `--check` with `--setup`, `--forget-key`, `--max-scored`, TERM, QUESTION, or PATH is an argument error: exit 2 and the one usage line. An option that has no effect in a mode conflicts with that mode, as `--timeout` does with `--setup` | src/cli.rs (inferred); FEATURE-06 spec "Out of scope" (the limit has no effect on `--check`); FEATURE-04 spec decision 26 |
| 19 | Cost limit variable | The check does not read `JAG_MAX_SCORED`, because its call scores one fixed result | docs/superpowers/specs/2026-10-13-STORY-18-cost-limit-on-scoring-design.md "Out of scope" |
| 20 | Secrecy | No output shows the key, the value of `JAG_BASE_URL`, or the body of a response. The key goes only into the `Authorization` header | system-design.md CV-6; FEATURE-04 spec decisions 16 and 29; src/config/model.rs |
| 21 | Write path | The check finds all five results first, then writes. A reader that closes the pipe gives no message and keeps the exit code. Another write error gives exit 2, as in a search | system-design.md CV-4; src/cli.rs (`write_failure`) |
| 22 | Test seams | `check::run` takes the environment, the file system, and the call as arguments, so unit tests give fakes. End-to-end tests use the fake server of FEATURE-04 and whole seconds | FEATURE-01 spec decision 17; FEATURE-04 spec decision 25; FEATURE-03 spec decision 15 |
| 23 | Crates | No new crate | 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md; 0014-call-the-model-service-with-ureq-and-serde-json-on-std.md |

## Design

### Files

| File | Change |
|---|---|
| `src/check.rs` | New. C-check: the five items, the results of the call, the exit code, and the text |
| `src/lib.rs` | Add `pub mod check;` |
| `src/cli.rs` | Add `--check`, its conflicts, its usage line, and the dispatch |
| `src/config.rs` | Add `pub(crate) fn key_file_mode` |
| `src/scorer.rs` | Add `pub fn probe` and its fixed result and question |
| `tests/check.rs` | New. End-to-end tests with the fake server |
| `README.md` | Add the section "Check that jag is ready" |

### The command line (C-cli)

Add a flag to `Cli`:

```rust
/// Check that jag is ready: the key, the service, and the key file
#[arg(long, conflicts_with_all = ["setup", "forget_key", "cost_limit", "term", "question", "paths"])]
check: bool,
```

clap conflicts work in both directions, so `--setup --check` also gives the
argument error. Each argument error writes the one `USAGE_ERROR` line and
exits 2 (FEATURE-04 decision 26). `--timeout` and `--retry-wait` stay
allowed with `--check`.

The usage text gets one more line:

```
jag [--timeout SECONDS] [--retry-wait SECONDS] [--max-scored COUNT] TERM "QUESTION" [PATH...]
       jag --check [--timeout SECONDS] [--retry-wait SECONDS]
       jag --setup
       jag --forget-key
```

`run` dispatches `--check` before the search, after `--forget-key`:

```rust
fn check(cli: &Cli, env: &Env, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<u8, Failure> {
    // Usage errors first (decision 17). The timeout is checked first.
    let timeout = parse_option(cli.timeout.as_deref(), TimeSetting::Timeout)?;
    let retry_wait = parse_option(cli.retry_wait.as_deref(), TimeSetting::RetryWait)?;
    let settings = model_settings(env, (timeout, retry_wait)).map_err(Failure::from)?;
    let settings = ScoreSettings::new(settings.base_url, settings.model, settings.times);
    let report = check::run(env, &RealFs, &|key| probe(key, &settings));
    // Write the lines, then the failures (decisions 14 and 21).
    ...
    Ok(report.exit_code())
}
```

Only an exit-2 error goes back as `Err(Failure)` to `finish`. The check
writes the failures of the items itself, because there can be more than
one. It writes all of standard output first, then the error stream.

### The key file (C-config)

```rust
/// The path and the mode (`mode & 0o777`) of the key file, or `None` when
/// there is no `$HOME` or no file. jag never reads the text of the file here.
pub(crate) fn key_file_mode(env: &Env, fs: &dyn Fs) -> Result<Option<(PathBuf, u32)>, ConfigError>
```

It uses `open_key_file` and `OpenFile::mode`, the first two steps of
`find_key`. So a file that is not a regular file gives `KeyFileNotRegular`
before the open, and an open or `fstat` error gives `KeyFileUnreadable`.
It never calls `read_to_string`. Only C-config touches the key file (CV-6).

### The call (C-scorer)

```rust
/// The question and the one result of the readiness check call.
pub const CHECK_QUESTION: &str = "Is jag ready?";
pub const CHECK_PATH: &str = "jag-check";
pub const CHECK_LINE: &[u8] = b"jag is ready";

/// One small call that tests the key, the credit, and the service
/// (FEATURE-02). It has the CV-5 retry and deadline of `score`.
pub fn probe(key: &ApiKey, settings: &ScoreSettings) -> Result<(), ScoreError> {
    let snippet = Snippet {
        source: Source::File(PathBuf::from(CHECK_PATH)),
        lines: vec![SnippetLine::new(1, CHECK_LINE, true)],
    };
    score(&[snippet], CHECK_QUESTION, key, settings).map(|_| ())
}
```

The body has the `jag-p1` form with one result and one question, `s1`.
`parse_scores` with a count of 1 reads the answer. A score of any value
from 0 to 1 passes: the check tests the service, not the answer. The
fake server `FakeServer::constant` answers this call with no change.

### The five items (C-check)

```rust
pub enum Status { Pass, Fail, NotChecked }

pub struct Item {
    pub status: Status,
    /// The text after the status, with its note in brackets.
    pub text: String,
    /// `Some` only for `Fail`. Its `exit_code` is the code of the item.
    pub failure: Option<Failure>,
}

pub struct Report { pub items: [Item; 5] }

pub fn run(env: &Env, fs: &dyn Fs, call: &dyn Fn(&ApiKey) -> Result<(), ScoreError>) -> Report
```

`run` does these steps:

1. Item 5: `key_file_mode(env, fs)`.
   - `Ok(None)`: pass, with the note "no key file".
   - `Ok(Some((path, mode)))` and `mode & 0o077 == 0`: pass.
   - `Ok(Some((path, mode)))` and another mode: fail with
     `ConfigError::KeyFileOpen { path, mode }`.
   - `Err(error)`: fail with `error`.
2. Item 1:
   - When item 5 failed and `env.set_variable()` is `None`: not checked
     (decision 10). jag does not call `find_key`.
   - Else `find_key(env, fs)`. `Ok((key, source))`: pass, with the source.
     `Err(error)`: fail with `error` (exit 3).
3. Items 2, 3, and 4: when item 1 passed, `call(&key)` one time. Else all
   three are not checked, and jag makes no call.

The key states and items 1 and 5:

| Key variables | Key file | Item 1 | Item 5 |
|---|---|---|---|
| One holds a good key | None, or no `$HOME` | pass, the variable name | pass, "no key file" |
| One holds a good key | Private | pass, the variable name | pass |
| One holds a good key | Others can read it | pass, the variable name | fail, `KeyFileOpen` |
| One holds a bad key | Any | fail, `KeyFormatBad` | as the key file gives |
| None set | None, or no `$HOME` | fail, `KeyMissing` | pass, "no key file" |
| None set | Private, good key | pass, the path | pass |
| None set | Private, but no key, bad YAML, bad key, or a read error | fail, `KeyMissing`, `KeyFileBad`, `KeyFormatBad`, or `KeyFileUnreadable` | pass |
| None set | Others can read it, not a regular file, or jag cannot open it | not checked | fail, `KeyFileOpen`, `KeyFileNotRegular`, or `KeyFileUnreadable` |

A variable counts as set by the rule of `find_key`: an empty or blank
variable is not set (FEATURE-01 decision 14).

### The call results

| Result of `call` | Item 2 | Item 3 | Item 4 |
|---|---|---|---|
| `Ok(())` | pass | pass | pass |
| `KeyNotAccepted` (401, 403) | fail, exit 4 | not checked | pass |
| `NoCredit` (402) | pass | fail, exit 5 | pass |
| `TimedOut`, `CannotConnect`, `RateLimited`, `ServerFailed`, `Redirect`, `UnexpectedStatus`, `BadResponse` | not checked | not checked | fail, exit 6 |
| No call (item 1 did not pass) | not checked | not checked | not checked |

A 401 or a 402 comes from the service, so the service is reachable. The
failure of each item is `Failure::from(error)`, with the FEATURE-03 text.

### The exit code (C-check)

```rust
impl Report {
    /// 0 when all pass. Else the code of the first item that fails, in
    /// the order of DECISION-04 (decision 13).
    pub fn exit_code(&self) -> u8
}
```

An item that is not checked has no code. Examples: no key gives 3. A
refused key and an open key file that is not the source give 4, because
item 2 comes before item 5. An open key file that is the source gives 3.

### Output

Each item line is the status in a field of 12 characters, then the text:
`format!("{status:<12}{text}")`. The status words are `pass`, `fail`, and
`not checked`. The item texts and their notes:

| Item | Text | Note on pass | Note on not checked |
|---|---|---|---|
| 1 | `a key is present` | ` (JAG_TYPESAFE_API_KEY)`, ` (TYPESAFE_API_KEY)`, or ` (<path>)` | ` (jag cannot use the key file: see the last item)` |
| 2 | `the service accepts the key` | none | ` (jag has no key to send)` or ` (the call to the service failed)` |
| 3 | `the account has credit` | none | ` (jag has no key to send)`, ` (the service did not accept the key)`, or ` (the call to the service failed)` |
| 4 | `the service is reachable` | none | ` (jag has no key to send)` |
| 5 | `other users cannot read the key file` | ` (no key file)` when there is none | never |

A line with `fail` has no note: the error stream gives the cause and the
fix. `<path>` is `Path::display` of the key file. When all five items
pass, one more line follows: `jag is ready.`

All items pass, with the key in a variable and no key file. Standard
output, exit 0, and an empty error stream:

```
pass        a key is present (JAG_TYPESAFE_API_KEY)
pass        the service accepts the key
pass        the account has credit
pass        the service is reachable
pass        other users cannot read the key file (no key file)
jag is ready.
```

No key. Exit 3, and no call:

```
fail        a key is present
not checked the service accepts the key (jag has no key to send)
not checked the account has credit (jag has no key to send)
not checked the service is reachable (jag has no key to send)
pass        other users cannot read the key file (no key file)
```

```
jag: no TypeSafe API key found.
Get a key at https://console.typesafe.ai/keys, then run `jag --setup`,
or set JAG_TYPESAFE_API_KEY.
```

A refused key, and a key file that others can read but that is not the
source. Exit 4:

```
pass        a key is present (JAG_TYPESAFE_API_KEY)
fail        the service accepts the key
not checked the account has credit (the service did not accept the key)
pass        the service is reachable
fail        other users cannot read the key file
```

```
jag: the TypeSafe API did not accept the key (HTTP 401).
Check the key at https://console.typesafe.ai/keys, then run jag --setup.
jag: /home/user/.config/jag/config.yml can be read by other users (mode 0644).
Run `chmod 600 /home/user/.config/jag/config.yml`.
```

### Acceptance criteria and their tests

Unit tests in `src/check.rs` give `run` a fake environment, a fake file
system, and a fake call. Unit tests in `src/config.rs` test
`key_file_mode`. End-to-end tests in `tests/check.rs` run the binary with
`jag` or `jag_bare` of `tests/common`, a temporary `$HOME`, and the fake
server.
A test with a key variable uses `jag(home, url)`. A test with no key
variable (no key, or the key in the key file) uses `jag_bare(home)` and
sets `JAG_BASE_URL` to the fake server, so that no call goes to the real
service. Each end-to-end test also checks that no output holds `SENTINEL`,
`127.0.0.1`, or `SECRET-BODY-TEXT` (decision 20).

- CRITERION-09 (each item shows pass or fail):
  - Unit: one case for each row of the two tables above. The test checks
    the status of each of the five items.
  - End-to-end: a key in `JAG_TYPESAFE_API_KEY`, a key file with mode
    `0644`, and a server that gives 402. Standard output is exactly
    `pass`, `pass`, `fail`, `pass`, `fail` for items 1 to 5. So each item
    is examined, and each shows pass or fail.
  - End-to-end: a private key file (mode `0600`) is the source. The item 1
    line is `pass        a key is present (<path>)`.
- CRITERION-10 (each failure names its problem and its fix):
  - End-to-end, one test for each item that can fail, with the exact error
    stream: no key (item 1, the `KeyMissing` text), HTTP 401 (item 2),
    HTTP 402 (item 3), HTTP 500 two times (item 4), and a key file with
    mode `0644` (item 5, the `chmod 600` fix).
  - End-to-end: the 402 case above has two blocks on the error stream, in
    item order: the credit block, then the key file block.
  - End-to-end: a key file with mode `0644` that is the source. Item 1 is
    not checked, item 5 fails, and the error stream holds the key file
    block one time. The server gets no request, and the output does not
    hold the key in the file.
- CRITERION-11 (jag is ready, exit 0): end-to-end, the "all items pass"
  example. Standard output is exactly the six lines, the error stream is
  empty, and the exit code is 0.
- CRITERION-67 (the code of the first cause):
  - Unit: `exit_code` gives 3, 4, 5, 6, and 3 when only item 1, 2, 3, 4,
    or 5 fails, and 0 when none fails.
  - Unit: item 2 and item 5 fail. The code is 4.
  - End-to-end: the exit codes of the CRITERION-10 tests are 3, 4, 5, 6,
    and 3. The 402 case with the open key file exits 5.
  - End-to-end: `--timeout 1 --retry-wait 1` and a server that waits 3
    seconds. Item 4 fails with the timeout text, and the exit code is 6.
- CRITERION-12 (one call that costs almost nothing):
  - End-to-end: when all items pass, the server gets exactly one request.
    It is `POST /v1/systemone`, its body is less than 400 bytes, its JSON
    has exactly one question, and its `Authorization` header holds the
    key.
  - End-to-end: with no key, the server gets no request.
  - End-to-end: a server that gives 503, then 200. The server gets two
    requests (the one call and its CV-5 retry), and the exit code is 0.

Other tests:

- End-to-end: `--check needle`, `--check --setup`, `--check --forget-key`,
  and `--check --max-scored 5` give exit 2, the one usage line, and no
  standard output.
- End-to-end: `--check --timeout 0` and `JAG_BASE_URL=http://example.com`
  give exit 2 with their FEATURE-03 and FEATURE-04 texts, no item line,
  and no request.
- Unit: `key_file_mode` gives `None` for a missing file and for no
  `$HOME`, the mode for a file, and `KeyFileNotRegular` for a folder. A
  fake `OpenFile` whose `read_to_string` panics shows that it never reads
  the text.

## Build order

1. C-config: `key_file_mode` and its unit tests.
2. C-scorer: `probe` and its constants.
3. STORY-06: `src/check.rs` with `Status`, `Item`, `Report`, `run`, and
   the line text, with the unit tests of the two tables. Then `--check`
   in `src/cli.rs`, with its conflicts, the usage line, and the dispatch.
   Then the end-to-end tests of CRITERION-09 and CRITERION-10.
4. STORY-07: `Report::exit_code` and its unit tests, the `jag is ready.`
   line, and the end-to-end tests of CRITERION-11 and CRITERION-67.
5. STORY-08: the end-to-end tests of CRITERION-12.
6. The other tests, and the README section "Check that jag is ready"
   with the command, the first example of "Output", and the exit codes
   0 and 2 to 6.

`cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` must
pass on the CI runs of `ubuntu-latest` and `macos-latest`.

## Assumptions

- HTTP 402 means no credit for the check call, as for a search. The
  TypeSafe docs list 401, 422, 429, and 529, but not 402 or 403. From
  src/scorer/client.rs and the FEATURE-03 spec "Assumptions".
- A 401 or a 403 comes from the service, so item 4 passes. From
  src/scorer/client.rs (the status is the answer of the service).
- The call is about 210 bytes, so about 70 input tokens. From
  src/scorer/prompt.rs (`request_body`) and FEATURE-04 decision 22 (bytes
  divided by 3).
- `Env::set_variable` gives the same answer as `find_key` for "a key
  variable is set". From src/config.rs.
- An option that has no effect in a mode conflicts with that mode. From
  src/cli.rs (`--timeout` and `--max-scored` conflict with `--setup` and
  `--forget-key`).

## Risks

- A key in a variable and a key file that other users can read: the check
  exits 3, but a search exits 0, because a search does not open the file.
  Signal: a script that runs `jag --check` before an agent run stops, but
  searches work. Mitigation: the fix line names `chmod 600 <path>`. The
  check protects the key in the file (CONSTRAINT-02).
- TypeSafe gives another status for no credit. Signal: a user with no
  credit sees item 2 or item 4 fail, not item 3. Mitigation: change the
  status map in src/scorer/client.rs. The check follows it.
- TypeSafe adds a minimum price for each call, or a free health endpoint.
  Signal: a change on the TypeSafe pricing page. Mitigation: change only
  `probe`.
- A failure that CV-5 tries again doubles the cost of the check. Signal:
  none is necessary, because the cost stays less than $0.00001.

## Out of scope

- A structured form of the check: a later change. FEATURE-11 adds the
  structured output option, and each command that has no structured form
  refuses it. So `--check` with that option is an argument error: exit 2
  (DECISION-07, CV-1).
- Ctrl-C during the call, and exit 130: FEATURE-15.
- An AI gateway and its words in the item lines: FEATURE-21.
- A cost line for the check call: FEATURE-20.
- Settings other than the key in the config file: DF-1.
- A check of `JAG_MAX_SCORED`: the check makes no search (decision 19).
- A setup that tests the key: FEATURE-01 decision 13 says no. The user
  runs `jag --check` after `jag --setup`.
