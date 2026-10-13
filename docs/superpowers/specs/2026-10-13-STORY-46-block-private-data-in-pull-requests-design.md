---
title: "Block private data in the repository: the private-data check on each pull request"
feature: FEATURE-17
stories: [STORY-46]
kind: build
lifecycle_state: draft
session: 21508ce9-788b-40ed-be8d-03797016b20b
---

# Block private data in the repository: the private-data check on each pull request

## Problem

A check on each pull request fails when the change adds a key, case content,
private evaluation data, or an internal project name (FEATURE-17,
CONSTRAINT-01).

- STORY-46: an agent builder wants each pull request checked for private
  data, so that the public repository never holds keys, case content,
  private evaluation data, or internal project names.
  - CRITERION-74: a pull request that adds private data fails the check, and
    the pull request cannot merge.
  - CRITERION-75: for each finding, the check names the file and the line,
    and it never shows the found value in its output.

The check must find internal names, but the repository must not hold those
names. So the list of private words and patterns is not a file in the
repository.

FEATURE-17 merges before the FEATURE-01 code (roadmap PR #6), so the
repository has no Rust project when this check lands.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Where the pattern list lives | The GitHub Actions secret `PRIVATE_PATTERNS`. One pattern on each line | human:404pilo, 2026-10-13 (Q-r) |
| 2 | Scanner | gitleaks, a pinned release binary, with its default rules for keys and one added rule for each private pattern. `--redact` hides each found value. Decision 13 puts the two rule sets in two configs | human:404pilo, 2026-10-13 (Q-s) |
| 3 | Workflow file | Separate files: `.github/workflows/private-data.yml` for the check and `.github/workflows/private-data-tests.yml` for its tests. FEATURE-01 adds `ci.yml` as its spec says and does not change these files | human:404pilo, 2026-10-13 (Q-t); two files from PR #14 review round 3: a skipped job reports success |
| 4 | What a pull request scan covers | The tree of the checkout, each commit of the PR, and the PR title, the PR body, and the commit messages. The tree is the merge commit of the PR and `main` (`refs/pull/<number>/merge`), not the PR head. This is correct: it is the tree that the merge gives. Do not change it to `HEAD_SHA` | human:404pilo, 2026-10-13 (Q-t) |
| 5 | Output | Only the place (file and line, or commit, or "PR text") and a rule id such as `private-pattern-3`. Never the value, never the pattern | CRITERION-75; human:404pilo, 2026-10-13 (Q-t) |
| 6 | No secret | With no `PRIVATE_PATTERNS`, the check fails | human:404pilo, 2026-10-13 (Q-t) |
| 7 | Language and tests | A bash script and a bash test script. Test data is invented and made at test time | human:404pilo, 2026-10-13 (Q-t) |
| 8 | Merge block | A chores task makes the check a required status check for `main`. The user gives the list to chores with `gh secret set PRIVATE_PATTERNS`. A test PR proves that a failing `private-data` run blocks the merge. Before the repository goes public, chores sets an Actions event policy that allows `pull_request_target` | human:404pilo, 2026-10-13 (Q-t) |
| 9 | CI system | GitHub Actions on each pull request | system-design.md, Deployment, release path (deployment-3) |
| 10 | gitleaks version | `v8.30.1` (latest release on 2026-10-13). The workflow holds the SHA-256 of `gitleaks_8.30.1_linux_x64.tar.gz` as a fixed value: `551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb`. It does not trust a checksums file from the same release | This spec, from Q-s and PR #14 review round 2 |
| 11 | Pattern form | Each non-blank line is a Go regular expression (RE2). The check adds `(?i)`, so a match ignores case. A line that starts with `#` is a comment. The check removes a `\r` and the spaces at the end of a line | This spec, from Q-r |
| 12 | No way to hide a finding | The check refuses a `.gitleaksignore` file in the tree, runs gitleaks with `--ignore-gitleaks-allow`, and uses only its own configs with `-c` | This spec, from Q-s: see "Ways to hide a finding" |
| 13 | Two configs | `keys.toml` extends the default config. `private.toml` holds only the private rules, with no `[extend]` and no allowlist. The global allowlist of the default config skips `.svg`, `.png`, `.pdf`, `package-lock.json`, `vendor/`, and more, and it applies to each rule of a config that extends it | This spec, from PR #14 review round 1 (verifier test with gitleaks v8.30.1) |
| 14 | No gitleaks output in the log | The check checks each pattern by itself before the scans and reports a bad one by its line number only. It sends all gitleaks output to the private folder and prints only "gitleaks failed (exit N)" | This spec, from PR #14 review round 1: a bad RE2 pattern makes gitleaks panic and print the pattern |
| 15 | Trust model | The `private-data` job runs on `pull_request_target`. The workflow, `check.sh`, and the gitleaks configs come from `main`. The PR files are data only: the job never runs, sources, or uses as config a file of the PR. The PR tree goes to a separate folder, and no PR file is in the folder that the job runs from. `permissions: contents: read`, and no step gives a token to PR files. A change to the check takes effect only after it merges | human:404pilo, 2026-10-13 (Q-v) |
| 16 | Output for a fork pull request | When the pull request comes from a fork, the output is only "private-data: private data found": no file, no line, no rule id, and no count. This is an exception to CRITERION-75 for forks only. A fork author can make many pull requests to guess a private word, and the place of each finding helps that guess | human:404pilo, 2026-10-13 (Q-ac) |

## Design

### Files

```
.github/workflows/private-data.yml        the check, on pull_request_target and push
.github/workflows/private-data-tests.yml  the tests, on pull_request
scripts/private-data/check.sh     the check
scripts/private-data/test.sh      the tests of the check
scripts/private-data/README.md    how to run the check on your machine
```

### Workflow

`.github/workflows/private-data.yml`:

- Triggers:
  - `pull_request_target` with the types `opened`, `synchronize`,
    `reopened`, and `edited` (a new title or body runs the check again).
  - `push` to `main`.
- `permissions: contents: read`. The workflow uses no cache.
- Each `actions/checkout` step is pinned to v7.0.1 by its full commit SHA,
  with the version in a comment. v7.0.1 has the `allow-unsafe-pr-checkout`
  input:
  `uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1`
- Job `jobs.private-data`, with `name: private-data`, on `ubuntu-latest`.
  The job has no `if:`. GitHub reports a skipped job as a success, so a
  skipped required check does not block a merge. The check-run name `private-data` is the context of the required check
  of decision 8. Do not change the job id or the name later, because the
  ruleset of `main` names it.
  1. `actions/checkout` with no `ref`, `path: base`, and
     `persist-credentials: false`. On `pull_request_target`, this is the
     code of `main`. On `push`, it is the pushed commit of `main`.
     `check.sh` comes from this folder only.
  2. On `pull_request_target` only: `actions/checkout` with
     `ref: refs/pull/<number>/merge`, `path: pr`, `fetch-depth: 0`, and
     `persist-credentials: false`. No step runs a file of `pr`, sources
     it, or reads it as config. Git and gitleaks only read it as data.
     The token of the checkout is not kept in `pr/.git/config`. Leave
     `lfs` at its default, `false`.
     For a pull request from a fork, `actions/checkout` refuses this ref
     on `pull_request_target`. Its pattern
     `^refs/pull/[0-9]+/(?:head|merge)$` covers `/merge` too, and only
     `allow-unsafe-pr-checkout: true` turns the refusal off (actions/checkout,
     `src/unsafe-pr-checkout-helper.ts`, `assertSafePrCheckout`). The
     workflow sets `allow-unsafe-pr-checkout: true` on this step only.
     This is safe because no step runs a file of `pr` (decision 15).
  3. Download `gitleaks_8.30.1_linux_x64.tar.gz` from the gitleaks release
     to a folder outside `base` and `pr`. Check it with `sha256sum -c`
     against the fixed SHA-256 of decision 10, which the workflow holds in
     its text. Stop on a mismatch.
  4. Run `base/scripts/private-data/check.sh`. Give each input through
     `env:` and never through `${{ }}` inside the `run:` text, because a PR
     title can hold shell text:
     - `PRIVATE_PATTERNS: ${{ secrets.PRIVATE_PATTERNS }}`
     - `PR_TITLE: ${{ github.event.pull_request.title }}`
     - `PR_BODY: ${{ github.event.pull_request.body }}`
     - `BASE_SHA: ${{ github.event.pull_request.base.sha }}`
     - `HEAD_SHA: ${{ github.event.pull_request.head.sha }}`
     - `GITLEAKS: <path of the binary>`
     - `REPO_DIR`: `pr` on `pull_request_target`, `base` on `push`.
     - `FORK_PR: ${{ github.event_name == 'pull_request_target' && github.event.pull_request.head.repo.id != github.repository_id }}`.
       It is `true` for a fork pull request. When the fork is deleted, the
       head repository is empty, and the value is also `true`. On `push`,
       it is `false`, so the push run on `main` prints the place of each
       finding.
     On `push`, `BASE_SHA`, `HEAD_SHA`, `PR_TITLE`, and `PR_BODY` are
     empty. The script then runs the tree scan, the path scan, and the
     link scan only.
`.github/workflows/private-data-tests.yml`:

- Trigger: `pull_request` with the types `opened`, `synchronize`, and
  `reopened`. `permissions: contents: read`.
- Job `jobs.private-data-tests`, with `name: private-data-tests`, on
  `ubuntu-latest`: check out the PR as usual, download gitleaks the same
  way, then run `scripts/private-data/test.sh`. It gets no secret and uses
  invented patterns only. It runs the PR version of the tests, so a change
  to the check is tested before it merges.

### First run (bootstrap)

On `pull_request_target`, GitHub runs the workflow file of `main`. When
the FEATURE-17 pull request is open, `main` has no such file. So the
FEATURE-17 pull request gets no `private-data` run. The checks before its
merge are these:

- The `private-data-tests` job runs on that pull request (`pull_request`)
  with invented patterns and no secret.
- The implementer runs `check.sh` on a local machine against the history
  of `main` and of the branch, with the private list from a local file, as
  `scripts/private-data/README.md` tells. The implementer reports only the
  exit code and the number of findings.
- The review of the pull request.

The first `private-data` run is the `push` run on `main` after the merge.
It runs the tree scan, the path scan, and the link scan. Chores reads the
check-run name from that run and makes `private-data` a required check
(decision 8).

The same rule holds for each later change to `check.sh` or to the
workflow: the change takes effect only after it merges. Until then, the
`private-data` job of its pull request runs the version of `main`.

### The check (`check.sh`)

`check.sh` uses `set -euo pipefail` and never uses `set -x`. It reads each
optional input with `${VAR:-}`: `BASE_SHA`, `HEAD_SHA`, `PR_TITLE`,
`PR_BODY`, and `REPO_DIR`. A local run then needs only `PRIVATE_PATTERNS`
and `GITLEAKS`. `REPO_DIR` is the folder to scan, `.` by default. Each git
command and each scan of steps 5 to 8 reads `REPO_DIR`, with `git -C
"$REPO_DIR"`. The check never runs or sources a file of `REPO_DIR`
(decision 15).

0. When `HEAD_SHA` is set and the commit of `REPO_DIR` does not have
   `HEAD_SHA` as its second parent, write "private-data: the merge commit
   of the pull request is not ready; run the job again" and exit 2.
   GitHub makes `refs/pull/<number>/merge` a short time after a push.

1. When `PRIVATE_PATTERNS` is empty or holds no pattern line, write
   "private-data: the PRIVATE_PATTERNS secret is not set. Ask the owner to
   set it." to the error stream and exit 2 (decision 6).
2. Make a private temporary folder with `mktemp -d`, mode `0700`. Remove it
   on exit with `trap`. All files that hold patterns, all gitleaks output,
   and all reports stay in this folder. Never write them to the
   repository, to an artifact, or to the log.
3. Write two configs in that folder (decision 13):
   - `keys.toml`: the default rules of gitleaks for keys, and nothing more.
     ```toml
     [extend]
     useDefault = true
     ```
   - `private.toml`: one `[[rules]]` table for each pattern line, numbered
     in list order. It has no `[extend]` and no `[allowlist]`, so no path
     and no file type is skipped.
     ```toml
     [[rules]]
     id = "private-pattern-1"
     description = "private pattern 1"
     regex = '''(?i)<line 1>'''
     ```
   A pattern line that holds `'''` cannot go in a TOML literal string:
   write "private-data: pattern line N cannot be used" and exit 2.
4. Check each pattern before the scans (decision 14). For each line N,
   write a config with only that rule, and run
   `gitleaks stdin -c <tmp>/one-N.toml` on an empty input. Send its
   standard output and its error stream to `<tmp>/pattern-N.log`. When
   gitleaks does not exit 0, write "private-data: pattern line N is not a
   valid RE2 regular expression" and exit 2. A bad pattern makes gitleaks
   panic and write the full pattern to its error stream, so the check never
   prints that log.
5. Make a copy of the tree. Read `git ls-files -s -z` first:
   - When an entry has mode `160000` (a submodule), write "private-data:
     remove the submodule at entry N of git ls-files; the check does not
     accept submodules" and exit 1. The check does not print the path.
   - For each entry with mode `120000` (a symbolic link), read the link
     target with `git cat-file blob <object>` for the link scan of step 6.
   - For each entry with mode `100644` or `100755`, write
     `git cat-file blob <object>` to `<tmp>/tree/<path>`, and make the
     parent folders first.
   The copy holds the stored bytes of each file. It does not use the work
   tree, because a `.gitattributes` file of the pull request changes the
   work tree: for example, `working-tree-encoding=UTF-16LE` writes a file
   as UTF-16, and gitleaks then does not find the word. `git cat-file blob`
   applies no attribute. `git archive` is not used, because it skips a
   file that `.gitattributes` marks `export-ignore`. The copy has no `.git`
   folder and no untracked file.
6. Run the scans. Each scan runs gitleaks two times: one time with
   `keys.toml` and one time with `private.toml`. Each run has
   `--redact --no-banner --log-level error --report-format json
   --report-path <tmp>/<scan>-<config>.json --exit-code 0
   --ignore-gitleaks-allow --gitleaks-ignore-path <tmp>/no-ignore`.
   `<tmp>/no-ignore` is an empty folder (decision 12). Each run sends its
   standard output and its error stream to `<tmp>/<scan>-<config>.log`.
   - Tree: `gitleaks dir <tmp>/tree`.
   - Paths: the output of `git -c core.quotePath=false ls-files`, one path
     on each line, piped to `gitleaks stdin`. A file name can hold an
     internal project name, and the other scans read only the content of a
     file. `core.quotePath=false` keeps a non-ASCII name as it is, so a
     pattern can match it.
   - Links: the target of each symbolic link of step 5, one target on each
     line in `git ls-files -s` order, piped to `gitleaks stdin`. The tree
     scan does not read the target of a link.
   - Commits (only when `BASE_SHA` is set):
     `gitleaks git --log-opts="--text --no-textconv --no-ext-diff --diff-merges=first-parent $BASE_SHA..$HEAD_SHA" .`
     Without `--diff-merges=first-parent`, gitleaks does not read the diff
     of a merge commit, so a private word that a merge adds passes. The
     other three options stop a `.gitattributes` file of the pull request
     from hiding a diff: `* -diff` makes git print "Binary files differ"
     in place of the lines, and a `textconv` or a `diff` driver changes
     them.
   - Text (only when `BASE_SHA` is set): the PR title, a blank line, the PR
     body, a blank line, then `git log --format=%B "$BASE_SHA..$HEAD_SHA"`,
     piped to `gitleaks stdin`.
7. With `--exit-code 0`, gitleaks exits 0 when it finds data and 1 when it
   fails (gitleaks `cmd/root.go`, `findingSummaryAndExit`). A bad config
   makes it panic with exit 2. So when a run does not exit 0, write only
   "private-data: gitleaks failed (exit N)" and exit 2. Never print the
   log of the run.
8. Find the paths that hold private data before the output. Remove the
   `<tmp>/tree/` part from each `File` of the tree findings. Then put each
   distinct `File` of the tree and commit findings, one on each line, through
   `gitleaks stdin` with both configs. Call each path with a finding a
   hidden path. Then read each report with `jq`. Print one line for each
   finding, with only these fields:
   - Tree: `<File>:<StartLine>: <RuleID>`. The script removes the
     `<tmp>/tree/` part from `File`.
   - Paths: `file path number <StartLine> in git ls-files order: <RuleID>`
   - Links: `link number <StartLine> in git ls-files -s order: <RuleID>`
   - Commits: `<Commit 12 chars> <File>:<StartLine>: <RuleID>`
   In the tree and commit lines, the check writes "a file whose path holds
   private data" in place of a hidden path. The path scan already gives
   the number of that path.
   - Text: `PR text or commit message, line <StartLine>: <RuleID>`
   Never print `Secret`, `Match`, or `Line`. They are redacted, but the
   script does not depend on that.
   When `FORK_PR` is `true`, the check prints none of these lines
   (decision 16).
9. With one finding or more: print "private-data: N findings. Remove the
   private data, then push again." and exit 1. When `FORK_PR` is `true`,
   print only "private-data: private data found" and exit 1. With none:
   print "private-data: no findings" and exit 0.

A rule id never holds the pattern, so the output never shows a private
word (decision 5). GitHub also hides the secret value in the log, but the
check does not depend on that: a gitleaks message can show a pattern in a
changed form, for example with `(?i)` in front or with Go escapes. So no
gitleaks output reaches the log (decision 14).

### Limits

- gitleaks does not read binary files. With `--max-archive-depth 0`, it
  does not look inside an archive, for example a `.zip` or a `.tar.gz`. A
  private word inside such a file passes the content scans. The path scan
  still reads its name. A reviewer must check each binary file and each
  archive that a pull request adds.
- Guessing from a fork: pass or fail still shows if a guess is correct.
  A fork author can push a guess of a private word and read the result.
  Decision 16 makes guessing slower, because the output gives no place
  and no count, but it does not stop it. FEATURE-19 looks at this again,
  for example with a rate limit or an approval step for fork runs.
- Git LFS files: the checkout does not fetch LFS content, so the check
  reads only the small pointer file, not the stored file. A reviewer must
  check each LFS file that a pull request adds.
- A pull request can only add a link target as text. The link scan reads
  that text. The check never follows a link out of the checkout.
- The `keys.toml` runs keep the global allowlist of the default config. So
  a key in, for example, `package-lock.json` or an `.svg` file is not found.
  The private patterns have no allowlist (decision 13).

### Ways to hide a finding

gitleaks has three ways to skip a finding. A pull request must not use them
to pass the check (decision 12):

- A `.gitleaksignore` file: gitleaks reads it from the ignore path and from
  the root of the scanned source. Before the scans, the check lists the
  tracked files with `git ls-files`. When a file named `.gitleaksignore` is
  in the list, the check writes "private-data: remove the ignore file at
  file path number N in git ls-files order; the check does not accept ignore
  files" and exits 1. The check does not print the path, because the path
  can hold private data.
- A `gitleaks:allow` comment on a line: `--ignore-gitleaks-allow` turns it
  off.
- A `.gitleaks.toml` file in the tree: `-c` comes first in the config order
  of gitleaks, so the check never reads that file.

### Local run

`scripts/private-data/README.md` tells a maintainer to set
`PRIVATE_PATTERNS` from a private file and to run `check.sh` with
`GITLEAKS` set to a local gitleaks binary. It holds no pattern.

### Acceptance criteria and their tests

`test.sh` makes a temporary git repository for each case. It builds every
test value at run time, so the repository holds no key-shaped string and no
private word for the tree scan to find. Test patterns are invented, for
example `jagtest-private-word-[0-9]{4}`.

| Criterion | Test |
|---|---|
| CRITERION-74 A pull request that adds private data fails the check | `test.sh`: (a) a file that holds a word that matches a test pattern: exit 1; (b) a file that holds a key-shaped value that the default gitleaks rules find, built at run time (for example `ghp_` and 36 random letters and digits): exit 1; (c) the private word only in a commit that a later commit removes: exit 1 (commit scan); (d) the private word only in `PR_TITLE`: exit 1 (text scan); (e) the word in a different case: exit 1; (f) clean change: exit 0; (g) no `PRIVATE_PATTERNS`: exit 2; (h) a test word on line 1 of `a.txt`, a line that also holds `gitleaks:allow`: exit 1, and the output holds `a.txt:1: private-pattern-1`; (i) a `.gitleaksignore` file in the tree as file path number 1: exit 1, and the output holds "remove the ignore file at file path number 1"; (j) the test word in `b.svg` and in `package-lock.json`: exit 1, with one finding for each file; (k) the test word only in a file name, as file path number 1: exit 1, and the output holds `file path number 1 in git ls-files order: private-pattern-1` and does not hold the test word (path scan); (l) a file `c.txt` that `.gitattributes` marks `export-ignore` and that holds the test word on line 1, with no `BASE_SHA`: exit 1, and the output holds `c.txt:1: private-pattern-1` (tree scan); (m) the test word in a file name and in the content of that file: exit 1, the output holds "a file whose path holds private data", and it does not hold the test word; (n) a tracked symbolic link whose target holds the test word: exit 1, and the output holds `link number 1 in git ls-files -s order: private-pattern-1`; (o) a merge commit in the PR that adds the test word, with `BASE_SHA` set and the word removed again in a later commit: exit 1 (commit scan); (p) a submodule entry: exit 1, and the output does not hold its path; (q) `HEAD_SHA` set to a commit that is not the second parent of the commit of `REPO_DIR`: exit 2; (r) all cases run with `REPO_DIR` set to a folder that is not the working folder; (s) a `.gitattributes` with `* -diff`, and the test word in a commit that a later commit removes: exit 1 (commit scan); (t) a `.gitattributes` with `*.txt working-tree-encoding=UTF-16LE`, and the test word in `d.txt`: exit 1, and the output holds `d.txt:1: private-pattern-1` (tree scan); (u) case (a) with `FORK_PR=true`: exit 1, and the output is only the line "private-data: private data found". "Cannot merge" comes from the required check of decision 8; the chores task proves it with one test PR. |
| CRITERION-75 The check names the place and never shows the value | `test.sh`: for cases (a) to (e), the output holds the file and the line (or the commit, or "PR text"), and the rule id, and it does not hold the test word, the key value, or the pattern text. A pattern line with `'''` gives exit 2 and the line number only. A pattern line that is not valid RE2 (for example `jagtest-word(unclosed`) gives exit 2, and the output holds the line number and does not hold the pattern text. |

## Build order

1. `check.sh` with the pattern check, the tree scan, the path scan, and
   the link scan, and decisions 5, 6, and 11 to 14. `test.sh` cases (a),
   (b), (e) to (n), (p) to (r), (t), (u), and the bad pattern cases of
   CRITERION-75.
2. The commit scan and the text scan. `test.sh` cases (c), (d), (o), and
   (s).
3. The two workflow files. Prove that `private-data-tests` runs on
   the PR of this feature. Run `check.sh` locally as "First run" says, and
   report the exit code.
4. After the merge, prove that the `push` run of `private-data` ran on
   `main`. It fails until the secret is set; this is decision 6.
5. Send the chores request of decision 8: the required check, a test PR
   that proves that a failing `private-data` run blocks the merge, and,
   before the repository goes public, an Actions event policy that allows
   `pull_request_target`.

## Assumptions

- The secret value keeps its line breaks when GitHub gives it to the job
  through `env:`.
- On `pull_request_target`, a pull request from a fork also gets the
  secret. This is safe because the job runs no file of the pull request
  (decision 15).
- The text scan reads the PR body as the PR author wrote it. A later edit of
  the body runs the check again (`edited` trigger).

## Risks

- A pattern is too wide and matches common text. Signal: the check fails on
  a clean pull request. Mitigation: the output gives the rule id, so the
  owner can find the line of the list and change it.
- A new gitleaks version changes the report fields. Signal: `jq` finds no
  `RuleID`. Mitigation: the version is pinned (decision 10), and `test.sh`
  runs on each pull request.
- Case content or private evaluation data that no pattern names. Signal:
  a review finds such text that the check passed. Mitigation: the owner
  adds a pattern for it, for example a case id form or a question text, to
  `PRIVATE_PATTERNS`. The check finds only what the default rules and the
  list name.
- GitHub changes the runner image, and `jq` or `tar` is not on the
  runner. Signal: the `private-data-tests` job fails at the
  first command. Mitigation: install the tool in the workflow.
- A later change runs a file of `pr`, or checks out the PR into `base`.
  The secret is then open to the PR author. Signal: a workflow diff that
  adds a `run:` step that names `pr`, or a `ref:` on the first checkout.
  Mitigation: decision 15; a reviewer refuses such a change.
- In a public repository, the default Actions event policy of GitHub
  blocks `pull_request_target`. GitHub enforces it from 2026-11-02
  (docs.github.com, "Securely using pull_request_target", "Default policy
  for pull_request_target"). Signal: a pull request shows no
  `private-data` run, and the merge stays blocked. Mitigation: the chores
  step of decision 8.
- GitHub does not start `pull_request_target` for some branch names, for
  example a name that looks like a SHA. Signal: a pull request shows no
  `private-data` run. The required check then blocks the merge, which is
  safe. Mitigation: rename the branch.
- A key form that the default rules do not know. Signal: a review finds a
  key that the check passed. Mitigation: add a pattern for that form to
  `PRIVATE_PATTERNS`.

## Out of scope

- The scan of the full history before the repository goes public, and the
  history cleanup: STORY-47, FEATURE-19.
- The check of the published packages (CRITERION-84): FEATURE-18.
- The branch rule and the secret value: the chores role (decision 8).
- The Rust CI workflow: FEATURE-01.
