---
title: "Search a folder as grep -r does: parallel folder walk, noise skip, and skip count"
feature: FEATURE-05
stories: [STORY-16, STORY-17]
kind: build
lifecycle_state: draft
inherits: [QS-3, C-search, CV-3]
session: 21508ce9-788b-40ed-be8d-03797016b20b
---

# Search a folder as grep -r does: parallel folder walk, noise skip, and skip count

## Problem

jag searches each subfolder of a folder, skips binary, hidden, and ignored
files, and says how many files it skipped (FEATURE-05, DECISION-14,
CAPABILITY-02).

- STORY-16: a shell power user gives a folder, and jag searches each file in
  that folder and in each subfolder, as `grep -r` does (CRITERION-22).
- STORY-17: jag skips binary, hidden, and ignored files (CRITERION-23). The
  folder walk counts the files it skipped and gives the notice text
  (CRITERION-24). FEATURE-04 shows the notice to the user (CRITERION-85).

This spec builds a library in the `search` module (C-search). It adds no
command-line option. FEATURE-04 connects it to `jag TERM "QUESTION"
[PATH...]`.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Match and walk code | jag links the ripgrep libraries into its binary: `ignore` walks the folders, `grep-searcher` reads each file and finds binary data, and `grep-regex` gives the matcher. `grep-matcher` gives the `Matcher` trait that these crates share. jag never starts the `rg` program or any other program | 0001-find-matches-with-the-ripgrep-libraries.md |
| 2 | What jag searches and skips | jag searches each subfolder. It skips binary files (files with binary data), hidden files, and ignored files | DECISION-14 |
| 3 | Skip rules | The default filters of the `ignore` crate, as in ripgrep: `.gitignore` (inside a git repository), `.ignore`, `.git/info/exclude`, the global git ignore file, and the ignore files of parent folders. Hidden means a name that starts with `.`. jag follows a symbolic link that the user gives as a root. It does not follow a symbolic link inside the walk. `grep -r` does the same | 0001-find-matches-with-the-ripgrep-libraries.md; grep -r; human:404pilo, 2026-10-13 (Q-g) |
| 4 | Notice stream | In text mode, the caller writes the skip notice to the error stream. The structured field comes with FEATURE-11 | system-design.md CV-3 |
| 5 | Code form | A module of C-search in the one Cargo package | 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md (PR #5) |
| 6 | Read errors | A file or folder that jag cannot read gives a warning, and the walk goes on, as `grep -r`. It does not count as skipped. The caller writes the warning to the error stream | grep -r; human:404pilo, 2026-10-13 (Q-g) |
| 7 | Skip count unit | jag counts each entry where it skips it. A skipped folder counts as 1. jag does not walk into a skipped folder to count its files | human:404pilo, 2026-10-13 (Q-h) |
| 8 | Notice text | `jag: skipped N files and folders (B binary, H hidden, I ignored)`. No notice when N is 0 | human:404pilo, 2026-10-13 (Q-h) |
| 9 | Feature border | FEATURE-05 builds the walk, the search of each file, the skip count, and the notice text as a library. FEATURE-04 uses it and shows the notice | human:404pilo, 2026-10-13 (Q-i); roadmap PR #6 |
| 10 | Walk order | Parallel walk with the `ignore` crate (`WalkBuilder::build_parallel`). The order in which files are read is not fixed | human:404pilo, 2026-10-13 (Q-j) |
| 11 | Order of the scan result | `scan` sorts its result by path (byte order of the path), then by line number, before it returns | This spec, from Q-j: see the reason below |

Reason for decision 11: the parallel walk (decision 10) gives the files in a
new order on each run. Without a sort, two results with the same score can
change places from one run to the next. Then the output of the same search
changes. An agent that runs a search two times must get the same output.
The sort costs little: the scan keeps only matching lines, and FEATURE-06
limits how many are scored. A later feature that streams lines in walk order,
for example FEATURE-12 plain grep, must sort them or say that the order is
not fixed.

## Design

### Files

```
src/search.rs           pub mod walk; re-exports scan, ScanReport, SkipCounts
src/search/walk.rs      the parallel walk, the per-file search, the counts
tests/walk.rs           tests over real folders in tempfile
```

`Cargo.toml` adds `ignore`, `grep-searcher`, `grep-regex`, and
`grep-matcher`, at the latest stable version of each on crates.io when the
implementer starts. These four crates are the ripgrep libraries of
decision 1.

ADR-0001 names `ignore`, `grep-searcher`, and `grep-regex`. ADR-0012 says
that each new outside crate needs a reason in the pull request. The reason
for `grep-matcher`: it holds the `Matcher` trait. `grep-regex` implements
that trait, and `grep-searcher` takes it. `scan` names the trait in its
signature, so that FEATURE-04 can give any matcher. The crate comes from
the same ripgrep source as the other three. `grep-searcher` and
`grep-regex` already depend on it, so it adds no new code to the build.
The pull request of the FEATURE-05 build repeats this reason.

### Public interface

```rust
pub struct ScanOptions {
    pub threads: usize,          // 0: the ignore crate picks the count
    pub global_ignore: bool,     // read the global git ignore file
    pub parent_ignore: bool,     // read the ignore files of parent folders
}

impl Default for ScanOptions {    // threads 0, global_ignore and parent_ignore true
    fn default() -> Self;
}

pub struct MatchLine {
    pub line_number: u64,        // 1-based
    pub text: Vec<u8>,           // the line, with no line terminator
}

pub struct FileMatches {
    pub path: PathBuf,           // as the walk gives it: root joined with the rest
    pub lines: Vec<MatchLine>,   // sorted by line_number
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct SkipCounts { pub binary: u64, pub hidden: u64, pub ignored: u64 }

impl SkipCounts {
    pub fn total(&self) -> u64;
    pub fn notice(&self) -> Option<String>;   // decision 8
}

pub enum WarningKind {
    Read(std::io::ErrorKind),    // an error with an io::ErrorKind
    IgnoreRule,                  // a bad rule or line in an ignore file
    Other,                       // any other error of the ignore crate
}

pub struct WalkWarning {
    pub path: Option<PathBuf>,   // None when the error has no path
    pub kind: WarningKind,
}

impl WalkWarning {
    pub fn message(&self) -> String;          // see "Warnings"
}

pub struct ScanReport {
    pub files: Vec<FileMatches>,   // only files with a match; sorted (decision 11)
    pub skipped: SkipCounts,
    pub warnings: Vec<WalkWarning>, // by path (None first), then by message()
}

pub fn scan<M>(roots: &[PathBuf], matcher: &M, options: &ScanOptions) -> ScanReport
where M: grep_matcher::Matcher + Sync;
```

FEATURE-04 builds the matcher from the term (DECISION-33) and turns
`FileMatches` into results with the lines around them (DECISION-10). This
spec needs only the matching lines.

### The walk

1. Build one `ignore::WalkBuilder` with each root of `roots`. Keep the
   default filters (decision 3). Set `follow_links(false)` and
   `threads(options.threads)`. Set `git_global(options.global_ignore)` and
   `parents(options.parent_ignore)`.
2. FEATURE-04 uses `ScanOptions::default()`, so the two ignore options stay
   on, as decision 3 says. Only the tests turn them off.
3. Run `build_parallel().run(...)`. Each thread has its own visitor, its own
   `grep_searcher::Searcher`, and its own buffers. The threads share five
   counters of type `AtomicU64`: `binary`, `hidden_children`, `children`,
   `yielded`, and `yielded_hidden`. They also share three `Mutex<Vec<_>>`:
   the files, the warnings, and the error paths of "The skip count".
4. For each `Ok(entry)`:
   - When `entry.error()` is `Some`, add a warning for that error (see
     "Warnings"). This is an error in an ignore file of the folder, for
     example a bad glob. The walk uses the other rules of the file.
   - A folder: count its children (see "The skip count"), then go on.
   - A regular file (`file_type().is_file()`): search it (see "The search of
     one file").
   - Any other type, for example a symbolic link below a root: do nothing.
5. For each `Err(error)`: add a warning for the error (see "Warnings"), then
   go on (decision 6).
6. After the walk, sort `files` (decision 11). Sort `warnings` by path, then
   by `message()`, so that the order is fixed.

The `ignore` crate reads each root with `fs::metadata`, which follows a
symbolic link (0.4.33, `DirEntryRaw::from_path`). So a root that is a link
to a file has the file type, and jag searches it. A root that is a link to
a folder has the folder type, and the walk goes into it. This agrees with
`grep -r`, which follows a link that the user names. The crate yields each
root with no filter.

### Warnings

jag makes a `WalkWarning` from an `ignore::Error` as follows.

`path`: jag goes down through `WithPath`, `WithDepth`, and
`WithLineNumber`. It also goes into a `Partial` that holds one error. The
path is the path of the first `WithPath` that it finds. For `Error::Loop`,
the path is the `child` path. Else the path is `None`.

A `Partial` with two or more errors has no path, unless a `WithPath` holds
the `Partial`. In that case the warning has no path. This is a rare case:
one ignore file with two or more bad rules.

`kind`, in this order:

1. `Read(kind)` when `error.io_error()` gives an `io::Error`.
2. `IgnoreRule` when `error.is_partial()` is true.
3. `IgnoreRule` when jag finds `WithLineNumber`, `Glob`,
   `UnrecognizedFileType`, or `InvalidDefinition` as it goes down.
4. `Other` in all other cases, for example for `Loop`.

`message()` gives one line. It holds no error text from the crate, so the
warning stays short and the same on each platform:

| kind | path | message |
|---|---|---|
| `Read(k)` | `Some(p)` | `jag: cannot read <p>: <k>` |
| `Read(k)` | `None` | `jag: cannot read a file: <k>` |
| `IgnoreRule` | `Some(p)` | `jag: a rule in <p> is not valid; jag uses the other rules` |
| `IgnoreRule` | `None` | `jag: an ignore rule is not valid; jag uses the other rules` |
| `Other` | `Some(p)` | `jag: cannot walk <p>` |
| `Other` | `None` | `jag: cannot walk a folder` |

`<k>` is the `Display` text of `io::ErrorKind`, for example
`permission denied`.

With `parent_ignore` off, the crate can still read the ignore files of
parent folders, because the git options are on. A bad rule in such a file
can then give an `IgnoreRule` warning. The tests do not depend on it,
because no parent of a temporary folder holds a bad rule.

### The search of one file

1. Configure the searcher with `BinaryDetection::quit(b'\0')` and line
   numbers on.
2. Search the file with a sink that keeps each matching line in a local
   `Vec<MatchLine>`.
3. When the sink gets `binary_data`, drop the local lines, add 1 to
   `binary`, and stop the search of this file. The searcher can report
   matches before it finds the binary byte, so jag drops them. A file with
   binary data gives no match.
4. An open or read error gives a `WalkWarning` with `Read(kind)` and the
   path. The file gives no match.
5. A file with one match or more goes into `files`.

### The skip count

The `ignore` crate skips a hidden or ignored entry and does not report it.
jag counts it as follows:

1. For each folder that the walk yields, read its children with
   `std::fs::read_dir`. Add the count of children to `children`. Add the
   count of children whose name starts with `.` to `hidden_children`. Also
   keep the path of each yielded folder.
2. For each `Ok` entry that the walk yields at depth 1 or more, add 1 to
   `yielded`. When its name starts with `.`, also add 1 to
   `yielded_hidden`.
3. For each `Err` with a path and with `error.depth()` of 1 or more, keep
   its path. Do not add to a counter yet.
4. After the walk, look at each kept `Err` path that is not the path of a
   yielded folder. For each one, add 1 to `yielded`. When its name starts
   with `.`, also add 1 to `yielded_hidden`.
5. Then, with `saturating_sub` for each subtraction:
   - `hidden = hidden_children - yielded_hidden`
   - `ignored = children - yielded - hidden`
6. A failure of `read_dir` in step 1 adds no children.

Reason for `yielded_hidden`: an ignore rule can whitelist a hidden child,
for example `!.github/` in `.ignore`. The crate then yields that child
(0.4.33, `Ignore::matched_dir_entry`). Step 1 counts it as hidden, so step
2 takes it out again. Without this, `hidden` is 1 too high and `ignored`
is 1 too low.

Reason for steps 3 and 4: the crate gives 2 items for a folder that it
cannot read (0.4.33, `Worker::run_one`). First it gives `Ok` for the
folder. Then it gives `Err` with the same path and the same depth. The
parent lists that folder as 1 child, so jag must count it 1 time.

- An error of a folder read has the path of a yielded folder. Step 4 does
  not count it.
- An error from a child that the crate cannot stat
  (`DirEntryRaw::from_entry`) has the path of the child. No `Ok` comes for
  that child, so step 4 counts it.
- An error from one bad item of a folder list has the path of the parent
  folder. Step 4 does not count it. The `read_dir` of step 1 also gets no
  child for that item.
- Errors with no depth or no path come from parent ignore files or from
  roots. They are not children.

These steps use paths, not the order of the items. So they do not depend
on the walk order or on the thread that sees each item.

A skipped folder counts as 1, because jag does not read its children
(decision 7). A file with binary data counts in `binary` only. It was
yielded, so it is not in `ignored`.

Step 1 reads the list of each folder a second time. The `ignore` crate
gives no call for a skipped entry, and its `filter_entry` runs after the
skip. This is OK for this spec, because a folder list is small next to the
file reads. jag still reads each file one time. FEATURE-16 measures the
cost on a large folder and can change it.

### Notice text

`SkipCounts::notice` returns `None` when `total()` is 0. Else it returns:

```
jag: skipped 14 files and folders (3 binary, 4 hidden, 7 ignored)
```

Each count is shown, also a count of 0. The text has no path and no file
text.

### Acceptance criteria and their tests

All tests build a folder in a `tempfile::TempDir`. A test that needs
`.gitignore` creates an empty `.git` folder at the top. Then the `ignore`
crate sees the folder as a git repository. No test calls the `git`
program. Each test calls `scan` with `global_ignore: false` and
`parent_ignore: false`. Then the global git ignore file of the developer
does not change the counts. The ignore files above the temporary folder
also do not change them, for example when `TMPDIR` is in a git
repository. One more test proves that
`ScanOptions::default()` has both options on. No test changes `HOME` or
another variable of the process. Tests compare paths after the sort of decision 11, or as sets. No
test depends on the walk order.

| Criterion | Test |
|---|---|
| CRITERION-22 jag searches each subfolder | `tests/walk.rs`: a tree `a.txt`, `x/b.txt`, `x/y/c.txt`, `x/y/z/d.txt`, each with the word `needle`. `scan` with the root and the matcher for `needle` returns the 4 files, each with line 1, and `skipped.total()` is 0. |
| CRITERION-23 jag skips noise files | One tree with: `.hidden.txt` and `.cache/e.txt` (hidden); `.git/` and `.gitignore` that names `build/` and `*.log`; `build/f.txt` and `g.log` (ignored); `bin.dat` with `needle` and then a NUL byte (binary); `ok.txt` (searched). Each file holds `needle`. `scan` returns only `ok.txt`. |
| CRITERION-24 The folder walk counts the files it skipped | The tree of CRITERION-23 gives `SkipCounts { binary: 1, hidden: 4, ignored: 2 }`: hidden counts `.hidden.txt`, `.cache`, `.git`, and `.gitignore`; ignored counts `build` and `g.log`. `notice()` is `Some("jag: skipped 7 files and folders (1 binary, 4 hidden, 2 ignored)")`. Unit: `SkipCounts::default().notice()` is `None`. |

More tests:

- A skipped folder with 50 files counts as 1 (decision 7).
- `.ignore` works with no `.git` folder.
- A NUL byte after 100 KB of text, after a matching line: the file gives
  no match and counts as binary.
- A tree with `.git/`, `.gitignore` that names `*.log`, `g.log`, `ok.txt`,
  and a subfolder `locked/` with mode `0o000`: one `WalkWarning` with
  `Read(PermissionDenied)` and the path of `locked`, `ok.txt` is searched,
  and `skipped` is `SkipCounts { binary: 0, hidden: 2, ignored: 1 }`
  (hidden: `.git` and `.gitignore`; ignored: `g.log`). `locked` is not in
  `skipped`. Skip this test when the tests run as root.
- Whitelisted hidden folder: a tree with `.ignore` that holds `*.tmp` and
  `!.github/`, a folder `.github/` with `w.yml`, `.env`, `t.tmp`, and
  `ok.txt`. `.github/w.yml` and `ok.txt` are searched. `skipped` is
  `SkipCounts { binary: 0, hidden: 2, ignored: 1 }` (hidden: `.ignore` and
  `.env`; ignored: `t.tmp`).
- Two warnings with the same path come in the order of `message()`.
- A bad glob in `.ignore` (for example `a[`) gives one `WalkWarning` with
  `IgnoreRule` and the path of that `.ignore`, and the walk goes on.
- `WalkWarning::message` gives the text of the table in "Warnings" for each
  of the six rows.
- A symbolic link below the root to a file with `needle` is not searched and
  is not in `skipped`.
- A root that is a file is searched.
- A root that is a symbolic link to a file is searched.
- A root that is a symbolic link to a folder: each file in the target folder
  is searched, and each result path starts with the root path as the user
  gave it.
- Order: `scan` on a tree of 200 files with 8 threads, 20 times, gives the
  same `ScanReport` each time (decision 11).

## Build order

1. Add the four crates. Add `src/search.rs` and `src/search/walk.rs` with
   the types and an empty `scan`.
2. The parallel walk and the search of one file, with the CRITERION-22 test
   (STORY-16).
3. Binary detection and the hidden and ignored skip, with the CRITERION-23
   test (STORY-17).
4. The skip count and the notice text, with the CRITERION-24 test
   (STORY-17).
5. The sort of the result, the warnings, and the other tests.

This spec needs the project of the FEATURE-01 spec
(`docs/superpowers/specs/2026-10-13-STORY-01-store-a-key-one-time-design.md`).
Build it after that project is on `main`.

## Assumptions

- The skip rules are the defaults of the `ignore` crate, so `.gitignore`
  works only inside a git repository, as in ripgrep. `.ignore` works in
  each folder.
- A symbolic link below a root is not searched and is not counted. `grep -r`
  also skips it with no message. A symbolic link that the user names as a
  root is followed, as `grep -r` does.
- A name that starts with `.` is hidden. jag does not read the hidden flag
  of macOS.
- `scan` reads each file one time. Binary detection happens in that read,
  so jag does not read a file two times.

## Risks

- The skip count is wrong when files change during the walk. Signal: an
  `ignored` count that changes between two runs on a folder that does not
  change. Mitigation: `saturating_sub` keeps the count at 0 or more; the
  count is a notice and never changes the exit code.
- A later version of the `ignore` crate skips a child for a reason that is
  not hidden and not an ignore rule. Signal: the CRITERION-24 test fails
  after a crate update. Mitigation: the test pins the counts of a fixed
  tree.
- The sort of decision 11 needs all matches in memory before the first
  output. Signal: FEATURE-16 measures a slow first line on a large folder.
  Mitigation: FEATURE-06 limits the scored results, and FEATURE-16 can
  measure and change the sort.
- The term follows the pattern rules of `rg`, not of grep (RK-1). This spec
  takes the matcher from the caller and does not change that risk.

## Out of scope

- The command line, the term, and the matcher: FEATURE-04.
- The lines around a match, and joined results: FEATURE-04 and FEATURE-08.
- Writing the notice and the warnings to the error stream, and the
  end-to-end notice test (CRITERION-85): FEATURE-04.
- The form of the result path (DECISION-15): FEATURE-08.
- A mix of files and folders, a path that does not exist, and piped text:
  FEATURE-14.
- The structured skip field: FEATURE-11.
- Plain grep output in a fixed order: FEATURE-12.
- Speed measurement and tuning of the walk: FEATURE-16.
