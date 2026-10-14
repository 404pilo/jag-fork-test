---
title: "Show the cost of a search: the cost line, the cost field, the cost with no figure, and the cost of a failed or stopped search"
feature: FEATURE-20
stories: [STORY-53, STORY-54, STORY-55, STORY-56]
kind: build
gh_issue: 115
lifecycle_state: draft
inherits: [QS-3, IF-1, IF-2, IF-4, IF-5, C-cli, C-config, C-scorer, C-output, CV-1, CV-2, CV-3, CV-4, CV-5, CV-6, CV-7]
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Show the cost of a search: the cost line, the cost field, the cost with no figure, and the cost of a failed or stopped search

## Problem

CAPABILITY-07: "After each search, jag reports how many results it scored
and what the search cost."

- STORY-53: "As a shell power user (PERSONA-04), I want a cost line after
  each search, so that I know what each search costs."
  - CRITERION-88: "In text mode, jag writes one cost line to the error
    stream after each search, also when the output goes to a pipe."
  - CRITERION-89: "The cost line shows the count of results that jag
    scored, the count that the cost limit skipped, and the billed amount in
    US dollars."
- STORY-54: "As an agent builder (PERSONA-01), I want the cost as a field
  of structured output, so that my agent can track what its searches cost."
  - CRITERION-90: "In structured mode, the output always has a cost field
    with the same counts and amount as the text cost line."
- STORY-55: "As an agent builder (PERSONA-01), I want a clear cost report
  when jag makes no model call or the service gives no cost figure, so that
  the search still works."
  - CRITERION-91: "When jag makes no model call, it reports a cost of zero
    and the reason."
  - CRITERION-92: "When the service gives no cost figure and jag cannot
    estimate one, jag shows the count, marks the cost as not available, and
    the search does not fail."
  - CRITERION-97: "When the service gives no billed amount, jag shows an
    estimate in US dollars and labels it as an estimate."
- STORY-56: "As an agent builder (PERSONA-01), I want to know what a failed
  or stopped search cost, so that no cost is hidden from me."
  - CRITERION-93: "When a search fails or the user stops it, the error
    gives the cost that jag paid up to that point. In structured mode, the
    cost is a field of the error."

The PRD decisions:

- DECISION-40: "jag shows the cost in US dollars. It uses the amount that
  the model service billed. When the service gives no billed amount, jag
  shows an estimate and labels it as an estimate."
- DECISION-41: "In text mode, jag writes a cost line to the error stream
  after each search, also when the output goes to a pipe. In structured
  mode, the cost is always a field."
- DECISION-42: "The report counts results. It shows the results that jag
  scored and the results that the cost limit skipped. The two counts add up
  to all the results that the term found."
- DECISION-43: "When a search makes no model call, jag reports a cost of
  zero and gives the reason, such as "no question"."
- DECISION-44: "When a search fails or the user stops it, the error message
  gives the cost that jag paid up to that point. In structured mode, the
  cost is a field of the error."
- DECISION-45: "When the model service gives no cost figure and jag cannot
  estimate one, jag shows the count and marks the cost as not available. A
  missing cost figure never fails the search."

Today jag does not read the cost of a call. `src/scorer/client.rs` reads
only `answers` from the response, and `usage` goes away. `score` in
`src/scorer.rs` gives the first failure and drops the result of each call
that ended. So a failed search loses its cost. FEATURE-15 built the stop
on Ctrl-C. Its stop failure and its stop document have no cost yet.
FEATURE-12 built plain grep: with no `--ask`, jag makes no model call. Its
spec gives the cost line and the reason "no question" to this feature.
FEATURE-09 built the size limit, which counts each field of the document.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Scope | One spec for STORY-53 to STORY-56. The line, the field, and the cases with no figure use one cost type | docs/adlc.md (one feature, one spec) |
| 2 | The figure | The cost is the sum, over each answered attempt, of `usage.input_tokens` times the price of the model. Output tokens are free. The service gives no amount in money | DECISION-40; system-design.md IF-1 ("The response gives the probabilities and the billed usage"); TypeSafe docs, 09-api-reference.md (`usage`) and 12-models.md ("Rough cost math": 0.042 US dollars for 1 million input tokens, output tokens are free) (inferred) |
| 3 | The price | 42 billionths of a US dollar for each input token of `jev-1.13.0`. `price_of(model)` in src/config/model.rs, next to `DEFAULT_MODEL`, gives `Some(42)` for `DEFAULT_MODEL` and `None` for each other name. A unit test fails when `DEFAULT_MODEL` has no price | ADR-0009 (each release pins the model, so the price changes with the pin); TypeSafe docs, 12-models.md; ADR-0002 ("The price is $0.042 for each 1M input tokens"); the earlier prototype (0.042 US dollars for 1 million input tokens) (inferred) |
| 4 | The model and the route | The price goes by the model name that jag sends (`settings.model`), compared exactly with `DEFAULT_MODEL`. A blank `JAG_MODEL` counts as not set, as now. The route, the TypeSafe API or a gateway, does not change the price | src/config/model.rs (`model_settings`: "A blank variable counts as not set"); CV-7; system-design.md IF-2 ("The same calls as IF-1") and the flow C-gateway to C-typesafe: the model service is TypeSafe on both routes, and the gateway passes the calls on; the FEATURE-21 spec (on `main`), "The route" ("So the fake server of the tests, on `127.0.0.1`, is a gateway": a price by route makes each end-to-end test "not available") (inferred) |
| 5 | The attempts that count | Each attempt that the service answers with HTTP 200, also a retry, and also an attempt that ends after another call fails. An attempt with another status, a timeout, or a lost connection adds nothing. An attempt that `Run::attempt` does not start after the deadline adds nothing. After a Ctrl-C, jag does not wait for the calls that run, so they add nothing | DECISION-40 (jag knows a bill only from an answer); DECISION-44 ("up to that point"); CV-5 (the one retry); src/scorer.rs (`std::thread::scope` waits for each call in flight after a failure); the FEATURE-15 spec decision 12 ("jag does not wait for the calls that run") (inferred) |
| 6 | A count that jag cannot read | For an answered attempt with no `usage`, a `null` count, a count that is not a whole number, or a body that jag cannot read, jag counts its own estimate: the UTF-8 bytes of the request body divided by 3, rounded down. The total is then an estimate | DECISION-40; CRITERION-97; TypeSafe docs, 09-api-reference.md ("Token usage, when reported"; each count can be null); FEATURE-04 spec decision 22 and src/scorer/prompt.rs `estimate` (the ratio: bytes divided by 3) (inferred) |
| 7 | The kind of the figure | One kind for each search, in this order: `no_call` (jag made no model call), `not_available` (the model has no price), `estimate` (one answered attempt or more had no count that jag can read), `billed` (each other case, also when no attempt got an answer) | DECISION-40; DECISION-43; DECISION-45; CRITERION-91; CRITERION-92; CRITERION-97 (inferred) |
| 8 | A missing count never fails the search | jag reads `usage.input_tokens` apart from the answers. `parse_scores` does not change. A bad count gives no failure and no retry | DECISION-45; CRITERION-92; src/scorer/client.rs (`call` gives `BadResponse` when `parse_scores` gives `None`) |
| 9 | The counts | "Scored" is `results_scored` and "skipped" is `results_skipped`: K and S of the cost-limit notice. N is `results_found`. With a question, K + S = N. With no question, K and S are 0, so the line says `scored 0 of N results, skipped 0`, and the reason `no question` tells why (decision 33). No new count field | DECISION-42; FEATURE-11 spec decision 14; the FEATURE-12 spec decision 14 (with no question, `results_scored` and `results_skipped` are 0); README.md, the table of fields ("With no question, `results_found` is the count of matching lines, and the other two are `0`"); tests/output.rs (the `criterion_51_*` tests expect K also when each call fails) (inferred) |
| 10 | The amount | In US dollars, with 6 decimals, for example `$0.002016`. jag adds the exact amount in billionths of a dollar as whole numbers. Then it rounds the sum one time to millionths: 500 billionths or more go up. The line and the field use the same whole number | ADR-0002 ("200 results cost about $0.009 in the worst case and about $0.002 in a typical case"); system-design.md, the row "QS-3 and METRIC-05 cost"; README.md (the check call costs "less than $0.00001"), so 2 decimals show $0.00 for each search; the earlier prototype (6 decimals) (inferred) |
| 11 | The cost line | One line: `jag: cost <amount>: scored K of N results, skipped S.` The amount part is `$0.002016`, `$0.002016 (estimate)`, `not available`, or `$0.000000 (<reason>)`. The line holds only numbers and fixed words, and never the words `cost limit` | DECISION-41; CRITERION-88; CRITERION-89; CRITERION-91; CRITERION-92; CRITERION-97; CV-3; CV-6; FEATURE-06 spec decision 16 (the words "scored L of N results, skipped S"); tests/output.rs (each line on the error stream starts with `jag: `); tests/cost_limit.rs (no `cost limit` when nothing is skipped) (inferred) |
| 12 | The place of the line | The last line on the error stream: after the progress line goes away, after the skip notice, the warnings, the cost-limit notice, and the size-limit notice, after the results, and after a failure text or the stop failure | DECISION-41 ("after each search"); DECISION-44 ("the error message gives the cost"); the FEATURE-15 spec, "Other features" ("the cost line comes after the progress line is removed"); src/cli.rs (`search`: "The size notice comes after the cost notice, and before the output") (inferred) |
| 13 | The runs that write the line | In text mode, each search that gets to the count of its results: the call of `record` in `search` or in `plain` (and in `write_report` of FEATURE-13), which sets `results_found` and calls `stop.count`. These end with exit 0, 1, 4, 5, 6, 7, or 8, with 2 for a failed write or for a path that does not exist (FEATURE-14), or by SIGINT after a Ctrl-C. A run that ends before the count writes no line: an argument error, no key, a bad setting, a bad term, a Ctrl-C during the walk, `--setup`, `--forget-key`, `--check`, `--help`, and `--version`. A closed pipe writes no line (decision 21) | CRITERION-88 ("after each search"); src/cli.rs (`search` and `plain`: the scan gives no failure, so each search that gets past the term gets to the count); tests/usage.rs, tests/check.rs, and tests/failure.rs (no key) keep their exact texts (inferred) |
| 14 | No model call | The amount is `$0.000000`, and the line gives the reason: `no question` (no `--ask`: plain grep, decision 33), `no match` (the term matched no line), or `no readable match` (jag could read no line of a match). `no question` wins: plain grep with no match gives `no question`, because `search` gives the work to `plain` before the scan (src/cli.rs line 453) | DECISION-43 ("such as "no question""); CRITERION-91; src/cli.rs (`plain`; `snippets.is_empty()` in `search`); the FEATURE-12 spec, "Other features" ("a search with no model call reports a cost of zero and the reason "no question"") (inferred) |
| 15 | A failed search | The failure text stays as it is. The cost line comes after it, with the cost of each answered attempt. In structured mode, the error document has the `cost` field. "A field of the error" is a field of the error document, because an error is a document of the same form | DECISION-44; CRITERION-93; system-design.md IF-5 ("An error is a JSON document of the same form") and CV-1; FEATURE-11 spec decision 21 (inferred) |
| 16 | The cost-limit notice | It stays as it is, before the calls. The cost line comes after the search. When the limit applies, the two lines show the same counts | FEATURE-06 spec decision 16 (the notice) and decision 18 ("The notice still shows when exit 1 wins"); src/cli.rs (jag writes the notice before the calls, so it also shows when a call fails); DECISION-41 (inferred) |
| 17 | The field | `cost`, an object, after `warnings_left_out` and before `results`, in each document: `micro_usd` (the amount as a whole number of millionths of a US dollar, or `null` when not available), `kind` (`billed`, `estimate`, `not_available`, or `no_call`), and `reason` (`null`, `no_question`, `no_match`, `no_readable_match`, or `not_started`) | CRITERION-90 ("always has a cost field"); DECISION-41; system-design.md IF-5; CONSTRAINT-05; ADR-0008; FEATURE-11 spec decisions 5 ("A new field goes before `results`") and 14; FEATURE-11 spec, "The document" (each number is a whole number) and "Out of scope" ("The cost of a search as a field (DECISION-41): FEATURE-20") (inferred) |
| 18 | A document before the count | The argument-error document, the error document of each failure before the count, and the stop document of a Ctrl-C before the count have `"cost":{"micro_usd":0,"kind":"no_call","reason":"not_started"}`. This is the default value | FEATURE-11 spec decision 21 ("0 for a step that did not run"); src/cli.rs (`parse` and `run` start from `Summary::default()`); src/cli/stop.rs (`counted` starts as `Summary::default()`) (inferred) |
| 19 | No new ADR | The field is an addition under ADR-0008: fields are only added. FEATURE-11 added its fields in the same way | ADR-0008; the FEATURE-11 spec (no ADR for its fields) (inferred) |
| 20 | Structured mode and the error stream | jag writes nothing on the error stream, as now. When jag cannot write the document, and the reader did not close the pipe, it writes the failure text of FEATURE-11 decision 25 and then the cost line | CV-3; FEATURE-11 spec decision 25; DECISION-44 (inferred) |
| 21 | A closed pipe | When the reader closes standard output, jag writes no cost line and no other text, in both modes. The watch thread ends jag by SIGPIPE with no message, or `emit` finds the closed pipe during its write and gives `Emitted::Closed`, so `finish` writes no line | DECISION-39 ("with no message when the pipe closes"); CV-4; the FEATURE-15 spec decision 15; src/cli.rs (`emit`) |
| 22 | A stopped search | A Ctrl-C after the count and before the search flow takes the output: in text mode, the stop writes the stop failure and then the cost line. In structured mode, the stop document has the `cost` field. The cost holds each attempt that ended before the stop. `Stop` holds the usage of the search, so the stop thread can read it. A Ctrl-C before the count writes the stop failure only, and its document has `not_started` (decision 18). A Ctrl-C after the search flow takes the output ends jag at once with no more text, so the line can be absent or cut | DECISION-44; CRITERION-93; the FEATURE-15 spec, decision 12 (the stop failure and the stop document), decision 14 ("When the search flow owns the output, a later Ctrl-C ends jag at once, with no text. The output can then be cut"), "Other features" ("DECISION-44 adds the cost that jag paid to the stop failure and to the stop document. The cost count must then be where the stop thread can read it, for example in `Stop`"), and "Out of scope" ("The cost of a stopped search (DECISION-44) and the cost line (DECISION-41): FEATURE-20") (inferred) |
| 23 | Exit codes | No change. The cost never changes the exit code or the end by a signal | ADR-0007; system-design.md IF-4; DECISION-45 |
| 24 | Secrecy | The line and the field hold only numbers and fixed words: no key, no address, and no body | CV-6; tests/secrecy.rs |
| 25 | The place of the code | The cost type and the sum in src/scorer/cost.rs, because C-scorer owns the entity Cost. The price next to `DEFAULT_MODEL` in src/config/model.rs. The line and the field in src/output.rs, because C-output writes the cost line. The usage, the model, and the end of the write of the search in `Stop` of src/cli/stop.rs. `Emitted` in src/cli.rs. `finish` in src/cli.rs and `Stop::on_signal` write the line | system-design.md C-scorer, C-output, and the data table (the row "Score, Cost": "C-scorer", "Memory, for one search."); ADR-0011; the FEATURE-15 spec, "Other features"; src/cli.rs (`run` reads `stop.size_limit()` after `claim_or_wait`, and gives that value of the search flow to `finish` in the same way) (inferred) |
| 26 | The shape of the code | `score_with_usage` has the 6 parameters of `score` and a seventh, `usage: &Mutex<Usage>`. Each attempt adds its bill to it when the attempt ends. `score` keeps its signature and gives a new usage, so `probe` and the tests of tests/score.rs do not change | src/scorer.rs (`score` has 6 parameters since FEATURE-15); tests/score.rs (inferred) |
| 27 | The fake server | `Reply::scores` keeps 1 input token for each call, so each amount of the current tests is `$0.000000`: 7 calls give 294 billionths, and only 12 calls or more give 500 billionths or more. A new `Reply::scores_with_usage` sets the `usage` value of the body: a count, `null`, no `usage`, or a text | tests/common/fake_server.rs; decision 10 (inferred) |
| 28 | The size of the document | The largest `cost` field adds 74 characters. The largest document with no result goes from 619 to 693 characters, less than the smallest size limit of 1,000. The default document goes from 310 to 373 characters. A `billed` field of 0 adds 53 characters. Each count includes the new line. The bound counts use the values of the bound test | FEATURE-11 spec decision 27; src/output.rs (`the_largest_document_with_no_result_is_less_than_1000_characters`); src/config/limit.rs (`MIN_SIZE_LIMIT`, FEATURE-09 on `main`) (computed) |
| 29 | FEATURE-09 (the size limit, on `main`) | The size limit counts the `cost` field. The cost line goes to the error stream, so it does not count. On the error stream: the cost-limit notice, the size-limit notice, then the cost line. This spec changes the tests of FEATURE-09 that check the error stream of a text search or the exact size of a document (see "Tests that change") | DECISION-22; ADR-0008; the FEATURE-09 spec, "Out of scope" ("In text mode the line goes to the error stream (DECISION-41), so it does not count. The field is part of the document, so it counts (DECISION-22)"); the FEATURE-09 code on `main` (PR #167) (inferred) |
| 30 | FEATURE-21 (the gateway, spec on `main`, code not on `main`) | `--model`, `config.model`, and a gateway use the rule of decision 4: `settings.model` holds the name from each source. A gateway that gives no `usage` gives an estimate with the pinned name, and `not available` with another name. An answered attempt with no count still adds its estimate in tokens. The FEATURE-21 code needs no change for the cost, because `price_of` does not read the route. Its decision 27 order (the settings before the key) stays: `stop.set_model` comes after the settings and the key. The new tests of this spec get the key from `jag` and `jag_std` of tests/common, which the FEATURE-21 code changes to the gateway key. The code that merges second changes each test that sets `JAG_TYPESAFE_API_KEY` for the fake server itself to `JAG_GATEWAY_API_KEY`. In a test that both specs change, it keeps the text of FEATURE-21 and adds the cost line. The code that merges second adds to the README section "The cost of a search", after "Output tokens are free.": "Through an AI gateway, the amount is the price of TypeSafe. It does not include a fee of the gateway." It also changes "the model that JAG_MODEL names" to "the model that --model, JAG_MODEL, or config.model names". | the FEATURE-21 spec (on `main`): decision 27, "Risks" ("A gateway gives no `usage` in its response. FEATURE-20 reads it (DECISION-45)"), "Build order" item 3, "The merge with other work" ("FEATURE-20 (spec not merged): the cost of a call through a gateway. Its code keeps the order of decision 27" and "The code that merges second changes each test that sets `JAG_TYPESAFE_API_KEY` for the fake server to `JAG_GATEWAY_API_KEY`"), "Tests that change" ("`jag` and `jag_std` set `JAG_GATEWAY_API_KEY` to `KEY` in place of `JAG_TYPESAFE_API_KEY`"), and "Out of scope" ("The cost of a call through a gateway: FEATURE-20"); 0015-read-the-base-url-and-the-model-name-from-config-base-url.md (accepted: `config.model`, and "the option wins over the variable, the variable wins over the file, and the file wins over the default"); 0017-use-a-separate-key-for-an-ai-gateway.md (accepted: the gateway key); decision 4 (inferred) |
| 31 | The check | `--check` writes no cost line, also after a Ctrl-C | CRITERION-88 (after each search); README.md (the check call costs "less than $0.00001"); the FEATURE-02 spec, "Out of scope" ("A cost line for the check call: FEATURE-20"); the FEATURE-15 spec decision 16 (the check keeps the default action of SIGINT) (inferred) |
| 32 | The README | A section "The cost of a search" after "In a terminal and in a pipe" and before "Speed". A `cost` row in the table of fields, the sentence on the error stream in structured mode, the Ctrl-C item of "In a terminal and in a pipe", and an item in "Plain grep with no question" | FEATURE-11 spec decision 29 (the README section "Output for an agent", with the fields); the FEATURE-15 spec decision 20 (the README section "In a terminal and in a pipe"); the FEATURE-12 spec (the README section "Plain grep with no question"); README.md on `main` ("Speed", from the FEATURE-16 code, is the last section) (inferred) |
| 33 | Plain grep | With no `--ask`, jag makes no model call. In text mode, the line is `jag: cost $0.000000 (no question): scored 0 of N results, skipped 0.` In structured mode, the field is `{"micro_usd":0,"kind":"no_call","reason":"no_question"}`. N is `results_found`, the count of matching lines. `plain` sets the cost after the scan and before `record` | DECISION-43 ("such as "no question""; the refused alternative "Report no cost"); DECISION-41 ("after each search"); CRITERION-91; the FEATURE-12 spec decision 14 and "Other features" ("FEATURE-20 adds the cost line and the cost field. This spec writes neither"); src/cli.rs (`plain`) (inferred) |
| 34 | `-l`, `-c`, and `-q` (FEATURE-13, PR #170 open) | The three options work only in plain grep, so they get the line of decision 33 after their output. N is `results_found` as their flow counts it. With `-q`, jag writes nothing on standard output, and the line goes to the error stream with the notices and the warnings. The code that merges second sets the cost before the call of `record` in `write_report` (src/cli/grep.rs in PR #170). The code that merges second also changes the help of `-q` to `Write nothing to standard output: give only the exit code (not with --ask or --json)`. | the FEATURE-13 spec decision 21 ("The skip notice and the warnings still go to the error stream, as in grep"), decision 22 (the options work only in plain grep with no `--json`), and "Other features" ("`-l`, `-c`, and `-q` make no model call, so they have no cost"); DECISION-43 (the refused alternative "Report no cost": no cost is a cost of zero); PR #170 (`write_report`, not merged) (inferred) |
| 35 | FEATURE-14 (spec on `main`, code not on `main`) | A path that does not exist gives its warning with the other warnings of the walk, before the output, and exit 2 at the end. The cost line still comes last. When no path exists, the search finds no result: `(no match)` with `--ask`, and `(no question)` with no `--ask`. Text from standard input gets the same cost as a file | the FEATURE-14 spec decision 7 (the warning "as each warning of the walk"; "When no path exists, jag finds no result, makes no model call, and exits with 2") and decision 3 (an empty pipe gives the no-result path); decisions 12 and 14 (inferred) |
| 36 | Story states | `ready` in the spec PR. FEATURE-04 and FEATURE-11 are done, and no ASK is open | docs/adlc.md rule 11 |

## Design

### Files

```
src/config/model.rs          DEFAULT_MODEL_PRICE and price_of, next to DEFAULT_MODEL
src/scorer/cost.rs           new: Bill, Usage, NoCallReason, Cost, and Cost::of
src/scorer/client.rs         read_input_tokens; call gives the bill of the attempt
src/scorer.rs                score_with_usage and the usage of Run; score gives a new usage
src/cli/stop.rs              the usage, the model, and the end of the write of the
                             search; the cost of a stop
src/output.rs                Summary.cost, the cost field, and cost_line
src/cli.rs                   Emitted; search and plain set summary.cost; emit gives
                             the end of the write; finish writes the line
tests/common/fake_server.rs  Reply::scores_with_usage
tests/cost.rs                new: the end-to-end tests of this feature
tests/stop.rs                the end-to-end tests of the cost of a stop
tests/output.rs, tests/failure.rs, tests/plain_grep.rs, tests/size_limit.rs,
tests/terminal.rs, tests/usage.rs
                             the tests that change (see "Tests that change")
README.md                    the section, the row, and four sentences
```

### The price (C-config)

```rust
/// The price of one input token of `DEFAULT_MODEL`, in billionths of a US
/// dollar: 0.042 US dollars for 1 million input tokens. Output tokens are
/// free. Change it when `DEFAULT_MODEL` changes (ADR-0009).
pub const DEFAULT_MODEL_PRICE: u64 = 42;

/// The price of one input token of `model`, or `None` when jag does not
/// know it (decisions 3 and 4).
pub fn price_of(model: &str) -> Option<u64> {
    (model == DEFAULT_MODEL).then_some(DEFAULT_MODEL_PRICE)
}
```

### The bill of one attempt (C-scorer)

In `src/scorer/cost.rs`:

```rust
/// What jag knows of the bill of one attempt (decisions 5 and 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bill {
    /// No answer with HTTP 200: another status, a timeout, or a lost
    /// connection. It adds nothing.
    NoAnswer,
    /// HTTP 200, and `usage.input_tokens` is a whole number.
    Tokens(u64),
    /// HTTP 200, and jag cannot read the count. jag counts its estimate.
    NoCount,
}
```

In `src/scorer/client.rs`:

- `Client::call` gives the bill with the result:
  `(Result<Vec<f64>, Attempt>, Bill)`. Before a 200 status the bill is
  `NoAnswer`. After a 200 status it is `Tokens(n)` or `NoCount`. A body
  that jag cannot read after a 200 status (`read_to_vec` fails) gives
  `NoCount`. A body for which `parse_scores` gives `None`, so that `call`
  gives `BadResponse`, still gives `Tokens(n)` or `NoCount`: the service
  did the work.
- `read_input_tokens(bytes: &[u8]) -> Option<u64>` reads the body as a
  `serde_json::Value`, then `usage.input_tokens` with `as_u64`. No `usage`,
  no count, `null`, a negative number, a fraction, a text, or a body that
  is not JSON gives `None`.
- `parse_scores` does not change (decision 8).

### The cost of a search (C-scorer)

In `src/scorer/cost.rs`:

```rust
/// What the attempts of one search used (memory, for one search).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    /// The input tokens that the service gave for the answered attempts.
    pub tokens: u64,
    /// The estimate for the answered attempts with no count (decision 6).
    pub estimated_tokens: u64,
    /// The count of answered attempts with no count.
    pub no_count: u64,
}

impl Usage {
    /// Add the bill of one attempt. `body_bytes` is the size of the body of
    /// its request. The sums saturate at `u64::MAX`.
    pub fn add(&mut self, bill: Bill, body_bytes: usize) { /* ... */ }
}

/// Why jag made no model call (DECISION-43).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoCallReason {
    /// The run ended before jag counted the results.
    #[default]
    NotStarted,
    /// The search has no question: plain grep (FEATURE-12).
    NoQuestion,
    /// The term matched no line.
    NoMatch,
    /// jag could read no line of a match.
    NoReadableMatch,
}

/// The cost of one search (DECISION-40 to DECISION-45). The amounts are in
/// millionths of a US dollar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cost {
    Billed(u64),
    Estimate(u64),
    NotAvailable,
    NoCall(NoCallReason),
}

impl Default for Cost {
    fn default() -> Self {
        Cost::NoCall(NoCallReason::NotStarted)
    }
}

impl Cost {
    /// The cost of the attempts of a search with `model`, in the order of
    /// decision 7. The caller gives `NoCall` when there was no call.
    pub fn of(usage: &Usage, model: &str) -> Cost {
        let Some(price) = price_of(model) else {
            return Cost::NotAvailable;
        };
        let billionths = usage
            .tokens
            .saturating_add(usage.estimated_tokens)
            .saturating_mul(price);
        let millionths = billionths.saturating_add(500) / 1000;
        if usage.no_count > 0 {
            Cost::Estimate(millionths)
        } else {
            Cost::Billed(millionths)
        }
    }
}
```

An unknown model with no answered attempt gives `NotAvailable`, not
`Billed(0)`. This keeps one order for each case (decision 7).

### The calls (C-scorer)

In `src/scorer.rs`:

- `Run` gets `usage: &'a Mutex<Usage>`. `Run::attempt` adds the bill of
  each attempt, with `body.len()`, when `Client::call` gives its result.
  This occurs before `Run::call` gives its result, so before the thread
  starts its next call. The first attempt and the retry count apart, and an
  attempt that ends after the first failure also counts (decision 5). An
  attempt that `Run::attempt` does not start after the deadline adds
  nothing.
- `pub fn score_with_usage(snippets, question, key, settings, stop,
  on_call_end, usage: &Mutex<Usage>) -> Result<Vec<f64>, ScoreError>`. It
  runs the calls as `score` does now.
- `pub fn score(...)` keeps its 6 parameters. It calls `score_with_usage`
  with a new `Mutex<Usage>` and drops it. `probe` and tests/score.rs do
  not change (decision 26).

### The stop (C-cli)

`Stop` in `src/cli/stop.rs` gets three fields and five methods:

```rust
/// The bills of the model calls of the search. `score_with_usage` adds to
/// it while the calls run, and a stop reads it (FEATURE-20).
usage: Mutex<Usage>,
/// The model name of the calls. `None` until the search flow has results
/// to score.
model: Mutex<Option<String>>,
/// How the write of the search flow to standard output ended.
emitted: Mutex<Emitted>,

/// The usage that the search flow gives to `score_with_usage`.
pub fn usage(&self) -> &Mutex<Usage>;
/// The search flow calls it before the count, when it has results to score.
pub fn set_model(&self, model: &str);
/// The cost that jag paid up to now: `Cost::of(usage, model)` after
/// `set_model`, and else the cost in the copy of `count`.
pub fn cost(&self) -> Cost;
/// `emit_claimed` calls it after its write.
pub fn set_emitted(&self, emitted: Emitted);
pub fn emitted(&self) -> Emitted;
```

`on_signal` takes a copy of `counted` and sets its `cost` to `cost()`. In
text mode, it writes the stop failure, then the line of `cost_line` when
there is one, then it flushes. In structured mode, the stop document holds
the cost. `on_closed_pipe` does not change: it writes nothing (decision
21).

### The search flow (C-cli)

The line numbers are those of src/cli.rs on `main`. `record` (line 665)
copies the summary to the stop with `context.stop.count(summary)` (line
684). So the search flow sets `summary.cost` before `record`, and a
Ctrl-C after the count finds the cost.

In `search` (line 431):

1. After `read_snippets` (line 481) and before `record` (line 484):

   ```rust
   summary.cost = if snippets.is_empty() {
       Cost::NoCall(no_call_reason(total))
   } else {
       stop.set_model(&settings.model);
       stop.cost()
   };
   ```

   `no_call_reason` gives `NoMatch` when `total` is 0, and
   `NoReadableMatch` when it is not. `stop.cost()` is then `Billed(0)` or
   `NotAvailable`. `set_model` comes before `ScoreSettings::new` (line
   508), which moves `settings.model`. So the no-match document of
   `no_result` (line 494) has its reason.
2. The call: `score_with_usage(..., stop.usage())` in place of
   `score(...)` (line 512).
3. After the scoring block, and before the `match` on its result (line
   521): `summary.cost = stop.cost();`. So the failure path and the
   document find the cost.
4. The rest does not change.

In `plain` (line 569): after the scan (line 590) and before `record` (line
593), `summary.cost = Cost::NoCall(NoCallReason::NoQuestion);`. A bad size
limit and a bad term end the run before the scan, so they keep
`not_started`. FEATURE-13 (PR #170, not merged) adds `write_report` in
src/cli/grep.rs, which calls `record` for `-l`, `-c`, and `-q`. The code
that merges second sets the same cost after the scan and before that call
(decision 34).

A run that ends before the count keeps the default
`Cost::NoCall(NoCallReason::NotStarted)`. So do `--setup`, `--forget-key`,
and `--check`.

### The end of the write (C-cli)

In `src/cli.rs`:

```rust
/// How the write of standard output ended (decisions 20 and 21).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Emitted {
    /// The write worked, or the run wrote nothing on standard output.
    #[default]
    Written,
    /// The reader closed the pipe (CV-4).
    Closed,
    /// Another write error: `emit` wrote the failure text.
    Failed,
}
```

- `emit` (line 331) gives `(u8, Emitted)`: the code as now, and how the
  write ended. `parse` (line 268) uses only the code.
- `emit_claimed` (line 716) calls `stop.set_emitted` with the value of
  `emit`, and gives the code as now. So `search`, `plain`, `no_result`, and
  `write_report` of FEATURE-13 keep their return types.
- `run` (line 167) reads `stop.emitted()` after `claim_or_wait`, as it reads
  `stop.size_limit()` now, and gives it to `finish`. Each other branch gives
  `Emitted::Written`.

### The line (C-output, C-cli)

`pub fn cost_line(summary: &Summary) -> Option<String>` in `src/output.rs`.
`D` is `m / 1_000_000`, and `DDDDDD` is `m % 1_000_000` with 6 digits.

| `summary.cost` | Line |
|---|---|
| `Billed(m)` | `jag: cost $D.DDDDDD: scored K of N results, skipped S.` |
| `Estimate(m)` | `jag: cost $D.DDDDDD (estimate): scored K of N results, skipped S.` |
| `NotAvailable` | `jag: cost not available: scored K of N results, skipped S.` |
| `NoCall(NoQuestion)` | `jag: cost $0.000000 (no question): scored 0 of N results, skipped 0.` |
| `NoCall(NoMatch)` | `jag: cost $0.000000 (no match): scored 0 of 0 results, skipped 0.` |
| `NoCall(NoReadableMatch)` | `jag: cost $0.000000 (no readable match): scored K of N results, skipped S.` |
| `NoCall(NotStarted)` | `None`: no line |

K is `results_scored`, N is `results_found`, and S is `results_skipped`.
Examples: 2016 gives `$0.002016`, 0 gives `$0.000000`, and 12345678 gives
`$12.345678`.

`finish` (line 301) gets the `Emitted` value of `run`, and writes the line
as the last write of the run. Today its `Ok` arm writes nothing. With this
feature, it writes the line after each arm:

```rust
let (code, emitted) = match result {
    Ok(code) => (code, emitted),
    Err(failure) if mode == Mode::Json => {
        let code = failure.exit_code;
        let buffer = error_document(summary, failure, limit);
        emit(stdout, stderr, &buffer, code)
    }
    Err(failure) => {
        let _ = write!(stderr, "{}", failure.render());
        (failure.exit_code, emitted)
    }
};
// The cost line is the last line on the error stream (decisions 12, 20,
// and 21).
let due = match mode {
    Mode::Text => emitted != Emitted::Closed,
    Mode::Json => emitted == Emitted::Failed,
};
if due {
    if let Some(line) = cost_line(summary) {
        let _ = writeln!(stderr, "{line}");
    }
}
ExitCode::from(code)
```

- In text mode, the line comes after the results or after the failure
  text. `no_result` writes nothing in text mode and gives `Ok(1)`, so the
  line of a search with no result also comes from `finish`.
- `NoCall(NotStarted)` gives no line. This keeps `--check`, `--setup`,
  `--forget-key`, no key, a bad setting, and a bad term with the texts of
  today.
- `run` calls `finish` after `claim_or_wait`, so the search flow owns the
  output (FEATURE-15 decision 14).
- jag writes the line as it writes the notices: a failed write of the line
  changes nothing, and the exit code stays.

### The field (C-output)

`Summary` gets `pub cost: Cost`. `Document` gets `cost: CostField` after
`warnings_left_out` and before `results`:

```rust
#[derive(Serialize)]
struct CostField {
    micro_usd: Option<u64>,
    kind: &'static str,
    reason: Option<&'static str>,
}
```

| `Cost` | `micro_usd` | `kind` | `reason` |
|---|---|---|---|
| `Billed(m)` | `m` | `"billed"` | `null` |
| `Estimate(m)` | `m` | `"estimate"` | `null` |
| `NotAvailable` | `null` | `"not_available"` | `null` |
| `NoCall(NoQuestion)` | `0` | `"no_call"` | `"no_question"` |
| `NoCall(NoMatch)` | `0` | `"no_call"` | `"no_match"` |
| `NoCall(NoReadableMatch)` | `0` | `"no_call"` | `"no_readable_match"` |
| `NoCall(NotStarted)` | `0` | `"no_call"` | `"not_started"` |

The default document ends with
`"warnings_left_out":0,"cost":{"micro_usd":0,"kind":"no_call","reason":"not_started"},"results":[]}`.
`micro_usd` is a whole number, so the rule "each number in the document is
a whole number" stays true. The error object keeps `cause` and `fix` only.
`write_document` fits the document in the size limit with the field
(decision 29).

### Each end of a run and its output

| End of the run | Exit | Error stream in text mode, in order | `cost` in structured mode |
|---|---|---|---|
| Argument error, no key, a bad setting, a bad term | 2 or 3 | The failure text only, as now | `no_call`, `not_started` |
| `--setup`, `--forget-key`, `--check` | as now | as now | no document |
| Plain grep (no `--ask`), with or without a match | 0, 1, or 8 | The skip notice and the warnings when they show, the size-limit notice when it shows, then `jag: cost $0.000000 (no question): scored 0 of N results, skipped 0.` | `no_call`, `no_question` |
| `-l`, `-c`, or `-q` (FEATURE-13, PR #170) | 0, 1, or 2 | The notices and the warnings, then the line with `(no question)` | no document: the options do not work with `--json` |
| No match | 1 | The skip notice and the warnings when they show, then `jag: cost $0.000000 (no match): scored 0 of 0 results, skipped 0.` | `no_call`, `no_match` |
| Matches, and jag can read none | 1 | The skip notice and the warnings, then the line with `(no readable match)` | `no_call`, `no_readable_match` |
| Scores | 0 or 1 | The skip notice and the warnings, then the line | `billed`, `estimate`, or `not_available` |
| Scores, and the cost limit applies | 7 or 1 | The same, with the cost-limit notice before the calls, then the line | the same |
| Scores, and the size limit leaves out results | 8, or 7 when the cost limit also applies | The same, then the size-limit notice, then the line | the same |
| A path that does not exist (FEATURE-14, code not on `main`) | 2 | The warning of the path with the other warnings, then the line | the document with the cost, and `exit_code` 2 |
| A model call fails | 4, 5, or 6 | The notices, the failure text (cause and fix), then the line with the cost of the answered attempts | The error document with that cost |
| jag cannot write standard output | 2 | The failure text of FEATURE-11 decision 25, then the line | The same text, then the line (decision 20) |
| Ctrl-C after the count, before the search flow takes the output | the end by SIGINT (a shell shows 130) | The notices, the stop failure, then the line with the cost of the attempts that ended before the stop | The stop document with that cost |
| Ctrl-C before the count | the end by SIGINT | The stop failure only, as now | The stop document with `no_call`, `not_started` |
| Ctrl-C after the search flow takes the output | the end by SIGINT | Nothing more (FEATURE-15 decision 14): the line can be absent or cut | Nothing more: the document can be cut, as now |
| The reader closes standard output | the end by SIGPIPE, or the code of the search during the write | Nothing more: no line, because jag exits "with no message when the pipe closes" (DECISION-39) | Nothing more, as now |

### Acceptance criteria and their tests

The end-to-end tests are in tests/cost.rs, with the fake server and the
helpers `jag` and `jag_std` of tests/common, unless a test names another
file. Each search with a question uses
`--ask QUESTION`, as the other tests do. In the tests, standard output is a
pipe. 33 short results give 2 calls: one of 32 results and one of 1 result
(src/scorer/prompt.rs `split_batches`).

- CRITERION-88 (one cost line after each search, also in a pipe):
  - End-to-end: a text search with a good answer. The error stream is
    exactly one line, the cost line. Exit 0.
  - End-to-end: `JAG_MAX_SCORED=1` and 2 results. The error stream is the
    cost-limit notice, then the cost line, as the last line. Exit 7.
  - End-to-end: plain grep, `jag needle dir`, with 3 matching lines. The
    error stream is exactly
    `jag: cost $0.000000 (no question): scored 0 of 3 results, skipped 0.`
    Exit 0.
  - End-to-end: the same searches with `--json`. The error stream is empty.
- CRITERION-89 (the counts and the billed amount):
  - End-to-end: 33 results, and each call gives `input_tokens` 24000. The
    line is `jag: cost $0.002016: scored 33 of 33 results, skipped 0.`
    (48,000 tokens times 42 is 2,016,000 billionths).
  - End-to-end: the same with `JAG_MAX_SCORED=32`. The line is
    `jag: cost $0.001008: scored 32 of 33 results, skipped 1.` Exit 7.
  - End-to-end: the first attempt of a call gives HTTP 503, and the retry
    gives 200 with 1000 tokens. The amount is `$0.000042`: the retry counts
    one time, and the failed attempt adds nothing.
  - Unit (src/output.rs): `cost_line` for `Billed(2016)` with K 150, N 200,
    and S 50 gives `jag: cost $0.002016: scored 150 of 200 results,
    skipped 50.` `Billed(0)` gives `$0.000000`, and `Billed(12345678)` gives
    `$12.345678`. No line holds `cost limit`.
  - Unit (src/scorer/cost.rs): 499 billionths give 0, 500 give 1, 1499 give
    1, and 1500 give 2. 48,000 tokens give `Billed(2016)`. A sum at
    `u64::MAX` saturates and does not overflow.
  - Unit (src/config/model.rs): `price_of(DEFAULT_MODEL)` is `Some(42)`.
    `price_of("jev-1.13")` and `price_of("")` are `None`.
- CRITERION-90 (the same counts and amount in the field):
  - End-to-end: the same search in text mode and with `--json`, in 5 cases:
    billed, estimate (no `usage`), not available (`JAG_MODEL=jev-9.9.9`),
    no match, and plain grep. The amount of the line times 1,000,000 is
    `micro_usd`, and `not available` is `null`. `(estimate)` is
    `"kind":"estimate"`. The reason words, with `_` for each space, are
    `reason`. K, N, and S are `results_scored`, `results_found`, and
    `results_skipped`.
  - Unit (src/output.rs): the field for each `Cost` value of the table. The
    key lists get `cost` in `TOP_LEVEL` and a new list
    `COST = ["micro_usd", "kind", "reason"]`.
- CRITERION-91 (zero and the reason when there is no call):
  - End-to-end: a term that matches nothing, with `--ask`. The server gets
    no request. The line is `jag: cost $0.000000 (no match): scored 0 of 0
    results, skipped 0.` Exit 1. With `--json`, `cost` is
    `{"micro_usd":0,"kind":"no_call","reason":"no_match"}`.
  - End-to-end: plain grep with a match and with no match. The server gets
    no request. The lines are `jag: cost $0.000000 (no question): scored 0
    of 3 results, skipped 0.` (exit 0) and `jag: cost $0.000000 (no
    question): scored 0 of 0 results, skipped 0.` (exit 1). With `--json`,
    `cost` is `{"micro_usd":0,"kind":"no_call","reason":"no_question"}` in
    both, `results_scored` is 0, and `results_skipped` is 0.
  - End-to-end (tests/usage.rs): `--json` with an option that jag does not
    know. The argument-error document has
    `{"micro_usd":0,"kind":"no_call","reason":"not_started"}`, and the error
    stream stays empty.
  - Unit (src/cli.rs): `no_call_reason(0)` is `NoMatch`, and
    `no_call_reason(2)` is `NoReadableMatch`. A search that changes a file
    between the scan and the read is not stable in a test, so a unit test
    covers the second reason. `cost_line` gives the line with
    `(no readable match)`.
- CRITERION-92 (not available, and the search does not fail):
  - End-to-end: `JAG_MODEL=jev-9.9.9`, a name with no price. Exit 0, and
    the results are the same as with the pinned model. The line is
    `jag: cost not available: scored 2 of 2 results, skipped 0.` With
    `--json`, `micro_usd` is `null` and `kind` is `"not_available"`.
  - Unit (src/scorer/cost.rs): `Cost::of` with an unknown model is
    `NotAvailable`, also when `no_count` is more than 0.
- CRITERION-97 (an estimate with a label):
  - End-to-end: the server gives no `usage`. Exit 0, and the results are
    as usual. The line has `(estimate)`. The amount is the sum, over the
    requests that the server got, of the bytes of the body divided by 3
    (rounded down), times 42, rounded as in decision 10. With `--json`,
    `kind` is `"estimate"`.
  - End-to-end: `"input_tokens":null` and `"input_tokens":"12"`. The same
    estimate, and exit 0.
  - End-to-end: 33 results. The call of 32 results gives 24000 tokens, and
    the call of 1 result gives no `usage`. The kind is `estimate`, and the
    amount is 24000 plus the estimate of the second call, times 42.
  - Unit (src/scorer/client.rs): `read_input_tokens` gives `Some(n)` for a
    whole number and for 0. It gives `None` for no `usage`, no
    `input_tokens`, `null`, `-1`, `2.5`, `"12"`, and a body that is not
    JSON. The tests of `parse_scores` stay.
  - Unit (src/scorer/cost.rs): `Usage::add` for each `Bill`. `Cost::of`
    gives `Estimate` when `no_count` is more than 0.
- CRITERION-93 (the cost of a failed or stopped search):
  - End-to-end: 33 results. The call of 32 results gives 24000 tokens. The
    call of 1 result gives HTTP 500 two times. Exit 6. The error stream
    ends with `jag: the TypeSafe API failed (HTTP 500).`, `Try again
    later.`, and `jag: cost $0.001008: scored 33 of 33 results, skipped
    0.` With `--json`, the error document has
    `"cost":{"micro_usd":1008,"kind":"billed","reason":null}`, and the
    error stream is empty.
  - End-to-end: HTTP 401 for each call. Exit 4. The line is
    `jag: cost $0.000000: scored 2 of 2 results, skipped 0.`
  - End-to-end (Linux only): standard output to `/dev/full`, in text mode
    and with `--json`. The error stream is the write failure text, then the
    cost line. Exit 2.
  - End-to-end (tests/stop.rs): a server that answers the first call at
    once with `input_tokens` 24000, and holds each other call for 30
    seconds. 200 results give 7 calls, 6 at the same time. The thread of
    the first call adds its bill before it starts the seventh call, so the
    test sends SIGINT after the server has 7 calls. The error stream is the
    stop failure, then `jag: cost $0.001008: scored 200 of 200 results,
    skipped 0.` The end is by SIGINT. With `--json`, the stop document has
    `"cost":{"micro_usd":1008,"kind":"billed","reason":null}`, and the
    error stream is empty.
  - Test (tests/score.rs, with the fake server): `score_with_usage` with a
    failure. The usage holds each answered attempt, also an attempt that
    ends after the failure. Use the shape of
    `a_late_batch_that_fails_ends_the_search_at_the_deadline` (line 395).
  - Unit (src/cli/stop.rs): after `count`, `set_model(DEFAULT_MODEL)`, and
    a bill of 24000 tokens, `on_signal` in text mode writes the stop
    failure, then `jag: cost $0.001008: scored 200 of 200 results, skipped
    0.` In structured mode, the document has the same cost. With no
    `set_model`, `cost()` is the cost of the copy of `count`.

### Tests that change

The fake server gives 1 input token for each call, so the amount of each
line below is `$0.000000` (decision 27). The line numbers are those of
`main`.

- tests/output.rs:
  - `criterion_44_text_is_the_default_also_in_a_pipe` (line 101): the
    check at line 108 becomes the text
    `jag: cost $0.000000: scored 2 of 2 results, skipped 0.` with `\n` at
    the end.
  - `TOP_LEVEL` (line 357) and `assert_top_level` (line 375): add `cost`.
  - `failure_of` (line 412): it drops the last line when that line starts
    with `jag: cost $` or `jag: cost not available`. The cost-limit notice
    starts with `jag: cost limit`, so the check names the two forms. The
    calls of `assert_error_document` at line 447 (a bad term) and line 467
    (no key) give a text with no cost line, because the run ends before the
    count. The call at line 489 (HTTP 401, 402, and 500) gives a text with
    the line
    `jag: cost $0.000000: scored 2 of 2 results, skipped 0.`
    `assert_error_document` (line 418) still expects an empty error stream
    for the JSON run.
  - `criterion_51_a_document_that_cannot_be_written_gives_the_text_failure_and_exit_2`
    (line 521): the check at line 544 becomes the write failure text, then
    `jag: cost $0.000000: scored 2 of 2 results, skipped 0.`
  - No change: `criterion_45_*` (line 127: each line starts with `jag: `,
    and the cost line does too), `criterion_46_*` (line 155: a JSON run), `criterion_50_*` (line 246: it finds each
    line by its start, and the cost line does not start with
    `jag: cost limit reached`), `criterion_49_*` (line 390: the JSON run
    has an empty error stream, and the text run has no check of the error
    stream), and
    `criterion_51_an_argument_error_document_that_cannot_be_written_gives_the_text_failure`
    (line 553: the run ends before the count).
- tests/failure.rs `assert_failure` (line 65): it gets one more argument,
  the expected cost line, or `None`. The error stream is the cause, the
  fix, then the line.
  - Line 86 (no key, exit 3): `None`.
  - Lines 100, 115, 129, 147, 160, 174, 188, and 202 (one file with one
    match, and no answered attempt):
    `jag: cost $0.000000: scored 1 of 1 results, skipped 0.`
  - Line 216 (`a_body_that_jag_cannot_read_gives_exit_6`: HTTP 200 and a
    body that jag cannot read): the line with `(estimate)`. The test
    computes the amount from the body of the one request that the server
    got (decision 6).
  - Lines 417 and 431 (`many_files`: 40 results in 2 calls, and the first
    call gets scores):
    `jag: cost $0.000000: scored 40 of 40 results, skipped 0.`
- tests/plain_grep.rs:
  - `criterion_53_the_matching_lines_come_as_grep_gives_them` (line 77,
    check at line 83), `criterion_54_the_options_of_the_model_have_no_effect`
    (line 265, check at line 275), and
    `criterion_68_no_key_is_needed_and_none_is_asked_for` (line 296, check
    at line 301):
    `jag: cost $0.000000 (no question): scored 0 of 3 results, skipped 0.`
  - `criterion_53_no_match_exits_1_and_a_bad_term_exits_2` (line 141,
    check at line 146):
    `jag: cost $0.000000 (no question): scored 0 of 0 results, skipped 0.`
    The bad term (line 150) ends the run before the count, so it gets no
    line.
  - `criterion_54_the_document_has_the_lines_and_no_score` (line 225) and
    `criterion_54_a_search_with_no_line_gives_the_document_with_exit_1`
    (line 254) can add the check of
    `{"micro_usd":0,"kind":"no_call","reason":"no_question"}`.
  - No change: the JSON runs (lines 230 and 443), the argument errors
    (lines 433, 462, and 481), and the checks with `contains` (lines 150,
    161, 288, 337, 404, and 492).
- tests/size_limit.rs, the text runs:
  - `criterion_36_39_41_42_the_ranked_folder_with_a_limit_of_1000` (line
    174, check at line 180): `notice(8, 30, "1000")`, then
    `jag: cost $0.000000: scored 30 of 30 results, skipped 0.`
  - `criterion_36_the_limit_counts_characters_and_not_bytes` (line 199,
    check at line 206):
    `jag: cost $0.000000: scored 10 of 10 results, skipped 0.`
  - `criterion_37_the_default_limit_is_20000` (line 215, check at lines
    225 to 231): the size-limit notice is now the line before the last
    line. The last line is
    `jag: cost $0.000000: scored 200 of 200 results, skipped 0.`
  - `criterion_37_the_user_can_raise_the_limit_or_turn_it_off` (line 235,
    check at line 247): the same line for each of the 4 runs.
  - `criterion_37_the_option_wins_over_the_variable_and_a_blank_variable_is_the_default`
    (line 252, check at line 265):
    `jag: cost $0.000000: scored 30 of 30 results, skipped 0.`
  - `criterion_40_the_top_result_is_cut_and_the_notice_says_so` (line 285,
    check at lines 297 to 302): the notice, then
    `jag: cost $0.000000: scored 2 of 2 results, skipped 0.`
  - `criterion_40_a_cut_with_no_other_result_gives_exit_8` (line 306,
    check at lines 312 to 317): the notice, then
    `jag: cost $0.000000: scored 1 of 1 results, skipped 0.`
  - `a_limit_that_the_top_result_fits_in_gives_no_cut` (line 321, check at
    line 326): `jag: cost $0.000000: scored 2 of 2 results, skipped 0.`
  - `criterion_41_a_search_inside_the_limit_gives_no_notice` (line 332,
    check at line 340):
    `jag: cost $0.000000: scored 30 of 30 results, skipped 0.`
  - `criterion_66_the_cost_limit_code_wins_and_both_notices_show` (line
    345, check at lines 351 to 357): the cost-limit notice, the size-limit
    notice, then
    `jag: cost $0.000000: scored 10 of 30 results, skipped 20.`
  - `plain_grep_has_the_size_limit_and_makes_no_call` (line 610, check at
    line 622): `notice(8, 30, "1000")`, then
    `jag: cost $0.000000 (no question): scored 0 of 30 results, skipped 0.`
- tests/size_limit.rs, the exact sizes of structured mode. The field
  `,"cost":{"micro_usd":0,"kind":"billed","reason":null}` has 53
  characters:
  - `criterion_104_the_document_has_the_fields_of_the_notice` (line 392):
    823 becomes 876 (line 402). A third result adds more than 200
    characters, so the document still holds 2 results (`names(0, 1)`), and
    `left_out` stays 28.
  - `criterion_104_a_document_inside_the_limit_has_the_values_of_nothing_left_out`
    (line 414): 6812 becomes 6865 (line 419).
- tests/size_limit.rs, no change: the checks with `contains` (lines 280,
  336, 369, and 662), the JSON runs (lines 410, 486, and 644), and the
  checks that compare a size with the limit (for example line 442).
- tests/terminal.rs `criterion_63_always_colors_in_a_pipe_as_grep_does`
  (line 100, check at line 105):
  `jag: cost $0.000000: scored 1 of 1 results, skipped 0.` The other tests
  of the file do not change: `criterion_62_a_pipe_gets_no_progress_line`
  (line 70) looks for `the model scored`, which the cost line does not
  hold.
- tests/stop.rs:
  - `criterion_64_ctrl_c_stops_jag_at_once_in_text_mode` (line 93, check
    at line 106): `STOPPED`, then
    `jag: cost $0.000000: scored 200 of 200 results, skipped 0.` No call
    ended, so the cost is 0.
  - `criterion_64_ctrl_c_gives_the_error_document_in_structured_mode`
    (line 112) does not change. It can add the check of
    `{"micro_usd":0,"kind":"billed","reason":null}`.
- tests/usage.rs
  `an_argument_error_with_json_writes_one_document_with_no_value` (line
  499) and `json_with_no_term_gives_an_error_document` (line 553): add the
  check of `not_started`. The error stream stays empty.
- src/output.rs:
  - `a_document_has_one_compact_line` (line 838): the exact text gets
    `"cost":{"micro_usd":0,"kind":"no_call","reason":"not_started"}` before
    `"results"`.
  - `TOP_LEVEL` (line 942) and `no_field_of_the_form_is_removed_or_renamed`
    (line 966): add `cost` and the list `COST`.
  - `the_largest_document_with_no_result_is_less_than_1000_characters`
    (line 1037): the summary that the test builds field by field gets
    `cost: Cost::Estimate(u64::MAX)`, the largest field. The bound stays,
    because 693 is less than 1,000. `full_summary` (line 901) uses
    `..Summary::default()`, so it does not change.
- src/cli/stop.rs: `a_signal_in_text_mode_writes_the_stop_failure_to_the_error_stream`
  (line 290) does not change: no count, so no line.
  `a_signal_in_structured_mode_writes_one_document_to_standard_output`
  (line 310) does not change. It can add the check of `not_started`.

These tests do not change: tests/check.rs (no line for `--check`),
tests/cost_limit.rs (the notice stays, and the line never holds
`cost limit`, lines 164 and 196), tests/score.rs (`score` keeps its
signature), the tests of `parse_scores` in src/scorer/client.rs, the
closed-pipe tests of tests/stop.rs (lines 176 and 181), tests/secrecy.rs,
tests/search.rs (its checks of the error stream use `contains`),
tests/speed.rs (it checks the exit code and the count of calls, and
writes the error stream only in its messages), and tests/walk.rs.

### The README

A new section after "In a terminal and in a pipe" and before "Speed":

```
## The cost of a search

After each search, jag writes one cost line on the error stream, also when
the output goes to a pipe. It is the last line there:

    jag: cost $0.002016: scored 150 of 200 results, skipped 50.

The amount is in US dollars. jag adds the input tokens that TypeSafe gives
for each answered call, at 0.042 US dollars for 1 million input tokens, the
price of jev-1.13.0. Output tokens are free. A call that gets no answer, for
example after a timeout, adds nothing. After a failure or a Ctrl-C, the line
comes after the message, and it counts the calls that ended before it.

- "(estimate)" after the amount: TypeSafe gave no count for a call, so jag
  counted the bytes of that call and divided them by 3.
- "cost not available": jag does not know the price of the model that
  JAG_MODEL names. The search does not fail.
- "$0.000000 (no question)": jag made no model call. The words in brackets
  give the reason: "no question" (no --ask), "no match", or "no readable
  match".

"Scored" and "skipped" are the counts of the cost-limit notice. With no
question, jag scores nothing, so the line says "scored 0 of N results,
skipped 0", and N is the count of matching lines. With --json, the cost is
the field "cost" of the document. A command that ends before the search
counts its results, and --check, write no cost line. When the reader closes
the pipe, jag writes no cost line.
```

In "Plain grep with no question", a new item at the end of the list: "jag
still writes the cost line on the error stream, with a cost of zero and
the reason `no question`. See "The cost of a search"."

In the table of fields, a new row before `results`:
`| cost | An object with micro_usd (the amount in millionths of a US dollar, or null when not available), kind (billed, estimate, not_available, or no_call), and reason (null, no_question, no_match, no_readable_match, or not_started) |`,
with each name in backticks as in the other rows. After the table: "A
later version can add a kind or a reason, so agent code must accept a
value that it does not know."

The sentence "then it writes that failure there, as text" becomes "then it
writes that failure and the cost line there, as text".

In "In a terminal and in a pipe", the Ctrl-C item: "jag writes
`jag: the search was stopped.`" becomes "jag writes
`jag: the search was stopped.` and the cost line". "an error document with
`exit_code` 130" becomes "an error document with `exit_code` 130 and the
cost".

## Build order

1. STORY-53, the line for each end of a search with a question (C-config,
   C-scorer, C-output, C-cli):
   1. `price_of` and its unit test.
   2. `Bill`, `read_input_tokens`, and `Client::call` with the bill, with
      the unit tests.
   3. `Usage`, `Cost`, and `Cost::of`, with the unit tests.
   4. `score_with_usage` and the usage of `Run`. `score` gives a new usage.
   5. The usage, the model, the end of the write, and `cost` of `Stop`.
   6. `Summary.cost`, `cost_line`, the cost in `search`, `Emitted` and
      `emit`, and the line in `finish`.
   7. `Reply::scores_with_usage`. The end-to-end tests of CRITERION-88 and
      CRITERION-89. The tests that change in tests/output.rs,
      tests/failure.rs, tests/terminal.rs, and the text runs of
      tests/size_limit.rs with a question.
2. STORY-54, the field (C-output): `CostField` in the document, the key
   lists, the bound test, `a_document_has_one_compact_line`, the sizes of
   tests/size_limit.rs (876 and 6865), the README row, and the end-to-end
   tests of CRITERION-90.
3. STORY-55, no call, the estimate, and not available (C-cli, C-scorer):
   the reasons in `search` and in `plain`, the changes in
   tests/plain_grep.rs, in tests/usage.rs, and in
   `plain_grep_has_the_size_limit_and_makes_no_call` of
   tests/size_limit.rs, and the tests of CRITERION-91, CRITERION-92, and
   CRITERION-97.
4. STORY-56, a failed or stopped search (C-cli): the line after a failed
   write in JSON mode, the cost in `on_signal`, the change in
   tests/stop.rs, the README sentences, and the tests of CRITERION-93.
5. The README section "The cost of a search" and the item in "Plain grep
   with no question".

## Assumptions

- The billed amount is the count of input tokens that the service gives,
  times the published price (decision 2). From system-design.md IF-1 ("the
  billed usage") and the TypeSafe docs (12-models.md). The response has no
  amount in money (09-api-reference.md lists `model`, `answers`, and
  `usage`).
- Only `jev-1.13.0` has a price (decisions 3 and 4). From the TypeSafe docs
  (12-models.md: "Current model: Jev 1.13") and src/config/model.rs
  (`DEFAULT_MODEL`). A check of the route can come later with no change to
  the form, because `not_available` exists. So this choice is cheap to
  change.
- An unknown model with no answered attempt shows `not available`, not
  `$0.000000` (decision 7). One order for each case is simpler than one
  more branch.
- An attempt with no answer adds nothing (decision 5). From
  src/scorer/client.rs: jag reads no body for a status other than 200.
- The estimate is the bytes of the request body divided by 3 (decision 6).
  FEATURE-04 spec decision 22 uses this ratio for `state` and the longest
  question, because the limit of the API applies to them. The bill covers
  the whole request, so this spec applies the ratio to the whole body. For
  ASCII text the estimate is high: a token is about 4 bytes. A high
  estimate is safer than a low one, and the estimate has its label.
- With no question, the counts are 0 and 0 (decisions 9 and 33). From the
  FEATURE-12 spec decision 14 and the README table of fields. DECISION-42
  says that the two counts add up to N. That holds for each search with a
  question. With no question, the cost limit skips nothing, and the reason
  `no question` tells why nothing was scored. The line and the field keep
  the values of FEATURE-12, so CRITERION-90 holds.
- `no question` wins over `no match` (decision 14). From src/cli.rs:
  `search` gives the work to `plain` before the scan, so plain grep has no
  other reason.
- 6 decimals (decision 10). From ADR-0002 and README.md (the check call).
- The words of the line (decision 11). From FEATURE-06 spec decision 16,
  tests/output.rs (each line starts with `jag: `), and tests/cost_limit.rs
  (no `cost limit` when nothing is skipped).
- The line is the last line, also after a failure text (decision 12).
  DECISION-44 says that the error message gives the cost. A fixed place
  also lets a script read the line with `tail -n 1`.
- A run that ends before the count writes no line (decision 13). From
  tests/usage.rs, tests/check.rs, and tests/failure.rs (no key): no story
  asks for a line there, nothing was paid, and jag has no counts for the
  line.
- "A field of the error" is a field of the error document (decision 15).
  From system-design.md IF-5. A copy in `error` gives two places for one
  figure. A field can be added later but never removed (CONSTRAINT-05),
  so the smaller form keeps both ways open.
- The cost-limit notice stays (decision 16). From FEATURE-06 spec decisions
  16 and 18, and tests/cost_limit.rs.
- The names and the form of the field (decision 17). From FEATURE-11 spec
  decision 5 and src/output.rs (each number is a whole number).
  Millionths keep the field and the line the same, digit for digit.
- No new ADR (decision 19). From ADR-0008 and the FEATURE-11 spec.
- A failed write gets the line (decision 20). From FEATURE-11 spec decision
  25: when standard output does not work, the error stream gets text.
- The cost of a stop holds the attempts that ended before the stop
  (decision 22). From the FEATURE-15 spec decision 12: jag does not wait
  for the calls that run. The usage is in `Stop`, as the FEATURE-15 spec
  "Other features" says.
- The end of the write is in `Stop` (decision 25). From src/cli.rs: `run`
  reads `stop.size_limit()` after `claim_or_wait` in the same way, and each
  write of the search flow goes through `emit_claimed`. So `search`,
  `plain`, and `write_report` of FEATURE-13 keep their return types.
- The fake server (decision 27). From tests/common/fake_server.rs:
  `Reply::scores` gives 1 input token for each call.
- No line for `--check` (decision 31). From CRITERION-88 and README.md.
- The README (decision 32). From FEATURE-11 spec decision 29, the
  FEATURE-15 spec decision 20, the FEATURE-12 spec, and README.md on
  `main` ("Speed" is the last section).
- `-q` gets the line on the error stream (decision 34). From the FEATURE-13
  spec decision 21: `-q` keeps the notices and the warnings on the error
  stream, as grep does.
- A path that does not exist (decision 35). From the FEATURE-14 spec
  decision 7: its warning is a warning of the walk.

## Risks

- TypeSafe changes the price with no new model version, and jag shows the
  old price. Signal: the bill in the TypeSafe console is not the sum of the
  cost lines. Mitigation: one constant next to `DEFAULT_MODEL`. A release
  changes it, and the unit test fails when `DEFAULT_MODEL` has no price.
- The service bills a call that times out at jag, or a call that runs at a
  Ctrl-C. The figure does not include it. Signal: the console shows more
  than the cost lines, in a time with timeouts or stops. Mitigation: the
  README says that only answered calls count.
- The estimate is high for ASCII text. Signal: an estimate is about one
  third more than the billed figure of the same search. Accepted: the
  estimate has its label.
- A gateway adds a fee, or uses another price. The figure does not include
  it. Signal: a gateway user reports a bill that is not the figure.
  Mitigation: a model name other than the pinned one gives
  `not available`. The section "The cost of a search" says that the amount
  does not include a fee of a gateway (decision 30).
- A script expects an empty error stream after a text search, also plain
  grep and `jag -q`, and now gets the cost line. Signal: a user report.
  Mitigation: DECISION-41 refused an option that hides the line. The line
  is the last line on the error stream, and it starts with `jag: cost $`
  or `jag: cost not available`. `-q` has the most risk, because a script
  that uses `-q` reads only the exit code. A later change can remove the
  line for `-q` with no change to the form.
- The merges with the code of FEATURE-13 (PR #170), FEATURE-21, and
  FEATURE-14 change the same code, tests, and README. Signal: a test fails
  after the second merge, or a search with `-l`, `-c`, or `-q` gives no
  cost line. Mitigation: decisions 30, 34, and 35 say what the code that
  merges second changes.
- A later model has a price that is not a whole number of billionths for
  each token. Signal: the release cannot write the price as a whole number.
  Mitigation: the code then uses a smaller unit inside. The field does not
  change.

## Out of scope

- A cost line for `--check`: no story asks for it (decision 31).
- The price of other models and of a gateway: a later feature, when
  TypeSafe has another model or a gateway gives a price.
- The token counts in the output: no story asks for them, and DECISION-40
  refused service units as the figure.
- The reuse of scores (DECISION-46): release 2.
- The gateway settings: FEATURE-21 (decision 30).
- The options `-l`, `-c`, and `-q`: FEATURE-13 (decision 34).
- The search of standard input and a path that does not exist: FEATURE-14
  (decision 35). Their results use the same cost.
