---
title: "Size limit on the text that jag returns: the limit option, the fit of whole results, the cut of the top result, the notice, and exit 8"
feature: FEATURE-09
stories: [STORY-25, STORY-26, STORY-27, STORY-28]
kind: build
gh_issue: 86
lifecycle_state: draft
inherits: [IF-4, IF-5, IF-6, C-cli, C-config, C-output, C-search, CV-2, CV-3, CV-4, CV-6, CV-7, DF-1]
session: 84086903-189a-4495-bbe8-0cfb867635cd
---

# Size limit on the text that jag returns: the limit option, the fit of whole results, the cut of the top result, the notice, and exit 8

## Problem

jag never writes more characters to standard output than a size limit. A
default limit always applies, and the user can raise it or turn it off. To
stay inside the limit, jag leaves out whole results, with the lowest rank
first. When the top result alone is larger than the limit, jag cuts it,
marks the cut, and says so. A notice gives the count of the results that
jag left out and tells how to raise the limit. Leaving out results gives
its own exit code (FEATURE-09, CAPABILITY-04, DECISION-19 to DECISION-26,
DECISION-47).

- STORY-25: an agent builder wants jag to never write more characters to
  standard output than a size limit, with a default that always applies,
  so that the output of jag fits the context of the agent.
  - CRITERION-36: the size limit is a count of characters, and jag never
    writes more characters to standard output than the limit.
  - CRITERION-37: jag always applies a default size limit, and the user can
    raise it or turn it off.
  - CRITERION-38: labels, the structured wrapper, and notices on standard
    output count toward the limit, and text on the error stream does not.
- STORY-26: an agent builder wants jag to leave out whole results with the
  lowest rank first, give a notice with their number and how to raise the
  limit, and exit with its own code.
  - CRITERION-39: to stay inside the limit, jag leaves out whole results,
    starting with the lowest rank.
  - CRITERION-41: when jag leaves out results, a notice gives their number
    and tells the user how to raise the limit.
  - CRITERION-42: when jag leaves out results to stay inside the limit, it
    exits with its own exit code above 2.
  - CRITERION-66: when a search reaches the cost limit and also leaves out
    results, jag exits with the cost-limit code, and the notice says that
    results were left out.
  - CRITERION-104: in structured output, when jag leaves out results or
    cuts the top result, the fields `left_out`, `top_result_cut`,
    `size_limit_fix`, and `warnings_left_out` give the counts and the fix
    text of the size-limit notice, and the error stream holds no size-limit
    notice.
- STORY-27: an agent builder wants jag to cut the top result and say so
  when that result alone is larger than the limit, so that the agent still
  gets the best answer.
  - CRITERION-40: when the top result alone is larger than the limit, jag
    cuts it, marks the cut, and says so.
- STORY-28: an agent builder wants a limit of zero, a negative limit, or a
  value that is not a number to give a usage error, so that a typo does not
  hide the output.
  - CRITERION-43: a limit of zero, a negative limit, or a value that is not
    a number gives a usage error and exit code 2.

The code on `main` writes each result in the `grep -n` form of FEATURE-08
(`src/output.rs` `write_text`). The search keeps all output in memory until
the end (CV-2). FEATURE-06 made `SearchOutcome.left_out` and the exit order
1, 7, 8, 0, but `left_out` is always false. FEATURE-11 (PR #139) writes
the JSON document of `--json`, with its 4 size-limit fields at the values
for "nothing left out or cut". This spec adds the limit setting, the fit of
whole results, the cut of the top result, the notice, the value of
`left_out`, and the values of the 4 size-limit fields. METRIC-06 guards the
cut: jag must not save room by cutting the line that answers the question.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Spec grouping | One spec for STORY-25 to STORY-28. The fit, the cut, and the notice use one function of `src/output.rs` | docs/adlc.md (one feature gives one spec) |
| 2 | Unit of the limit | Characters. A character is a character of the text that `String::from_utf8_lossy` gives: each UTF-8 character counts as 1, and each bad byte sequence counts as 1 | DECISION-19; CRITERION-36; FEATURE-08 spec decision 9 |
| 3 | What counts | Each character that jag writes to standard output: the no-good-answer line, each path, each `:` and `-`, each line number, the text, each cut mark, each `--` line, and each `\n`. In structured mode, the full document. The error stream does not count: notices, warnings, the cost notice, and errors | DECISION-22; CRITERION-38; DECISION-29; system-design.md CV-3 |
| 4 | Names of the setting | The option `--max-chars COUNT` and the variable `JAG_MAX_CHARS`. The earlier prototype used `--max-output` | Inferred: FEATURE-06 spec decision 1 (human:404pilo, Q-aw: the name says what jag counts, as in `--max-scored`); DECISION-19 (the unit is characters) |
| 5 | Turn the limit off | The flag `--no-max-chars`. The variable has no value that turns the limit off, because each value that is not a number is a usage error. A very large value has almost the same effect | Inferred: DECISION-20 and CRITERION-37 (the user can turn the limit off); DECISION-25 (a value that is not a number is a usage error, so no value such as `off` can exist); the `--no-` form of grep options such as `--no-filename` |
| 6 | Default limit | 20,000 characters | human:404pilo, 2026-10-14 (the answer to the FEATURE-09 ASK: the default of 20,000 and the smallest limit of 1,000); the earlier prototype (its default output limit is 20,000 characters); DECISION-20 (a default that is sized for the context of an agent) |
| 7 | Values of the limit | A whole number of 1,000 or more: ASCII digits only, after a trim. A number that is too large for `usize` gives `usize::MAX`, so the message stays true. Zero, a negative number, a number below 1,000, and a value that is not a number give exit 2. The earlier prototype moved a value outside its range into the range. This spec gives exit 2 in place of that, because of DECISION-25 | DECISION-25; CRITERION-43; FEATURE-06 spec decision 3 (digits only). The minimum: human:404pilo, 2026-10-14 (the answer to the FEATURE-09 ASK); the earlier prototype (its smallest output limit is 1,000 characters); DECISION-22 and DECISION-30 (the structured document is always complete and inside the limit); the no-good-answer line (72 characters) always fits |
| 8 | Setting order | `--max-chars` or `--no-max-chars`, then `JAG_MAX_CHARS`, then the default. No key in the config file. An option hides a bad variable | system-design.md CV-7 and DF-1; FEATURE-06 spec decision 5 |
| 9 | Where jag checks the setting | The option: in step 1, after `--max-scored`, before the key. The variable: in step 3, after `JAG_MAX_SCORED` | FEATURE-06 spec decision 6; FEATURE-03 spec decision 11 |
| 10 | Option conflicts | `--max-chars` with `--no-max-chars`, and each of them with `--check`, `--setup`, or `--forget-key`, give the fixed line of the argument error rule and exit 2. An option that has no effect in a mode conflicts with that mode | FEATURE-02 spec decision 18; FEATURE-06 spec ("The limit setting"); `src/cli.rs` `USAGE_ERROR` |
| 11 | Secrecy of a bad value | No error message shows the value of the option or of the variable. The notice of decision 15 shows a valid limit. A valid limit holds only digits, so it is not a key. So this spec applies FEATURE-06 spec decision 7 to error messages only | system-design.md CV-6; FEATURE-06 spec decision 7; DECISION-24 (the notice helps the agent to raise the limit) |
| 12 | Which results stay | The most results from the top rank that fit whole, in rank order. jag leaves out each result after them. It never writes a lower result in place of a higher result that does not fit | DECISION-21; CRITERION-39 |
| 13 | Cut of the top result | Only when no result fits whole. jag writes only the top result, cut, and leaves out each other result. The cut keeps the first matching line (see "The cut of the top result"). One mark ` [+N chars]` at the end of the last line that jag writes, in place of the mark of that line. N is the count of the characters of the result text that the output does not show, with 1 for the end of each line that jag left out | DECISION-23; CRITERION-40; METRIC-06 (jag must not cut the line that answers); DECISION-21 (no other result is cut); FEATURE-08 spec decisions 3 and 12 (the mark form) |
| 14 | Top result with no room for one line | When the path, the line number, and the mark of the first matching line do not fit, jag writes nothing of the top result and leaves it out. A path that jag cuts does not open the file | This spec; CAPABILITY-03 (the user opens the file at the line) |
| 15 | Notice | One line on the error stream: `jag: size limit reached: showed K of N results, left out R. To see more, use --max-chars with a number larger than L, or --no-max-chars.` With a cut: `left out R, and cut the top result.` N is the count of the ranked results, and L is the limit in digits. The limit is a valid number, so it is not a secret | DECISION-24; CRITERION-41; DECISION-23 ("says that it cut the result"); CV-3; FEATURE-06 spec decision 16 (the form of the cost notice) |
| 16 | When the notice shows | When jag leaves out a result or cuts the top result, also when exit 1 or exit 7 wins. It comes after the cost notice and before jag writes standard output, so it shows also when the reader closes the pipe | DECISION-47 ("The notice still says that jag left out results"); CRITERION-66; FEATURE-06 spec decision 18; system-design.md CV-4 |
| 17 | Exit code | R > 0 or a cut of the top result sets `left_out`, which gives exit 8. So a cut with R = 0 also gives exit 8: the agent sees from the exit code alone that it did not get all of the best result. The order 1, 7, 8, 0 does not change | human:404pilo, 2026-10-14 (the answer to the FEATURE-09 ASK: exit 8 also when jag cuts only the top result); DECISION-26 ("An agent can then see from the exit code alone that it did not get all the results"); system-design.md IF-4 ("8 results left out by the size limit"); DECISION-47; FEATURE-06 spec decision 18 |
| 18 | Structured mode | The limit counts the full document. The FEATURE-11 code is on `main` (PR #139), so the FEATURE-09 code connects the two parts (see "Structured mode"). FEATURE-11 writes the top-level fields `left_out`, `top_result_cut`, `size_limit_fix`, and `warnings_left_out` with the values for "nothing left out or cut". This spec sets their values. The results fit first, then the warnings. jag never cuts a document. A path in an error document gets shorter when the document does not fit | DECISION-22; DECISION-28; DECISION-30; CONSTRAINT-05; CRITERION-104; 0008-write-structured-output-as-one-json-document.md; system-design.md CV-3 and IF-5; the FEATURE-09 and FEATURE-11 split of the orchestrator, 2026-10-13, and the FEATURE-11 answers of specifier-2 (the writer, the field names, and the bound); the FEATURE-11 spec (PR #136, decisions 16, 18, 21, 27, and 31); the FEATURE-11 code (PR #139, `src/output.rs` `Summary`) |
| 19 | Place of the code | `src/config/limit.rs` (C-config): the setting. `src/output.rs` (C-output): the fit, the cut, and the notice. `src/search/snippet.rs` (C-search): the character count. `src/cli.rs` (C-cli): the options and the search flow. No new crate | system-design.md C-output ("applies the size limit and writes the notices"); 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md; 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md |
| 20 | Names in the code | The code says "size limit" (`SizeLimit`, `size_limit`, `SizeLimitBad`, `DEFAULT_SIZE_LIMIT`, `MIN_SIZE_LIMIT`), the name of the PRD. Only the option texts, the variable name, and the message texts hold `max-chars` | FEATURE-06 spec decision 21 (the same rule for "cost limit"); CAPABILITY-04 |
| 21 | Cost of the fit | The fit runs after the scores, in memory. It reads no file and makes no call. In text mode, it counts each result one time. In structured mode, it writes the document to memory one time for each size that it tries: about log2(N) + 2 times for the results, one time for each warning that it tries, and one time for each step of the cut. No document is larger than the first one | system-design.md CV-2 and QS-3 |
| 22 | Story state | `ready` in this spec PR. On `main`, PR #156 set STORY-25 to STORY-28 to `backlog` until this spec merges | docs/adlc.md rule 11: the spec is in this PR, FEATURE-08 is `done` (PR #130), so FEATURE-09 is not blocked, and the two ASKs of this spec have their answers (human:404pilo, 2026-10-14) |

## Design

### Files

```
src/config/limit.rs     C-config: SizeLimit, the size limit constants, parse_size_limit, size_limit
src/config.rs           C-config: ConfigError::SizeLimitBad and its texts
src/search/snippet.rs   C-search: char_count, one rule with cut_line
src/output.rs           C-output: SizeReport, TextReport, fitting_count, the cut, write_text, write_lines, size_notice, the fit of a document
src/cli.rs              C-cli: --max-chars, --no-max-chars, the search flow, plain grep, left_out, the known limit
tests/size_limit.rs     end-to-end tests of CRITERION-36 to CRITERION-42, CRITERION-66, and CRITERION-104
tests/usage.rs          the limit argument tests (CRITERION-43) and the help
```

No new crate. This spec uses these items of the earlier specs:

- FEATURE-08: `Snippet`, `SnippetLine` (`text`, `is_match`, `cut`),
  `Source::label`, `cut_line`, `MAX_LINE_CHARS`, `NO_GOOD_ANSWER`, and the
  text form of `write_text`.
- FEATURE-06: `SettingSource`, the `parse_cost_limit` pattern,
  `SearchOutcome`, `search_exit_code`, and `tests/cost_limit.rs`.
- FEATURE-04: the fake server, `rank`, and the search flow.
- FEATURE-11: `Summary`, `write_json`, `Mode`, `error_document`, `finish`,
  and the test helpers of `tests/output.rs`.
- FEATURE-12: `plain`, `write_lines`, and `write_json` with
  `results: &[&Snippet]` (see "Other features").

### The limit setting (C-cli, C-config)

The `Cli` struct gets two options, after `--max-scored`:

```rust
/// The most characters that jag writes to standard output, 1,000 or more (default 20,000)
#[arg(long = "max-chars", value_name = "COUNT", conflicts_with_all = ["check", "setup", "forget_key"])]
size_limit: Option<String>,

/// Turn off the size limit on standard output
#[arg(long = "no-max-chars", conflicts_with_all = ["size_limit", "check", "setup", "forget_key"])]
no_size_limit: bool,
```

- The type of `--max-chars` is `String`. jag parses the text itself, so
  that the message is its own and never shows the value (decision 11).
- clap conflicts work in both directions, so the `--check` flag of
  FEATURE-02 does not change.
- The usage text does not change. Since the FEATURE-12 code (PR #160),
  its search lines are `jag [OPTIONS] TERM [PATH...]` and
  `jag [OPTIONS] --ask QUESTION TERM [PATH...]`, with no option name. The
  help lists `--max-chars <COUNT>` and `--no-max-chars`.

`src/config/limit.rs` adds:

```rust
pub const SIZE_LIMIT_OPTION: &str = "--max-chars";
pub const SIZE_LIMIT_OFF_OPTION: &str = "--no-max-chars";
pub const SIZE_LIMIT_VARIABLE: &str = "JAG_MAX_CHARS";
pub const DEFAULT_SIZE_LIMIT: usize = 20_000;
pub const MIN_SIZE_LIMIT: usize = 1_000;

/// The size limit of a search (CAPABILITY-04).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeLimit {
    /// The most characters on standard output.
    Chars(usize),
    /// `--no-max-chars`: no limit.
    Off,
}

impl SizeLimit {
    /// The count of characters, or `None` for no limit.
    pub fn chars(self) -> Option<usize>;
}

pub fn parse_size_limit(text: &str, source: SettingSource) -> Result<usize, ConfigError>;
pub fn size_limit(env: &Env, option: Option<SizeLimit>) -> Result<SizeLimit, ConfigError>;
```

`parse_size_limit`:

1. Trim the text.
2. The text must be 1 or more ASCII digits and nothing else. So `1000`,
   `20000`, `01000`, and ` 40000 ` pass. `0`, `999`, `0999`, `-1`, `+1000`,
   `1e4`, `20,000`, `1000.0`, `20000 chars`, `off`, `none`, and a blank
   text fail.
3. Parse it as `usize`. A number that is too large for `usize` gives
   `usize::MAX`. The value must be `MIN_SIZE_LIMIT` or more.
4. Each failure gives `ConfigError::SizeLimitBad { source }`.

`size_limit` gives the option when it is set (decision 8). Else it reads
`JAG_MAX_CHARS`. A variable that is not set or is blank gives
`Chars(DEFAULT_SIZE_LIMIT)`, as FEATURE-01 decision 14 says. A value that is
not text, or that `parse_size_limit` refuses, gives
`SizeLimitBad { source: Variable }`. Else it gives `Chars(value)`.

`ConfigError::SizeLimitBad` gives exit 2. Standard error is exactly
`jag: <cause>\n<fix>\n`:

| Source | Cause | Fix |
|---|---|---|
| Option | `the value of --max-chars is not a whole number of 1,000 or more.` | `Use a number such as --max-chars 40000, or turn the limit off with --no-max-chars.` |
| Variable | `JAG_MAX_CHARS is not a whole number of 1,000 or more.` | `Set JAG_MAX_CHARS to a number such as 40000, or unset it.` |

Build the texts from `SIZE_LIMIT_OPTION`, `SIZE_LIMIT_OFF_OPTION`, and
`SIZE_LIMIT_VARIABLE`.

clap can read `--max-chars -1` as an unknown option. Then jag gives the
fixed line of the argument error rule. Both texts give exit 2, so
CRITERION-43 holds. `--max-chars=-1` gives the text of the table.

### The count of characters (C-search)

```rust
/// The count of characters of `String::from_utf8_lossy(bytes)`, with no copy (decision 2).
pub fn char_count(bytes: &[u8]) -> usize;
```

`cut_line` and `char_count` share one private function that gives the
width in bytes of each character, from `bytes.utf8_chunks()`. So the cut
and the count use one rule.

The count of joined texts is the sum of their counts when each text ends
with `\n`. A bad sequence holds only bytes of 0x80 or more, so it never
holds `\n`, and the decode starts again after each `\n`. So the fit can add
the counts of the results.

### The fit (C-output)

```rust
/// What the size limit did to the output of one search.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SizeReport {
    /// The results that jag wrote, with a cut top result.
    pub shown: usize,
    /// The results that the size limit left out.
    pub left_out: usize,
    /// The N of the mark of a cut top result. 0 for no cut.
    pub top_result_cut: u64,
}

/// What `write_text` wrote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextReport {
    pub no_good_answer: bool,
    pub size: SizeReport,
}

/// Write the results in rank order, inside the size limit. `paint` is the color of
/// FEATURE-15, as on `main`. `limit` is `None` for no limit.
pub fn write_text(
    out: &mut dyn Write,
    ranked: &[(Snippet, f64)],
    paint: Option<Paint>,
    limit: Option<usize>,
) -> io::Result<TextReport>;

/// The largest k from 0 to `count` with `size(k) <= limit`. `size(k)` must grow with k.
pub fn fitting_count(count: usize, limit: usize, size: impl Fn(usize) -> usize) -> usize;
```

`write_text`:

1. The label line: `NO_GOOD_ANSWER` and `\n` when the top score is below
   `THRESHOLD`, as now. It counts 72 characters.
2. The block of each result: its lines in the FEATURE-08 form. The block
   of each result after the first starts with `--\n`. Count each block with
   `char_count`.
3. With no limit, write the label and all blocks. `shown` is N, and the
   other values are 0.
4. Else `room` is `limit.checked_sub(label count)`. `size(k)` is the sum
   of the first k block counts. `K = fitting_count(N, room, size)`. When
   `room` is `None`, the label alone does not fit: jag writes nothing, and
   each result is left out. Only a limit below 72 does this, and the
   setting does not allow it, but the unit tests use small limits.
5. When K is 1 or more: write the label and the first K blocks.
   `shown = K`, `left_out = N - K`, and `top_result_cut = 0`.
6. When K is 0 and N is 1 or more: write the label and the cut of the top
   result (next section). `shown` is 1, or 0 for decision 14.
   `left_out = N - shown`. With N = 0, jag writes nothing, as now.

`fitting_count` can use a binary search. The text fit can also add the
blocks one at a time. The structured mode needs the binary search, because
there each `size(k)` writes a full document.

Example: 30 results of 113 characters each, and a limit of 1,000. With
`--\n`, k results need `116k - 3` characters. 8 results need 925, and 9
results need 1,041. So K = 8, and jag leaves out 22 results.

### The cut of the top result (C-output, STORY-27)

The cut applies only when K = 0: the top result alone does not fit in the
room. jag writes only the top result, and leaves out each other result
(decision 13). The full text of a line is its length in the file:
`char_count(&line.text) + line.cut`. Steps 4 and 5 use it. Steps 1 and 2
measure each line in its output form: the path, the separators, the line
number, the kept text, its own mark, and `\n`.

1. The first line: the first line of the result. When the lines from the
   first line through the first matching line, whole, and the mark do not
   fit, jag leaves out lines from the top, one at a time, until they fit,
   or until the first matching line is the first line. It never leaves
   out the first matching line in this step. The lines before the first
   matching line are lines around the match, so the matching line stays
   (METRIC-06).
2. jag writes whole lines, in order, from the first line, while the lines
   that it wrote, the next line, and the mark fit together. A line that
   jag writes whole keeps its own mark from FEATURE-08.
3. When the first matching line does not fit whole with the mark, jag
   writes its path, its separators, its line number, and the first
   characters of its text that fit with the mark. The cut ends at a
   character, never inside one.
4. The mark ` [+N chars]` goes at the end of the last line that jag
   writes, in place of the FEATURE-08 mark of that line. N is the count of
   the characters of the result text that the output does not show: the
   text of each line that jag left out, with 1 for its line end, and the
   rest of the text of the last line. A whole line that keeps its own mark
   does not count in N. So N is 1 or more, also when jag leaves out only
   empty lines.
5. To find the room for the mark, use the digits of the full count of the
   result: the text of each line, with 1 for each line end. So the output
   can be some characters below the limit, but never above it.
6. When step 3 cannot write the path, the separators, the line number, and
   the mark, jag writes nothing of the result (decision 14). Then `shown`
   is 0, and standard output holds only the label line, or nothing.

The mark has the form of the FEATURE-08 mark, so a script finds each cut
with one pattern. For a cut top result, N counts the text of the full
result that the output does not show and that no other mark shows.

Example 1, the end of the result: a limit of 1,000, a good score, and the
top result `d/top.txt`, lines 1 to 7. Each line has 200 characters, and
line 4 is the matching line. Each line takes 213 characters in the output,
so the result needs 1,491. The full count is 1,407, so the room for the
mark is 14 (` [+1407 chars]`). Lines 1 to 4 and the mark need 866, so line
1 stays. Lines 1 to 4 take 852. Line 5 and the mark do not fit. So jag
writes lines 1 to 4, and line 4 ends with ` [+603 chars]`: the 600
characters and the 3 line ends of lines 5, 6, and 7. The output has 865
characters.

Example 2, lines before the match: the same result with 300 characters in
each line. Each line takes 313 characters, so the result needs 2,191. The
room for the mark is 14. Lines 1 to 4 and the mark need 1,266, and lines 2
to 4 and the mark need 953. So jag leaves out line 1, writes lines 2, 3,
and 4, and line 4 ends with ` [+1204 chars]`, for lines 1, 5, 6, and 7.
The output has 953 characters.

### The notice (C-output)

```rust
/// The notice of the size limit. `None` when there is no limit, or when jag left out no
/// result and cut nothing.
pub fn size_notice(size: &SizeReport, limit: Option<usize>) -> Option<String>;
```

N is `shown + left_out`, the count of the ranked results. The notice is
one line, with no value from the command line other than the limit
(decision 15):

- `jag: size limit reached: showed 8 of 30 results, left out 22. To see more, use --max-chars with a number larger than 1000, or --no-max-chars.`
- With a cut: `jag: size limit reached: showed 1 of 3 results, left out 2, and cut the top result. To see more, use --max-chars with a number larger than 1000, or --no-max-chars.`
- A cut with no other result: `... showed 1 of 1 results, left out 0, and cut the top result. ...`
- Decision 14: `... showed 0 of 3 results, left out 3. ...`

Build the texts from `SIZE_LIMIT_OPTION` and `SIZE_LIMIT_OFF_OPTION`. The
word is always `results`, as in the cost notice.

### The search flow (C-cli)

The search flow of FEATURE-06 changes in these steps:

- Step 1: after `--max-scored`, read the size limit option.
  `--no-max-chars` gives `Some(SizeLimit::Off)`. `--max-chars` gives
  `Some(SizeLimit::Chars(n))` with `parse_size_limit(text,
  SettingSource::Option)`. A failure gives exit 2 before the key (decision
  9). `SearchArgs` gets `size_limit: Option<SizeLimit>`.
- Step 3: after `cost_limit(env, args.cost_limit)`, add
  `let size = size_limit(env, args.size_limit)?;`. `size` is a
  `SizeLimit`. A failure gives exit 2.
- Step 14, in text mode: `let report = write_text(&mut buffer, &ranked,
  paint, size.chars())`. When `size_notice(&report.size, size.chars())` gives a
  text, write it to the error stream with `stop.notice`, as the other
  notices of `search`, before jag writes standard output (decision 16).
  Then write the buffer, as now. The exit code is
  `search_exit_code(SearchOutcome { no_good_answer: report.no_good_answer,
  cost_limit_reached, left_out: report.size.left_out > 0 ||
  report.size.top_result_cut > 0 })` (decision 17).
- In structured mode, each document of a search goes through the steps of
  "Structured mode": the document of step 14, the document with no result
  (exit 1), and the error document of `finish`. `run` holds the limit that
  jag knows. It starts as the default, and `search` changes it after it
  reads the option in step 1 and the variable in step 3. `finish` gets it.

The cost notice comes in step 11, before the scores. So when both limits
apply, the error stream holds the cost notice, then the size notice.

### The exit code of a search (C-cli)

`search_exit_code` does not change. `left_out` is now true when the size
limit left out one result or more, or cut the top result:

| no good answer | cost limit reached | left out | Exit |
|---|---|---|---|
| yes | any | any | 1 |
| no | yes | any | 7 |
| no | no | yes | 8 |
| no | no | no | 0 |

A cut of the top result with no result left out also gives exit 8
(decision 17). The notice, the mark, and `top_result_cut` in structured
mode show the cut.

### Structured mode (C-output, IF-5): the link with FEATURE-11

FEATURE-11 (PR #139, on `main`) owns the structured option `--json`, the
one JSON document of IF-5, its writer `output::write_json(out, summary,
results)`, and its field names. Its notices are named numbers, and its
warnings are `{path, text}` objects in `warnings`. Its `Summary` holds the
4 size-limit fields, and it writes them with the values for "nothing left
out or cut": 0, 0, `""`, and 0. No text in the document starts with
`jag: `. This spec owns the limit rule, and it sets the values of the 4
fields in each document (DECISION-28, CONSTRAINT-05, CRITERION-104):

| Field | Value |
|---|---|
| `left_out` | A whole number: R, the count of the results that the size limit left out (CRITERION-39) |
| `top_result_cut` | A whole number: the N of the cut of the top result. 0 when jag did not cut it (CRITERION-40) |
| `size_limit_fix` | A text: the second sentence of "The notice", for example `To see more, use --max-chars with a number larger than 1000, or --no-max-chars.` An empty text when jag left out no result and no warning, and cut nothing (CRITERION-41, CRITERION-104) |
| `warnings_left_out` | A whole number: the count of warnings that the size limit left out (step 5) |

The FEATURE-11 tests of the field names already hold the 4 names. The
FEATURE-09 code builds these steps in one function of `src/output.rs`.
Each write of a document in the search flow calls it (see "The search
flow"). The argument error document of `parse` comes before jag reads the
limit. It holds only fixed texts and no warning, so it always fits, and it
does not change. The steps:

1. The limit counts each character of the document that jag writes: the
   wrapper, the field names, the counts, the results, the warnings, and
   the `\n` at the end (DECISION-22).
2. Before each write to a buffer, jag sets the fields of the summary:
   `left_out`, `top_result_cut`, `size_limit_fix`, `warnings_left_out`,
   and the exit code of the document, with the order of "The exit code of
   a search".
3. jag writes the document with all N results and all warnings to a
   buffer and counts it. When it fits, jag writes it.
4. Else jag will leave out or cut something, so it sets `size_limit_fix`
   to its text. It fits the results first, with no warning in the
   document: `fitting_count` finds the largest K from 0 to N whose
   document fits. Each of these documents holds the same `size_limit_fix`
   text. `left_out` is N - K, so it can lose one digit when K grows. But
   one more result adds more characters than one digit, so the size grows
   with K. Call `write_json` with the first K results.
5. Then jag adds the warnings, in order, while the document still fits. A
   warning that jag adds stays whole. `warnings_left_out` counts the other
   warnings. A warning is not a result, so it gives no exit 8 (IF-4).
6. When K is 0 and N is 1 or more, jag cuts the top result with the steps
   of "The cut of the top result", before step 5. The size of the document
   takes the place of the size of the text. The JSON line holds the kept
   text and the cut field of FEATURE-11 for that line. The JSON text holds
   no ` [+N chars]` mark. `top_result_cut` holds N.
7. A document with no result is still complete. jag never cuts a document
   (DECISION-30, ADR-0008). The document with no result and no warning
   always fits (see "Assumptions").
8. An error document holds no result. When the failure comes after the
   scan, it holds the warnings that jag found (FEATURE-11 decision 21).
   jag adds them with step 5. Its limit is the limit that jag knows when
   the error comes: the option or the variable when jag already read it,
   else the default. When the error document with no warning does not fit,
   jag sets `size_limit_fix` to its text and makes the paths in the error
   texts shorter. It finds the largest length M with which the document
   fits: each path that is longer than M becomes `...` and the last M - 3
   characters of the path. So jag cuts the longest path first, and two
   long paths get the same length. A path that shows two times gets the
   same form at each place. Only a very long path can make an error
   document with no warning larger than 1,000 characters.
9. jag writes no size notice to the error stream in structured mode
   (CV-3).

Tests of the connection. With the test in the CRITERION-104 row of
"Acceptance criteria and their tests", they are the tests of CRITERION-104.
Each end-to-end test runs with `--json`:

- For limits from 1,000 to 20,000, with many results: the document never
  has more characters than the limit, it always parses, and `left_out` is
  N - K, with K the largest count that fits.
- The cut: the folder of CRITERION-40 with `--max-chars 1000`. The document
  holds part of `top.txt`, `left_out` is 1, `top_result_cut` is more than
  0, `size_limit_fix` is the text of the CRITERION-104 row, and jag exits
  8. With only `top.txt`: `left_out` is 0, `top_result_cut` is more than
  0, `exit_code` is 8, and jag exits 8 (decision 17).
- Warnings: 50 files that jag cannot read, one file that matches, and
  `--max-chars 1000`. With a server that gives 0.8: the document fits,
  `warnings_left_out` is more than 0, `size_limit_fix` holds its text, and
  jag exits 0, because a warning is not a result. With a server that gives
  500 to each call (exit 6), and with a term that matches no file (exit
  1): the document fits, and `warnings_left_out` is more than 0. A user
  who can read each file gets no warning, so these tests check the
  warnings only when jag cannot read the files, as `notice_folder` of
  `tests/output.rs` does.
- The FEATURE-09 code changes the FEATURE-11 unit test
  `the_largest_document_with_no_result_is_less_than_1000_characters` in
  `src/output.rs`: it compares with `MIN_SIZE_LIMIT` in place of the
  number 1000. Its fix text of 120 characters is longer than the longest
  real fix text: 95 characters, with a limit of 20 digits.
- An error document with a key file path of more than 1,000 characters
  and `--max-chars 1000` fits and parses.

### Acceptance criteria and their tests

All end-to-end tests use the fake server and the test setup of
`tests/cost_limit.rs`: `HOME` in a `tempfile` folder, the key text
`SENTINEL-SENTINEL-SENTINEL`, `JAG_BASE_URL` of the fake server, no proxy
variables, and `env_remove` of `JAG_MAX_CHARS` and `JAG_MAX_SCORED` unless
the test sets them. In each test, standard output and standard error do not
hold `SENTINEL`. Each test gives `--ask q`, so it is a question search,
unless it is a test of plain grep (FEATURE-12, see "Other features").

Each test folder holds its files in a folder `d`. The tests run jag with
`current_dir` in the test folder and the path `d`, so each path is
`d/<name>`, and each count is the same on each machine. The test of
CRITERION-32 in `tests/search.rs` shows this path form (`dir` gives
`dir/sub/a.txt`).

The "ranked folder": 30 files `r00.txt` to `r29.txt`. File `rNN.txt` holds
one line of 100 characters: `needle NN ` and then `x` characters. The
server gives the snippet of `rNN.txt` the score `0.9 - NN / 100`, so the
rank order is `r00` to `r29`. Each result takes 113 characters
(`d/r00.txt:1:`, 100 characters, and `\n`), and each `--\n` takes 3.

The "big folder": 200 files `b000.txt` to `b199.txt`, each with one line
of 100 characters that starts with `needle`. The server gives each snippet
0.8. The output with all results is more than 20,000 characters.

| Criterion | Test |
|---|---|
| CRITERION-36 jag never goes over the size limit | `tests/size_limit.rs`: the ranked folder with `--max-chars 1000`. Standard output has 925 characters, at most 1,000, and holds `r00` to `r07`. With `--max-chars 1041`, standard output has exactly 1,041 characters and holds 9 results: the limit is a count that the output can reach. The "UTF-8 folder" (10 files `u0.txt` to `u9.txt`, each with one line: `needle ` and 93 `é` characters, and the score 0.8): all 10 results take 1,147 characters and 2,077 bytes. `--max-chars 1147` shows all 10 results and exits 0, so the limit counts characters, not bytes. Unit tests of `write_text` (see "More tests"). |
| CRITERION-37 A default size limit always applies | The big folder with no setting: standard output has at most 20,000 characters, exit 8, and the notice ends with `larger than 20000, or --no-max-chars.` With `--max-chars 40000`: all 200 results, exit 0, and no notice. With `--no-max-chars`: all 200 results and exit 0. With `JAG_MAX_CHARS=40000`: all 200 results. With `--max-chars 1000` and `JAG_MAX_CHARS=40000`: at most 1,000 characters (the option wins). With `--no-max-chars` and `JAG_MAX_CHARS=abc`: all results and exit 0 (the option hides the bad variable). With `JAG_MAX_CHARS=` (blank): the default. Unit tests of `size_limit`. |
| CRITERION-38 All text on standard output counts | The ranked folder with `--max-chars 1041`. A server that gives 0.8 to each snippet: 9 results, exactly 1,041 characters. A server that gives 0.2 to each snippet: the label line and 8 results, 997 characters, exit 1. The label takes the room of the ninth result. The error stream holds the notice, and its characters do not count. The structured wrapper counts: the ranked folder with `--json --max-chars 1000` gives a document with 2 results, where the text holds 8 (see CRITERION-104). Unit tests: the `--\n` lines and the marks count. |
| CRITERION-39 jag leaves out whole results with the lowest rank first | The ranked folder with `--max-chars 1000`: standard output holds `r00` to `r07` in rank order, and no line of `r08` to `r29`. Each result that shows has all its lines. Unit test: a large second result and a small third result. The second result does not fit, so the third result does not show, although it fits. |
| CRITERION-40 jag cuts a top result that is larger than the limit | A folder with `top.txt` of Example 1 and `low.txt` with one line `needle`. The server gives `top.txt` 0.9 and `low.txt` 0.5. With `--max-chars 1000`: standard output is lines 1 to 4 of `top.txt`, line 4 ends with ` [+603 chars]`, and it has 865 characters. Exit 8. The error stream holds `jag: size limit reached: showed 1 of 2 results, left out 1, and cut the top result. To see more, use --max-chars with a number larger than 1000, or --no-max-chars.` With only `top.txt`: the same output, exit 8 (decision 17), and `showed 1 of 1 results, left out 0, and cut the top result.` Unit tests of each step of the cut (see "More tests"). |
| CRITERION-41 The notice gives the count and how to raise the limit | The ranked folder with `--max-chars 1000`: the error stream holds exactly one line `jag: size limit reached: showed 8 of 30 results, left out 22. To see more, use --max-chars with a number larger than 1000, or --no-max-chars.` The ranked folder with no setting gives no `size limit` text. |
| CRITERION-42 Leaving out results has its own exit code | The ranked folder with `--max-chars 1000` exits 8. With `--max-chars 4000` it exits 0. |
| CRITERION-66 Cost-limit code wins when both limits apply | The ranked folder with `--max-scored 10 --max-chars 1000` exits 7. The error stream holds `jag: cost limit reached: scored 10 of 30 results, skipped 20.`, then `jag: size limit reached: showed 8 of 10 results, left out 2. To see more, use --max-chars with a number larger than 1000, or --no-max-chars.` Standard output holds 8 results. The unit tests of `search_exit_code` stay. |
| CRITERION-104 The size-limit notice is named fields in structured output | `tests/size_limit.rs`: the ranked folder with `--json --max-chars 1000`. Standard output is one JSON document of 823 characters, at most 1,000. Its results are `r00` and `r01`. `left_out` is 28, `top_result_cut` is 0, `warnings_left_out` is 0, and `size_limit_fix` is exactly `To see more, use --max-chars with a number larger than 1000, or --no-max-chars.` The `exit_code` field is 8, and jag exits 8. The error stream is empty, so it holds no size-limit notice. The ranked folder with `--json` and no setting: 30 results (6,812 characters), `left_out` 0, `top_result_cut` 0, `size_limit_fix` `""`, `warnings_left_out` 0, and exit 0. The tests of the connection (see "Structured mode") also prove this criterion. |
| CRITERION-43 A size limit that is not valid is a usage error | `tests/usage.rs`: `--max-chars` with `0`, `999`, `0999`, `abc`, `1.5`, `1e4`, `+1000`, `20,000`, `off`, and a blank text exits 2 with the option text of the table. `--max-chars=-1` exits 2 with the same text. `--max-chars -1` exits 2. `JAG_MAX_CHARS` with each of these values exits 2 with the variable text. The server gets no request. Unit tests of `parse_size_limit`. |

More tests:

- `char_count` (unit tests): `""` gives 0, `"abc"` gives 3, `"é"` (2 bytes)
  gives 1, `b"a\xffb"` gives 3, `b"\xe2\x82"` gives 1, and
  `b"\xe2\x82\n"` gives 2. The count of a joined text of lines that end
  with `\n` is the sum of the counts of the lines.
- `write_text` with a limit (unit tests, with limits below 1,000; the
  minimum is a rule of the setting, not of `write_text`):
  - For each limit from 1 to 2,000, for lists with the label, with cut
    lines, with bad bytes, and with 2-byte characters: the output has at
    most `limit` characters, and the report is right: `shown + left_out`
    is N.
  - With no limit, and with a limit equal to the full count, the output is
    the same as the output of FEATURE-08, and the report has `left_out` 0.
  - The `--\n` lines and the marks of FEATURE-08 count.
- The cut (unit tests): Example 1 and Example 2 give their outputs and N.
  A matching line of 300 characters and a room for half of it: the line
  ends inside its text, at a character, with the mark. A last line with
  its own mark: the mark of the cut takes its place, and N holds its cut.
  A room that is too small for the path and the mark: no output, `shown`
  0, `left_out` N. A result that starts with its matching line: step 1
  leaves out no line.
- `size_notice` (unit tests): each form of "The notice", and `None` for a
  report with `left_out` 0 and `top_result_cut` 0.
- Limit values (unit tests of `parse_size_limit`): `1000`, `20000`,
  `01000`, ` 40000 `, and `99999999999999999999999` (gives `usize::MAX`)
  pass. Each fail value of "The limit setting" fails. Each case runs for
  both sources.
- The option wins (unit tests of `size_limit`): `Chars` and a variable,
  `Off` and a bad variable, a variable only, a blank variable, and
  neither.
- Limit secrecy (`tests/usage.rs`): `--max-chars SENTINEL-SENTINEL-SENTINEL`
  exits 2, and `JAG_MAX_CHARS=SENTINEL-SENTINEL-SENTINEL` exits 2. The error
  stream does not hold `SENTINEL`.
- Order: `--max-chars abc` with no key exits 2 (decision 9).
  `JAG_MAX_CHARS=abc` with no key exits 3, because the key comes first.
  With no `--ask`, `JAG_MAX_CHARS=abc` with no key exits 2, because plain
  grep reads no key.
- Conflicts: `--max-chars 5000 --no-max-chars`, `--check --max-chars 5000`,
  `--check --no-max-chars`, `--setup --max-chars 5000`, and
  `--forget-key --no-max-chars` exit 2 with the fixed line and no standard
  output.
- `jag --help` shows `--max-chars <COUNT>` and `--no-max-chars`.
- Plain grep with the limit: the ranked folder with no `--ask` and
  `--max-chars 1000`. Each line takes 113 characters, with no `--`, so
  standard output holds the 8 lines of `r00` to `r07` (904 characters).
  The error stream holds `jag: size limit reached: showed 8 of 30 results,
  left out 22. To see more, use --max-chars with a number larger than
  1000, or --no-max-chars.`, and jag exits 8. The server gets no request.
  With `--json`: the document has at most 1,000 characters and parses,
  `left_out` is N - K with K the largest count that fits, and jag exits 8.
- A model failure with the limit: the ranked folder with
  `--max-chars 1000`, and the server gives 500 to each call. jag exits 6,
  standard output is empty, and the error stream holds no size notice.

#### Changed tests

- `tests/usage.rs` `help_and_version_exit_0` checks the help. It also
  checks `--max-chars <COUNT>` and `--no-max-chars`.
- The unit tests of `write_lines` in `src/output.rs` call it with the
  limit `None`. They keep their texts.
- The unit tests of `write_text` in `src/output.rs` call it with the limit
  `None`. They keep their texts. They read `TextReport.no_good_answer` in
  place of the `bool`.
- Each end-to-end test that writes more than 20,000 characters now exits 8.
  Run the full suite. Such a test gets `--no-max-chars`, unless it is about
  the limit. The tests of `tests/cost_limit.rs` write fewer than 20,000
  characters: 200 results of about 80 characters or less, also with the
  long temporary paths of macOS.

### Other features

- FEATURE-12 (code on `main`, PR #160; spec PR #151;
  0018-take-the-question-only-from-the-ask-option.md): the command form is
  `jag [OPTIONS] TERM [PATH...]`, and the question comes only from
  `--ask QUESTION`. With no `--ask`, the search is plain grep: `plain` in
  `src/cli.rs`, with no key and no model call. The FEATURE-12 code merged
  first, so the FEATURE-09 code adapts, as the FEATURE-12 spec says in its
  "Other features":
  - Each end-to-end test of a question search gives `--ask`.
  - The default limit applies to plain grep too (DECISION-20). `plain`
    has no step 3, so it reads the limit at its start, with
    `size_limit(env, args.size_limit)`. It reads no key, so a bad
    `JAG_MAX_CHARS` gives exit 2 with no key.
  - `write_lines` gets the last parameter `limit: Option<usize>` and gives
    a `SizeReport`, as `write_text`. Each line is one result, with no `--`
    and no label line. The fit, the cut of the first line, and the notice
    are the same as for `write_text`. The exit code is
    `search_exit_code(SearchOutcome { no_good_answer: false,
    cost_limit_reached: false, left_out })`: 8 when the limit leaves out a
    line or cuts the first line, else 0.
  - `write_json` takes `results: &[&Snippet]`. The fit of "Structured
    mode" calls it with the first K results, in both flows.
- FEATURE-15 (done, code on `main`): text mode can have color. The color
  codes are characters on standard output, so they count toward the limit
  (DECISION-22). The fit and the cut count the colored text that
  `write_text` gives, with its codes. The size notice goes through
  `stop.notice`, after the progress line goes away. Source: the FEATURE-15
  spec, decisions 3 and 14, and its "Other features".

## Build order

0. The FEATURE-08 code (PR #129), the FEATURE-06 code (PR #124), the
   FEATURE-02 code (PR #132), the FEATURE-11 code (PR #139), and the
   FEATURE-12 code (PR #160) are on `main`. This spec uses `cut_line`,
   `write_text`, `SettingSource`, `SearchOutcome`, `search_exit_code`, the
   `--check` flag, `Summary`, `write_json`, `plain`, and `write_lines`.
1. `char_count` in `src/search/snippet.rs`, with its unit tests.
2. STORY-28: `SizeLimit`, `parse_size_limit`, `size_limit`, and
   `SizeLimitBad`, with their unit tests. Then `--max-chars`,
   `--no-max-chars`, the help, and the steps 1 and 3 of the search flow,
   with the tests of CRITERION-43 in `tests/usage.rs`.
3. STORY-25 and STORY-26: `SizeReport`, `TextReport`, `fitting_count`,
   `write_text` with the limit, `size_notice`, and step 14 of the search
   flow, with their unit tests and the end-to-end tests of CRITERION-36 to
   CRITERION-39, CRITERION-41, CRITERION-42, and CRITERION-66. Then the
   changed tests.
4. STORY-27: the cut of the top result, with its unit tests and the
   end-to-end test of CRITERION-40.
5. Structured mode: the steps of "Structured mode", the known limit of
   `run`, and the 3 writes of a document, with the end-to-end test of
   CRITERION-104 and the tests of the connection.
6. Plain grep: the limit in `plain` and in `write_lines` (see "Other
   features"), with the test of plain grep in "More tests".

## Assumptions

- The names `--max-chars`, `--no-max-chars`, and `JAG_MAX_CHARS` (decisions
  4 and 5). Source: FEATURE-06 spec decision 1, the human answer Q-aw. The
  earlier prototype used `--max-output` and had no off form.
- The structured document of FEATURE-11 with no result, no warning, and
  no error fits in 1,000 characters. Source: the FEATURE-11 spec (PR #136,
  decision 27) gives at most 403 characters, with each count at 20
  digits. The 3 size-limit fields other than `left_out` add at most 194
  characters (`top_result_cut`: 38; `size_limit_fix` with a limit of 20
  digits: 115; `warnings_left_out`: 41). So the bound is 597. The FEATURE-11
  unit test `the_largest_document_with_no_result_is_less_than_1000_characters`
  checks it on `main`.
- The fake server can give each snippet its own score from its text.
  Source: `tests/common/fake_server.rs` (`Reply::scores`).

## Risks

- With the default of 20,000 characters, many searches with many long
  results exit 8. Signal: issues that ask why jag exits 8, or agents that
  read each code above 0 as a failure. Mitigation: the notice says how to
  see more. A later change of the default is a measured change with
  `jag-bench` (METRIC-02 and METRIC-06).
- A cut with no result left out gives exit 8 (decision 17), and the notice
  says `left out 0`. A script that reads exit 8 only as "results left out"
  does not see that the cause is the cut. Signal: a report of exit 8 with
  `left out 0`. Mitigation: the notice says `and cut the top result`, and
  structured mode gives `top_result_cut`.
- The mark of a cut top result is at the end of the result, also when
  step 1 left out lines before the match. Signal: a reader who thinks that
  the text after the mark is the only missing text. Mitigation: the line
  numbers show the missing lines, and N counts all missing text that no
  other mark shows.
- A write of a document in the search flow that does not use the steps of
  "Structured mode" can be larger than the limit. Signal: the test of
  CRITERION-104 or a test of the connection fails. Mitigation: one function
  of `src/output.rs` fits each document of a search.
- The FEATURE-11 document with no result can grow larger than 1,000
  characters, for example with a new field. Signal: the FEATURE-11 unit
  test `the_largest_document_with_no_result_is_less_than_1000_characters`
  fails. Mitigation: that PR raises `MIN_SIZE_LIMIT` and the message texts,
  before the first release.
- A very long path leaves no room for one line of the top result (decision
  14). Signal: the notice says `showed 0 of`. Mitigation: a larger limit.

## Out of scope

- The JSON document, its fields, its writer, and the JSON form of each
  error: FEATURE-11 (PR #139). This spec sets only the values of the 4
  size-limit fields. "Structured mode" gives the link.
- Plain grep with no question: FEATURE-12 (on `main`). This spec gives it
  only the limit (DECISION-20). See "Other features".
- Piped text: FEATURE-14.
- The cost line and the cost field: FEATURE-20. In text mode the line goes
  to the error stream (DECISION-41), so it does not count. The field is
  part of the document, so it counts (DECISION-22).
- A key in the config file for the limit: DF-1.
- The measure of METRIC-06 with `jag-bench`: C-bench, which no feature
  plans yet.
- A size limit on the lines of `--check` (FEATURE-02). `--check` writes no
  result, so the two size options conflict with it (decision 10).
