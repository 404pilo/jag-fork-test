---
title: "Rank matching lines by a question: the search command, results with lines around them, and the no-good-answer label"
feature: FEATURE-04
stories: [STORY-13, STORY-14, STORY-15]
kind: build
lifecycle_state: draft
inherits: [QS-1, QS-2, QS-3, QS-4, IF-1, IF-2, IF-3, IF-4, IF-6, C-cli, C-search, C-scorer, C-output, CV-1, CV-2, CV-3, CV-4, CV-5, CV-6, CV-7]
session: 21508ce9-788b-40ed-be8d-03797016b20b
---

# Rank matching lines by a question: the search command, results with lines around them, and the no-good-answer label

## Problem

The user runs `jag TERM "QUESTION" [PATH...]`. jag ranks the lines that
match the term by how well they answer the question (FEATURE-04,
CAPABILITY-02, DECISION-31). When no result answers the question well, jag
shows the best results with a clear label and exits with code 1
(DECISION-11).

- STORY-13: a knowledge seeker runs jag with a term, a question, and paths,
  and gets the matching lines in order of how well they answer the
  question.
  - CRITERION-18: jag accepts `jag TERM "QUESTION" [PATH...]`, with the
    term first as in grep.
  - CRITERION-19: the best answer comes first.
  - CRITERION-20: the term is a regular expression, as in grep.
  - CRITERION-71: a term with no matches makes no model call, writes no
    results, and exits with code 1.
  - CRITERION-85: a ranked search of a folder says how many files it
    skipped.
- STORY-14: each result holds the matching line with a few lines before
  and after it, and near matches join into one result.
  - CRITERION-21: a result holds the lines around the match.
- STORY-15: jag labels the best results as weak and exits with code 1 when
  no result answers well.
  - CRITERION-29: weak results have a clear label.
  - CRITERION-30: no good answer exits with code 1.

This spec connects the folder walk of FEATURE-05 and the key lookup of
FEATURE-01 to a model client. It builds the search command, the results,
the scorer, and the text output.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Results to score before FEATURE-06 | At most 200 results: the first 200 in path and line order. When there are more, jag writes "jag: scored 200 of N results" to the error stream. FEATURE-06 adds the BM25 pre-rank, the option, and exit 7 | human:404pilo, 2026-10-13 (Q-u); 0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md |
| 2 | Scoring prompt | Shape B: all results of a call go in `state`, with one `noul` question for each result. No `criteria`. A neutral question text. Version id `jag-p1` | human:404pilo, 2026-10-13 (Q-x); 0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md |
| 3 | No-good-answer threshold | 0.4 on the top score. Tune it with `jag-bench` on `jag-p1` before release 1 | human:404pilo, 2026-10-13 (Q-y); 0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md |
| 4 | Lines around a match | 3 lines before and 3 lines after. Windows that touch or overlap join. A result has at most 12 lines | human:404pilo, 2026-10-13 (Q-z); DECISION-10 |
| 5 | Results that jag shows | All scored results, in rank order. FEATURE-09 cuts them to the size limit later | human:404pilo, 2026-10-13 (Q-aa) |
| 6 | Place of the no-good-answer label | The first line on standard output, before the results | human:404pilo, 2026-10-13 (Q-ae); DECISION-11 |
| 7 | Model failures | All codes of ADR-0007 now: 4 key not accepted, 5 no credit, 6 service down, with the retry of CV-5. FEATURE-03 adds `--timeout`, `JAG_TIMEOUT`, and the final texts | human:404pilo, 2026-10-13 (Q-af); 0007-number-the-exit-codes-above-2-from-3-to-8.md; CV-5 |
| 8 | HTTP and JSON crates | `ureq` 3 with rustls and the machine certificate store (feature `platform-verifier`, `RootCerts::PlatformVerifier`), 6 standard threads, and `serde_json` | human:404pilo, 2026-10-13 (Q-ag); 0014-call-the-model-service-with-ureq-and-serde-json-on-std.md (PR #21) |
| 9 | Service address and model | The variables `JAG_BASE_URL` and `JAG_MODEL`. Defaults `https://api.typesafe.ai/v1` and `jev-1.13.0` | human:404pilo, 2026-10-13 (Q-ah); IF-2; 0009-pin-the-jev-model-version-in-each-release.md |
| 10 | Spec grouping | One spec for STORY-13, STORY-14, and STORY-15. A result (STORY-14) is the unit that jag scores (STORY-13), and the label (STORY-15) reads the top score | human:404pilo, 2026-10-13 (Q-ai) |
| 11 | Calls | 32 results at most in each call, 6 calls in parallel | 0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md (Q-aj, inherited) |
| 12 | Threshold form | A fixed value, with no option | 0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md (Q-aj, inherited) |
| 13 | Prompt form | One built-in prompt with a version id, with no option | 0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md (Q-aj, inherited) |
| 14 | Model | The pinned model `jev-1.13.0`. `jev-latest` points to it on 2026-10-13 | 0009-pin-the-jev-model-version-in-each-release.md; TypeSafe docs (Q-aj, inherited) |
| 15 | Match search | `scan` of the FEATURE-05 spec finds the matches. The term is a regular expression with the rules of `rg` (RK-1). It is case-sensitive, as in grep. `-F` comes with FEATURE-13 | 0001-find-matches-with-the-ripgrep-libraries.md; docs/superpowers/specs/2026-10-13-STORY-16-search-a-folder-as-grep-r-design.md (Q-aj, inherited) |
| 16 | Failure, stream, and key rules | CV-2: a model failure fails the full search, with no partial result. CV-3: results go to standard output, and notices go to the error stream. CV-6: the key goes only into the `Authorization` header | system-design.md CV-2, CV-3, CV-6 (Q-aj, inherited) |
| 17 | Key lookup | `find_key` of the FEATURE-01 spec. No key gives exit 3 | docs/superpowers/specs/2026-10-13-STORY-01-store-a-key-one-time-design.md (Q-aj, inherited) |
| 18 | Key before the scan | jag finds the key before it reads a file. So a missing key exits 3 before any file read. Only an argument error of step 1 (exit 2) comes before it | This spec, from the FEATURE-04 ASK |
| 19 | Paths and plain grep | Until FEATURE-14, a search needs one PATH or more. Until FEATURE-12, a term with no question is a usage error, exit 2 | This spec, from the FEATURE-04 ASK |
| 20 | Lines around a match | jag plans the results from the match line numbers, with no read. Then it reads only the files of the first 200 results a second time, to get the lines around each match | This spec, from the FEATURE-04 ASK |
| 21 | Long lines | In the request, jag cuts each line at 300 characters. The text output shows the full line. FEATURE-08 shows the cut | This spec, from the FEATURE-04 ASK |
| 22 | Call size | jag closes a call when the next result makes its estimate more than 24,000 tokens. The docs limit `state` and the longest question to 32k tokens. 32 results (decision 11) is the upper limit of ADR-0002, and the token limit can close a call sooner. This does not change the cost or the 6 parallel calls. ADR-0002 did not know the 32k limit of the API. The estimate is the UTF-8 bytes divided by 3, so that text that is not ASCII (for example CJK) does not go past the limit | This spec, from the FEATURE-04 ASK; TypeSafe docs, Limits |
| 23 | No match | No call, nothing on standard output, exit 1 | This spec, from the FEATURE-04 ASK; CRITERION-71 |
| 24 | Notices | After the scan, jag writes the skip notice and the warnings to the error stream | This spec, from the FEATURE-04 ASK; CRITERION-85 |
| 25 | Model test server | A fake server on 127.0.0.1 that uses only `std`. No new test crate | This spec, from the FEATURE-04 ASK |
| 26 | Argument errors | An argument error never repeats a value from the command line, because the value can be a key. One exception (2026-10-14): the error of an unknown option names the option when the text that clap gives has the form of an option, and never shows a value (FEATURE-13 spec decision 18) | CV-6; PR #17 review (verifier-1): clap repeats an unexpected argument; human:404pilo, 2026-10-14 (hitl answer FEATURE-13 "The name of an unknown option in the error": the exception) |
| 27 | Text form of a result | The form of `grep -n` with lines around the match: `path:N:text` for a matching line, `path-N-text` for another line, and `--` between results. No score in text mode | This spec; DECISION-15; FEATURE-08 owns the final form |
| 28 | Rank order | By score, highest first. Equal scores: by path (byte order), then by first line | This spec; the same reason as FEATURE-05 decision 11 |
| 29 | Plain `http` | `JAG_BASE_URL` can use `http` only when the host is exactly `127.0.0.1`, `localhost`, or `[::1]`. Each other `http` address, and each address with a user name, gives exit 2, because the key goes in the clear | This spec; CV-6 |
| 30 | Question | A blank question gives exit 2. A question of more than 12,000 bytes (UTF-8) gives exit 2, because each call holds it. By the estimate of decision 22, this is about 4,000 tokens | This spec; decision 22 |
| 31 | Search deadline | One deadline of 9 seconds for all model calls of a search. No call and no retry starts after it. Each attempt waits at most the time that is left. At the deadline, jag exits 6 | This spec; CV-5 (fail within 9 seconds); PR #23 review |

## Design

### Files

```
src/cli.rs                  C-cli: the search form, the argument error rule, the search flow
src/config.rs               C-config: adds ModelSettings and model_settings()
src/search/snippet.rs       C-search: Snippet, plan_windows, read_snippets (lines around a match, join)
src/scorer.rs               C-scorer: score(), PROMPT_VERSION, THRESHOLD, rank()
src/scorer/prompt.rs        the request body of jag-p1, the batches
src/scorer/client.rs        the ureq agent, one call, the retry, the status map
src/output.rs               C-output: the text form, the label
tests/search.rs             end-to-end tests with the fake server
tests/usage.rs              the argument error tests
tests/common/fake_server.rs the fake model server (std only)
```

`src/lib.rs` adds `pub mod scorer;` and `pub mod output;`. `search.rs`
adds `pub mod snippet;`.

`Cargo.toml` adds `ureq` 3 and `serde_json`. Keep rustls as the TLS of
`ureq`, and add no `native-tls`. Turn on the `ureq` feature
`platform-verifier`, so that jag uses the certificate store of the
machine (ADR-0014). Without the feature, `RootCerts::PlatformVerifier`
compiles, but the first HTTPS call panics with "requires feature:
platform-verifier". Turn off each other default feature of `ureq` that this
spec does not use. The pull request names the reason for each
crate: 0014-call-the-model-service-with-ureq-and-serde-json-on-std.md
(PR #21) and ADR-0012.

This spec uses these items of the earlier specs:

- FEATURE-05: `scan`, `ScanOptions::default()`, `ScanReport`,
  `FileMatches`, `MatchLine`, `SkipCounts::notice`, `WalkWarning`,
  `WalkWarning::message`.
- FEATURE-01: `config::find_key`, `config::Env`, `config::RealFs`,
  `ApiKey`, `ApiKey::expose` (`pub(crate)`), `Failure`, `Failure::render`.

### Command line (C-cli)

The `Cli` struct of FEATURE-01 gets three positional arguments (IF-3,
CRITERION-18):

```rust
term: Option<String>,        // TERM
question: Option<String>,    // QUESTION
paths: Vec<PathBuf>,         // PATH...
```

- `--setup` and `--forget-key` conflict with each positional argument.
- The help text shows the usage `jag TERM "QUESTION" [PATH...]`. Set it
  with `#[command(override_usage = ...)]`, because clap derive gives
  `jag [OPTIONS] [TERM] [QUESTION] [PATHS]...`.
- A term with no question gives exit 2 (decision 19): "jag: a search needs
  a question. Use: jag TERM \"QUESTION\" PATH...".
- A term and a question with no path gives exit 2 (decision 19): "jag: a
  search needs a path. Searching standard input comes later.".
- A blank question, or a question of more than 12,000 bytes (UTF-8),
  gives exit 2 (decision 30). The message does not show the question.

#### The argument error rule (CV-6)

clap repeats a value from the command line in many errors. For example,
`jag --setup <key>` gives "unexpected argument '<key>' found". So jag never
writes the text of a clap error (decision 26):

1. Parse with `Cli::try_parse()`, never with `Cli::parse()`.
2. For `ErrorKind::DisplayHelp`, `ErrorKind::DisplayVersion`, and
   `ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand`, call `print()` on
   the error, then exit with `error.exit_code()`. These texts hold no value
   from the command line.
3. For each other kind, write one fixed line to the error stream and exit
   with 2:
   `jag: the command line is not valid. Run jag --help to see the usage.`
   Do not write the error, its context, its suggestions, or its "tip"
   lines.

The same rule holds for each message of this spec. A message never shows
the term, the question, `JAG_BASE_URL`, or `JAG_MODEL`. A bad regular
expression gives only "jag: the term is not a valid regular expression.",
exit 2. The error text of `grep-regex` quotes the pattern, so jag does not
write it.

### Settings (C-config)

```rust
pub struct ModelSettings { pub base_url: String, pub model: String }
pub fn model_settings(env: &Env) -> Result<ModelSettings, ConfigError>;
```

- `JAG_BASE_URL`: trim it. Blank counts as not set, as FEATURE-01 decision
  14 says. Remove each `/` at the end. Default `https://api.typesafe.ai/v1`.
- Parse the address with `ureq::http::Uri`. Do not compare a prefix of
  the text. The address must have an authority with no `@` (no user name
  and no password). The scheme must be `https`, or `http` with a host that
  is exactly `127.0.0.1`, `localhost`, or `[::1]` (decision 29). Compare
  the whole host, not a part of it. A test proves the form that
  `Uri::host` gives for `[::1]`. Else: `BaseUrlBad`,
  exit 2, cause "JAG_BASE_URL is not an https address", fix "Set
  JAG_BASE_URL to an https address, or unset it". The text does not show
  the value, because an address can hold a user name and a password.
- `JAG_MODEL`: trim it. Blank counts as not set. Default `jev-1.13.0`.
- These two have no option and no key in the config file yet (CV-7, DF-1).
- The call goes to `<base_url>/systemone`.

### The search flow (C-cli)

All errors of the search go through one function. The `finish` function
of FEATURE-01 changes to take a `Failure`. `impl From<ScoreError> for
Failure` gives exit 4, 5, or 6. `ConfigError` keeps its `From`. Each usage
case of this spec (no question, no path, a bad question, a bad regular
expression) builds a `Failure` with exit 2.

`cli::run` runs a search in this order:

1. Parse the arguments (see "Command line").
2. Find the key with `find_key` (decision 18). An error gives its failure
   and exit 3.
3. Read the model settings. An error gives exit 2.
4. Build the matcher: `grep_regex::RegexMatcherBuilder::new()` with
   `line_terminator(Some(b'\n'))`, then `build(term)`. An error gives exit
   2 (see "The argument error rule").
5. `scan(paths, &matcher, &ScanOptions::default())`.
6. `plan_windows(&report.files)` (see "Results"). This step reads no
   file. N is the count of planned results after the join, not the count
   of matching lines (decision 1).
7. Keep the first 200 planned results, in path and line order. Drop the
   rest.
8. `read_snippets(&kept)`. This reads only the files of the kept results
   (decision 20).
9. Write to the error stream, in this order: the skip notice of
   `report.skipped.notice()`, each warning of the scan, then each warning
   of `read_snippets`, with `message()`. This is CRITERION-85.
10. When there is no snippet: write nothing to standard output, make no
    call, and exit 1 (CRITERION-71).
11. When N is more than 200, write "jag: scored 200 of N results" to the
    error stream.
12. `scorer::score(...)`. A failure gives its message and its exit code,
    and nothing goes to standard output (CV-2).
13. `scorer::rank(...)` (decision 28).
14. `output::write_text(...)` to standard output, then exit 0, or exit 1
    when the label shows.

jag keeps all output in memory until step 14. So a failure before step 14
writes no partial result (CV-2). For Ctrl-C, see "Assumptions".

When a kept file changed after the scan, `read_snippets` can drop a
snippet. Then fewer than 200 snippets go to the model, and jag does not
take more from the dropped rest. The notice still says N.

The warnings of the scan do not change the exit code in this spec.
`grep -r` gives exit 2 after a read error. FEATURE-14 decides that rule
for jag.

### Results (C-search)

```rust
pub struct SnippetLine { pub number: u64, pub text: Vec<u8>, pub is_match: bool }
pub struct Snippet { pub path: PathBuf, pub lines: Vec<SnippetLine> }   // IF-6
pub const CONTEXT_LINES: u64 = 3;
pub const MAX_LINES: u64 = 12;
pub struct Window { pub path: PathBuf, pub start: u64, pub end: u64, pub matches: Vec<u64> }
pub fn plan_windows(files: &[FileMatches]) -> Vec<Window>;
pub fn read_snippets(windows: &[Window]) -> (Vec<Snippet>, Vec<WalkWarning>);
```

The name is `Snippet`, not `Result`, so that it does not hide
`std::result::Result`. A `Snippet` is one result of IF-6. The marks on
cut lines come with FEATURE-08.

`plan_windows` uses only the match line numbers. It reads no file. For
each `FileMatches`, in the order of the scan, it joins the matches into
windows, in line order. There is one rule. For a match on line `m`:

- First match: the window is `[max(1, m - 3), m + 3]`.
- Each next match: when `m <= start + 11` and `m - 3 <= end + 1`, join.
  Add `m` to the matches of the window, and set
  `end = min(max(end, m + 3), start + 11)`.
- Else: close the window as a result. The new window is
  `[max(m - 3, end + 1), m + 3]`. It holds line `m`, and it does not
  repeat a line of the result before it.

So a window never has more than 12 lines (decision 4). The after lines of
a match that do not fit in the 12 lines are dropped. Examples:

- 10 and 14 give `[7, 17]`.
- 10 and 16 give `[7, 18]`, and 10, 13, and 16 also give `[7, 18]`.
- 1, 4, 7, 9, 11, and 14 give `[1, 12]` and `[13, 17]`.
- 10 and 30 give `[7, 13]` and `[27, 33]`.

`read_snippets` reads each file of the kept windows one time, in order
(decision 20):

1. Read the file with `std::fs::read`. A read error gives a `WalkWarning`
   with `Read(kind)` and the path. The windows of the file give no result.
2. Split the bytes on `\n`. Line N is item N - 1. When the file ends with
   `\n`, drop the empty item after the last `\n`. So a file of 5 lines
   that ends with `\n` has `L = 5`, not 6. Keep a `\r` at the end of a line,
   as grep does.
3. The file can change after the scan. Clamp the end of each window at
   `L`. Drop a match that is past `L`. Drop a window with no match left.
4. Each window that is left gives one `Snippet` with its lines. `is_match`
   is true for each match of the window.

The scan gives the files sorted by path and the lines sorted by number.
So the results are in path and line order.

### Scorer (C-scorer)

```rust
pub const PROMPT_VERSION: &str = "jag-p1";
pub const THRESHOLD: f64 = 0.4;
pub const MAX_SCORED: usize = 200;

pub struct ScoreSettings {
    pub base_url: String,
    pub model: String,
    pub timeout: Duration,      // 4 s (CV-5)
    pub retry_wait: Duration,   // 1 s (CV-5)
    pub deadline: Duration,     // 9 s (CV-5, decision 31)
}

pub fn score(snippets: &[Snippet], question: &str, key: &ApiKey,
             settings: &ScoreSettings) -> Result<Vec<f64>, ScoreError>;
pub fn rank(snippets: Vec<Snippet>, scores: Vec<f64>) -> Vec<(Snippet, f64)>;
```

`score` gives one score for each snippet, in the order of the input.
`timeout`, `retry_wait`, and `deadline` are fields so that unit tests can
make them short. `cli` sets 4 s, 1 s, and 9 s. The deadline is
`2 x timeout + retry_wait`: the time of one call, its wait, and its retry.
FEATURE-03 keeps this sum when it adds the timeout option. FEATURE-03 adds the option and the
variable. C-check and C-bench call the same `score` and the same client
later (ADR-0014).

#### The request of `jag-p1`

The body of each call:

```json
{
  "model": "<model>",
  "state": {
    "question": "<QUESTION>",
    "snippets": [
      { "n": 1, "path": "src/a.txt", "lines": "line 1 text\nline 2 text" }
    ]
  },
  "questions": {
    "s1": { "type": "noul", "instructions": "Does snippet 1 answer the question?" }
  }
}
```

- `n` counts from 1 in each call. The id of question `n` is `s<n>`.
- The API does not send question ids to the model. So the instruction
  text names the snippet number, and the same number is in `state`.
- The instruction text is the constant
  `Does snippet <n> answer the question?`. It has no `criteria`. It holds
  no word of a domain and no text of the earlier prototype.
- `path` is the path as the scan gives it, with `to_string_lossy`.
- `lines` holds the lines of the snippet, joined with `\n`. jag makes each
  line text with `String::from_utf8_lossy`, then cuts it to 300 characters
  at a character border (decision 21).

#### Batches

1. Take the snippets in order. Put each one in the current call.
2. The estimate of a call is the count of UTF-8 bytes of its `state`
   JSON and of its longest question, divided by 3. For CJK text, one
   character is 3 bytes and about one token, so the estimate is not too
   low (decision 22).
3. Close the call before a snippet that makes the estimate more than
   24,000, or when the call has 32 snippets (decisions 11 and 22).
4. One snippet always fits. 12 lines of 300 characters give at most
   21,600 bytes of JSON, when each character is a control character that
   JSON writes as 6 bytes. That is 7,200 tokens. With a question of 12,000
   bytes (4,000 tokens) and a path, the estimate stays below 24,000.

With full snippets of long lines, a call holds fewer than 32 snippets. 200
such snippets need about 8 calls, in two rounds of 6.

#### Parallel calls

- `std::thread::scope` with at most 6 threads. The threads take the next
  call number from an `AtomicUsize`.
- A shared `AtomicBool` stop flag: after the first failure, no thread
  starts a new call. A call that already runs comes to its end. Before a
  retry, a thread reads the stop flag. When the flag is set, the thread
  does not retry, and the call gives its first failure.
- The deadline (decision 31): `score` sets `end = Instant::now() +
  deadline` one time. Before each attempt, also a retry, a thread
  computes the time left. When no time is left, the call fails as a
  timeout (exit 6) and sets the stop flag. Else the attempt uses
  `min(timeout, time left)` as its timeout, through the request config of
  `ureq` (`.config().timeout_global(...)`).
- Each result goes to the slot of its call number. So the scores map back
  to the snippets in order.
- When a call fails, `score` gives the failure of the lowest call number
  that failed. So the message is the same on each run.

#### The client

One `ureq::Agent` for the search:

```rust
Agent::config_builder()
    .tls_config(TlsConfig::builder().root_certs(RootCerts::PlatformVerifier).build())
    .timeout_global(Some(settings.timeout))
    .max_redirects(0)
    .http_status_as_error(false)
    .build()
    .new_agent()
```

Each call is `POST <base_url>/systemone` with these headers:
`Authorization: Bearer <key>`, `Content-Type: application/json`,
`Accept: application/json`, and `User-Agent: jag/<version>`. Only this
code calls `ApiKey::expose` (CV-6).

The response:

```rust
#[derive(Deserialize)]
struct Response { answers: HashMap<String, Answer>, usage: Option<Usage> }
#[derive(Deserialize)]
struct Answer { noul: f64 }
#[derive(Deserialize)]
struct Usage { input_tokens: u64 }
```

serde ignores the other fields. jag reads `usage` but does not use it yet
(FEATURE-20). Each `s<n>` of the call must have an answer with a finite
`noul` from 0 to 1. Else the body is not readable.

#### Failures and the retry

| Case | Retry (CV-5) | Exit | Cause text |
|---|---|---|---|
| HTTP 200 with a readable body | - | - | - |
| HTTP 401 or 403 | no | 4 | the TypeSafe API did not accept the key (HTTP <status>) |
| HTTP 402 | no | 5 | the TypeSafe account has no credit (HTTP 402) |
| Timeout (`ureq::Error::Timeout`) | yes | 6 | the TypeSafe API did not answer in time |
| Each other `ureq` error, for example no connection | yes | 6 | jag cannot connect to the model service |
| HTTP 429 | yes | 6 | the TypeSafe API refused more calls (HTTP 429) |
| HTTP 500 to 599 | yes | 6 | the TypeSafe API failed (HTTP <status>) |
| HTTP 300 to 399 | no | 6 | the model service sent a redirect |
| Each other status, for example 408 or 422 | no | 6 | the TypeSafe API gave HTTP <status> |
| HTTP 200 with a body that jag cannot read | no | 6 | the TypeSafe API gave a response that jag cannot read |

- A retry waits `retry_wait`, then tries one more time. The second failure
  gives the exit of its own case. CV-5 does not retry 408, so this spec
  does not.
- The fixes: exit 4 "Check the key at https://console.typesafe.ai/keys,
  then run jag --setup." Exit 5 "Add credit to the TypeSafe account."
  Exit 6 "Try again later." These texts are short forms. FEATURE-03 writes
  the final texts.
- A failure text never holds the response body, the error text of `ureq`,
  the address, or the key. The body can repeat the request, and the
  `ureq` text can hold the address.

#### Rank

`rank` sorts the snippets by score, highest first, with `f64::total_cmp`.
Equal scores sort by path (byte order of the path), then by the first line
(decision 28). The top score is the first score after the sort.

### Text output (C-output)

```rust
pub const NO_GOOD_ANSWER: &str =
    "jag: no good answer. The results below do not answer the question well.";
pub fn write_text(out: &mut dyn Write, ranked: &[(Snippet, f64)]) -> io::Result<bool>;
```

- When the top score is below `THRESHOLD`, the first line is
  `NO_GOOD_ANSWER`, and `write_text` returns true (decision 6). A top score
  of exactly 0.4 is a good answer.
- Then each result, in rank order (decision 5). Each line of a result is
  `path:N:text` for a matching line, and `path-N-text` for another line
  (decision 27). `text` is the bytes of the line as the file holds them.
- A line `--` comes between two results.
- A write error with `ErrorKind::BrokenPipe` ends jag with no message and
  with the exit code of the search (CV-4).

Example:

```
src/net.rs-38-    let wait = Duration::from_secs(1);
src/net.rs:41:    // retry one time after a timeout
src/net.rs-42-    retry(call, wait)
--
docs/notes.md:7:The client waits one second, then tries again.
```

### Acceptance criteria and their tests

All end-to-end tests run the `jag` binary with `assert_cmd`. Each test:

- sets `HOME` to a `tempfile` folder;
- sets `JAG_TYPESAFE_API_KEY` to `SENTINEL-SENTINEL-SENTINEL`, and calls
  `env_remove("TYPESAFE_API_KEY")`;
- sets `JAG_BASE_URL` to `http://127.0.0.1:<port>/v1` of the fake server;
- calls `env_remove` for `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`,
  `NO_PROXY`, and their lower-case forms, because `ureq` reads them.

The fake server (`tests/common/fake_server.rs`) binds `127.0.0.1:0` with
`std::net::TcpListener`. It reads each request head and its
`Content-Length` body, and it keeps each request for the test. It replies
with HTTP/1.1 and `Connection: close`. The test gives a reply rule, for
example a status, or a score from the text of each snippet. The server
parses the request with `serde_json` to find the snippets. It counts the
calls that run at the same time.

The key text `SENTINEL-SENTINEL-SENTINEL` passes the private-data check.
Do not use a key text that looks like a real key.

| Criterion | Test |
|---|---|
| CRITERION-18 The command takes the term, the question, and the paths | `tests/search.rs`: `jag needle "which line says how to retry" <dir>`, with a server that gives 0.8 to each snippet, exits 0, and the server gets one call. `jag --help` shows `jag TERM "QUESTION" [PATH...]`. Unit: a term with no question, and a term and a question with no path, give exit 2 and make no call. |
| CRITERION-19 The best answer comes first | Three files with `needle`. The server gives 0.9 to the snippet of `b.txt` and 0.1 and 0.3 to the others. The first result on standard output is from `b.txt`, then the 0.3 result, then the 0.1 result. Unit: `rank` with equal scores orders by path, then by the first line. |
| CRITERION-20 The term is a regular expression | The term `ne+dle` finds `needle` and `neeedle`, and not `Needle` (case-sensitive). The term `(` gives exit 2, stderr does not hold the term, and the server gets no call. |
| CRITERION-21 A result holds the lines around the match | Unit tests of `plan_windows`: each example of "Results" (10 and 14; 10 and 16; 10, 13, and 16; 1, 4, 7, 9, 11, and 14; 10 and 30). A match on line 2 starts at line 1. Unit tests of `read_snippets`: a match on the last line ends at the last line. A file of 5 lines that ends with `\n`, with a match on line 5, gives lines 2 to 5 and no line 6. The same file with no `\n` at the end gives the same lines. A file that has fewer lines than a match number drops that match. End-to-end: the output holds `path-9-` and `path:10:` lines and `--` between two results. |
| CRITERION-29 Weak results have a clear label | The server gives 0.2 to each snippet. The first line of standard output is `NO_GOOD_ANSWER`, and the results follow. |
| CRITERION-30 No good answer exits with code 1 | The same test exits 1. A top score of 0.4 exits 0 with no label. A top score of 0.39 exits 1. |
| CRITERION-71 A term with no matches makes no model call | A term with no match: exit 1, empty standard output, and the server gets no call. With no key and no match: exit 3 (decision 18). |
| CRITERION-85 A ranked search says how many files it skipped | A folder with `.hidden.txt`, a binary file, and `ok.txt`, each with `needle`: stderr holds `jag: skipped 2 files and folders (1 binary, 1 hidden, 0 ignored)`. The notice also shows when no file matches. |

More tests:

- The argument error rule (decision 26): `jag --setup
  SENTINEL-SENTINEL-SENTINEL`, `jag --setup=SENTINEL-SENTINEL-SENTINEL`,
  `jag --SENTINEL-SENTINEL-SENTINEL`, and `jag --forget-key needle
  SENTINEL-SENTINEL-SENTINEL` each exit 2. stderr is the fixed line and
  does not hold `SENTINEL`. `jag --help` and `jag --version` exit 0.
- Key secrecy: in each end-to-end test, standard output and stderr do not
  hold `SENTINEL`. The server finds the key only in the `Authorization`
  header, and never in the body.
- Failures: 401 and 403 give exit 4. 402 gives exit 5. 500 two times gives
  exit 6 and 2 calls. 500, then 200, gives exit 0 and 2 calls. 429 two
  times gives exit 6. 422 gives exit 6 and 1 call. 302 gives exit 6 and 1
  call. A body that is not JSON, an answer that is missing, and a `noul`
  of 1.5 each give exit 6. In each case, standard output is empty, and
  stderr does not hold the response body.
- TLS: set `JAG_BASE_URL` to `https://127.0.0.1:<port>/v1` of a std
  `TcpListener` that accepts the connection and closes it. jag exits 6,
  and stderr does not hold `panicked`. Without the `platform-verifier`
  feature, this test panics, because jag sets up TLS before the
  handshake. The implementer proves this one time: remove the feature,
  run the test, see it fail, and say so in the pull request.
- Timeout: a unit test of `score` with a `timeout` of 200 ms and a
  `retry_wait` of 50 ms. The server waits 1 s. `score` gives a
  `ScoreError` that maps to exit 6, after 2 calls.
- Stop flag: a unit test of `score` with 10 calls. The first call gets
  401 at once. Each other call gets 500 after 100 ms. The server gets 6
  first calls or fewer, and no retry starts after the 401 reply. The
  result maps to exit 4.
- Parallel: a unit test of `score` with 20 calls. Each reply waits 100 ms.
  The server never sees more than 6 calls at the same time.
- Deadline: a unit test of `score` with 8 calls, a `timeout` of 200 ms, a
  `retry_wait` of 50 ms, and a `deadline` of 450 ms. Each reply waits
  150 ms. The first reply to calls 7 and 8 is 500, and the second reply
  is 200 with a score. The retries start at about 350 ms and have 100 ms
  left, so they time out. `score` gives a `ScoreError` that maps to exit
  6 in less than 1 s. With no deadline, the retries succeed and the
  result is exit 0.
- Each unit test of `score` sets `deadline = 2 x timeout + retry_wait`
  when it does not say another value. The Stop flag and Parallel tests
  set a deadline of 5 s.
- Score limit: 250 files, each with one match. The server gets exactly 200
  snippets. stderr holds `jag: scored 200 of 250 results`. Standard output
  holds 200 results.
- Batches: unit tests of the batch split. 70 short snippets give calls of
  32, 32, and 6. Full snippets of 12 lines of 300 characters give calls
  with an estimate of 24,000 or less. 32 snippets of 12 lines of 80 CJK
  characters (about 92,000 bytes) give two calls or more, each with an
  estimate of 24,000 or less.
- Question limit: a question of 12,000 bytes is accepted. A question of
  12,001 bytes gives exit 2. 4,001 CJK characters (12,003 bytes) give
  exit 2.
- Long line: a matching line of 1,000 characters. The request holds 300
  characters of it. Standard output holds all 1,000.
- Request form: the body has `model` from `JAG_MODEL`, and one `s<n>`
  question for each snippet with the instruction text of `jag-p1`.
- Settings: `http://example.com/v1`, `http://localhost.example.net/v1`,
  `http://127.0.0.1@example.net/v1`, and `https://user@example.com/v1` in
  `JAG_BASE_URL` each give exit 2, and stderr does not hold `example`.
  `http://127.0.0.1:<port>/v1`, `http://localhost:<port>/v1`, and
  `http://[::1]:<port>/v1` are accepted. A `/` at the end of
  `JAG_BASE_URL` is removed.
- Warnings: a file that `read_snippets` cannot read gives the warning on
  stderr after the scan notices, and the other files are scored.
- Second read: a unit test plans 250 windows in 250 paths and keeps the
  first 200. The last 50 paths do not exist. `read_snippets` gives 200
  snippets and no warning, so it did not read the dropped paths.

## Build order

0. ADR-0014 (PR #21), the FEATURE-01 project (PR #20), and the FEATURE-05
   library (PR #22, merge 719a6f1) are on `main`.
1. `plan_windows` and `read_snippets` with their unit tests (STORY-14,
   CRITERION-21).
2. The settings, the request of `jag-p1`, and the batches, with their unit
   tests.
3. The client, the retry, the status map, and the parallel calls, with
   the fake server and the failure tests.
4. The search form in `cli`, the argument error rule, the search flow, and
   the end-to-end tests of CRITERION-18, CRITERION-19, CRITERION-20,
   CRITERION-71, and CRITERION-85 (STORY-13).
5. `rank`, the label, and the text output, with the tests of CRITERION-29
   and CRITERION-30 (STORY-15).

## Assumptions

- The TypeSafe docs list 401, 422, 429, and 529 as errors. They do not
  list 402 or 403. jag maps them as ADR-0007 says. 529 is in the 5xx
  range, so jag retries it.
- This spec adds no signal handler. The default action of Ctrl-C stops
  jag with no output, because jag keeps all output in memory until the
  end. A shell then shows exit 130. A program that starts jag sees the
  signal, not exit code 130 (see "Risks").
- A file can change between the scan and the second read. jag then uses
  the lines that it reads, and drops a match that is past the end.

## Risks

- The threshold 0.4 belongs to the earlier prototype prompts, not to
  `jag-p1`. Signal: the `jag-bench` decoy AUC on `jag-p1` is below 0.85
  (QS-2). Mitigation: tune the threshold with `jag-bench` before release 1.
  A new value needs a new prompt version (ADR-0003).
- Rate limits. The docs give 100K tokens and 80 requests each second, and
  they say that the limits change (docs.typesafe.ai, Limits and pricing).
  6 calls in parallel stay far below 80. Signal: exit 6 with HTTP 429 in
  normal use. Mitigation: the retry of CV-5.
- The deadline of 9 seconds (decision 31) can stop a slow search that
  would succeed, for example 8 slow calls in two rounds. Signal: exit 6
  with "did not answer in time" and no 429 or 5xx. Mitigation: FEATURE-03
  measures it and sets the final timeout.
- With no signal handler, a program that starts jag sees the signal
  SIGINT and not exit code 130 (CV-4). Signal: a caller reports it.
  Mitigation: a later feature adds a handler. A new crate for it needs a
  reason in the pull request (ADR-0012).
- A user types a key as a PATH. The scan warning then shows it as a path
  that jag cannot read. Signal: a key text in a "cannot read" warning.
  Mitigation: FEATURE-14 decides the message for a path that does not
  exist.
- Before FEATURE-06, a broad term scores only the first 200 results in
  path order, so the answer can be missing. Signal: the "scored 200 of N"
  notice. Mitigation: FEATURE-06.

## Out of scope

- The BM25 pre-rank, the cost limit option, and exit 7: FEATURE-06.
- The final path form and the marks on cut lines: FEATURE-08.
- The size limit and exit 8: FEATURE-09.
- Structured output, `prompt_version`, and the structured label:
  FEATURE-11.
- Plain grep with no question: FEATURE-12.
- `-F`, `-A`, `-B`, and `-C`: FEATURE-13.
- Standard input, a mix of files and folders, and a path that does not
  exist: FEATURE-14.
- The cost line and the use of `usage.input_tokens`: FEATURE-20.
- An AI gateway with its own settings: FEATURE-21.
- `--timeout`, `JAG_TIMEOUT`, and the final failure texts: FEATURE-03.
- `--check`: FEATURE-02.
