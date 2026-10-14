---
title: "Fast enough to replace grep: a typical search in under 5 seconds, kept by a speed test and proved by a measure with the model service"
feature: FEATURE-16
stories: [STORY-44]
kind: build
gh_issue: 105
lifecycle_state: draft
inherits: [QS-3, IF-4, C-cli, C-search, C-prerank, C-scorer, CV-5, CV-6]
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Fast enough to replace grep: a typical search in under 5 seconds, kept by a speed test and proved by a measure with the model service

## Problem

A shell user goes back to grep when a search is slow (NEED-12,
CAPABILITY-06). METRIC-03 sets the target: a typical search returns in
under 5 seconds.

- STORY-44: a shell power user (PERSONA-04) wants a typical search to
  return in under 5 seconds, so that they do not go back to grep.
  - CRITERION-65: one search at the default cost limit, on a medium code
    repository, with a working service, returns in under 5 seconds.

CONSTRAINT-06 gives the scope of the target: one search at the default
cost limit, on a medium code repository, while the model service works.
The wait before a retry (CONSTRAINT-04) does not count. QS-3 measures "the
full search, from the start of the command to the last line". Its risk is
M, "because the time of one model call with 32 snippets and the time of
the file search are estimates, not measurements". ADR-0002 says: "Measure
it before release 1."

No source said what a medium code repository is: not the PRD, the system
design, the ADRs, or the merged specs. The user answered on 2026-10-14:
about 15,000 files, as Go 1.25.0 (decision 2).

Today, jag has no speed test and no measure. Earlier specs left notes for
FEATURE-16: the second read of each folder list and the sort of all
matches (FEATURE-05), BM25 on many results and a search with a large
limit (FEATURE-06), one `String` for each word of the pre-rank (review of
PR #124), the second read of plain grep (FEATURE-12, PR #151), and an
early stop of `-q` and `-l` (FEATURE-13, PR #153). This spec measures each
one (decisions 13 to 16).

The three probes of this spec (see "The evidence") show that the model
calls decide the time. On Go 1.25.0 (14,490 files), the local part takes
0.05 to 0.11 s with 3 CPUs or more, and at most 0.25 s with 1 CPU: the
walk, the search of the files, the pre-rank, the read of the snippets,
and the output. A typical search makes 7 calls in two rounds. So the time
is about two times the time of one call, plus the local part. The target
holds while one call of 32 results takes less than about 2.4 s.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | A typical search | A search with a term and the option `--ask QUESTION`, in text mode, at the default cost limit of 200 results and the default size limit of 20,000 characters, on a medium code repository (decision 2). The user gives the repository as one folder or as the list of its files (a shell glob). The term finds more than 200 results, so the pre-rank runs, and the model scores 200 results in 7 calls: 6 calls of 32 and 1 call of 8, with 6 calls at the same time. So the search waits for two rounds of calls. jag calls the TypeSafe API directly, with no gateway. The files are in the file cache of the system, because a user searches the repository that they work in. No call gets a retry | CRITERION-65; CONSTRAINT-06; system-design.md QS-3; DECISION-12 (the default cost limit is 200 results); FEATURE-09 spec decision 6 (the default size limit is 20,000 characters; its code is on `main`, PR #167); DECISION-36 (any mix of files and folders); 0018-take-the-question-only-from-the-ask-option.md (the command form `jag [OPTIONS] TERM [PATH...]`); 0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md ("A typical search needs two rounds of calls"); the probes of 2026-10-14 (each of the 5 searches of decision 8 found more than 200 results on tokio 1.53.3, Django 5.2, and Go 1.25.0, exited with 7, and made 7 calls) (inferred) |
| 2 | The size of a medium code repository | About 15,000 files, as Go 1.25.0: 14,490 files, 3.48 million lines, and 133.8 MB. The test tree of decision 7 has `FILES = 15_000`, and the measure of decision 8 clones Go at the tag `go1.25.0`, the commit `6e676ab2b809d46623acb5988248d95d1eb7939c` | human:404pilo, 2026-10-14 (hitl answer FEATURE-16 "The size of a medium code repository": about 15,000 files, as Go 1.25.0; not the recommended option, which was about 7,000 files, as Django 5.2); the first probe of 2026-10-14 (the file counts of 13 public repositories at fixed tags, from 222 to more than 60,000 files); `git ls-remote https://github.com/golang/go.git refs/tags/go1.25.0` and a clone of the tag (2026-10-14: the commit `6e676ab2b809d46623acb5988248d95d1eb7939c`, 14,490 files of 133,779,333 bytes) |
| 3 | The time that counts, and the pass rule | The wall time from the start of the jag process to its end, as a shell measures it. The measure runs each of its 5 searches one time to warm up the file cache, and then 5 times. A search passes when the median of its 5 times is under 5.0 s. The measure passes when each of the 5 searches passes. The median drops a rare slow run, for example a run with a retry, whose wait does not count (CONSTRAINT-06). The measure also gives the slowest run | system-design.md QS-3 (Artifact: "the full search, from the start of the command to the last line"); CONSTRAINT-06; CRITERION-65 (inferred) |
| 4 | The budget of the 5 seconds | The local part takes at most 1 s. The two rounds of calls get the other 4 s, so one round can take 2 s. The CI test of decision 6 checks both parts with the fake model service | 0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md ("about 1 second of model time, which leaves time for the file search inside 5 seconds"); system-design.md CV-5 (one call times out after 4 s); the probes of 2026-10-14 (the local part took 0.025 to 0.092 s on three repositories with 12 CPUs, and 0.049 to 0.252 s on Go 1.25.0 with 12 to 1 CPUs; with a delay of 2.4 s for each call, the search took 4.86 to 4.92 s; with a delay of 2 s, the search of the test tree took 4.08 to 4.10 s) (inferred) |
| 5 | The model calls | No change: 32 results in each call and 6 calls at the same time (ADR-0002), on one `ureq` agent for each search (ADR-0014). A change waits for the result of the measure (Risk 1) | 0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md; 0014-call-the-model-service-with-ureq-and-serde-json-on-std.md |
| 6 | The CI speed test | The new file `tests/speed.rs` holds one test. Only a release build runs it (`#[cfg_attr(debug_assertions, ignore = ...)]`). It makes the test tree of decision 7 and runs three checks, one after the other, with the fake model service: (a) a typical search takes under 1 s when the service answers at once; (b) the same search takes under 5 s when the service answers each call after 2 s, with 7 calls and 6 calls at the same time; (c) the search of the tree as the list of its files takes under 1 s. The list of check (c) has 15,000 paths such as `d0000/f0.txt`: 315,142 bytes of arguments with their pointers, less than a third of the 1 MiB that macOS allows, so the test gives the list on the command line and needs no folder in its place. A new step of the CI job `rust` runs `cargo test --locked --release --test speed` on `ubuntu-latest` and on `macos-latest`. A debug build has no optimization (`opt-level` 0), so its time says nothing about the release binary that a user runs | CRITERION-65; decision 4; the second probe of 2026-10-14 (the size of the argument list; the times with 1 to 12 CPUs); xnu `bsd/sys/syslimits.h` (`ARG_MAX` is `1024 * 1024` on macOS); `.github/workflows/ci.yml` (`cargo test --locked`, a debug build, is the only test step); `Cargo.toml` (no profile settings); `tests/failure.rs` and `tests/score.rs` (tests that assert a time, for example `started.elapsed() < Duration::from_secs(5)`); `tests/common/fake_server.rs` (`Reply::after` and `most_at_once`) (inferred) |
| 7 | The test tree | `FILES` files in folders of 10 files (`d0000/f0.txt` to `d1499/f9.txt` for 15,000 files), in a temporary folder outside the repository, so that no ignore file of the repository applies. Each file has 240 lines of about 40 bytes, so the tree has about the line count and the size of the repository of decision 2: 3.6 million lines and 142.6 MB. Line 120 of each file holds the term `error`, so the search finds `FILES` results. The answer of decision 2 sets `FILES` and `LINES`. The tree uses about 190 MB of the disk of the runner, and the test removes it at its end | decision 2; the second probe of 2026-10-14 (a file of Go 1.25.0 has 240 lines of 38.5 bytes on average; a script made the tree of 15,000 files of 9,509 bytes in 0.21 s, and it used 190.5 MB on an ext4 disk) (inferred) |
| 8 | The measure with the model service | The new script `scripts/speed/measure.py`: Python 3, standard library only. The user runs it with their own key. It clones Go at the tag `go1.25.0` with `git clone --depth 1`, checks the commit of decision 2, runs each of the 5 searches of the Design one time to warm up and then 5 times, one search at a time, and prints a table and one result line. It exits with 0 when the measure passes, 1 when it fails, and 2 when jag fails or an argument is wrong. jag gets the environment of the user with no `JAG_BASE_URL`, `JAG_MODEL`, `JAG_TIMEOUT`, `JAG_RETRY_WAIT`, `JAG_MAX_SCORED`, and `JAG_MAX_CHARS`, so it uses its defaults and calls the TypeSafe API directly. The script never reads, prints, or sets the key: it copies the environment of the user and removes only those six settings, so jag finds the key as it always does | CRITERION-65 ("with a working service"); 0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md (Consequences: "Measure it before release 1"); system-design.md CV-6 (only C-config reads the key); `src/config/model.rs` and `src/config/limit.rs` on `main` (the settings that jag reads, with `JAG_MAX_CHARS` of FEATURE-09, PR #167); `scripts/adr/` (the repository already has Python 3 scripts) (inferred) |
| 9 | The record of the result | The README section "Speed" gets one dated line for each measure: the date, the jag version, the system with its CPU count, the repository with its file count, the medians, and the slowest run. The first line comes in the pull request of step 6 of the Build order | decision 3; FEATURE-15 spec (a README section for each feature) (inferred) |
| 10 | When STORY-44 is done | After step 6 of the Build order: the measure passes, and its line is on `main`. A code merge alone does not finish STORY-44. When the measure fails, STORY-44 does not go to `done` and keeps its state, and the specifier sends the numbers to the orchestrator as an ASK with the options of Risk 1 | CRITERION-65 ("with a working service": the fake service of the CI test does not prove it) (inferred) |
| 11 | The list of folders in `finish` | `Shared.folder_paths` becomes a `Mutex<HashSet<PathBuf>>`. Today `finish` searches the whole list for each error path, so its time grows with the count of errors times the count of folders. This is not on the path of a typical search, because a code repository has few folders that jag cannot read. A search of a home folder or of `/` can have many | `src/search/walk.rs` (`finish` calls `folder_paths.contains(&path)` on a `Vec`); the first probe of 2026-10-14 (20,000 folders: 48 ms with no folder that jag cannot read, 156 ms with 500, 915 ms with 2,000; 40,000 folders with 2,000: 1,589 ms; `rg -n` took 38 to 62 ms on each tree) |
| 12 | One walk for all roots | `scan` keeps one `WalkBuilder` for all roots, as on `main`. Each walk of the `ignore` crate starts its threads and stops them, and that takes about 1.4 to 2.3 ms. So a walk for each root makes a shell glob of 1,000 files take 1.4 to 2.3 s. The FEATURE-12 code (PR #160) put the order of the command line on `main` in the one walk: the root of a file is `path.ancestors().nth(entry.depth())`, `order_files` gives each file the places of its root, and `warning_place` gives each warning the place of the first root that holds its path. The FEATURE-14 spec (PR #142, decision 6) keeps it. This feature does not change that code. Check (c) of decision 6 fails when a change walks each root alone | `src/search/walk.rs` on `main` (`scan` adds each root to one `WalkBuilder`; `order_files`, `warning_place`, and `finish(shared, roots)`); ignore 0.4.33 `src/walk.rs` (`WalkParallel::visit` puts each root on its stack and starts `threads()` workers for each walk; `threads()` gives the CPU count, at most 12, for 0); FEATURE-14 spec decision 6 and Risk 9 (on `main`, PR #142); the first probe of 2026-10-14 (Go files as roots, the `ignore` walk with the searcher of jag: 1,000 roots took 4.5 ms in one walk, and 2.33 s with a walk for each root with 12 threads, 1.67 s with 4, and 1.44 s with 2; 5,000 roots took 20 ms against 11.8 s; jag on `main` with 5,000 file roots took 66 ms); the second probe of 2026-10-14 (jag on `main` 157c8f2 with the 14,462 file paths of Go took 0.121 s, and with the 15,000 file paths of the test tree 0.082 s) |
| 13 | The other costs that earlier specs left to FEATURE-16 | No change to: the second read of each folder list (`count_children`), the sort of all matches in memory, one `String` for each word in the pre-rank, BM25 on many results, and the read of each whole file in `read_snippets`. With all of them, the local part of a typical search on Go 1.25.0 takes about 0.1 s with 12 CPUs, and at most 0.25 s with 1 CPU: a quarter of the budget of decision 4 | FEATURE-05 spec ("FEATURE-16 measures the cost on a large folder and can change it"; "FEATURE-16 can measure and change the sort"); FEATURE-06 spec, Risks (BM25 on a very large N); the review of PR #124 (`words` in `src/prerank.rs`); the probes of 2026-10-14 (Go, `error`: 54,475 matching lines, 33,198 results, 0.092 s in the first probe, and 0.103 s with 12 CPUs and 0.252 s with 1 CPU in the second; 20,000 folders: jag 48 ms, `rg -n` 44 ms) |
| 14 | The second read of plain grep | No change. FEATURE-12 (PR #151, decision 9) reads each matching file a second time to give its lines. That read takes 20 to 43 ms on Django and on Go. Plain grep makes no model call, so it stays far under 5 s | PR #151 decision 9 and Risk 4 ("FEATURE-16 can remove the second read"); the first probe of 2026-10-14 (a read of each file with a match, on one thread: Django, 1,112 to 1,303 files of 22 to 23 MB, 19.5 to 22.2 ms; Go, 1,928 to 4,551 files of 45 to 63 MB, 22.4 to 43.1 ms) |
| 15 | An early stop of `-q` and `-l` | Not built. The target does not need it: a search of the 14,490 files of Go takes 0.05 to 0.11 s with 3 CPUs or more, and at most 0.25 s with 1 CPU, so a stop at the first match saves less than that | FEATURE-13 spec (PR #153: "Out of scope" and Risk 7 name FEATURE-16 for it); the probes of 2026-10-14 |
| 16 | A search with a large limit | No change. A search with `--max-scored N` needs one more round of calls for each 192 results. The README says so. The deadline of CV-5 (9 s with the defaults) stops a search that needs more time, and `--timeout` makes the deadline longer | FEATURE-06 spec ("The time of a search with a large limit: FEATURE-16", and its risk "A limit near 2,000 can reach the deadline of CV-5"); system-design.md CV-5; 0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md (6 calls of 32 at the same time) |
| 17 | Docs | The README section "Speed": the target, where the time goes, the time of a larger limit, the limit of the system on a long list of files, how to run the measure and its cost, and the dated lines of decision 9 | decision 9; Risk 9; FEATURE-15 spec (a README section for each feature); METRIC-05 (at most $0.02 for each search, an assumption) (inferred) |
| 18 | Story state | STORY-44 goes to `ready` in this PR. The answer of 2026-10-14 closes the ASK of decision 2, so no open ASK blocks it. FEATURE-16 has FEATURE-04 and FEATURE-06 in `blocked_by`, and both are `done`. The spec merges with this PR | docs/adlc.md rule 11 (a spec PR can set its own stories to `ready`); the orchestrator ANSWER of 2026-10-14 ("STORY-44 → ready if rule 11 allows") |

## Design

### Files

- `src/search/walk.rs` (C-search): `Shared.folder_paths` is a `HashSet`,
  and a new unit test of `finish` (decision 11).
- `tests/speed.rs` (new): the test tree and the three checks (decisions 6
  and 7).
- `.github/workflows/ci.yml`: the step
  `cargo test --locked --release --test speed` (decision 6).
- `scripts/speed/measure.py` (new): the measure with the model service
  (decision 8).
- `README.md`: the section "Speed" (decision 17).

No other file of `src/` changes.

### The evidence

Three probes ran on 2026-10-14 on Linux x86_64 with 12 CPUs and a warm
file cache. A fake model service on 127.0.0.1 gave a score for each result.
Each time is the median of 5 runs after 1 warm-up. jag got a made-up key
and a temporary `HOME`.

The first probe used a release build of `main` at daca59a, with all 12
CPUs. The `ignore` crate walks with 12 threads on this machine. The local
part: a typical search with a service that answers at once. The 5
searches are the searches of the measure (decision 8).

| Repository | Files | Lines | Size | jag | `rg -n` with the same term |
|---|---|---|---|---|---|
| tokio 1.53.3 | 867 | 192,901 | 6.1 MB | 0.025 to 0.030 s | 0.006 to 0.013 s |
| Django 5.2 | 6,881 | 1,089,232 | 44.7 MB | 0.043 to 0.051 s | 0.027 to 0.034 s |
| Go 1.25.0 | 14,490 | 3,475,476 | 133.8 MB | 0.059 to 0.092 s | 0.030 to 0.061 s |

Each of the 15 searches found more than 200 results (385 to 33,198),
scored 200, made 7 calls, and exited with 7.

The full search with a delay for each call (the search `error`, mean of 3
runs):

| Delay of each call | Django 5.2 | Go 1.25.0 |
|---|---|---|
| 0.5 s | 1.058 s | 1.133 s |
| 1.0 s | 2.064 s | 2.126 s |
| 2.0 s | 4.061 s | 4.119 s |
| 2.4 s | 4.860 s | 4.921 s |

So the time is two times the delay plus the local part.

The second probe used a release build of `main` at 157c8f2, with the form
`jag --ask QUESTION TERM PATH...` (ADR-0018). `taskset` gave jag 12, 4, 3,
2, or 1 CPUs, so the `ignore` crate walked with that count of threads. A
runner of GitHub has 2 CPUs (`ubuntu-latest` for a private repository), 4
CPUs (`ubuntu-latest` for a public repository), or 3 CPUs (`macos-latest`).
`taskset` sets the count of CPUs, not their speed. The test tree is the
tree of decision 7. A list is the paths of all files, relative to the
folder, with no hidden part, as a shell glob gives them.

| Search | 12 CPUs | 4 CPUs | 3 CPUs | 2 CPUs | 1 CPU |
|---|---|---|---|---|---|
| Go 1.25.0, one folder, the 5 searches | 0.059 to 0.103 s | 0.049 to 0.104 s | 0.054 to 0.111 s | - | - |
| Go 1.25.0, one folder, `error` | 0.103 s | 0.104 s | 0.111 s | 0.206 s | 0.252 s |
| Go 1.25.0, the list of its 14,462 files with no hidden part, `error` | 0.121 s | 0.120 s | 0.125 s | 0.156 s | 0.244 s |
| Test tree, one folder, `error` | 0.080 s | 0.063 s | 0.068 s | 0.103 s | 0.179 s |
| Test tree, the list of its 15,000 files, `error` | 0.082 s | 0.077 s | 0.080 s | 0.129 s | 0.153 s |

Each search exited with 7 and made 7 calls. The 5 searches found 3,051 to
33,198 results on Go, and 15,000 on the test tree. `rg -n error` took
0.068 s on the folder of Go and 0.059 s on its list, with 12 CPUs. A
clone of the tag `go1.25.0` (a git repository, so its `.gitignore`
applies) gave the same counts in 0.055 to 0.105 s. With a delay of 2 s
for each call, the search of the test tree took 4.08 to 4.10 s, as one
folder and as the list, with 12 and with 3 CPUs (the median of 3 runs).

The argument list of jag, with 8 bytes for each pointer: 678,840 bytes for
the list of Go, and 315,142 bytes for the list of the test tree. macOS
allows 1 MiB (1,048,576 bytes) for the arguments and the environment
together, and Linux allowed 2 MiB on the probe machine (`getconf
ARG_MAX`).

The test tree: 15,000 files of 9,509 bytes, 142.6 MB of data and 190.5 MB
on an ext4 disk. A Python script made it in 0.21 s.

The third probe compared a release build of `main` at e536cd0, which has
the size limit of FEATURE-09 (PR #167), with the build at 157c8f2. The two
builds ran in turns on the searches of the second probe: the 5 searches
on the folder of Go with 12 CPUs, `error` on the folder of Go with 3, 2,
and 1 CPUs, and `error` on the list of Go and on the test tree with 12,
2, and 1 CPUs. Each median of e536cd0 was within 0.01 s of the median of
157c8f2, from 0.062 to 0.161 s. With the default size limit of 20,000
characters, e536cd0 showed 17 to 49 of the 200 results, the error stream
gave the notice of the cost limit and the notice of the size limit, and
each search exited with 7. With 1 and 2 CPUs, this probe was faster than
the second probe, for example 0.161 s against 0.252 s on the folder of Go
with 1 CPU. Other jobs on the probe machine can cause such a difference,
so the budgets use the larger times of the second probe.

### The list of folders in `finish` (C-search, decision 11)

`src/search/walk.rs`:

```rust
/// Paths of the folders that the walk yielded.
folder_paths: Mutex<HashSet<PathBuf>>,
```

`visit_entry` inserts the path of each folder into the set. `finish` keeps
its code: `folder_paths.contains(&path)` is now a lookup in the set. The
counts do not change.

### One walk for all roots (C-search, decision 12)

`scan` keeps one `WalkBuilder` with all roots, as on `main`. It does not
start a walk for each root. The FEATURE-12 code (PR #160) put the order of
the command line in the one walk, and the FEATURE-14 spec keeps it (PR
#142, decision 6). `src/search/walk.rs` on `main`:

- The root of a file at depth `d` is `path.ancestors().nth(d)`, because
  the walk joins the name of each child to the path of its folder.
- `order_files` maps each root path to its places on the command line. A
  path that the user gives two times has two places. The walk gives its
  files two times, and the copies of one file under one root take the
  places of that root, one each, in order. The copies are equal, so the
  order of the threads does not change the report.
- `warning_place` gives a warning the place of the first root, in the
  order of the command line, that its path starts with
  (`Path::starts_with`). The depth of an error does not give its root: an
  error in the list of a folder has the path of the folder and the depth
  of its children. A warning with no path, or with a path under no root
  (for example an error of an ignore file above a root, which has an
  absolute path), takes place 0. So with one root, each warning has place
  0. With roots that overlap, for example `dir dir/sub`, a warning under
  `dir/sub` takes the place of `dir`.
- `finish(shared, roots)` sorts the files by place, then by path, and the
  warnings by place, then by path, then by message.

This feature does not change that code. Check (c) of decision 6 fails
when a change walks each root alone (Risk 6).

### The CI speed test (decisions 6 and 7)

`tests/speed.rs`:

```rust
//! The time of a typical search (STORY-44, CRITERION-65). Only a release
//! build runs it: `cargo test --release --test speed`.

/// The count of files of a medium code repository (decision 2).
const FILES: usize = 15_000;
const LINES: usize = 240;
/// The most time of the local part of a typical search (decision 4).
const LOCAL: Duration = Duration::from_secs(1);
/// The time of one call in check (b), and the target of CRITERION-65.
const CALL: Duration = Duration::from_secs(2);
const TARGET: Duration = Duration::from_secs(5);
const QUESTION: &str = "where does the code retry after an error";

#[test]
#[cfg_attr(debug_assertions, ignore = "a speed test runs only in a release build")]
fn a_typical_search_returns_in_under_5_seconds() { ... }
```

The test makes the tree one time in a `tempfile::TempDir`, with
`common::write`: 15,000 files and 142.6 MB. A line of a file is, for
example, `    let value_17 = add(value_16, 17);`. Line 120 is
`    return handle_error(value_119);`. Then the test runs the checks one
after the other, so that no check slows another. Each check measures the
time of `output()` of the `jag` command of `tests/common/mod.rs`, with the
fake server and a temporary `HOME`:

- (a) The server (`FakeServer::constant(0.9)`) gives each result the
  score 0.9 at once. jag runs `--ask QUESTION error <tree>` one time to
  warm up, then one more time. The second run takes less than `LOCAL` and
  exits with 7. The server gets 14 calls: 7 for each run.
- (b) A new server gives the score 0.9 after `CALL`
  (`Reply::scores(request, |_| 0.9).after(CALL)`). The same search takes
  less than `TARGET`, exits with 7, and makes 7 calls. `most_at_once()` is
  6.
- (c) A new server as in (a). jag runs `--ask QUESTION error` with the
  paths of the `FILES` files, relative to the tree, in the current folder
  of the tree. The search takes less than `LOCAL`, exits with 7, and makes
  7 calls.

The score 0.9 is more than `THRESHOLD` (0.4), so the exit code is 7 and
not 1. The size limit of FEATURE-09 (20,000 characters) leaves out most of
the 200 results, but the exit code stays 7, because the cost limit wins
over the size limit. Each failure message gives the time. When the test
ends, also after a failed check, the `TempDir` removes the tree.

`.github/workflows/ci.yml`, after `- run: cargo test --locked`:

```yaml
      - run: cargo test --locked --release --test speed
```

### The measure with the model service (decision 8)

`scripts/speed/measure.py`:

```
python3 scripts/speed/measure.py --jag PATH [--repo FOLDER] [--runs N]
```

- `--jag`: the jag program to measure. The README uses
  `target/release/jag`.
- `--repo`: a folder to search. A `--repo` that is not a folder stops the
  script with exit 2 before the first run. With no `--repo`, the script
  clones `https://github.com/golang/go.git` at the tag `go1.25.0` with
  `git clone --depth 1 --branch go1.25.0 --quiet` into a new temporary
  folder. Then `git rev-parse HEAD` in the clone must give the commit of
  decision 2, `6e676ab2b809d46623acb5988248d95d1eb7939c`. When it gives
  another commit, the script exits with 2 and names the two commits. The
  script removes the folder at the end, also after a failure.
- `--runs`: the timed runs of each search. The default is 5.

The searches, in this order:

| Term | Question |
|---|---|
| `error` | where does the code retry after an error |
| `default` | where are the default settings read |
| `close` | what happens when a connection closes |
| `lock` | where can two goroutines wait on the same lock |
| `path` | how is a relative path made absolute |

On Go 1.25.0, the searches find 3,051 (`close`) to 33,198 (`error`)
results (the probes), so the pre-rank runs and the model scores 200
results in 7 calls (decision 1).

For each search, the script runs `jag --ask QUESTION TERM FOLDER` one time
to warm up, then `--runs` times, one run at a time. It measures each run
with `time.perf_counter()` around `subprocess.run`, with standard input
from `/dev/null`, and standard output and the error stream in pipes. The
exit codes 0, 1, 7, and 8 are a search that ended (IF-4). Another code
stops the script with exit 2: it prints the code and the error stream of
jag, which never holds the key (CV-6).

The output is a table with one row for each search (the term, the
median, the slowest run, and the exit codes), then a verdict line, then
the result line:

```
search    median   slowest  exit codes
<term>    <m.mm> s <s.ss> s <codes>
...
Target under 5 s: <met | not met>.
<date>: jag <version> on <system> <machine>, <n> CPUs, Go 1.25.0 (<n> files): medians <a> to <b> s, slowest run <c> s.
```

The last line is the line for the README. `jag --version` gives the
version, `platform.system()` and `platform.machine()` give the system, and
`os.cpu_count()` gives the CPU count. The file count counts the files of
the folder with no `.git`, with a comma for each group of three digits:
`14,490` for the clone of Go 1.25.0. With `--repo`, the line gives the
name of the folder in the place of "Go 1.25.0".

The implementer checks the script with no key and no model service. The
stand-ins are short shell scripts in a temporary folder, not in the
repository. Checks 1 to 3 use `--repo .`. Each check sets `JAG_BASE_URL`
and `JAG_MAX_CHARS` in the environment of the script:

1. A stand-in that writes its arguments and the names of its `JAG_*`
   variables to a file, and exits with 7 at once: the script prints the
   table and exits with 0. The file shows 30 runs (5 searches, 6 runs
   each) of `--ask QUESTION TERM FOLDER` in the order of the table, and no
   `JAG_BASE_URL` and no `JAG_MAX_CHARS`.
2. A stand-in that sleeps 5.1 s and exits with 7, with `--runs 1`: the
   script exits with 1.
3. A stand-in that writes `jag: no credit` to the error stream and exits
   with 5: the script prints the code and that line, and exits with 2.
4. The stand-in of check 1 with no `--repo` and `--runs 1`: the script
   clones Go, the file shows the path of the clone as FOLDER, the result
   line names `Go 1.25.0 (14,490 files)`, and the temporary folder is gone
   after the end.

### The user step (decision 10)

The chores role guides the user, one step at a time, after the code merge,
in a clone of the repository on `main`:

```
$ git switch main && git pull --ff-only
$ docker run --rm --user "$(id -u):$(id -g)" -v "$PWD:/w" -w /w rust:1.99 cargo build --release --locked
$ target/release/jag --check
$ python3 scripts/speed/measure.py --jag target/release/jag
```

The machine of the user has no Rust tools, so the build runs in the
Docker image `rust:1.99`. On a machine with Rust, `cargo build --release
--locked` does the same. The measure clones Go 1.25.0: it downloads about
36 MB and uses about 170 MB of disk until its end. Checks: the build ends
with `Finished`, `jag --check` exits with 0, and the measure exits with 0
(`echo $?`). The user gives the last two lines of the output to chores.
Chores opens a docs pull request that adds the result line to the README
section "Speed". After its merge, the planner sets STORY-44 to `done`.

### Other features

- FEATURE-14 (spec on `main`, PR #142): its decision 6 keeps one walk for
  all paths and gives each file and each warning the place of its path on
  the command line, as the FEATURE-12 code does (decision 12). The measure
  and the speed test always give a path, so jag never reads standard input
  (its decision 4). A path that does not exist gives exit 2 (its decision
  7), and the measure stops with exit 2. In one walk, roots that overlap,
  for example `dir dir/sub`, still give the files of `dir/sub` two times,
  as in grep, because the `ignore` crate walks each root. The only extra
  time is a second search of the files of `dir/sub`. Risk 9 of FEATURE-14
  and Risk 6 of this spec name check (c) as their signal.
- FEATURE-12 (code on `main`, PR #160): the form `jag --ask QUESTION TERM
  PATH...` of ADR-0018, and the order of the command line in the one walk
  (decision 12). Plain grep keeps its second read in `read_snippets`
  (decision 14).
- FEATURE-13 (spec on `main`, PR #153; code in PR #170, not merged): `-q`,
  `-l`, and `-c` search all paths, as its spec says (decision 15).
- FEATURE-09 (code on `main`, PR #167): the size limit of 20,000
  characters leaves out most of the 200 results of a typical search. The
  exit code stays 7, because the cost limit wins over the size limit. The
  fit works on the scores in memory, with about log2(N) + 2 writes of the
  document and no file read (its decision 21). The third probe of "The
  evidence" saw no change of the time larger than 0.01 s. A search that
  reaches the size limit and not the cost limit exits with 8, and the
  measure accepts 8.
- FEATURE-21 (spec PR #143, not merged): the target and the measure use
  the direct route to the TypeSafe API. When the code of FEATURE-21 is on
  `main`, the measure also removes its settings of the model service, so
  that jag uses the direct route. A gateway adds its own time.
- FEATURE-19 (spec PR #154, not merged): it changes other workflow files,
  not `ci.yml`. It moves the code to a new public repository, where
  `ubuntu-latest` has 4 CPUs and not 2.
- FEATURE-15 (code on `main`): in a pipe, jag shows no progress and no
  color. The measure runs jag in a pipe. In a terminal, the progress line
  adds no model call and no file read.
- C-bench (jag-bench): it measures the quality of the results
  (CAPABILITY-12), not the time.

### The README section "Speed"

- A typical search returns in under 5 seconds: one search with a question,
  at the default limit of 200 scored results, on a code repository of
  about 15,000 files, while the model service works.
- jag finds the matches in one pass. On Go 1.25.0, with 14,490 files, that
  takes about 0.1 s on a machine with 12 CPUs. Then the model scores 200
  results in 7 calls, 6 at the same time, so the search waits for two
  rounds of calls. The calls take most of the time.
- A larger `--max-scored` needs one more round of calls for each 192
  results.
- A long list of files can be too long for the system. macOS allows 1 MiB
  for the arguments of a command and its environment, so a shell glob of
  a repository with about 1.5 times the files of Go can fail with
  `argument list too long`. Then give the folder in the place of the list.
- To measure jag on your machine, run
  `python3 scripts/speed/measure.py --jag PATH` in a clone of the
  repository, with the path of the jag that you use, for example
  `target/release/jag` after `cargo build --release --locked`. The script
  clones Go 1.25.0 (a download of about 36 MB) and runs 5 searches 6 times
  each: 30 searches. Each search costs at most about $0.02, so the measure
  costs at most about $0.60. jag finds your key as it always does. The
  script exits with 0 when each search returns in under 5 seconds.
- The dated result lines (decision 9).

### Acceptance criteria and their tests

- CRITERION-65 (a typical search returns in under 5 seconds):
  - Unit (`src/search/walk.rs`): `finish` gets `children` 50,000,
    `yielded` 40,000, 40,000 folder paths, and 4,000 error paths. 2,000 of
    the error paths are folder paths, and 2,000 are not. `ignored` is
    8,000, and `finish` takes less than 1 s in a debug build. On `main`,
    `finish` compares each of the 4,000 error paths with each of the
    40,000 folder paths: 160 million comparisons.
  - Release (`tests/speed.rs`), check (a): a typical search with a service
    that answers at once takes less than 1 s (decision 4).
  - Release, check (b): with a service that answers each call after 2 s,
    the search takes less than 5 s, with 7 calls and 6 at the same time.
    This is CRITERION-65 for a model that needs 2 s for each call.
  - Release, check (c): the tree as the list of its 15,000 files takes
    less than 1 s (decision 12).
  - The measure (step 6 of the Build order): the script exits with 0 on
    `main`, with the key of the user and the TypeSafe API. This is the
    proof of CRITERION-65. The CI checks keep it true after it.

## Build order

1. Decision 11: the `HashSet` in `finish`, and its unit test.
2. Decisions 6 and 7: `tests/speed.rs` and the CI step. If a change of
   `scan` on `main` walks each root alone, check (c) fails: give `scan`
   one walk again, as decision 12 says, in this step.
3. Decision 8: `scripts/speed/measure.py`, and its four checks with
   stand-ins. The pull request gives their output.
4. Decision 17: the README section "Speed", with no result line yet.
5. The code pull request merges.
6. The user step: the user runs the measure on `main`, and a docs pull
   request adds the result line. Then STORY-44 is done (decision 10).

`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and the
new speed step must pass on the CI runs of `ubuntu-latest` and
`macos-latest`.

## Assumptions

- The times of the probes come from one machine: Linux x86_64, 12 CPUs, a
  warm file cache. `taskset` limits the count of CPUs, not their speed,
  and a CI runner has other jobs. With 2 CPUs, jag searched the test tree
  in 0.103 s as one folder and in 0.129 s as the list, and with 1 CPU in
  0.179 s and 0.153 s. So the budget of 1 s of checks (a) and (c) is more
  than 7 times the times with 2 CPUs, and more than 5 times the times
  with 1 CPU. Check (b) leaves about 0.9 s: two rounds of 2 s take 4.0 s,
  so the local part has the same budget of 1 s as in check (a). A third
  round of calls makes check (b) fail (6 s). Source: the second probe of
  2026-10-14.
- A runner of GitHub for a private repository has 2 CPUs and 8 GB of
  memory on `ubuntu-latest`, and 3 CPUs (M1) and 7 GB on `macos-latest`.
  For a public repository, `ubuntu-latest` has 4 CPUs and 16 GB. Each
  runner has 14 GB of disk. This repository is private today. Source:
  GitHub Docs, "GitHub-hosted runners"
  (docs.github.com/en/actions/reference/runners/github-hosted-runners),
  read on 2026-10-14; `gh api repos/ai-ready-future/jag --jq .private`
  gave `true` on 2026-10-14.
- The `ignore` crate gives `min(CPUs, 12)` threads for `threads(0)`, and
  each walk starts its own workers. The count of CPUs comes from
  `std::thread::available_parallelism`, which on Linux counts only the
  CPUs that `taskset` allows. Source: ignore 0.4.33, `src/walk.rs`
  (`WalkParallel::threads` gives
  `available_parallelism().map_or(1, |n| n.get()).min(12)`, and
  `WalkParallel::visit`), read on 2026-10-14; the Rust docs of
  `std::thread::available_parallelism`, read on 2026-10-14 (on Linux, it
  reads the affinity mask with `sched_getaffinity()` and the cgroup
  quotas).
- macOS counts each argument and each variable of the environment, with
  its pointer of 8 bytes, against `ARG_MAX`, which is 1 MiB. Source: xnu
  12377.1.9 (the `main` branch of github.com/apple-oss-distributions/xnu),
  read on 2026-10-14: `bsd/sys/syslimits.h` (`ARG_MAX` is
  `(1024 * 1024)` on macOS), `bsd/sys/param.h` (`NCARGS` is `ARG_MAX`),
  and `bsd/kern/kern_exec.c` (`exec_extract_strings`: "we charge the
  string length and argv[]/envv[] pointer slot to ip_argspace"; the space
  starts at `NCARGS`, and no space gives `E2BIG`).
- The walk joins the name of each child to the path of its folder, and a
  root has depth 0. So `entry.path().ancestors().nth(entry.depth())` is the
  root of a file. An error in the list of a folder does not follow this
  rule: it has the path of the folder and the depth of its children.
  Source: ignore 0.4.33, `src/walk.rs` (`DirEntryRaw::from_entry` takes
  the path of the `std::fs::DirEntry`, which is the folder joined with the
  name; `Work::read_dir` gives such an error `with_path(self.dent.path())`
  and `with_depth(self.dent.depth() + 1)`).
- The fake server of the tests adds no delay of its own, because it closes
  each connection after its reply. Source: the first probe of 2026-10-14 (a
  server that writes as `tests/common/fake_server.rs` does gave 26.6 ms
  for a typical search on tokio, against 21.1 ms for a server that sends
  each reply with no delay).
- `cargo test --release --test speed` builds the binary of jag in the same
  release build as the test, and `CARGO_BIN_EXE_jag` gives its path, so
  the test runs the release binary. Source: `tests/common/mod.rs`
  (`assert_cmd::cargo::cargo_bin!("jag")`); the Cargo docs ("Environment
  variables Cargo sets for crates", `CARGO_BIN_EXE_<name>`; `cargo test`:
  "Binary targets are automatically built if there is an integration test
  ... being selected").
- The machine of the user has Python 3, `git`, Docker, network access to
  GitHub and to the TypeSafe API, and about 200 MB of free disk for the
  clone of Go. Source: `scripts/adr/` (the
  repository needs Python 3 for its ADR scripts); CRITERION-65 ("with a
  working service"); a check on 2026-10-14: the `docker run` command of
  the user step built `main` (daca59a) in 32 s, and `target/release/jag
  --version` gave `jag 0.1.0` on the host.
- A typical search sends far fewer tokens than the rate limit of the
  service (100K tokens each second): 200 results of 7 to 12 lines of
  about 41 bytes are about 100,000 bytes, or about 33,000 tokens by the
  estimate of the FEATURE-04 spec (bytes divided by 3). Source:
  system-design.md strategy-1; FEATURE-04 spec decision 22.

## Risks

1. One model call takes more than about 2.4 s, so the measure fails
   (QS-3, risk M). Signal: the measure exits with 1. Options, each with
   its own decision: (a) one round of calls, with 7 calls at the same
   time or 6 calls of 34, which changes ADR-0002 and needs a new ADR; (b)
   open the connections to the service while the walk runs, so that the
   first round does not wait to open a secure connection; (c) a smaller
   default cost limit, which changes DECISION-12 and is a decision of the
   user.
2. The network path to the service is different for each user. Signal: an
   issue says that a search takes more than 5 s. Mitigation: the script is
   in the repository, so the user can run it and put its line in the
   issue.
3. A CI runner is slow or busy. Signal: a check of `tests/speed.rs` fails
   only some of the time. Mitigation: the budget of checks (a) and (c) is
   more than 7 times the times of the second probe with 2 CPUs, check (b)
   leaves about 0.9 s for the local part, the checks run one after the
   other, and check (a) warms up the file cache first.
4. The lines of the test tree are simpler than real code, so the search
   and the pre-rank can be faster on them. Signal: jag on a real
   repository of the same size, with a service that answers at once,
   takes much more time than check (a). Mitigation: the probes give the
   times on Go 1.25.0 with 1 to 12 CPUs (0.049 to 0.252 s), and the budget
   of 1 s is about 4 times the slowest of them or more.
5. A large file with a match is read whole two times: by the scan and by
   `read_snippets`. Signal: a slow search in a repository with a large
   generated file, for example a log or a data file. Mitigation: the user
   gives the folders of the code, or puts the file in an ignore file.
6. A later change of `scan` walks each root alone, for example in the
   code of FEATURE-14, against decision 6 of its spec. Signal: check (c)
   fails on its pull request. Mitigation: decision 12.
7. The release build and the test tree make each CI run longer. Signal:
   the CI job takes some minutes more, or a runner has no free disk.
   Mitigation: none for the build now. `actions/cache` saves only a new
   key, and the key changes only with `Cargo.lock` or
   `rust-toolchain.toml`, so each run builds the release crates again until
   one of them changes. Then the cache keeps `target/release`. The test
   tree has 142.6 MB of data, about 190 MB of the 14 GB of disk of each
   runner, and the test removes it at its end. On the probe machine, a
   script made the tree in 0.21 s, and the three checks take about 5 s,
   most of it the 4 s of check (b).
8. The service gives HTTP 429 to a search of the measure, and the retry
   gets HTTP 429 again, so jag exits with 6 (CV-5). Signal: the measure
   stops with exit 2 and shows the error of jag. Mitigation: the measure
   runs one search at a time, and a typical search sends about a third of
   the token limit of each second. The user runs the measure again later.
9. A long list of files passes the limit of the system for the arguments
   of a command. macOS allows 1 MiB for the arguments and the environment
   together. The arguments of jag with the list of the 14,462 files of Go
   took 678,840 bytes with their pointers, so a shell glob of a repository
   with about 1.5 times the files of Go, and paths as long, passes the
   limit. Linux allowed 2 MiB on the probe machine. Signal: the shell says
   `argument list too long`, and jag does not start. Mitigation: the user
   gives the folder in the place of the list, and the README section
   "Speed" says so. jag cannot change this limit. The list of check (c)
   takes 315,142 bytes, so the check stays under the limit on macOS.

## Out of scope

- A store of scores between searches (DECISION-46) and a search index
  (SCOPE-06).
- A change to the calls of ADR-0002: Risk 1.
- The time through an AI gateway: FEATURE-21.
- The first search after a start of the machine, when the files are not
  in the file cache (decision 1).
- An early stop of `-q` and `-l` (decision 15).
- A time target for plain grep: no PRD page gives one (decision 14).
- The time on Windows: the CI runs on Linux and macOS only.
