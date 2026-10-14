---
title: "Clear failure when the model is not available: the final failure texts, the exit codes, and the time options"
feature: FEATURE-03
stories: [STORY-09, STORY-10, STORY-11, STORY-12]
kind: build
lifecycle_state: draft
inherits: [QS-4, IF-3, IF-4, C-cli, C-config, C-scorer, CV-1, CV-2, CV-3, CV-5, CV-6, CV-7]
session: 21508ce9-788b-40ed-be8d-03797016b20b
---

# Clear failure when the model is not available: the final failure texts, the exit codes, and the time options

## Problem

When the key is missing or not valid, the account has no credit, or the
model service is down, jag says which cause it is and how to fix it. Each
cause has its own exit code. jag never falls back to a ranking that ignores
the question (FEATURE-03, CAPABILITY-01, DECISION-01, DECISION-05).

Before jag says that the service is down, it tries again one time. It fails
after a fixed time, and the user can change that time (CONSTRAINT-04).
When the credit or the service stops during a search, jag shows no partial
result (DECISION-06). jag fails the full search within the worst-case time
of its two time settings: `2 x timeout + retry wait`, from the first call.
With the defaults, that is 9 seconds (CONSTRAINT-07, as the user answered
in Q-as).

FEATURE-01 and FEATURE-04 built most of this: exits 3 to 6, the retry, and
"fail the full search". FEATURE-04 also built the deadline (its decision
31), but its retry wait can go past the deadline. The FEATURE-04 fix PR #30
corrects that, and the FEATURE-03 code depends on it. This spec adds the two
time options, makes the failure texts final, and adds the tests that prove
each criterion.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | The times that the user can change | Both. The time of one call: `--timeout SECONDS` and `JAG_TIMEOUT`. The wait before the retry: `--retry-wait SECONDS` and `JAG_RETRY_WAIT`. Defaults 4 s and 1 s. So CRITERION-16 and CV-5 both hold | human:404pilo, 2026-10-13 (Q-al); system-design.md CV-5 (the architect adds `--retry-wait` to CV-5 in a separate docs PR) |
| 2 | Form and range of the times | Whole seconds, from 1 to 60, for both options and both variables. Each other value gives exit 2, and the message does not show the value | human:404pilo, 2026-10-13 (Q-am) |
| 3 | Exit codes | 3 key missing, 4 key not accepted, 5 no credit, 6 service down. 0, 1, and 2 keep their grep meaning | 0007-number-the-exit-codes-above-2-from-3-to-8.md; DECISION-05 |
| 4 | Retry | One retry after a timeout, a failed connection, HTTP 429, or HTTP 5xx, after the retry wait | system-design.md CV-5; FEATURE-04 spec, failure table |
| 5 | Deadline | One deadline for all model calls of a search: `2 x timeout + retry wait`, from the first call. 9 seconds with the defaults, 180 seconds with the largest values. No fixed 10-second limit | human:404pilo, 2026-10-13 (Q-as); CONSTRAINT-07 (the planner rewrites it with this formula); DECISION-52 (PR #29); system-design.md QS-4 and CV-5 (main 82b1034, PR #25); FEATURE-04 fix PR #30 (the retry wait stops at the deadline); docs/superpowers/specs/2026-10-13-STORY-13-rank-matching-lines-by-a-question-design.md decision 31 |
| 6 | No fallback | A model failure gives no result that ignores the question | DECISION-01; system-design.md QS-4 |
| 7 | Full failure | A model failure fails the full search, with no partial result | DECISION-06; system-design.md CV-2 |
| 8 | Failure shape | Each failure has a cause, a fix, and an exit code. Text mode writes it to the error stream | system-design.md CV-1, CV-3; FEATURE-01 spec decision 11 |
| 9 | Setting order | The option wins over the variable. No key in the config file yet | system-design.md CV-7, DF-1 |
| 10 | Secrecy of a value | No message shows the value of a time option or a time variable | CV-6; FEATURE-04 spec decision 26 |
| 11 | Where jag checks the options | With the other arguments, before the key (step 1 of the FEATURE-04 search flow). The variables are checked with the model settings (step 3) | This spec; FEATURE-04 spec decision 18 |
| 12 | Fix texts | One fix for each cause, not one fix for each exit code (see "Failure texts") | This spec; CRITERION-13 |
| 13 | Credit page | The fix of exit 5 names `https://console.typesafe.ai` | This spec; see "Assumptions" |
| 14 | `--check` | The two time options also apply to the call of `--check`. FEATURE-02 uses them | This spec |
| 15 | Fast tests | No hidden option for tests. An end-to-end test uses whole seconds. A test under 1 second calls `score` with a `ScoreSettings`, as in FEATURE-04 | human:404pilo, 2026-10-13 (Q-am) |

## Design

### Files

```
src/cli.rs              C-cli: --timeout and --retry-wait, the order of the checks
src/config/model.rs     C-config: JAG_TIMEOUT, JAG_RETRY_WAIT, parse_seconds, Times
src/config.rs           C-config: ConfigError::TimeBad and its failure
src/scorer.rs           C-scorer: ScoreSettings::new takes the times; the final texts
tests/failure.rs        end-to-end tests of CRITERION-13 to CRITERION-17
tests/usage.rs          the time argument tests
```

No new crate.

### The time options (C-cli, C-config)

The `Cli` struct gets two options:

```rust
/// Seconds to wait for one model call, from 1 to 60 (default 4)
#[arg(long, value_name = "SECONDS", conflicts_with_all = ["setup", "forget_key"])]
timeout: Option<String>,

/// Seconds to wait before the one retry, from 1 to 60 (default 1)
#[arg(long = "retry-wait", value_name = "SECONDS", conflicts_with_all = ["setup", "forget_key"])]
retry_wait: Option<String>,
```

- The type is `String`, not a number. jag parses the text itself, so that
  the message is its own and never shows the value (decision 10). A clap
  error still gives the fixed line of the FEATURE-04 argument error rule.
- The usage text of the search line is
  `jag [--timeout SECONDS] [--retry-wait SECONDS] TERM "QUESTION" [PATH...]`.

`src/config/model.rs` adds:

```rust
pub const TIMEOUT_VARIABLE: &str = "JAG_TIMEOUT";
pub const RETRY_WAIT_VARIABLE: &str = "JAG_RETRY_WAIT";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(4);
pub const DEFAULT_RETRY_WAIT: Duration = Duration::from_secs(1);
pub const MIN_SECONDS: u64 = 1;
pub const MAX_SECONDS: u64 = 60;

pub enum TimeSetting { Timeout, RetryWait }
pub enum TimeSource { Option, Variable }
pub struct Times { pub timeout: Duration, pub retry_wait: Duration }

pub fn parse_seconds(text: &str, setting: TimeSetting, source: TimeSource)
    -> Result<Duration, ConfigError>;
```

`parse_seconds`:

1. Trim the text.
2. The text must be 1 to 3 ASCII digits and nothing else. So `4`, `60`,
   and `01` pass. `0.5`, `1.5`, `-1`, `+1`, `1e1`, `4s`, `1000`, and a
   blank text fail.
3. Parse it as `u64`. The value must be from `MIN_SECONDS` to
   `MAX_SECONDS`, both included.
4. Give `Duration::from_secs(value)`.
5. Each failure gives `ConfigError::TimeBad { setting, source }`.

`ModelSettings` gets `pub times: Times`. `model_settings(env, options)`
takes the parsed options as `(Option<Duration>, Option<Duration>)`. For
each of the two times:

- The option wins (decision 9).
- Else the variable: blank counts as not set, as FEATURE-01 decision 14
  says. A value that is not text, or that `parse_seconds` refuses, gives
  `TimeBad { source: Variable }`.
- Else the default.

`ConfigError::TimeBad` gives exit 2:

| Setting, source | Cause | Fix |
|---|---|---|
| Timeout, option | `the value of --timeout is not a whole number of seconds from 1 to 60.` | `Use a number such as --timeout 10.` |
| Timeout, variable | `JAG_TIMEOUT is not a whole number of seconds from 1 to 60.` | `Set JAG_TIMEOUT to a number such as 10, or unset it.` |
| Retry wait, option | `the value of --retry-wait is not a whole number of seconds from 1 to 60.` | `Use a number such as --retry-wait 2.` |
| Retry wait, variable | `JAG_RETRY_WAIT is not a whole number of seconds from 1 to 60.` | `Set JAG_RETRY_WAIT to a number such as 2, or unset it.` |

### The search flow (C-cli)

The FEATURE-04 search flow changes in two steps:

- Step 1: after the other argument checks, parse `--timeout`, then
  `--retry-wait`, with `parse_seconds(text, setting, Option)`. A failure
  gives exit 2 before the key (decision 11).
- Step 3: `model_settings(env, options)` gives the two times. The timeout
  variable is checked before the retry wait variable.
- `ScoreSettings::new(base_url, model, times)` sets `timeout`,
  `retry_wait`, and `deadline = 2 x timeout + retry_wait` (decision 5).
  With `--timeout 10 --retry-wait 5`, a search fails within 25 seconds.
  With the largest values, it fails within 180 seconds.
- `Run::call` in `src/scorer.rs` changes: before the retry wait, it
  compares the wait with the time that is left before the deadline. When
  the time left is not more than the retry wait, it skips the retry and
  gives the error of the first attempt. So no wait goes past the deadline,
  and a search fails within the deadline also when it has more than 6
  calls (decision 5). The FEATURE-04 fix PR #30 makes this change. The
  FEATURE-03 code starts after PR #30 merges.

### Failure texts (C-scorer, C-config)

Each cause gets its own fix (decision 12). A text never holds the key, the
address, a value of the command line, a variable, or the response body.
`KEYS_PAGE` is `https://console.typesafe.ai/keys`.

| Cause | Exit | Cause text | Fix text |
|---|---|---|---|
| No key (`ConfigError::KeyMissing`) | 3 | no TypeSafe API key found. | Get a key at `KEYS_PAGE`, then run `jag --setup`, or set `JAG_TYPESAFE_API_KEY`. (FEATURE-01, no change) |
| HTTP 401 or 403 | 4 | the TypeSafe API did not accept the key (HTTP `<status>`). | Check the key at `KEYS_PAGE`, then run `jag --setup`. |
| HTTP 402 | 5 | the TypeSafe account has no credit (HTTP 402). | Add credit at https://console.typesafe.ai, then try again. |
| Timeout or deadline | 6 | the TypeSafe API did not answer in time. | Try again later, or give each call more time with `--timeout`. |
| No connection | 6 | jag cannot connect to the model service. | Check the network connection and `JAG_BASE_URL`, then try again. |
| HTTP 429 | 6 | the TypeSafe API refused more calls (HTTP 429). | Wait one minute, then try again. |
| HTTP 500 to 599 | 6 | the TypeSafe API failed (HTTP `<status>`). | Try again later. |
| HTTP 300 to 399 | 6 | the model service sent a redirect. | Check `JAG_BASE_URL`, or unset it. |
| Other status, for example 422 | 6 | the TypeSafe API gave HTTP `<status>`. | Check `JAG_BASE_URL` and `JAG_MODEL`, or unset them. |
| HTTP 200 with a body that jag cannot read | 6 | the TypeSafe API gave a response that jag cannot read. | Check `JAG_BASE_URL`, or unset it. |

The backticks in the table are formatting. The no-key fix keeps the
FEATURE-01 text of `src/config.rs` exactly, with its backticks and its
line break. The other fix texts have no backticks.

The texts name the variables, not their values. FEATURE-21 (an AI gateway)
can change the word "TypeSafe API" when the address is not the default.

### Acceptance criteria and their tests

All end-to-end tests use the fake server and the test setup of the
FEATURE-04 spec: `HOME` in a `tempfile` folder, the key text
`SENTINEL-SENTINEL-SENTINEL`, `JAG_BASE_URL` of the fake server, and no
proxy variables. In each test, standard output and stderr do not hold
`SENTINEL`.

| Criterion | Test |
|---|---|
| CRITERION-13 Each model failure names its cause and fix | `tests/failure.rs`: one case for each row of "Failure texts". stderr is exactly `jag: <cause>\n<fix>\n`. The no-key case runs with no key variable and no key file. |
| CRITERION-14 Each model failure has its own exit code | The same cases give exits 3, 4, 5, and 6. A search with results exits 0, a search with no match exits 1, and a bad term exits 2. |
| CRITERION-15 jag never ranks without the model | For each failure case of CRITERION-13, standard output is empty, also when the files have matches. |
| CRITERION-16 jag tries again one time, and the user can change the wait | 500, then 200, gives exit 0 and 2 calls. With `--retry-wait 2`, the second call comes 2 s or more after the first. `JAG_RETRY_WAIT=2` gives the same. `--retry-wait 1` with `JAG_RETRY_WAIT=3` gives a gap of less than 3 s (the option wins). The server waits 3 s: `--timeout 1` gives exit 6 after 2 calls, in less than 5 s (1 s, the 1 s wait, and 1 s, with room for a slow machine). `JAG_TIMEOUT=1` gives the same, and `--timeout 1` with `JAG_TIMEOUT=30` gives the same. With `--timeout 5`, the search waits for the reply and exits 0. |
| CRITERION-17 A failure during a search fails the full search | 40 files, one match each, so 2 calls (32 and 8 results) in parallel. The server gives the first request that it gets as many scores as that request asks for, and 402 to each later request. jag exits 5, standard output is empty, and stderr holds the exit 5 text. The same with 500 to each later request (the retry also gets 500) gives exit 6. |

More tests:

- Time values (unit tests of `parse_seconds`): `1`, `4`, `60`, `01`, and
  ` 10 ` pass. `0`, `61`, `0.5`, `1.5`, `-1`, `+1`, `1e1`, `4s`, `1000`,
  `abc`, and a blank text fail. Each case runs for both settings.
- The option wins: unit tests of `model_settings` for each time, with an
  option and a variable, with a variable only, and with neither.
- Time secrecy (`tests/usage.rs`): `--timeout SENTINEL-SENTINEL-SENTINEL`,
  `--retry-wait SENTINEL-SENTINEL-SENTINEL`,
  `JAG_TIMEOUT=SENTINEL-SENTINEL-SENTINEL`, and
  `JAG_RETRY_WAIT=SENTINEL-SENTINEL-SENTINEL` each exit 2. stderr does not
  hold `SENTINEL`.
- Order: `--timeout abc` and `--retry-wait abc` with no key exit 2
  (decision 11). `JAG_TIMEOUT=abc` with no key exits 3, because the key
  comes first.
- `--setup --timeout 5` and `--forget-key --retry-wait 5` exit 2 with the
  fixed line.
- `jag --help` shows `--timeout <SECONDS>` and `--retry-wait <SECONDS>`.
- Deadline formula (CONSTRAINT-07, Q-as): `ScoreSettings::new` gives a
  deadline of `2 x timeout + retry_wait` for the defaults (9 s), for
  10 s and 5 s (25 s), and for 60 s and 60 s (180 s). The tests compare
  with the formula, not with a fixed 10 s.
- Deadline end to end: the server waits 10 s. `--timeout 1 --retry-wait 2`
  gives exit 6 after 2 calls, in less than 5 s. The formula gives 4 s,
  and 1 s is room for a slow machine.
- Deadline with more than 6 calls (unit test of `score`, short times):
  224 snippets, so 7 calls. The server waits longer than the timeout.
  `score` gives `TimedOut` within the deadline and a small margin, not
  the deadline and a retry wait.
- No test needs a time under 1 second through the binary (decision 15).
  The FEATURE-04 unit tests of `score` keep their short times.

## Build order

0. The FEATURE-04 code is on `main` (merge 61a8f22), and the FEATURE-04
   fix PR #30 (the retry wait stops at the deadline) is merged.
1. `parse_seconds`, `TimeBad`, and `ModelSettings.times`, with their unit
   tests (STORY-11).
2. The `--timeout` and `--retry-wait` options, the search flow steps 1 and
   3, `ScoreSettings::new`, and the retry wait that stops at the deadline
   (from PR #30), with the tests of CRITERION-16 and the deadline tests
   (STORY-11).
3. The final failure texts, with the tests of CRITERION-13 and
   CRITERION-14 (STORY-09, STORY-10).
4. The tests of CRITERION-15 and CRITERION-17 (STORY-09, STORY-12).

## Assumptions

- The TypeSafe docs do not give a page to add credit. The keys page is
  `https://console.typesafe.ai/keys`, so the exit 5 fix names
  `https://console.typesafe.ai`. Source: `src/config.rs` `KEYS_PAGE`.
- The docs do not list 402 or 403. jag maps them as ADR-0007 says.

## Risks

- A user sets the largest values. A failure then takes up to 180 s.
  Signal: a report of a slow failure. Mitigation: the range stops at 60 s,
  and the help text gives the defaults.
- The end-to-end tests of CRITERION-16 take some seconds each, because the
  smallest time is 1 s. Signal: a slow test run. Mitigation: keep these
  tests few; the unit tests of `score` cover the short times.
- CV-5 and QS-4 name the formula on `main` (82b1034, PR #25).
  CONSTRAINT-07 and DECISION-52 name it only after PR #29 merges. Signal:
  the PRD and this spec differ. Mitigation: merge PR #29 before or with
  this PR (decision 5).
- The FEATURE-03 code can start before PR #30 merges. Signal: the deadline
  test with 7 calls fails. Mitigation: build order step 0.
- The word "TypeSafe API" is not correct when `JAG_BASE_URL` points to a
  gateway. Signal: a gateway user reports it. Mitigation: FEATURE-21.
- The fix of exit 5 names the console home page, not a credit page.
  Signal: the TypeSafe docs add a credit page. Mitigation: change the text.

## Out of scope

- The readiness check `--check`: FEATURE-02. It uses the time options of
  this spec.
- Ctrl-C and exit 130: FEATURE-15.
- The cost limit and exit 7: FEATURE-06. The size limit and exit 8:
  FEATURE-09.
- A failure as a JSON document in structured mode: FEATURE-11.
- An AI gateway and its words in the texts: FEATURE-21.
- A key in the config file for the timeout: DF-1.
