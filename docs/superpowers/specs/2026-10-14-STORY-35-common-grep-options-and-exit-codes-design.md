---
title: "Common grep options and exit codes: the options of the matcher, the lines around a match, the file list, the count, the quiet mode, the file globs, the error of an unknown option, and the exit codes"
feature: FEATURE-13
stories: [STORY-35, STORY-36, STORY-37, STORY-38]
kind: build
gh_issue: 96
lifecycle_state: draft
inherits: [IF-3, IF-4, IF-5, IF-6, C-cli, C-search, C-output, CV-1, CV-2, CV-3, CV-6]
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Common grep options and exit codes: the options of the matcher, the lines around a match, the file list, the count, the quiet mode, the file globs, the error of an unknown option, and the exit codes

## Problem

jag must feel like grep. A shell user keeps their habits: the options and
the exit codes of grep (CAPABILITY-06).

- STORY-35: a shell power user wants jag to accept the grep options that
  they use most, with their grep meaning, and `-F` to read the term as fixed
  text, so that they keep their habits (DECISION-33, DECISION-34).
  - CRITERION-55: jag accepts `-i`, `-w`, `-F`, `-e`, `-A`, `-B`, `-C`,
    `-l`, `-c`, `-q`, `--include`, and `--exclude`, with their grep meaning.
  - CRITERION-56: with `-F`, jag reads the term as fixed text and not as a
    regular expression.
- STORY-36: a shell power user wants `-A`, `-B`, and `-C` to set the lines
  around each match (DECISION-50).
  - CRITERION-69: `-A`, `-B`, and `-C` set the lines around each match in a
    question search. With none of them, a result has the default context.
- STORY-37: a shell power user wants a usage error for an unknown option
  (DECISION-34).
  - CRITERION-57: any other option gives a usage error that names the
    option, and jag exits with code 2.
- STORY-38: a shell power user wants the grep exit codes 0, 1, and 2, so
  that their scripts that test grep exit codes keep working (DECISION-05).
  - CRITERION-58: jag exits with code 0 when it returns a result, code 1
    when it finds no match, and code 2 for a usage error, as grep does.

Today jag takes no grep option. The term is a regular expression, and it is
case-sensitive (FEATURE-04 spec decision 15). A result has 3 lines before
and 3 lines after the match, and at most 12 lines (FEATURE-04 spec decision
4). Each option that jag does not know gives one fixed line, `jag: the
command line is not valid. Run jag --help to see the usage.`, and exit 2.
That line names no option, because an argument error never repeats a value
from the command line: the value can be a key (FEATURE-04 spec decision 26,
which cites system-design.md CV-6). So CRITERION-57 and that rule disagreed.
On 2026-10-14 the user chose that jag names the option and never a value
(decision 18). This pull request adds that exception to FEATURE-04 spec
decision 26.

The question comes only from the option `--ask QUESTION` (FEATURE-12 spec
decision 1, `0018-take-the-question-only-from-the-ask-option.md`). In this
spec, a question search is a search with `--ask`, and plain grep is a
search with no `--ask`. The user answers of 2026-10-14 for this feature say
"with a question" for "with `--ask`".

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Spec grouping | One spec for STORY-35 to STORY-38. The four stories change the same command line and the same search flow | docs/adlc.md (one feature gives one spec) |
| 2 | Names of the options | The 12 options of DECISION-34, with the short names of grep. Each one also has its long name of GNU grep: `--ignore-case`, `--word-regexp`, `--fixed-strings`, `--regexp=TERM`, `--after-context=NUM`, `--before-context=NUM`, `--context=NUM`, `--files-with-matches`, `--count`, and `--quiet` with the alias `--silent`. `--include=GLOB` and `--exclude=GLOB` have only a long name, as in grep. A short option takes its value as the next word or joined (`-A2`), and short options join (`-iw`, `-iA2`), as in grep. A long option takes its value after `=` or as the next word, as in grep. Decision 37 gives a short option with `=`, for example `-e=foo` | DECISION-34; CRITERION-55 ("with their grep meaning"); FEATURE-15 spec decision 5 (`--colour`, an alias from the help of grep); GNU grep 3.12 (`grep --help`, checked on 2026-10-14); clap 4.6.7 (checked in Docker on 2026-10-14: `-iA2`, `-e-x`, and `--regexp=a` parse) (inferred) |
| 3 | The options of jag | "Any other option" (CRITERION-57) is each option that is not in DECISION-34 and that is not an option of jag from another decision: `--ask` (`0018-take-the-question-only-from-the-ask-option.md`), `--json` (DECISION-27), `--color` and `--colour` (DECISION-38), `--timeout` and `--retry-wait` (FEATURE-03 spec), `--max-scored` (FEATURE-06 spec), `--check`, `--setup`, and `--forget-key` (contracts-2), `--help` and `-h`, `--version` and `-V`, and `--max-chars` and `--no-max-chars` when the FEATURE-09 code merges | DECISION-27; DECISION-38; DECISION-34; system-design.md IF-3 and contracts-2; FEATURE-15 spec "Other features" ("The usage error for an unknown option must accept `--color` and `--colour`"); FEATURE-09 spec decisions 4 and 5 (PR #134, not merged) |
| 4 | `-h` and `-V` | `-h` and `--help` show the help, and `-V` and `--version` show the version, as today and as rg. In grep, `-h` removes the path from each line, and DECISION-34 does not keep that option (Risk 2) | `src/cli.rs` (clap gives `-h` and `-V`); FEATURE-12 spec "Out of scope" ("`-h`: DECISION-34 does not keep them"); ripgrep (`rg -h` shows the help) (inferred) |
| 5 | Grep options that jag does not keep | Each other option of grep is an unknown option (decision 18), for example `-r`, `-n`, `-H`, `-s`, `-v`, `-x`, `-o`, `-m`, `-L`, `-Z`, `--exclude-dir`, and `-NUM`. jag always searches a folder as `grep -r` does, and always shows the path and the line number, as `grep -H -n` does. So `jag -rn` gives exit 2 (Risk 1) | DECISION-34 ("Any other option is a usage error"; rejected: "jag ignores an unknown option with a warning", and "All GNU grep options"); FEATURE-05 spec; FEATURE-08 spec decisions 1 and 5 |
| 6 | The options with `--check`, `--setup`, and `--forget-key` | Each option of this spec conflicts with `--check`, `--setup`, and `--forget-key`. A conflict gives the fixed line and exit 2, or the argument error document with `--json` | FEATURE-15 spec decision 5 (`--color` conflicts with the same three options); FEATURE-11 spec decision 23; FEATURE-02 spec decision 18; `src/cli.rs` (an option that has no effect in a mode conflicts with that mode) |
| 7 | `-i` | `RegexMatcherBuilder::case_insensitive(true)`: upper case and lower case letters match as the same, with the Unicode rules of rg | DECISION-34; CRITERION-55; 0001-find-matches-with-the-ripgrep-libraries.md; grep-regex 0.1.14 docs (`case_insensitive`) |
| 8 | `-w` | `RegexMatcherBuilder::word(true)`: a match has no word character just before it and just after it. The grep-regex docs say that this is "subtly different than putting `\b` assertions on both sides": `-2` matches in `foo -2 bar`, as in grep | grep-regex 0.1.14 docs (`word`; context7 `/burntsushi/ripgrep` and docs.rs, 2026-10-14); GNU grep 3.12 (checked: `-w needle` matches `the needle.`, and not `needles`, `needle_x`, or `Needle`) |
| 9 | `-F` | `RegexMatcherBuilder::fixed_strings(true)`: each character of each term matches literally. `-i` and `-w` still apply | DECISION-33; CRITERION-56; grep-regex 0.1.14 docs (`fixed_strings`: "all characters ... are matched literally"); GNU grep 3.12 (checked: `-F 'needle.'`, `-w -F`, and `-i -F`) |
| 10 | `-e` | `-e TERM` gives a term. jag takes `-e` more than one time: a line matches when it matches one of the terms. `-e` takes a value that starts with `-`, as `-e -x` in grep. So `jag -e --ask q needle` has the term `--ask` and the paths `q` and `needle`, as `grep -e --foo` has the pattern `--foo`. With `-e`, no word of the command line is the term, so each word is a path, as each word is a file with `grep -e`. The question comes only from `--ask`, so no word is the question. `Cli.term` becomes `Option<OsString>`, because with `-e` it holds the first path, and a path can be a text that is not UTF-8. With no `-e`, a term that is not UTF-8 gives the failure of `argument_failure`, as clap gives today | GNU grep 3.12 (checked: `-e one -e three`, `-e -x`, and `-e needle a.txt b.txt` searches the two files); grep-regex 0.1.14 (`build_many`: "behaves as if all of the patterns given are joined together into a single alternation"); FEATURE-12 spec decision 1 and its "Other features" ("With `-e PATTERN`, the term is not a word of the command line, so each word is a path"); `0018-take-the-question-only-from-the-ask-option.md`; clap 4.6.7 (`allow_hyphen_values`; a copy of the `Cli` of this spec with `--ask`, run in `rust:1.99` on 2026-10-14: `-e needle a.txt b.txt` gives the term word `a.txt` and the path `b.txt`, `--ask q -e needle dir` and `-e needle --ask q dir` give the question `q` and the word `dir`, `-e --ask q needle` gives the term `--ask` and the words `q` and `needle`, and a word that is not UTF-8 after `-e` parses) (inferred) |
| 11 | Code of the matcher | One function, `build_matcher`, makes the matcher of both flows with `build_many` of the terms and the options of decisions 7 to 9. A term that is not valid gives the failure of today: `the term is not a valid regular expression.` and `Check the term. It uses the regular expression rules of ripgrep.`, exit 2. The `paint` function of FEATURE-15 uses the same matcher, so jag colors each match of each option | `src/cli.rs` (`RegexMatcherBuilder::new().line_terminator(Some(b'\n')).build(args.term)`); FEATURE-15 spec decision 8 and "Other features" ("`-i`, `-w`, `-F`, and `-e` change the matcher, and decision 8 colors with that same matcher") |
| 12 | Values of `-A`, `-B`, and `-C` | A whole number of 0 or more: ASCII digits only, after a trim of white space. A number that is too large for `u64` counts as `u64::MAX`, because grep takes a very large number. Each other value gives exit 2, also `-1`, `+1`, and an empty value. The cause is `the value of -A is not a whole number of 0 or more.` (with `-B` or `-C` for those options), and the fix is `Give a number of lines, for example 3.` The message never shows the value. It can show `-A`, because `-A` is a name of jag and not a text of the user (decision 18 is about a text of the user). grep takes `+1` and ` 1`. jag does not take `+1` | FEATURE-06 spec decision 3 and FEATURE-09 spec decision 7 (digits only, after a trim); FEATURE-04 spec decision 26 (no value in a message); GNU grep 3.12 (checked: `-A abc` and `-A ''` give exit 2, and `-A +1`, `-A ' 1'`, and `-C 99999999999999999999` work); clap 4.6.7 (checked in Docker: with `allow_hyphen_values`, `-A -1` gives the value `-1`) (inferred) |
| 13 | Which value applies | `-A` and `-B` win over `-C` for their side, in each order, as grep: `-C1 -A3` and `-A3 -C1` both give 1 line before and 3 lines after. When an option shows two times, the last one wins. With only `-A`, a match has 0 lines before it. With only `-B`, it has 0 lines after it | GNU grep 3.12 (checked: `-C1 -A3`, `-A3 -C1`, `-A1`, `-C 2 -A 1 -B 0`); clap 4.6.7 (checked in Docker: `-A 2 -A 5` gives 5) |
| 14 | Lines around a match in a question search | With none of the three options: 3 lines before, 3 lines after, and at most 12 lines in a result, as today. With one or more of them and `--ask`: the values of decision 13, from 0 to 12 for each side. A larger value gives exit 2, the cause `the value of -C is more than 12 with --ask.` (with the name of the option that gave the value), and the fix `Give a number from 0 to 12.` Windows that touch or overlap join, as today. A result has at most max(12, B + 1 + A) lines, so at most 25 lines. The model reads all the lines of a result | human:404pilo, 2026-10-14 (hitl answer FEATURE-13 "Lines around a match in a search with a question": the recommended option, "with a question" means "with `--ask`"); DECISION-50; DECISION-10; CRITERION-69; FEATURE-04 spec decisions 4, 21, and 22 ("One snippet always fits") |
| 15 | Lines around a match in plain grep | Each value of decision 12 works. A match and the lines around it form one group. Groups that touch or overlap join, with no limit of lines, as in grep. jag writes `--` between two groups, also when the groups are in two files, and also with `-C 0`, as grep. A line that does not match has `-` after the path and after the line number, and a match has `:`, as `write_text` writes them today and as grep. With none of the three options, plain grep stays as FEATURE-12 spec decision 6: one line for each match, and no `--` | DECISION-50 ("as in grep"); CRITERION-55; FEATURE-12 spec decisions 6 and 7 and its "Other features"; `src/output.rs` (`write_text`: `:` for a match, `-` for each other line); GNU grep 3.12 (checked: `-C1`, `-A0`, `-C0`, and `-C1` on two files) |
| 16 | Plain grep with lines around a match, and `--json` | One result for each group of decision 15, with all its lines and the mark `is_match` on each line, as a result of a question search. `results_found` is the count of groups. jag adds no field | FEATURE-12 spec decision 14 (one line for each result when no option gives lines around a match); FEATURE-11 spec (each result has its lines); system-design.md IF-5; CONSTRAINT-05 (inferred) |
| 17 | Code of the windows | `plan_windows(files, context)` gets `ContextLines { before, after, max_lines: Option<u64> }`. `ContextLines::DEFAULT` is 3, 3, and `Some(12)`, so the windows of today do not change. A question search with an option gives `Some(max(12, before + 1 + after))`. Plain grep with an option gives `None`, so groups that touch or overlap always join, and two groups never touch. All sums saturate, because a value can be `u64::MAX` | `src/search/snippet.rs` (`plan_windows`, `CONTEXT_LINES`, `MAX_LINES`); FEATURE-12 spec decision 9 (`line_windows` for plain grep with no option) (inferred) |
| 18 | The error of an unknown option | jag names the option, and never a value. For the clap error `UnknownArgument`, jag reads the text of `ContextKind::InvalidArg`. clap gives only the text before `=` there, and for a group of short options only the one unknown letter. When that text has the form of an option, the cause is `the option NAME is not valid.` and the fix is `Run jag --help to see the options.`, exit 2. The form of an option: `-` and one ASCII letter or digit, or `--` and 1 to 30 ASCII letters, digits, or `-`. The limit of 30 is above the longest long option of GNU grep 3.12 (21 characters after `--`: `--dereference-recursive`) and of jag (18: `--files-with-matches`). A longer text, for example a key, gives the fixed line. Each other text, and each other argument error, gives the fixed line of today. With `--json`, the error document holds the same cause and fix. This pull request adds this exception to FEATURE-04 spec decision 26 | human:404pilo, 2026-10-14 (hitl answer FEATURE-13 "The name of an unknown option in the error": the recommended option). The card showed a conflict: DECISION-34 ("jag names the option and exits with code 2") and CRITERION-57 against FEATURE-04 spec decision 26 (an argument error never repeats a value from the command line, because the value can be a key). FEATURE-04 spec decision 26 cites system-design.md CV-6, but CV-6 says only that no option takes the key. The test `an_argument_error_writes_one_fixed_line_with_no_value` in `tests/usage.rs` checks today that `--SENTINEL-SENTINEL-SENTINEL` gives the fixed line. GNU grep 3.12 writes `unrecognized option '--nope=SECRET'` (checked), and its longest long option has 21 characters after `--` (`grep --help`, checked on 2026-10-14). clap 4.6.7 gives `--nope` for `--nope=SECRET`, `-Z` for `-iZx`, and the full text for `--sk-0123456789abcdef0123456789abcdef` (checked in Docker on 2026-10-14; the first two again with `--ask` in the `Cli`) |
| 19 | `-l` | In plain grep, jag writes the path of each file with a match, one path on each line, in the order of the scan. A file that the user names two times shows two times, as in grep. The options of decisions 12 to 15 have no effect. With `-c`, `-l` wins, as in grep | GNU grep 3.12 (checked: `-l`, `-l -c`, `-l -C2`, and `-l needle a.txt a.txt`); FEATURE-14 spec decision 6 (PR #142: a file named two times gives its results two times) |
| 20 | `-c` | In plain grep, jag writes `path:N` for each file that it searched, also for a file with no match (`path:0`), in the order of the scan. N is the count of matching lines. The path is always there, also for one file, because jag always writes the path, as `grep -H` (grep with one file and no `-H` writes only N). The options of decisions 12 to 15 have no effect. Piped text with no byte gives `(standard input):0` and exit 1, as `grep -H -c`, because jag searches the empty text (FEATURE-14 spec decision 3) and this decision writes `path:0` for it | GNU grep 3.12 (checked: `-c needle a.txt b.txt` gives `a.txt:2` and `b.txt:0`; `-H -c` gives `(standard input):1`, and `(standard input):0` with exit 1 for an empty pipe; `-c -C2` gives the count only; `-c` counts lines, not matches); FEATURE-08 spec decisions 1 and 5; FEATURE-12 spec decision 7 (the path on each line, as `grep -H`); human:404pilo, 2026-10-14 (hitl answer FEATURE-14 "Pipe sends no text": jag searches the empty text, no match, exit 1) |
| 21 | `-q` | jag writes nothing on standard output. It exits with 0 when a line matches, and with 1 when no line matches. A line that matches gives 0 also when a path does not exist, as `grep -q`. This is the one exception to the rule of FEATURE-14 spec decision 7 (a path that does not exist gives exit 2 at the end). jag still writes the warning of the missing path. With no match and a missing path, the code is 2, as in grep. The skip notice and the warnings still go to the error stream, as in grep (grep `-q` still writes its errors; `-s` is not kept). With `-l` or `-c`, `-q` wins, as in grep. jag scans all the paths, and then exits. So it writes the warning of each missing path, also after a match (grep stops at the first match) | The orchestrator DECISION of 2026-10-14 (no ASK: the sources decide it, and the user can reverse it); human:404pilo, 2026-10-14 (hitl answer FEATURE-14 "One path does not exist": "as grep does"); CRITERION-55 ("with their grep meaning"); STORY-38 ("so that their scripts that test grep exit codes keep working"); GNU grep 3.12 (`man grep`, `-q`: "Exit immediately with zero status if any match is found, even if an error was detected"; checked on 2026-10-14: `-q needle missing.txt a.txt` writes the error and exits with 0, `-q needle a.txt missing.txt` writes no error and exits with 0, `-q zzz missing.txt a.txt` exits with 2, and `-q -l` and `-q -c`); CRITERION-85 (in text mode, notices go to the error stream); FEATURE-16 spec decision 15 (PR #155: no early stop; a scan of the 14,490 files of Go 1.25.0 takes less than 0.1 s) (inferred) |
| 22 | `-l`, `-c`, and `-q` with `--ask` or with `--json` | The three options work only in plain grep (no `--ask`) with no `--json`. With `--json`: the cause `the option -l does not work with --json.` and the fix `Remove --json, or remove -l.` With `--ask`: the cause `the option -l does not work with --ask.` and the fix `Remove --ask, or remove -l.` Both give exit 2, and jag checks `--json` first. The message names the option that wins: `-q`, then `-l`, then `-c`. `-q` with `--json` must be a usage error, because structured mode always writes one full document | human:404pilo, 2026-10-14 (hitl answer FEATURE-13 "File list, count, and quiet options with a question": the recommended option, "with a question" means "with `--ask`"); DECISION-30 (structured mode always writes a complete output); FEATURE-11 spec decision 23 (an option gives its structured form, or it refuses `--json`); DECISION-32 (plain grep makes no model call) |
| 23 | Binary files with `-l` and `-c` | grep lists and counts the binary files that match. jag skips each binary file (DECISION-14), so `-l` and `-c` do not show them, and the skip notice counts them, as today | DECISION-14; FEATURE-05 spec decisions 2 to 8; GNU grep 3.12 (checked: `-c needle bin.dat a.txt` gives `bin.dat:1`) (inferred) |
| 24 | Code of `-l`, `-c`, and `-q` | `ScanOptions` gets `keep_files_with_no_match: bool`. When it is true, `search_file` keeps a `FileMatches` with no line for each file that it searched with no match and no binary data. Only `-c` sets it. `plan_windows`, `line_windows`, and `match_texts` give no window for a file with no line. The three options read no file after the scan. C-output gets `write_files` and `write_counts` | `src/search/walk.rs` (`search_file` keeps only a file with a match); `src/search/snippet.rs` (`plan_windows` loops on the lines of each file) (inferred) |
| 25 | The rule of `--include` and `--exclude` | The rule of GNU grep. Each option gives one glob. For each file, the last glob of the command line that matches the name of the file decides: `--include` searches the file, and `--exclude` skips it. When no glob matches, jag searches the file, unless the first glob is an `--include` glob. The globs apply to each file of a walk and to each file that the user names, as in grep. They do not apply to a folder (grep has `--exclude-dir` for that, and DECISION-34 does not keep it), and they do not apply to piped text. This is not the rule of FEATURE-14 spec decision 6 (jag searches a hidden or ignored file that the user names): the globs are a choice of the user, and the skip rules of jag do not change | GNU grep manual ("If contradictory --include and --exclude options are given, the last matching one wins. If no --include or --exclude options match, a file is included unless the first such option is --include"); GNU grep 3.12 (checked: `--exclude='*.md' --include='*.rs'` searches `a.txt`, `--include='*.rs' --exclude='c.*'` skips `c.rs`, `--include='*.rs' needle a.txt` skips `a.txt`, `--exclude=sub` does not skip the folder `sub`, and piped text is searched with `--exclude='*'`) |
| 26 | Glob syntax | A glob matches the name of the file: the last part of its path. jag uses `globset` with its default rules, the glob rules of rg: `*`, `?`, `[a-z]`, `[!a-z]`, `{a,b}`, and `**`. A glob is case-sensitive, as in grep. grep uses `fnmatch`, so two cases are not the same: `*.{rs,md}` and `**/*.rs` match in jag and not in grep. A glob that does not parse gives exit 2, the cause `the glob of --include is not valid.` (or `--exclude`), and the fix `Check the glob. jag uses the glob rules of ripgrep.` The message never shows the glob. grep takes a glob that does not parse and matches no file with it | globset 0.4.20 docs (context7 `/burntsushi/globset`, 2026-10-14: `Glob::new`, `compile_matcher`, `is_match`, the syntax); GNU grep 3.12 (checked: `--include='*.{rs,md}'`, `--include='**/*.rs'`, and `--include='['` match no file, and `--include='*.RS'` does not match `c.rs`); FEATURE-04 spec decision 26 (no value in a message) (inferred) |
| 27 | Code of the globs | `FileFilter` in the new file `src/search/filter.rs` holds the globs in the order of the command line. clap gives the values of `--include` and of `--exclude` in two lists, so `parse` reads the place of each value with `ArgMatches::indices_of` and joins the two lists in that order. `ScanOptions` gets `files: FileFilter`, and it loses `Copy`. `visit_entry` asks the filter before `search_file`. So a file that a glob skips is not in the skip notice: the walk gave it, and the user chose to skip it | clap 4.6.7 (checked in Docker: `indices_of` gives the places 2 and 6 of two `--exclude` values and the place 4 of an `--include` value); `src/search/walk.rs` (`visit_entry`, and the skip count of the entries that the walk gives); ignore 0.4.33 (an override glob of `--glob` "always overrides any other ignore logic", so it can search a hidden or ignored file) (inferred) |
| 28 | Color of `-l`, `-c`, and the groups | The colors of FEATURE-15 spec decision 7, byte for byte as GNU grep: the path in `35`, the `:` of `-c` in `36`, and the count with no color. The `--` between groups is in `36`, as today | FEATURE-15 spec decision 7 and "Out of scope" ("The color of the output of `-l` and `-c`: the feature that adds them"); GNU grep 3.12 (`--color=always -l` and `--color=always -c`, checked with `od -c` on 2026-10-14) (inferred) |
| 29 | Exit codes | 0 when a line matches, 1 when no line matches, and 2 for a usage error, in each flow and with each option. So `-c` with no match writes the `path:0` lines and exits with 1, as grep. A question search keeps its codes: 1 also for no good answer, and the codes 3 to 8 and 130 of IF-4. A path that does not exist gives a warning and exit 2 at the end, after the output of the other paths, also with `-l` and `-c`, as grep (FEATURE-14 spec decision 7). `-q` with a match is the one exception: exit 0 (decision 21). A warning for a file that jag cannot read does not change the code (FEATURE-14 spec decision 10): grep gives 2 there | DECISION-05; CRITERION-58; system-design.md IF-4; FEATURE-05 spec decision 6 (Q-g: the walk goes on, as `grep -r`); human:404pilo, 2026-10-14 (hitl answer FEATURE-14 "One path does not exist": search the other paths, as grep does, and exit 2 at the end); FEATURE-14 spec decisions 7 and 10 (PR #142, not merged); GNU grep 3.12 (checked: `-c zzz a.txt b.txt` and `-l zzz a.txt` exit with 1; `-l needle missing.txt a.txt` writes `a.txt` and exits with 2; `-c needle missing.txt a.txt b.txt` writes `a.txt:2` and `b.txt:0` and exits with 2) |
| 30 | Order of the checks of step 1 | The term (or `-e`), the paths, and the checks of the question of `--ask`, as FEATURE-12 spec decision 20 gives them. Then `-l`, `-c`, and `-q` with `--json` or with `--ask` (decision 22), the values of `-A`, `-B`, and `-C` (decisions 12 and 14), and the globs (decision 26). Then `--timeout`, `--retry-wait`, and `--max-scored`, as today. All come before the key. No message shows a value | `src/cli.rs` (`search_args`: "The time options come after the other checks and before the key"); FEATURE-04 spec decisions 18 and 26; FEATURE-12 spec decision 20 |
| 31 | Help and usage | The two usage lines of FEATURE-12 spec decision 22 stay: `jag [OPTIONS] TERM [PATH...]` and `jag [OPTIONS] --ask QUESTION TERM [PATH...]`. One new line comes after them: `jag [OPTIONS] -e TERM ... [PATH...]`, the form of `man grep` for `-e`. `--ask` is one of the OPTIONS of that line. Then the three lines of `--check`, `--setup`, and `--forget-key` of today. Each option gets one line of help (see "The command line"). The README gets the section "Grep options" | `0018-take-the-question-only-from-the-ask-option.md` (its Consequences: "The help, the README, and the usage line show `jag [OPTIONS] TERM [PATH...]` and `--ask QUESTION`"); FEATURE-12 spec decision 22; `src/cli.rs` (`override_usage`); GNU grep 3.12 (`man grep`: `grep [OPTION]... -e PATTERNS ... [FILE]...`, checked on 2026-10-14); FEATURE-11 and FEATURE-15 specs (a README section for each feature) (inferred) |
| 32 | Code place and crates | C-cli (`src/cli.rs`): the options, the parse, the checks, `build_matcher`, the flows of `-l`, `-c`, and `-q`, the error of an unknown option, and the help. C-search: `ContextLines` and `plan_windows` (`src/search/snippet.rs`), `FileFilter` (`src/search/filter.rs`), and the two fields of `ScanOptions` (`src/search/walk.rs`). C-output (`src/output.rs`): `write_files`, `write_counts`, and the `--` of `write_lines`. One new direct crate: `globset = "0.4.20"`. It is in the build already, through `ignore` 0.4.33, and it has the same author as `ignore` and `grep-regex`. The reason for the pull request (ADR-0012): the rule of decision 25 needs one matcher for each glob, in order, and the override rules of `ignore` are not that rule (an `ignore` override with one `--include` glob skips each file that matches no glob, also after an `--exclude` glob that comes first) | 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md; 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md ("Each new outside crate needs a reason in the pull request"); 0001-find-matches-with-the-ripgrep-libraries.md; `Cargo.lock` (globset 0.4.20 through ignore) |
| 33 | ADR | No ADR. Each choice adds an option, a value, or a message, and a later version can keep each one. The long names are aliases of the options of DECISION-34, as `--colour` is an alias of `--color`. The user answers of 2026-10-14 (decisions 14, 18, and 22) give usage errors, and a later version can widen each one with no break: a usage error can become a meaning. The JSON document gets no new field. The option `--ask` has its ADR already | docs/adr/CLAUDE.md ("If it is cheap to change your mind, do not write an ADR"); system-design.md IF-3 ("A removed or changed option needs a new major version") and IF-5; `0018-take-the-question-only-from-the-ask-option.md`; FEATURE-12 spec decision 24 |
| 34 | Story state | All four stories stay `backlog`. The user answers of 2026-10-14 close the three ASKs of decisions 14, 18, and 22, so no open ASK blocks a story. FEATURE-13 has FEATURE-12 in its `blocked_by` list, because `-l`, `-c`, `-q`, and the lines around a match in plain grep need the plain flow of FEATURE-12 (Build order step 5). So CRITERION-55 cannot pass before the FEATURE-12 code is on `main`. By rule 11, no story of FEATURE-13 is `ready` before FEATURE-12 is `done` | docs/adlc.md rule 11; the orchestrator ANSWER of 2026-10-14 ("STORY-35–38 stay backlog because FEATURE-13 is blocked_by FEATURE-12 (rule 11)"); verifier-1, pre-check of PR #153 (the `blocked_by` entry) |
| 35 | Other features | FEATURE-12 (the plain flow and the option `--ask`), FEATURE-14 (the order of the scan, piped text, an empty pipe, and a path that does not exist), FEATURE-09 (the size limit and its open point: exit 8 or 0), FEATURE-15 (color), FEATURE-11 (structured mode), FEATURE-16 (speed), and FEATURE-20 (the cost line): see "Other features". The user answered the open points of FEATURE-12 and FEATURE-14 on 2026-10-14, and this spec applies the answers to its options. It decides no open point of FEATURE-09 | The orchestrator TASK of 2026-10-14 ("Do not decide the open points of FEATURE-14 (empty pipe, missing path), FEATURE-12 (question or path) or FEATURE-09 (exit 8 or 0) — name the interaction"); the user answers of 2026-10-14 for FEATURE-12 and FEATURE-14; FEATURE-12 spec; FEATURE-14 spec (PR #142); FEATURE-09 spec (PR #134) |
| 36 | Tests | The new file `tests/grep_options.rs` holds the end-to-end tests of the five criteria. Unit tests go in `src/cli.rs`, `src/search/snippet.rs`, `src/search/filter.rs`, `src/search/walk.rs`, and `src/output.rs`. Three tests of `tests/usage.rs` change (decision 18). The expected bytes of grep in the tests come from GNU grep 3.12, checked on 2026-10-14. No test runs the `grep` program | FEATURE-11 and FEATURE-15 specs (one test file for each feature, the fake server, a temporary `$HOME`); `tests/common/mod.rs` (`jag_bare`, `jag`, `write`); `tests/usage.rs` (inferred) |
| 37 | A short option with `=` | A short option that takes a value reads one `=` just after its letter as the separator, as rg does. `-e=foo` gives the term `foo`, the same as `--regexp=foo`. `-ie=FOO` gives `-i` and the term `FOO`. `-e==foo` gives the term `=foo`. `-e=` gives an empty term, the same as `-e ''`, so each line matches. `-A=2`, `-B=2`, and `-C=2` give 2. A word that is the value of the option before it does not change: `-e -e=foo` gives the term `-e=foo`, as in grep. GNU grep reads the `=` as a part of the value: `-e=foo` gives the pattern `=foo`, and `-A=2` gives exit 2. jag does not copy this. Each option keeps its grep meaning. Only the form of the separator is not the same. clap gives this form today, so the code does not change. New tests record it, and the README tells the difference | PR #170 review (verifier-2, round 1, note 2: "The spec does not name the `=` form of a short option", and "A test that records the current behavior is enough for now"); `0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md` (clap reads the command line); decision 26 and "Out of scope" (the prefixes of a long option): jag does not copy each rare form of grep, and the README tells the difference; ripgrep 15.1.0 (checked on 2026-10-14: `rg -e=foo` gives `foo`, `rg -e=` matches each line, and `rg -A=2` gives 2); GNU grep 3.12 (`/usr/bin/grep`, checked on 2026-10-14: `-e=foo` matches only `=foo`, `-ie=FOO` gives `=FOO`, `-A=2` gives `=2: invalid context length argument` and exit 2, and `-e -e=foo` gives the pattern `-e=foo`); the jag binary of `main` eff47ff, built in `rust:1.99` (checked on 2026-10-14: `-e=foo`, `-ie=FOO`, `-e==foo`, `-e=`, `-A=2`, `-B=1`, `-C=1`, and `-e -e=foo` give the results of this decision) (inferred) |
| 38 | `--json` in an argument error | An argument error writes the error document only when clap reads `--json` as the option. So an argument error has the mode of the same command line with no error. jag reads the words from left to right, as clap does, also after the word that gives the error. A word that is the value of an option is not an option: `-e --json`, `--regexp --json`, `-ie --json`, `-A --json`, `-B --json`, and `-C --json` give the value `--json` (decisions 10 and 12). So `jag -e --json --nope dir` writes the text error of `--nope`. A `--` that is the value of an option does not end the options: in `jag -e -- --json --nope dir`, the term is `--`, `--json` is the option, and jag writes the error document. The other options with a value do not take a value that starts with `-`, so `--json` after them is the option, as today (FEATURE-12 spec decision 2: `jag --ask --json TERM PATH` writes the document). A value joined to its option is not the word `--json`: `-e--json` and `--regexp=--json` give the text error, as today. The option is only the word `--json`: `--json=1` is not the option, as today. This changes FEATURE-11 spec decision 22 ("`--json` before the first `--`"). That rule was correct before this spec, because no option took a value that starts with `-`. The code of `json_requested` changes | DECISION-27 (only `--json` gives structured mode); system-design.md CV-1; `src/cli.rs` (`enum Mode`: "Only `--json` gives `Json`"; `json_requested`: "After `--`, `--json` is a term or a path, not the option"); FEATURE-11 spec decision 22; decisions 10 and 12 (`allow_hyphen_values`); PR #170 review (verifier-2, round 1, note 3); the jag binary of `main` eff47ff, built in `rust:1.99` (checked on 2026-10-14: `-e --json a.txt` runs in text mode, but `-e --json --nope a.txt`, `--regexp --json --nope a.txt`, `-C --json --nope a.txt`, and `-ie --json --nope a.txt` write the error document; `-e -- --json a.txt` writes a document, but `-e -- --json --nope a.txt` writes the text error; `-ei --json --nope a.txt` writes the document, because `-e` takes `i`) (inferred) |

## Design

### Files

- `Cargo.toml`: `globset = "0.4.20"` in `[dependencies]` (decision 32).
- `src/cli.rs` (C-cli): the 12 options, the parse with `indices_of`, the
  words with `-e`, `build_matcher`, the checks of step 1, the flows of `-l`,
  `-c`, and `-q`, the error of an unknown option, and the help.
- `src/search/filter.rs` (new, C-search): `GlobKind`, `FileFilter`, and the
  rule of decision 25.
- `src/search/walk.rs` (C-search): `ScanOptions.files` and
  `ScanOptions.keep_files_with_no_match`, and the filter in `visit_entry`.
- `src/search/snippet.rs` (C-search): `ContextLines` and `plan_windows(files,
  context)`.
- `src/search.rs`: `pub mod filter;` and the exports.
- `src/output.rs` (C-output): `write_files`, `write_counts`, and the `--` of
  `write_lines` (FEATURE-12).
- `tests/grep_options.rs` (new): the end-to-end tests.
- `tests/usage.rs`: three tests change (see "Acceptance criteria and their
  tests").
- `README.md`: the section "Grep options".

### The command line (C-cli)

`Cli` gets these fields. The new constant `MODES` is `["setup",
"forget_key", "check"]`, the list of `--json` and `--color` today (decision
6). Each doc comment is the help line of the option.

```rust
    /// Match upper case and lower case letters as the same
    #[arg(short = 'i', long = "ignore-case", conflicts_with_all = MODES)]
    ignore_case: bool,

    /// Match only whole words
    #[arg(short = 'w', long = "word-regexp", conflicts_with_all = MODES)]
    word: bool,

    /// Read the term as fixed text, not as a regular expression
    #[arg(short = 'F', long = "fixed-strings", conflicts_with_all = MODES)]
    fixed: bool,

    /// Use TERM as the term. With more than one -e, a line matches one of the terms
    #[arg(short = 'e', long = "regexp", value_name = "TERM", allow_hyphen_values = true, conflicts_with_all = MODES)]
    terms: Vec<String>,

    /// Show NUM lines after each match
    #[arg(short = 'A', long = "after-context", value_name = "NUM", allow_hyphen_values = true, overrides_with = "after", conflicts_with_all = MODES)]
    after: Option<String>,

    /// Show NUM lines before each match
    #[arg(short = 'B', long = "before-context", value_name = "NUM", allow_hyphen_values = true, overrides_with = "before", conflicts_with_all = MODES)]
    before: Option<String>,

    /// Show NUM lines before and after each match
    #[arg(short = 'C', long = "context", value_name = "NUM", allow_hyphen_values = true, overrides_with = "context", conflicts_with_all = MODES)]
    context: Option<String>,

    /// Show only the path of each file with a match (not with --ask or --json)
    #[arg(short = 'l', long = "files-with-matches", conflicts_with_all = MODES)]
    files_with_matches: bool,

    /// Show only the count of matching lines of each file (not with --ask or --json)
    #[arg(short = 'c', long = "count", conflicts_with_all = MODES)]
    count: bool,

    /// Show nothing: give only the exit code (not with --ask or --json)
    #[arg(short = 'q', long = "quiet", alias = "silent", conflicts_with_all = MODES)]
    quiet: bool,

    /// Search only the files whose name matches GLOB
    #[arg(long, value_name = "GLOB", conflicts_with_all = MODES)]
    include: Vec<String>,

    /// Do not search the files whose name matches GLOB
    #[arg(long, value_name = "GLOB", conflicts_with_all = MODES)]
    exclude: Vec<String>,

    /// The globs of --include and --exclude, in the order of the command line
    #[arg(skip)]
    globs: Vec<(GlobKind, String)>,
```

`-A`, `-B`, and `-C` keep their text, as `--timeout` does. `search_args`
parses them (decision 12). `-q` has `alias = "silent"`, so the help shows
`--quiet` only. clap gives a value with `allow_hyphen_values` also when the
value starts with `-`: `-A -1` gives the value `-1`, and the check of
decision 12 refuses it.

`Cli.term` becomes `Option<OsString>` (decision 10). With no `-e`,
`search_args` gives the failure of `argument_failure` for a term that is
not UTF-8. `Cli.ask` stays `Option<String>` (FEATURE-12 spec decision 3).
The `conflicts_with_all` lists of `--check`, `--setup`, and `--forget-key`
keep `term`, `ask`, and `paths`, so they do not change.

A copy of this `Cli` with the `--ask` field of FEATURE-12, run with clap
4.6.7 in `rust:1.99` on 2026-10-14, gave these results:

- `-l --ask q needle dir` and `-C1 --ask q needle dir` parse. The check of
  decision 22 and the limit of decision 14 come in `search_args`.
- `-iA2 --ask q needle dir` gives `-i`, `-A 2`, the question, the term, and
  the path. `-A -1 --ask q needle dir` gives the value `-1` to `-A`.
- `--ask -C1 needle dir` is an argument error, and `--ask=-C1 needle dir`
  gives the question `-C1`, as FEATURE-12 spec decision 2 says.
- `-q` and `--silent` give `quiet`. `--json -c needle a.txt` parses, and
  decision 22 refuses it.
- `--check -i`, `--check --ask q`, and `--setup -l` are argument errors.
- `--exclude=*.md --include *.rs --ask q needle dir --exclude=c.*` gives
  the places 2 and 10 of the two `--exclude` values and the place 4 of the
  `--include` value, so `ordered_globs` gives the order of the command
  line.
- `--ask q needle -- -e dir` gives the paths `-e` and `dir`.

`parse` uses `Cli::command().try_get_matches()` and then
`Cli::from_arg_matches(&matches)`, so that it can read the order of the
globs:

```rust
/// The globs in the order of the command line (decision 27). clap gives the
/// values of each option in their own list, with the place of each value.
fn ordered_globs(matches: &ArgMatches) -> Vec<(GlobKind, String)> {
    let mut globs: Vec<(usize, GlobKind, String)> = Vec::new();
    for (id, kind) in [("include", GlobKind::Include), ("exclude", GlobKind::Exclude)] {
        let values = matches.get_many::<String>(id).into_iter().flatten();
        let places = matches.indices_of(id).into_iter().flatten();
        globs.extend(places.zip(values).map(|(place, glob)| (place, kind, glob.clone())));
    }
    globs.sort_by_key(|(place, _, _)| *place);
    globs.into_iter().map(|(_, kind, glob)| (kind, glob)).collect()
}
```

An error of `from_arg_matches` is an argument error: the fixed line, as
today.

The words with `-e` (decision 10). clap fills `term` and then `paths`.
`search_args` reads them as one list of words:

```rust
/// The words after the options: with no -e, the term and the paths. With
/// -e, each word is a path (decision 10).
fn words(cli: &Cli) -> Vec<&OsStr>
```

With `-e`, `search_args` takes the terms of `cli.terms`, and each word as a
path. With no `-e`, the first word is the term, as today, and each other
word is a path. The `SearchArgs` of FEATURE-12 gets `terms: Vec<&'a str>`
in the place of `term`, and `paths: Vec<PathBuf>` in the place of
`&'a [PathBuf]`, because with `-e` the first path comes from `Cli.term`.

### The matcher (C-cli)

```rust
/// The matcher of the terms, with -i, -w, and -F (decisions 7 to 11).
fn build_matcher(terms: &[&str], options: MatchOptions) -> Result<RegexMatcher, Failure> {
    RegexMatcherBuilder::new()
        .line_terminator(Some(b'\n'))
        .case_insensitive(options.ignore_case)
        .word(options.word)
        .fixed_strings(options.fixed)
        .build_many(terms)
        .map_err(|_| {
            usage_failure(
                "the term is not a valid regular expression.",
                "Check the term. It uses the regular expression rules of ripgrep.",
            )
        })
}
```

`build_many` of one term gives the matcher of `build` of that term. The
place of the matcher in the flow does not change: after the key and the
settings in a question search (FEATURE-04 spec decision 18), and before the
scan.

### The lines around a match (C-search, C-cli)

```rust
/// The lines around each match (FEATURE-13, decisions 13 to 17).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextLines {
    pub before: u64,
    pub after: u64,
    /// The most lines of one result. `None`: no limit (plain grep).
    pub max_lines: Option<u64>,
}

impl ContextLines {
    /// 3 lines before, 3 lines after, and at most 12 lines (FEATURE-04 spec
    /// decision 4).
    pub const DEFAULT: ContextLines = ContextLines {
        before: CONTEXT_LINES,
        after: CONTEXT_LINES,
        max_lines: Some(MAX_LINES),
    };
}

pub fn plan_windows(files: &[FileMatches], context: ContextLines) -> Vec<Window>
```

The rule of `plan_windows` stays, with the values of `context` in the place
of the constants:

- The first match `m` of a file opens the window `[max(1, m − B), m + A]`.
- The next match `m` joins the window when `m − B ≤ end + 1` and, with a
  limit L, `m ≤ start + L − 1`. Then `end` becomes `max(end, m + A)`, and,
  with a limit, at most `start + L − 1`.
- Else the window ends, and `m` opens a new window that starts at
  `max(m − B, end + 1)`.

Each sum uses `saturating_add` and each difference `saturating_sub`. With
`ContextLines::DEFAULT`, each window of today stays the same, so each unit test
of `plan_windows` of today passes with `ContextLines::DEFAULT`.

C-cli makes the context from the options:

```rust
/// The lines around a match (decisions 12 to 15). `question` is true in a
/// question search (with --ask).
fn context_lines(cli: &Cli, question: bool) -> Result<ContextLines, Failure>
```

- No option of the three: `ContextLines::DEFAULT` in a question search. Plain
  grep with no option keeps `line_windows` of FEATURE-12.
- Else: B is the value of `-B`, or of `-C`, or 0. A is the value of `-A`,
  or of `-C`, or 0 (decision 13).
- In a question search, a value above 12 gives the failure of decision 14,
  with the name of the option that gave it. Then `max_lines` is
  `Some(max(12, B + 1 + A))`.
- In plain grep, `max_lines` is `None`.

`read_snippets`, `match_texts`, and the pre-rank do not change. The
request of the model holds each line of a result (`snippet_json`). One
result of 25 lines of 300 characters gives at most 45,000 bytes of JSON,
with a control character as 6 bytes, so 15,000 tokens. With a question of
12,000 bytes (4,000 tokens), 5,000 tokens stay below the limit of 24,000
for the path and the other text of the call. A path of 4,096 bytes takes
about 1,400 tokens. So one result still fits in one call (FEATURE-04 spec
decision 22).

Example of a question search: `jag -C 1 --ask "which X?" X ctx.txt` on `ctx.txt`
with the lines `1` to `11` and `X` at lines 4 and 10 gives two results: the
lines 3 to 5, and the lines 9 to 11.

Plain grep with an option (FEATURE-12 flow): `plan_windows(&report.files,
context)` in the place of `line_windows`, then `read_snippets`. In text
mode, `write_lines` writes `--` between two results (see "The text"). With
`--json`, each group is one result (decision 16). Example: `jag -C 1 X
ctx.txt` writes, as `grep -n -H -C1 X ctx.txt` does:

```
ctx.txt-3-3
ctx.txt:4:X
ctx.txt-5-5
--
ctx.txt-9-9
ctx.txt:10:X
ctx.txt-11-11
```

### `-l`, `-c`, and `-q` (C-cli, C-output)

The checks of decision 22 come in step 1. In the plain flow of FEATURE-12,
after the scan, the counts, and the notices:

- `-q`: no output. The exit code is 0 when a file of `report.files` has a
  line, also when a path does not exist (decision 21). With no line, it is
  2 when a path does not exist, and else 1.
- `-l`: `write_files(&mut buffer, &report.files, colored)`.
- `-c`: the scan has `keep_files_with_no_match: true`. Then
  `write_counts(&mut buffer, &report.files, colored)`.
- With `-l` and `-c`, the exit code is 2 when a path does not exist, as
  FEATURE-14 spec decision 7 gives it for each search. Else it is 0 when a
  file has a line, and 1 when no file has a line. The output goes through
  `emit_claimed`, as each other output of a search.

```rust
/// Write the path of each file with a match, one path on each line, in the
/// order of `files` (decision 19). The path is in `PATH` when `colored`.
pub fn write_files(out: &mut dyn Write, files: &[FileMatches], colored: bool) -> io::Result<()>

/// Write `path:N` for each file of `files`, in that order: N is the count of
/// matching lines (decision 20). The path is in `PATH` and the `:` in
/// `SEPARATOR` when `colored`.
pub fn write_counts(out: &mut dyn Write, files: &[FileMatches], colored: bool) -> io::Result<()>
```

`write_files` skips each file with no line. Each path is
`path.as_os_str().as_encoded_bytes()`, as `Source::label` gives it. Example
on `a.txt` (`one needle`, `two`, `three needle`) and `b.txt` (`x`):

```
$ jag -l needle a.txt b.txt
a.txt
$ jag -c needle a.txt b.txt
a.txt:2
b.txt:0
```

With `--color=always`, `jag -c needle a.txt` writes
`\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K2\n`, the bytes
of `grep --color=always -c needle a.txt b.txt` for its first line.

### The file globs (C-search)

```rust
/// The option that gave a glob.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlobKind {
    Include,
    Exclude,
}

/// The globs of --include and --exclude, in the order of the command line
/// (FEATURE-13, decision 25).
#[derive(Clone, Debug, Default)]
pub struct FileFilter {
    globs: Vec<(GlobKind, GlobMatcher)>,
}

impl FileFilter {
    /// The filter of `globs`. A glob that does not parse gives its kind, and
    /// no text of the glob (decision 26).
    pub fn new(globs: &[(GlobKind, String)]) -> Result<FileFilter, GlobKind>

    /// True when jag searches a file with this name: the last glob that
    /// matches decides. When no glob matches, jag searches the file, unless
    /// the first glob is an --include glob.
    pub fn allows(&self, name: &OsStr) -> bool
}
```

`FileFilter::default()` has no glob, so `allows` is always true.
`search_args` makes the filter. The failure of a glob that does not parse
names `--include` or `--exclude` (decision 26).

`ScanOptions` gets two fields:

```rust
    /// The globs of --include and --exclude (FEATURE-13).
    pub files: FileFilter,
    /// Keep a `FileMatches` with no line for each searched file with no
    /// match (FEATURE-13, `-c`).
    pub keep_files_with_no_match: bool,
```

`ScanOptions` keeps `Clone`, `Debug`, and `Default`. It loses `Copy`,
`PartialEq`, and `Eq`: no code compares two `ScanOptions` values. The
closure of the walk gets `&ScanOptions`. In `visit_entry`, a file goes to
`search_file` only when `options.files.allows(entry.file_name())`. A folder
does not go through the filter. `entry.file_name()` of a root file is the
last part of its path, so the filter applies to a file that the user
names. The walk counts the entry before the filter, so a file that a glob
skips is not in the skip notice.

### The error of an unknown option (C-cli)

In `parse`, for the error kind `UnknownArgument`:

```rust
/// The failure of an unknown option: its name, when the text that clap
/// gives has the form of an option. Else the failure of each argument
/// error (decision 18).
fn unknown_option_failure(error: &clap::Error) -> Failure {
    match error.get(ContextKind::InvalidArg) {
        Some(ContextValue::String(name)) if is_option_form(name) => usage_failure(
            &format!("the option {name} is not valid."),
            "Run jag --help to see the options.",
        ),
        _ => argument_failure(),
    }
}

/// True for `-` and one ASCII letter or digit, or for `--` and 1 to 30
/// ASCII letters, digits, or `-`.
fn is_option_form(text: &str) -> bool
```

In text mode, jag writes `failure.render()` on the error stream, for
example:

```
jag: the option --nope is not valid.
Run jag --help to see the options.
```

With the option `--json` (decision 38), the error document holds this
cause and this fix, with `exit_code` 2. Each other kind of clap error keeps
the fixed line or the argument error document of today.

### The option `--json` in an argument error (C-cli)

`json_requested` reads the words of the command line before clap gives
the error, so it must find the same option `--json` that clap finds
(decision 38):

```rust
/// True when clap reads `--json` as the option: not as the value of an
/// option, and not after the `--` that ends the options (decision 38).
fn json_requested(args: impl IntoIterator<Item = OsString>) -> bool
```

It reads each word from left to right, and it does not stop at an
unknown option:

- `--` ends the options, unless it is the value of the option before it.
- `--NAME=VALUE` is one option with its value.
- `--NAME` is one option. When the option takes a value, the next word is
  its value if the option takes a value that starts with `-` (`--regexp`,
  `--after-context`, `--before-context`, and `--context`), or if the next
  word does not start with `-`, or if it is `-` alone.
- `-XYZ` is a group of short options. A letter with no value is one
  option. The first letter that takes a value (`e`, `A`, `B`, or `C`)
  takes the rest of the word as its value. When the rest is empty, it
  takes the next word, also a word that starts with `-`.
- An unknown option is an option with no value.
- `--color` takes its value only after `=`, so it never takes the next
  word.

The list of the options and their values can come from `Cli::command()`,
so that it stays the same as `Cli`. The implementer checks the clap API in
`rust:1.99`.

### The text (C-output)

FEATURE-12 gives `write_lines(out, results, paint)`. This spec adds one
parameter:

```rust
pub fn write_lines(
    out: &mut dyn Write,
    results: &[Snippet],
    paint: Option<Paint>,
    groups: bool,
) -> io::Result<()>
```

With `groups`, `write_lines` writes `--` (in `SEPARATOR` with color)
between two results, as `write_text` does. Each line that does not match
gets `-` in the place of each `:`, as in `write_text` (decision 15). With no option of decisions 12
and 13, plain grep gives `false`, so its bytes do not change. The label
line of no good answer never shows in plain grep.

### Other features

- FEATURE-12 (spec merged in 00c4d30, STORY-33 and STORY-34 ready): the
  plain flow, `line_windows`, `write_lines`, and the option `--ask`. The
  question comes only from `--ask`, so with `-e` each word is a path
  (decision 10). `-l`, `-c`, `-q`, and the lines around a match in plain
  grep need the plain flow (Build order step 5), so FEATURE-13 has
  FEATURE-12 in its `blocked_by` list (decision 34). The checks of this
  spec come after the checks of FEATURE-12 spec decision 20. The FEATURE-12
  code merges first, and the FEATURE-13 code builds on it.
- FEATURE-14 (spec PR #142, not merged): `scan` keeps one walk for all
  paths and gives each file the place of its path on the command line,
  so `-l` and `-c` write the files in that order. The one walk uses the
  `ScanOptions`, with its filter and its flag. For piped text, `-l` and
  `-c` write the label `(standard input)` in the place of the path, as
  `grep -H` does, and the globs do not apply. The user answered the two
  open points of FEATURE-14 on 2026-10-14. An empty pipe: jag searches the
  empty text, finds no match, and exits with 1 (FEATURE-14 spec decision
  3). So `-c` writes `(standard input):0` (decision 20), and `-l` and `-q`
  write nothing. A path that does not exist: a warning, the output of the
  other paths, and exit 2 at the end, as grep (FEATURE-14 spec decision 7).
  So `-l` and `-c` write the lines of the other paths and exit with 2, and
  `-q` with a match exits with 0 (decision 21). A read error gives a
  warning and does not change the exit code (FEATURE-14 spec decision 10).
- FEATURE-09 (spec PR #134, not merged): the size limit counts each
  character on standard output (FEATURE-09 spec decision 3), so it counts
  the lines of `-l` and `-c` and the lines around a match. The code that
  merges second gives the lines of `-l` and `-c` the size limit, with each
  line as one whole item. More lines around a match make a result larger,
  so fewer results fit. `--max-chars` and `--no-max-chars` are options of
  jag (decision 3). The open point of FEATURE-09 (exit 8 or 0) applies to
  the output of this spec as FEATURE-09 decides it. This spec does not
  decide it.
- FEATURE-15 (done): `--color` and `--colour` stay options of jag (decision
  3). The `paint` function uses the matcher of `build_matcher` (decision
  11). Decision 28 gives the color of `-l` and `-c`.
- FEATURE-11 (done): `-l`, `-c`, and `-q` refuse `--json` (decision 22).
  Plain grep with lines around a match gives one result for each group
  (decision 16). The argument error document holds the cause of decision
  18.
- FEATURE-16 (spec PR #155, not merged): `-q` and `-l` scan all the
  paths. FEATURE-16 builds no early stop at the first match (its decision
  15): a scan of the 14,490 files of Go 1.25.0 takes less than 0.1 s, so
  an early stop saves less than 0.1 s.
- FEATURE-20 (backlog): `-l`, `-c`, and `-q` make no model call, so they
  have no cost. A question search with lines around a match sends more
  text to the model, so it can cost more.

### The README section "Grep options"

The section comes after "In a terminal and in a pipe". It says:

- jag takes these grep options, with their grep meaning: `-i`, `-w`, `-F`,
  `-e`, `-A`, `-B`, `-C`, `-l`, `-c`, `-q`, `--include`, and `--exclude`.
  Each one also has its long name of GNU grep, for example
  `--ignore-case`.
- jag always searches folders, as `grep -r`, and always shows the path and
  the line number, as `grep -H -n`. So jag does not take `-r`, `-n`, or
  `-H`. Each other option, also `-r`, gives a usage error and exit 2.
  `-h` shows the help, as in rg.
- `-e TERM` gives a term. With more than one `-e`, a line matches one of
  the terms. With `-e`, each word after the options is a path, as in grep.
- With `--ask`, `-A`, `-B`, and `-C` take 0 to 12 lines, and a result has
  at most 25 lines. With no option, a result has 3 lines before and 3
  lines after the match, and at most 12 lines. With no `--ask`, the
  options take each number, and `--` stands between two groups of lines,
  as in grep.
- `-l`, `-c`, and `-q` work only with no `--ask` and with no `--json`.
  `-c` writes `path:N` also for one file.
- `--include` and `--exclude` match the name of the file, with the glob
  rules of ripgrep, for example `*.{rs,toml}`. They do not apply to
  folders or to piped text.
- jag skips binary files, so `-l` and `-c` do not show them.
- A short option takes its value after `=`, as in rg: `-e=foo` is the
  same as `--regexp=foo`. grep reads `-e=foo` as the term `=foo`. To
  search `=foo`, type `-e '=foo'` (decision 37).
- The exit codes: 0 for a match, 1 for no match (with `--ask`, also for no
  good answer), and 2 for a usage error, as in grep. A path that does not
  exist gives a warning and exit 2, but `-q` with a match exits with 0, as
  `grep -q`. A file that jag cannot read gives a warning, and the code
  does not change.

### Acceptance criteria and their tests

- CRITERION-55 (the options, with their grep meaning):
  - Unit (`src/cli.rs`): each short name and each long name parses to its
    field. `-iA2` gives `ignore_case` and `after` `2`. `-e -x` gives the
    term `-x`. `--silent` gives `quiet`. `ordered_globs` of
    `--exclude=*.md --include *.rs --exclude=c.*` gives the three globs in
    that order. Each option of this spec with `--check`, `--setup`, or
    `--forget-key` is an error.
  - Unit (`src/cli.rs`, decision 10): `search_args` of `-e needle a.txt
    b.txt` gives the terms `[needle]` and the paths `a.txt` and `b.txt`.
    `-e --ask q needle` gives the term `--ask` and the paths `q` and
    `needle`. With `-e`, a path that is not UTF-8 works. With no `-e`, a
    term that is not UTF-8 gives the failure of `argument_failure`.
  - Unit (`src/cli.rs`, decision 37): `-e=foo` gives the terms `[foo]`.
    `-ie=FOO` gives `ignore_case` and the terms `[FOO]`. `-e==foo` gives
    `[=foo]`. `-e=` gives `[""]`. `-e -e=foo` gives `[-e=foo]`. `-A=2`
    gives `after` `2`.
  - End-to-end (`tests/grep_options.rs`, decision 37): `jag -e=foo a.txt`
    on a file with the lines `foo` and `=foo` writes the two lines, exit 0.
    `jag -e '=foo' a.txt` writes only the line `=foo`.
  - Unit (`src/cli.rs`): `build_matcher` with `ignore_case` matches
    `Needle` for `needle`. With `word`, it matches `the needle.` and not
    `needles` or `needle_x`. `build_matcher(&["one", "three"], ...)`
    matches both lines.
  - Unit (`src/search/filter.rs`): the cases of decision 25:
    `[Exclude *.md, Include *.rs]` allows `a.txt` and `c.rs` and refuses
    `d.md`. `[Include *.rs, Exclude c.*]` refuses `c.rs`. `[Include *.rs]`
    refuses `a.txt`. No glob allows each name. `*.{rs,md}` allows `d.md`.
    `[` gives `Err(GlobKind::Include)`.
  - Unit (`src/search/walk.rs` or `tests/walk.rs`): with the filter
    `[Include *.rs]`, `scan` of a folder with `a.txt` and `dir/c.rs` gives
    only `dir/c.rs`, and the skip counts do not change. A root file `a.txt`
    with that filter gives no file. With `keep_files_with_no_match`, a file
    with no match gives a `FileMatches` with no line, and a binary file
    gives none.
  - Unit (`src/output.rs`): `write_files` and `write_counts` give the bytes
    of the examples, with and with no color. `write_lines` with `groups`
    writes `--` between two results, and with no `groups` it writes the
    bytes of FEATURE-12.
  - End-to-end (`tests/grep_options.rs`): `jag -i`, `-w`, and `-e one -e
    three` with `--ask` and the fake server: the request holds only the
    lines that match. `--include='*.rs'` on a folder sends only the lines of
    the `.rs` file. `jag --ask q --exclude='*.txt' needle a.txt` gives exit
    1 and no call.
  - End-to-end (`tests/grep_options.rs`, after the FEATURE-12 code):
    `jag -l needle a.txt b.txt` writes `a.txt\n`, exit 0. `jag -c needle
    a.txt b.txt` writes `a.txt:2\nb.txt:0\n`, exit 0. `jag -q needle
    a.txt` writes nothing, exit 0. `jag -c -l needle a.txt` writes
    `a.txt\n`. `jag -q -l needle a.txt` writes nothing. `jag -c -C2 X
    ctx.txt` writes `ctx.txt:2\n`. `jag -e needle a.txt b.txt` writes the
    matching lines of the two files. `jag -l --ask "a question" needle
    a.txt` and `jag --json -c needle a.txt` give exit 2 with the texts of
    decision 22 and no call.
- CRITERION-56 (`-F`):
  - Unit (`src/cli.rs`): `build_matcher(&["needle."], fixed)` matches
    `the needle.` and not `needles`. `build_matcher(&["a("], fixed)` gives
    a matcher. With no `-F`, `a(` gives the failure of the term.
  - End-to-end: `jag -F --ask q 'a(' file` sends the line `x a( y` to the
    model, and exits with 0. `jag --ask q 'a(' file` exits with 2.
- CRITERION-69 (the lines around a match in a question search):
  - Unit (`src/search/snippet.rs`): `plan_windows(files, ContextLines::DEFAULT)`
    gives the windows of today (each test of today). With `before: 0,
    after: 2, max_lines: Some(12)`, a match at line 5 gives `[5, 7]`. With
    `before: 12, after: 12, max_lines: Some(25)`, a match at line 30 gives
    `[18, 42]`. With `max_lines: None`, matches at lines 4 and 6 with
    `before: 1, after: 1` give one window `[3, 7]`, and matches at lines 4
    and 10 give two windows. With `after: u64::MAX`, the plan does not
    overflow.
  - Unit (`src/cli.rs`): `context_lines` gives `ContextLines::DEFAULT` for no option,
    before 1 and after 3 for `-C1 -A3` and for `-A3 -C1`, before 0 and
    after 5 for `-A 5`, and the failure of decision 14 for `-C 13` in a
    question search. `-A abc`, `-A -1`, `-A +1`, and `-A ''` give the
    failure of decision 12, and the text holds no value.
  - End-to-end (`tests/grep_options.rs`, fake server, with `--ask`): with no
    option, the request holds lines 1 to 7 for a match at line 4 (today). With `-C 5`,
    it holds lines 1 to 9 for a match at line 4 of a file of 20 lines. With
    `-A 2`, it holds lines 4 to 6. With `-C 13`, jag exits with 2, writes
    the text of decision 14, and makes no call.
  - End-to-end (after the FEATURE-12 code): `jag -C 1 X ctx.txt` writes the
    seven lines of the example, with `--`. `jag -C 0 X ctx.txt` writes
    `ctx.txt:4:X\n--\nctx.txt:10:X\n`. `jag --json -C 1 X ctx.txt` gives
    `results_found` 2 and two results of three lines.
- CRITERION-57 (an unknown option):
  - Unit (`src/cli.rs`): `is_option_form` is true for `-Z`, `-1`, `--nope`,
    and a `--` with 30 letters. It is false for `-`, `--`, `-é`, `--a_b`, a
    `--` with 31 letters, and `SENTINEL`.
  - End-to-end (`tests/grep_options.rs`): `--nope`, `--nope=SENTINEL`,
    `-iZx`, and `-r` give exit 2, an empty standard output, and the two
    lines of decision 18 with `--nope`, `--nope`, `-Z`, and `-r`. The text
    does not hold `SENTINEL`. `--json --nope` gives the error document with
    that cause and fix. `--SENTINEL-SENTINEL-SENTINEL-SENTINEL` (35
    characters after `--`) gives the fixed line. `--color=always`,
    `--colour`, `--json`, `--timeout 5`, and `--max-scored 10` are not
    unknown options.
  - Unit (`src/cli.rs`, decision 38): `json_requested` is false for
    `-e --json --nope`, `--regexp --json --nope`, `-ie --json --nope`,
    `-A --json --nope`, `-B --json --nope`, `-C --json --nope`,
    `-e--json --nope`, and `--regexp=--json --nope`. It is true for
    `-e -- --json --nope`, `-ei --json --nope`, `--ask --json t p`,
    `--include --json`, and `--nope --json`. The cases of
    `json_requested_reads_the_option_before_the_first_double_dash` do not
    change.
  - End-to-end (`tests/grep_options.rs`, decision 38): `jag -e --json
    --nope a.txt` writes nothing on standard output, writes the two lines
    of decision 18 with `--nope` on the error stream, and exits with 2.
    `jag -e -- --json --nope a.txt` writes the error document with the
    cause of decision 18, and exits with 2.
  - `tests/usage.rs`: in `an_argument_error_writes_one_fixed_line_with_no_value`,
    the case `--SENTINEL-SENTINEL-SENTINEL` becomes
    `--SENTINEL-SENTINEL-SENTINEL-SENTINEL`: today the shorter text has the
    form of an option, so it would show. In
    `an_argument_error_with_json_writes_one_document_with_no_value`, the
    case `--json --bogus` moves to the new test of `--json --nope`, because
    its cause names `--bogus`. In `json_after_a_double_dash_is_not_the_option`,
    the expected error text becomes `jag: the option --bogus is not
    valid.\nRun jag --help to see the options.\n`. `help_and_version_exit_0`
    checks the usage line of decision 31 and the help of each option. The
    test `criterion_51_an_argument_error_document_that_cannot_be_written_gives_the_text_failure`
    of `tests/output.rs` also uses `--json --bogus`. It does not change,
    because it checks only the failure of the write.
- CRITERION-58 (the exit codes):
  - End-to-end (`tests/grep_options.rs`): a question search with a good
    answer exits with 0, and with no match with 1. Each usage error of this
    spec exits with 2: a bad value of `-A`, an unknown option, a glob that
    does not parse, and `-l` with `--ask`. A search with a match and a
    file that jag cannot read exits with 0 and writes a warning: the test
    `a_file_that_jag_cannot_read_gives_a_warning_and_the_others_are_scored`
    of `tests/search.rs` proves it today for a question search, so it
    stays.
  - End-to-end (after the FEATURE-12 code): plain grep with a match exits
    with 0, and with no match with 1. `jag -c zzz a.txt b.txt` writes
    `a.txt:0\nb.txt:0\n` and exits with 1. `jag -l zzz a.txt` and `jag -q
    zzz a.txt` exit with 1 and write nothing. `jag -l needle dir` with a
    match and a file that jag cannot read exits with 0 and writes a
    warning. As in `tests/search.rs`, the test skips this check for a user
    who can read each file (root).
  - End-to-end (after the FEATURE-12 and FEATURE-14 code): `jag -q needle
    missing.txt a.txt` writes nothing on standard output, writes the
    warning of FEATURE-14 spec decision 7, and exits with 0. `jag -q zzz
    missing.txt a.txt` exits with 2. `jag -l needle missing.txt a.txt`
    writes `a.txt\n` and exits with 2. `jag -c needle missing.txt a.txt
    b.txt` writes `a.txt:2\nb.txt:0\n` and exits with 2. An empty pipe with
    `-c` writes `(standard input):0\n` and exits with 1. Piped text with
    `--exclude='*'` gives its match, exit 0.

## Build order

1. STORY-37: `is_option_form`, `unknown_option_failure`, and the change of
   `parse`. The three tests of `tests/usage.rs` change, and the new tests of
   CRITERION-57 pass. This step needs no other step.
2. STORY-35, the matcher: the fields `-i`, `-w`, `-F`, and `-e`, the
   conflicts of decision 6, `Cli.term` as `Option<OsString>`, `words`,
   and `build_matcher`. The question search uses them. The unit tests of
   the matcher and the end-to-end tests of `-i`, `-w`, `-e`, and `-F`
   pass.
3. STORY-35, the globs: `globset` in `Cargo.toml`, `src/search/filter.rs`,
   `ordered_globs`, `ScanOptions.files`, and the check in `visit_entry`.
   The unit tests of the filter and of the scan pass.
4. STORY-36, the question search: `ContextLines`, `plan_windows(files,
   context)`, `context_lines`, and the checks of decisions 12 and 14. Each unit
   test of `plan_windows` of today passes with `ContextLines::DEFAULT`. Then the
   new tests of CRITERION-69 for a question search pass.
5. When the FEATURE-12 code is on `main`: STORY-35 and STORY-36 in plain
   grep: the checks of decision 22, `keep_files_with_no_match`,
   `write_files`, `write_counts`, the flows of `-l`, `-c`, and `-q`, the
   lines around a match with `groups` in `write_lines`, and the document of
   decision 16. The tests that say "after the FEATURE-12 code" pass.
6. STORY-38: the tests of CRITERION-58. The exception of decision 21 and
   the tests that say "after the FEATURE-12 and FEATURE-14 code" need the
   warning of a path that does not exist (FEATURE-14 spec decision 7).
   When the FEATURE-14 code merges after the FEATURE-13 code, the
   FEATURE-14 code adds them.
7. The help, the usage line, and the README section "Grep options".
8. After the FEATURE-13 code (PR #170), one more pull request: the tests
   of decision 37, which record the code of today, and the README line of
   decision 37. Then the code of decision 38: `json_requested` reads the
   words as decision 38 says, and the tests of decision 38 pass. STORY-37
   is `done` on `main` (#192), so the planner decides how the board
   tracks this pull request.

## Assumptions

- The long names of GNU grep are aliases that a grep user expects, as
  `--colour` (decision 2). Source: FEATURE-15 spec decision 5; GNU grep
  3.12 `grep --help`.
- `-h` stays the help, as in rg (decision 4). Source: `src/cli.rs` (clap
  gives `-h`).
- `word(true)` of grep-regex is the rule of `grep -w` (decision 8). Source:
  grep-regex 0.1.14 docs; the GNU grep 3.12 checks.
- With `-e`, each word is a path (decision 10). Source: FEATURE-12 spec
  decision 1 and its "Other features"; GNU grep (with `-e`, each word is a
  file); the clap 4.6.7 run of 2026-10-14.
- A value of `-A`, `-B`, or `-C` has the digit rule of the other number
  options of jag (decision 12). Source: FEATURE-06 spec decision 3;
  FEATURE-09 spec decision 7.
- A joined result of a question search keeps the limit of today: when a
  window reaches its limit, the last match of the window can get fewer
  lines after it than A (decision 17). Source: `src/search/snippet.rs`
  (`plan_windows`).
- The document of plain grep with lines around a match has one result for
  each group, with no new field (decision 16). Source: FEATURE-12 spec
  decision 14; FEATURE-11 spec.
- `-q` and `-l` scan all paths, with no early stop (decision 21). Source:
  FEATURE-16 spec decision 15 (PR #155).
- `-q` with a match exits with 0 also when a path does not exist, the one
  exception to FEATURE-14 spec decision 7 (decision 21). Source: the
  orchestrator DECISION of 2026-10-14; the user answer of 2026-10-14 for
  FEATURE-14 ("as grep does"); CRITERION-55; STORY-38; `man grep` (`-q`)
  and the GNU grep 3.12 checks.
- With `-c`, an empty pipe gives `(standard input):0` and exit 1 (decision
  20). Source: decision 20 and FEATURE-14 spec decision 3 (jag searches
  the empty text); the GNU grep 3.12 check with `-H -c`.
- jag skips binary files also with `-l` and `-c` (decision 23). Source:
  DECISION-14; FEATURE-05 spec.
- `-c` needs the files with no match from the scan, and a flag of
  `ScanOptions` gives them only for `-c` (decision 24). Source:
  `src/search/walk.rs` (`search_file`).
- A glob matches the name of the file only, with the rules of `globset`,
  and a glob that does not parse is a usage error (decision 26). Source:
  globset docs; GNU grep 3.12 checks.
- A file that a glob skips is not in the skip notice (decision 27). Source:
  `src/search/walk.rs` (the walk counts each entry that it gives).
- The color of `-l` and `-c` is the color of GNU grep (decision 28).
  Source: GNU grep 3.12 bytes; FEATURE-15 spec decision 7.
- The new usage line of `-e` takes the form of `man grep` (decision 31).
  Source: `src/cli.rs` (`override_usage`); FEATURE-12 spec decision 22;
  GNU grep 3.12 (`man grep`).
- The message texts of decisions 12, 14, 18, 22, and 26 are new. Source:
  system-design.md CV-1 (a cause and a fix); the texts of `src/cli.rs`.
- The tests hold the expected bytes of GNU grep 3.12, and no test runs
  `grep` (decision 36). Source: FEATURE-11 and FEATURE-15 specs.
- A short option takes its value after one `=`, as in rg, and not as in
  grep (decision 37). Source: clap 4.6.7 and the jag binary of `main`
  eff47ff; ripgrep 15.1.0; decision 26.
- An argument error has the mode of the same command line with no error
  (decision 38). Source: DECISION-27; FEATURE-11 spec decision 22;
  `src/cli.rs` (`enum Mode`).

## Risks

1. A grep habit such as `jag -rn needle .` gives exit 2. Signal: reports
   or agent logs with `the option -r is not valid.` Mitigation: the error
   names the option, and the README says that jag always works as `grep
   -r -n -H`. A later version can accept `-r`, `-n`, and `-H` with no
   effect: an addition breaks no command.
2. A grep user types `jag -h needle dir` for no path on each line, and gets
   the help with exit 0. A script then reads the help as results. Signal: a
   report of help text in the output of a script. Mitigation: the README.
   Before the first release, a change of `-h` is cheap.
3. `-c` on a large folder writes one `path:0` line for each file, as grep.
   The size limit of FEATURE-09 then cuts the output. Signal: exit 8 or the
   size notice with `-c`. Mitigation: `--include`, or `-l`.
4. More lines around a match send more text to the model, so a search costs
   more. Signal: the cost of a search grows with `-C` (FEATURE-20).
   Mitigation: the limit of 12 lines on each side (decision 14).
5. A glob matches in jag and not in grep, or the reverse: braces, `**`, a
   glob that does not parse, and a glob with `/` on a file that the user
   names (grep also tries each part of the name after a `/`). Signal: a
   report that compares jag and grep. Mitigation: the README says that jag
   uses the glob rules of ripgrep on the name of the file.
6. A short secret that the user types as an option shows on the error
   stream (decision 18). Signal: none that jag can see. Mitigation: only a
   text of 30 characters or less with the form of an option shows, and no
   option of jag takes the key (CV-6).
7. `-q` scans all paths, so it is slower than `grep -q` on a large folder
   with an early match. On Go 1.25.0 (14,490 files), the full scan takes
   less than 0.1 s, so FEATURE-16 builds no early stop (its decision 15,
   PR #155). Signal: a user report of a slow `-q` on a very large folder.
   Mitigation: a later feature adds an early stop.
8. The code of FEATURE-14 and FEATURE-09 changes the same functions as
   this spec: `search`, `search_args`, `scan`, `parse`, and `write_lines`.
   The FEATURE-12 code comes first (decision 34). Signal: a merge conflict.
   Mitigation: the code that merges second joins the parts, and the tests
   of each feature stay.
9. `-e` takes the next word as its term also when the word starts with
   `-`, so `jag -e --ask q needle` searches the term `--ask` in the paths
   `q` and `needle` (decision 10). Signal: a report of a missing path `q`
   after `-e --ask`. Mitigation: grep does the same with `-e`, and the
   warning of FEATURE-14 spec decision 7 names the path that does not
   exist.
10. A script that tests only the exit code of `jag -q` does not see a
    mistyped path when another path has a match (decision 21). Signal: a
    report of a missed typo with `-q`. Mitigation: jag writes the warning
    of the missing path on the error stream, as `grep -q` writes its
    error.
11. A grep script with `-e=x` searches `x` in jag and `=x` in grep, with
    no error, and `-e=` matches each line in jag (decision 37). Signal: a
    report that compares jag and grep with `-e=`. Mitigation: the README
    tells the difference. Before the first release, a change to the form
    of grep is cheap. After the first release, it is a changed option and
    needs a new major version (IF-3).
12. The words that `json_requested` reads as values must stay the same as
    the options of `Cli` (decision 38). A new option that takes a value
    that starts with `-` changes the rule. Signal: an error document with
    no option `--json`, or a text error with it. Mitigation: the list of
    options comes from `Cli::command()`, and the unit tests of decision 38.

## Out of scope

- An early stop of `-q` and `-l` at the first match: FEATURE-16 does not
  build it (its decision 15: it saves less than 0.1 s). A later feature
  can add it.
- Each other option of GNU grep, for example `-r`, `-n`, `-H`, `-h` as
  "no path", `-v`, `-x`, `-o`, `-m`, `-L`, `-s`, `-Z`, `-f`,
  `--exclude-dir`, and `-NUM`: DECISION-34 does not keep them.
- The prefixes of a long option that GNU grep takes, for example
  `--ignore` for `--ignore-case`: no story asks for them.
- A term with a line break, which grep reads as two terms: no story asks
  for it. `-e` gives more than one term.
- A meaning of `-l`, `-c`, and `-q` with `--ask` or with `--json`: a later
  version (the user answer of 2026-10-14 gives a usage error, decision 22).
- `GREP_COLORS` and the colors: FEATURE-15.
- The rules of an empty pipe and of a path that does not exist: FEATURE-14.
  This spec applies them to its options (decisions 20, 21, and 29).
- Exit 8 or 0 for the size limit: FEATURE-09.
- The option `--ask`: FEATURE-12.
- The cost line: FEATURE-20.
