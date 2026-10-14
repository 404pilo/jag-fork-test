---
title: "Cost limit on scoring: the BM25 pre-rank, the limit option, the skip notice, and exit 7"
feature: FEATURE-06
stories: [STORY-18, STORY-19, STORY-20]
kind: build
lifecycle_state: draft
inherits: [QS-1, QS-3, QS-4, IF-3, IF-4, IF-6, C-cli, C-config, C-search, C-prerank, C-scorer, CV-2, CV-3, CV-5, CV-6, CV-7, DF-1]
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Cost limit on scoring: the BM25 pre-rank, the limit option, the skip notice, and exit 7

## Problem

When a term matches too many lines, jag pre-ranks the matches by word
match and scores only the best part, up to a cost limit. The user can
change the limit. jag says how many matches it skipped (FEATURE-06,
CAPABILITY-02, DECISION-12). When jag reaches the limit, it exits with its
own code. Exit 1 wins when nothing answers well (DECISION-13). The
cost-limit code wins over the code for results that the size limit leaves
out (DECISION-47).

- STORY-18: an operator wants jag to pre-rank a large set of matches by
  word match, score only the best ones, and say how many it skipped.
  - CRITERION-25: when the matches are more than the cost limit, jag
    pre-ranks them by word match and scores only the best ones, up to the
    limit.
  - CRITERION-27: when jag reaches the cost limit, it returns the ranked
    results and says how many matches it skipped.
- STORY-19: an agent builder wants a default cost limit that an option can
  change.
  - CRITERION-26: jag has a fixed default cost limit, stated as a count of
    results to score, and the user can change it with an option.
- STORY-20: an agent builder wants jag to exit with its own code when it
  reaches the cost limit, and that code to win over the left-out code.
  - CRITERION-28: when jag reaches the cost limit, it exits with its own
    exit code, and exit code 1 wins when nothing answers well.
  - CRITERION-66: when a search reaches the cost limit and also leaves out
    results, jag exits with the cost-limit code, and the notice says that
    results were left out.

FEATURE-04 built a temporary cut: it scores the first 200 results in path
order, writes "jag: scored 200 of N results", and exits 0 (FEATURE-04 spec
decision 1). This spec replaces that cut. It adds the BM25 pre-rank
(C-prerank), the option and the variable of the limit, the final skip
notice, exit 7, and the one function that gives the exit code of a search.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Name of the limit setting | The option `--max-scored COUNT` and the variable `JAG_MAX_SCORED`. The other options were `--candidates` / `JAG_CANDIDATES` and `--cost-limit` / `JAG_COST_LIMIT` | human:404pilo, 2026-10-14 (Q-aw) |
| 2 | Default limit | 200 results | 0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md; DECISION-12; system-design.md strategy-1 |
| 3 | Values of the limit | A whole number from 1 to 2,000: 1 to 4 ASCII digits and nothing else. Each other value gives exit 2. 2,000 results are about 63 calls, or 11 rounds of 6 calls. With the call times of ADR-0002 (about 0.5 s for each round), that is about 6 s, which fits the 9-second deadline. A search of 2,000 results costs about 10 times the ADR-0002 values: about $0.02 in a typical case and $0.09 in the worst case. The cap of METRIC-05 and the time of CONSTRAINT-06 apply to the default limit. A larger limit is the trade of cost for a wider search that STORY-19 gives to the user | Inferred: system-design.md CV-5 (the time settings take whole numbers in a fixed range; the limit uses the same form); FEATURE-03 spec decision 2; DECISION-12; METRIC-05; 0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md |
| 4 | Unit of the limit and of the counts | Results: the planned results after the join of near matches, not the matching lines. Scored + skipped = all results | DECISION-12 ("a count of results to score"); DECISION-42; FEATURE-04 spec decision 1 |
| 5 | Setting order | The option wins over the variable. No key in the config file | system-design.md CV-7, DF-1; FEATURE-03 spec decision 9 |
| 6 | Where jag checks the setting | The option: with the other arguments in step 1, after the time options, before the key. The variable: in step 3, after the model settings | FEATURE-03 spec decision 11; FEATURE-04 spec decision 18 |
| 7 | Secrecy of a value | No message shows the value of the option or of the variable | system-design.md CV-6; FEATURE-03 spec decision 10; FEATURE-04 spec decision 26 |
| 8 | Text that the pre-rank reads | Only the matching lines of a result, not the lines around them. The scan already holds the text of each matching line (`MatchLine.text`), so the pre-rank reads no file | Inferred: STORY-18 and CRITERION-25 ("pre-ranks the matches"); FEATURE-04 spec decision 20 (no file read before the cut); system-design.md QS-3, strategy-5 |
| 9 | Pre-rank method and code | BM25 on the question words. Own code in `src/prerank.rs`. No new crate | system-design.md strategy-5, C-prerank; 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md (module `prerank`); 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md (each new crate needs a reason) |
| 10 | Words | Make the text with `String::from_utf8_lossy`. Cut each matching line at 300 characters, as the request does. Lower case with `str::to_lowercase`. Split on each character that is not `char::is_alphanumeric`. No stop words, no stems | This spec. The cut is FEATURE-04 spec decision 21: the pre-rank reads the same text that the model gets |
| 11 | BM25 values | k1 = 1.2, b = 0.75. idf = ln(1 + (N - n + 0.5) / (n + 0.5)). The document frequency n and the mean length come from the N results of the search. Each question word counts one time, in the order of the question | This spec. These are the usual values. The idf form of Lucene `BM25Similarity` stays above 0 when a word is in most results, for example the term itself (context7, `/apache/lucene`) |
| 12 | Equal BM25 scores | By path (byte order), then by first line | FEATURE-04 spec decision 28; FEATURE-05 spec decision 11 (the scan gives this order) |
| 13 | Result with no question word | It scores 0 and stays in the list. jag can keep it when fewer than `limit` results score more than 0 | This spec |
| 14 | Order of the kept results | The kept results go to the model in path and line order. So the calls are the same on each run. The final order comes from the model scores | This spec; FEATURE-04 spec decision 28 |
| 15 | When the limit applies | Only when the results are more than the limit. A count equal to the limit gives no notice and no exit 7. When the results are not more than the limit, jag does no pre-rank | This spec; DECISION-13 (exit 7 means that jag did not score every result) |
| 16 | Skip notice | `jag: cost limit reached: scored L of N results, skipped S.` on the error stream, with S = N - L. It replaces "jag: scored 200 of N results" | DECISION-42; DECISION-13; system-design.md CV-3; CRITERION-27 |
| 17 | Exit code of the limit | 7 | 0007-number-the-exit-codes-above-2-from-3-to-8.md; system-design.md IF-4; DECISION-13 |
| 18 | Exit order | 1 wins, then 7, then 8, then 0. The notice still shows when exit 1 wins. One function gives the code, with unit tests for each case. FEATURE-09 sets "left out" and adds the end-to-end test of exit 8 | DECISION-13; DECISION-47; system-design.md IF-4; the rule of the user answer Q-ar (build only the part of this feature now, with a unit test) |
| 19 | Model failures | A model failure (exit 4, 5, or 6) gives its own code and no result, also when the limit applies. The notice is already on the error stream | system-design.md CV-2; FEATURE-03 spec decision 7 |
| 20 | Deadline | It stays `2 x timeout + retry wait` for each limit | system-design.md CV-5; CONSTRAINT-07; DECISION-52 ("This time covers all calls of the search") |
| 21 | Names in the code | The code says "cost limit" (`cost_limit`, `CostLimitBad`, `DEFAULT_COST_LIMIT`), the name of the PRD (DECISION-12). Only the option text, the variable name, and the message texts hold the name of decision 1 | This spec: one name for the concept in the code and the PRD |

## Design

### Files

```
src/prerank.rs          C-prerank: words, question_words, scores, best
src/search/snippet.rs   C-search: match_texts
src/config/limit.rs     C-config: the limit constants, parse_cost_limit, cost_limit
src/config.rs           C-config: ConfigError::CostLimitBad, SettingSource
src/config/model.rs     C-config: TimeSource becomes SettingSource
src/cli.rs              C-cli: --max-scored, the search flow, SearchOutcome, search_exit_code
src/scorer.rs           C-scorer: MAX_SCORED goes away
tests/cost_limit.rs     end-to-end tests of CRITERION-25 to CRITERION-28
tests/usage.rs          the limit argument tests
tests/search.rs         only_the_first_200_results_are_scored changes (see "Changed tests")
```

`src/lib.rs` adds `pub mod prerank;`. `src/config.rs` adds `mod limit;`.
No new crate.

This spec uses these items of the earlier specs:

- FEATURE-05: `scan`, `ScanReport`, `FileMatches`, `MatchLine`. The scan
  gives the files sorted by path (byte order) and the lines sorted by
  number (FEATURE-05 spec decision 11).
- FEATURE-04: `plan_windows`, `Window`, `read_snippets`, `score`, `rank`,
  `write_text`, `MAX_LINE_CHARS`, the fake server.
- FEATURE-03 (PR #33): the `parse_seconds` pattern, `TimeSource`,
  `ConfigError::TimeBad`, `model_settings(env, options)`, and the usage
  line with `--timeout` and `--retry-wait`.

### The limit setting (C-cli, C-config)

The `Cli` struct gets one option:

```rust
/// The most results that the model scores, from 1 to 2,000 (default 200)
#[arg(long = "max-scored", value_name = "COUNT", conflicts_with_all = ["setup", "forget_key"])]
cost_limit: Option<String>,
```

- The type is `String`, not a number. jag parses the text itself, so that
  the message is its own and never shows the value (decision 7). A clap
  error still gives the fixed line of the FEATURE-04 argument error rule.
- The usage text of the search line is
  `jag [--timeout SECONDS] [--retry-wait SECONDS] [--max-scored COUNT] TERM "QUESTION" [PATH...]`.

`src/config/limit.rs` adds:

```rust
pub const COST_LIMIT_OPTION: &str = "--max-scored";
pub const COST_LIMIT_VARIABLE: &str = "JAG_MAX_SCORED";
pub const DEFAULT_COST_LIMIT: usize = 200;
pub const MAX_COST_LIMIT: usize = 2_000;

pub fn parse_cost_limit(text: &str, source: SettingSource) -> Result<usize, ConfigError>;
pub fn cost_limit(env: &Env, option: Option<usize>) -> Result<usize, ConfigError>;
```

`parse_cost_limit`:

1. Trim the text.
2. The text must be 1 to 4 ASCII digits and nothing else. So `1`, `200`,
   `2000`, and `0400` pass. `0`, `2001`, `02000`, `1.5`, `-1`, `+1`,
   `1e3`, `2,000`, `200 results`, and a blank text fail.
3. Parse it as `usize`. The value must be from 1 to `MAX_COST_LIMIT`, both
   included.
4. Each failure gives `ConfigError::CostLimitBad { source }`.

`cost_limit` gives the option when it is set (decision 5). Else it reads
`JAG_MAX_SCORED`: blank counts as not set, as FEATURE-01 decision 14 says.
A value that is not text, or that `parse_cost_limit` refuses, gives
`CostLimitBad { source: Variable }`. Else it gives `DEFAULT_COST_LIMIT`.

The enum `TimeSource` of FEATURE-03 becomes `SettingSource`, with the same
two values `Option` and `Variable`, because the time settings and the
limit both use it. Change each use in `src/config.rs`, `src/config/model.rs`,
and `src/cli.rs`.

`ConfigError::CostLimitBad` gives exit 2:

| Source | Cause | Fix |
|---|---|---|
| Option | `the value of --max-scored is not a whole number from 1 to 2,000.` | `Use a number such as --max-scored 400.` |
| Variable | `JAG_MAX_SCORED is not a whole number from 1 to 2,000.` | `Set JAG_MAX_SCORED to a number such as 400, or unset it.` |

Build the texts from `COST_LIMIT_OPTION` and `COST_LIMIT_VARIABLE`.

`scorer::MAX_SCORED` goes away. `DEFAULT_COST_LIMIT` takes its place.
The scorer does not know the limit: the cut comes before the second read.

### The matching text of a result (C-search)

```rust
/// The text of the matching lines of each window, in the order of `windows`.
pub fn match_texts<'a>(files: &'a [FileMatches], windows: &[Window]) -> Vec<Vec<&'a [u8]>>;
```

`windows` must be the output of `plan_windows(files)`. `plan_windows` puts
each matching line of `files` in exactly one window, in order. So window k
holds the next `windows[k].matches.len()` matching lines, in the order of
`files` and their lines. `match_texts` walks the lines of `files` one time
and gives each window its lines. A `debug_assert!` checks that the line
numbers are the same as `window.matches`.

This adds no field to `Window` and reads no file (decision 8).

### The pre-rank (C-prerank)

```rust
pub const K1: f64 = 1.2;
pub const B: f64 = 0.75;

pub fn words(text: &str) -> Vec<String>;
pub fn question_words(question: &str) -> Vec<String>;
pub fn scores(texts: &[Vec<&[u8]>], question: &str) -> Vec<f64>;
pub fn best(texts: &[Vec<&[u8]>], question: &str, limit: usize) -> Vec<usize>;
```

`words(text)` (decision 10): lower-case the text with `str::to_lowercase`.
Split it on each character for which `char::is_alphanumeric` is false.
Drop the empty parts. Examples:

- `Retry-After: 30s, RÉSUMÉ` gives `retry`, `after`, `30s`, `résumé`.
- `retry_wait` gives `retry`, `wait`.
- `???` gives no word.

`question_words(question)`: the `words` of the question, each word one
time, in the order of its first use. `Retry the RETRY, retry_wait?` gives
`retry`, `the`, `wait`.

`scores(texts, question)`, for N results (decision 11):

1. The document of result k: for each of its matching lines, make the text
   with `String::from_utf8_lossy`, cut it at `MAX_LINE_CHARS` (300)
   characters, and take its `words`. The document is the words of all its
   matching lines.
2. Keep only two facts for each document: its length `dl` (the count of
   its words), and the count `tf` of each question word in it. Do not keep
   the other words.
3. `avgdl` is the mean `dl` of the N documents. When `avgdl` is 0, each
   score is 0.
4. For each question word q, `n(q)` is the count of documents that hold q
   one time or more. `idf(q) = ln(1 + (N - n(q) + 0.5) / (n(q) + 0.5))`.
5. The score of a document is the sum, over the question words in their
   order, of
   `idf(q) x tf x (K1 + 1) / (tf + K1 x (1 - B + B x dl / avgdl))`.
   A question word with `tf = 0` adds 0. Add in the order of the question
   words, so the sum is the same on each run.

`best(texts, question, limit)`:

1. When `texts.len() <= limit`, give `0..texts.len()` and compute nothing
   (decision 15).
2. Else compute `scores`. Sort the indices by score, highest first, with a
   stable sort and `f64::total_cmp`. The input is in path and line order
   (FEATURE-05 spec decision 11). So equal scores stay in path order, then
   in first-line order (decision 12).
3. Take the first `limit` indices. Sort them in increasing order (decision
   14) and give them.

A question with no word gives each result the score 0. `best` then gives
the first `limit` results in path and line order.

### The search flow (C-cli)

The search flow of FEATURE-04, as FEATURE-03 changes it, changes in these
steps:

- Step 1: after `--timeout` and `--retry-wait`, parse `--max-scored` with
  `parse_cost_limit(text, SettingSource::Option)`. A failure gives exit 2
  before the key (decision 6). `SearchArgs` gets
  `cost_limit: Option<usize>`.
- Step 3: after `model_settings(env, args.times)`, call
  `cost_limit(env, args.cost_limit)`. A failure gives exit 2.
- Step 6: `let windows = plan_windows(&report.files);` and
  `let total = windows.len();`. N is `total`.
- Step 7 (replaces "keep the first 200"): when `total > limit`, compute
  `match_texts(&report.files, &windows)`, then
  `prerank::best(&texts, args.question, limit)`, and keep the windows at
  those indices. Else keep all windows. `kept` is in path and line order.
- Step 8: `read_snippets(&kept)`. It reads only the files of the kept
  results (FEATURE-04 spec decision 20).
- Steps 9 and 10: no change. When there is no snippet, jag makes no call
  and exits 1, before the skip notice.
- Step 11 (replaces "scored 200 of N"): `let cost_limit_reached = total >
  limit;`. When it is true, write the skip notice to the error stream
  (decision 16):
  `jag: cost limit reached: scored {kept.len()} of {total} results, skipped {total - kept.len()}.`
- Steps 12 and 13: no change. A model failure gives its own exit code
  (decision 19).
- Step 14: `write_text` gives `no_good_answer`. The exit code is
  `search_exit_code(SearchOutcome { no_good_answer, cost_limit_reached,
  left_out: false })`.

Example: 250 results and the default limit give
`jag: cost limit reached: scored 200 of 250 results, skipped 50.`

### The exit code of a search (C-cli)

```rust
/// What a search that got its scores found.
struct SearchOutcome {
    no_good_answer: bool,
    cost_limit_reached: bool,
    /// FEATURE-09 sets this when the size limit leaves out results.
    left_out: bool,
}

/// IF-4 order: 1 wins, then 7, then 8, then 0 (DECISION-13, DECISION-47).
fn search_exit_code(outcome: SearchOutcome) -> u8;
```

| no good answer | cost limit reached | left out | Exit |
|---|---|---|---|
| yes | any | any | 1 |
| no | yes | any | 7 |
| no | no | yes | 8 |
| no | no | no | 0 |

This function covers only a search that got its scores. A usage error (2),
a key error (3), a model failure (4, 5, 6), and a search with no match (1)
end before it.

### Acceptance criteria and their tests

All end-to-end tests use the fake server and the test setup of the
FEATURE-04 spec: `HOME` in a `tempfile` folder, the key text
`SENTINEL-SENTINEL-SENTINEL`, `JAG_BASE_URL` of the fake server, no proxy
variables, and `env_remove("JAG_MAX_SCORED")` unless the test sets it. In
each test, standard output and stderr do not hold `SENTINEL`.

The folder of the "large folder" tests: 250 files `f000.txt` to `f249.txt`,
each with the line `needle`, and 5 files `z0.txt` to `z4.txt`. `z0.txt`
holds `needle retry`, and `z1.txt` to `z4.txt` hold `needle retry wait`.
The question is `how long is the retry wait`. So N = 255. In path order,
the `z` files come last.

| Criterion | Test |
|---|---|
| CRITERION-25 jag scores only the best matches | `tests/cost_limit.rs`: the large folder with the default limit. The server gets exactly 200 snippets, and they hold the 5 `z` files. The cut of FEATURE-04 did not send them. With `--max-scored 5`, the server gets exactly the 5 `z` snippets in one call, and the `path` fields of the request are in path order, `z0` to `z4`. Unit tests of `prerank` (see "More tests"). |
| CRITERION-26 The cost limit has a default that the user can change | The large folder: no setting gives 200 snippets. `--max-scored 10` gives 10. `JAG_MAX_SCORED=10` gives 10. `--max-scored 10` with `JAG_MAX_SCORED=20` gives 10 (the option wins). `JAG_MAX_SCORED=` (blank) gives 200. `--max-scored 2000` gives 255 and no notice. Unit tests of `parse_cost_limit` and `cost_limit` (see "More tests"). |
| CRITERION-27 jag says how many matches it skipped | The large folder with the default limit: stderr holds the line `jag: cost limit reached: scored 200 of 255 results, skipped 55.`, and standard output holds 200 results. With `--max-scored 5`: `scored 5 of 255 results, skipped 250.`, and 5 results. 200 files give no `cost limit` text on stderr. |
| CRITERION-28 Reaching the cost limit has its own exit code | The server gives 0.8 to each snippet. The large folder exits 7. 200 files exit 0, and 199 files exit 0. The server gives 0.2 to each snippet: the large folder exits 1, the first line of standard output is `NO_GOOD_ANSWER`, and stderr still holds the skip notice. |
| CRITERION-66 Cost-limit code wins when both limits apply | Unit tests of `search_exit_code`, one for each of the 8 cases of the table: each case with "no good answer" gives 1, "cost limit reached" with "left out" gives 7, "left out" alone gives 8, and none gives 0. FEATURE-09 sets `left_out`, and adds the end-to-end test and the left-out notice. |

More tests:

- Words (unit tests of `words` and `question_words`): each example of "The
  pre-rank".
- BM25 values (unit tests of `scores`, with a margin of 1e-9):
  - The texts `retry now` and `wait here`, the question `retry`: the
    scores are ln 2 (0.693147...) and 0.
  - The texts `retry`, `retry a b`, and `x y`, the question `retry`: the
    scores are 0.590862..., 0.390192..., and 0. A shorter result with the
    same word ranks higher.
  - The texts `retry` and `retry`: each score is ln 1.2 (0.182322...),
    which is more than 0. A word in each result still counts.
  - The question `retry retry` gives the same scores as `retry`.
  - Texts with no word (`???` and `!!`) give the scores 0 and 0, with no
    NaN.
  - A line of 300 characters (`x ` 150 times), then `retry`, scores 0,
    because the word is after the cut.
- `best` (unit tests):
  - The texts `needle`, `needle`, and `needle retry`, the question
    `retry`, and the limit 2 give `[0, 2]`: the best, then the first of
    the equal scores, in increasing order.
  - The question `???` and the limit 2 of 3 texts give `[0, 1]`.
  - 3 texts and the limit 3 give `[0, 1, 2]`.
- `match_texts` (unit tests): 2 files. The first file has matches on lines
  10, 14, and 30. The second has a match on line 2. `plan_windows` gives 3
  windows. `match_texts` gives the texts of lines 10 and 14, of line 30, and
  of line 2.
- Limit values (unit tests of `parse_cost_limit`): `1`, `200`, `2000`,
  `0400`, and ` 10 ` pass. `0`, `2001`, `02000`, `1.5`, `-1`, `+1`, `1e3`,
  `2,000`, `200 results`, `abc`, and a blank text fail. Each case runs for
  both sources.
- The option wins (unit tests of `cost_limit`): an option and a variable,
  a variable only, a blank variable, and neither.
- Limit secrecy (`tests/usage.rs`): `--max-scored SENTINEL-SENTINEL-SENTINEL`
  exits 2, and `JAG_MAX_SCORED=SENTINEL-SENTINEL-SENTINEL` exits 2. stderr
  does not hold `SENTINEL`. stderr is exactly `jag: <cause>\n<fix>\n` of
  the table in "The limit setting".
- Order: `--max-scored abc` with no key exits 2 (decision 6).
  `JAG_MAX_SCORED=abc` with no key exits 3, because the key comes first.
- `--setup --max-scored 5` and `--forget-key --max-scored 5` exit 2 with
  the fixed line.
- `jag --help` shows `--max-scored <COUNT>` and the usage line of "The
  limit setting".
- A failure with the limit: the large folder, and the server gives 500 to
  each call. jag exits 6, standard output is empty, and stderr holds the
  skip notice and the exit 6 text (decision 19).
- Same calls on each run: run the large folder with `--max-scored 40` two
  times. The calls run in parallel, so compare the requests as sets: the
  two runs send the same requests, and each request holds the same
  snippets in the same order.

#### Changed tests

- `tests/search.rs` `only_the_first_200_results_are_scored`: it changes
  name to `the_default_limit_scores_200_results`. 250 files give exit 7
  and the new notice
  `jag: cost limit reached: scored 200 of 250 results, skipped 50.` The
  server still gets 200 snippets, and standard output still holds 200
  results.
- The FEATURE-04 unit test of the second read in `src/search/snippet.rs`
  ("250 windows, keep the first 200") does not use `MAX_SCORED`, so it
  stays.

## Build order

0. The FEATURE-03 code (PR #33) is on `main`. This spec changes the same
   files (`src/cli.rs`, `src/config.rs`, `src/config/model.rs`) and uses
   its `parse_seconds` pattern and its usage line.
1. `words`, `question_words`, `scores`, and `best` in `src/prerank.rs`,
   with their unit tests (STORY-18).
2. `match_texts` in `src/search/snippet.rs`, with its unit test.
3. `SettingSource`, `parse_cost_limit`, `cost_limit`, and `CostLimitBad`,
   with their unit tests (STORY-19).
4. The `--max-scored` option, the search flow steps 1, 3, 6, 7, and 11,
   the removal of `MAX_SCORED`, the end-to-end tests of CRITERION-25,
   CRITERION-26, and CRITERION-27, and the changed test (STORY-18,
   STORY-19).
5. `SearchOutcome` and `search_exit_code` with the tests of CRITERION-28
   and CRITERION-66 (STORY-20).

## Assumptions

- The scan gives the files in path order (byte order) and the lines in
  line order. So the order of `plan_windows` is the tie order of decision
  12. Source: `src/search/walk.rs` (the sort by `path_key`) and FEATURE-05
  spec decision 11.
- `plan_windows` puts each matching line in exactly one window, in order.
  `match_texts` depends on it. Source: `src/search/snippet.rs`
  `plan_windows`.
- A file can change between the scan and the second read. Then
  `read_snippets` can drop a kept result, and fewer results go to the
  model than the notice says. The read warning shows it. This is the
  FEATURE-04 rule ("The notice still says N").

## Risks

- A limit near 2,000 can reach the deadline of CV-5, because the deadline
  does not grow with the limit (decision 20). Signal: exit 6 with "did not
  answer in time" and no 429 or 5xx, with a large `--max-scored`.
  Mitigation: the user sets a larger `--timeout`. FEATURE-16 measures the
  time of a call.
- BM25 on a very large N (for example 100,000 results) costs time before
  the first call. Signal: a slow search before the first call, in the
  FEATURE-16 measures. Mitigation: `scores` keeps only `dl` and the
  question word counts, and each line is cut at 300 characters.
- BM25 with no stems misses a word in another form (`retries` and
  `retry`). Signal: the `jag-bench` results show answers that the pre-rank
  cuts. Mitigation: a later change of the pre-rank, measured with
  `jag-bench`. The model scores the kept results, so only the cut is at
  risk.
- PR #33 (FEATURE-03) and this spec change the same files. Signal: a merge
  conflict in `src/cli.rs` or `src/config/model.rs`. Mitigation: build
  order step 0.
- The FEATURE-08 spec (draft) moves `MAX_LINE_CHARS` to
  `src/search/snippet.rs` and changes `rank` and `SnippetLine`. Signal: a
  merge conflict, or two constants for the cut. Mitigation: use the one
  constant `MAX_LINE_CHARS` where it is on `main` when the code starts.

## Out of scope

- Exit 8, the size limit, and the left-out notice: FEATURE-09. It sets
  `left_out` and adds the end-to-end test of CRITERION-66.
- The counts of scored and skipped results as fields of structured output:
  FEATURE-11 (DECISION-28).
- The cost line in money and tokens: FEATURE-20.
- The time of a search with a large limit: FEATURE-16.
- `--max-scored` for `--check`: `--check` makes one free call (FEATURE-02),
  so the limit has no effect on it.
- A key in the config file for the limit: DF-1.
- Stems, stop words, and other pre-rank changes: a later change, measured
  with `jag-bench`.
