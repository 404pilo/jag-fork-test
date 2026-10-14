---
title: "Go public with a clean history: the history scan, the clean history, and the move to a new public repository"
feature: FEATURE-19
stories: [STORY-47, STORY-52, STORY-60, STORY-61]
kind: build
gh_issue: 109
lifecycle_state: draft
session: 4502bbaa-b415-4568-8320-e2f93089c264
---

# Go public with a clean history: the history scan, the clean history, and the move to a new public repository

## Problem

Before jag goes public, a scan lists the private data in the history. A new
public repository gets a clean history, and a second scan finds none. The
private repository stays private (FEATURE-19, CONSTRAINT-01, DECISION-51).

- STORY-47: an agent builder wants each commit in the history checked for
  private data before jag goes public, so that no old commit shows it.
  - CRITERION-76: the scan finds each key, case content, private evaluation
    data, and internal project name in each commit, names its commit, file,
    and line, and never shows the value.
- STORY-52: an agent builder wants a new public repository with a clean
  history, so that no public commit holds private data, and the private
  repository stays as it is.
  - CRITERION-86: after the history scan lists each finding, one new history
    that has none of the findings is made for the new public repository. The
    new history is not made before the scan.
  - CRITERION-87: before the new repository becomes public, a second scan of
    each ref that it will have (each branch, each tag, and each `refs/pull/*`
    ref) finds no key, no case content, no private evaluation data, and no
    internal project name.
  - CRITERION-102: after the new public repository is created, the private
    repository stays private, and its history is not rewritten. Before the new
    repository is created, the private repository ai-ready-future/jag gets the
    name ai-ready-future/jag-private (ADR-0019).
- STORY-60: an agent builder wants the private-data check to run on each fork
  pull request in the new public repository, so that no fork pull request
  skips the check.
  - CRITERION-99: before the new repository becomes public, its Actions event
    policy allows `pull_request_target` for the private-data check, and a test
    fork pull request shows that the check runs.
- STORY-61: an agent builder wants the board and all issues of jag, open and
  closed, to become public with the new repository, so that outside
  contributors can see them.
  - CRITERION-100: the Project stays private until the new repository is
    public. After the new repository becomes public, the Project is linked to
    it and becomes public. The private repository, ai-ready-future/jag-private,
    stays private.
  - CRITERION-101: before the new repository and the Project become public,
    the board items, the issues, and their comments hold no key, no case
    content, no private evaluation data, and no internal project name. The
    comments of an issue move with the issue.
  - CRITERION-103: while the new repository is still private, each issue of
    the private repository, open or closed, moves to it with
    `gh issue transfer`. Before the issues move, a scan checks each issue and
    its comments. It finds no key, no case content, no private evaluation
    data, and no internal project name. When the scan finds one, no issue
    moves. Before the issues move, the new repository has the same labels and
    milestones, so that each issue keeps them.

DECISION-51 gives the order. The new repository is created as a private
repository. A scan checks each issue of the private repository and its
comments, and no issue moves while the scan finds private data. Then, while
the new repository is private, all issues move to it, open and closed. Then
the board items and the issues are checked, and the second scan runs. Then the
new repository becomes public, and after it the board.

The new public repository gets the name `ai-ready-future/jag` (decision 11,
ADR-0019). So the private repository first gets the name
`ai-ready-future/jag-private` (step B2). In Part A of the steps,
`ai-ready-future/jag` is the private repository. From step B3, it is the new
public repository.

Old commits hold private data, for example internal names in the first
versions of `docs/adlc.md` and DECISION-03 (roadmap PR #12, Q-o). The list
that finds this data is the secret `PRIVATE_PATTERNS` (FEATURE-17 spec
decision 1). No agent reads it. So each scan and the clean step run in GitHub
Actions with the secret, and they print only places, rule ids, and counts.

Some steps only the user can do: change the name of a repository, create a
repository, set a secret, set an organization setting, and change a
visibility. The chores session guides the user through each of them. "The
go-public steps" names each step and its owner.

Facts on 2026-10-14 (`main` at `daca59a`):

- `main` has 81 commits. The repository has no tag and no note.
- Two binary files are in old commits: two `.pyc` files in
  `scripts/adr/__pycache__/`. A later commit removes them.
- 1 commit of 81 has an author address that is not the address of the other
  80 commits.
- 59 of 81 commit titles end with `(#N)`: the number of a pull request of
  ai-ready-future/jag.
- The last `private-data` run on `main` (the push of `daca59a`) passed. So the
  last tree of `main` holds no finding.
- The board (PROJECT-01) has 85 items. Each item is an issue: 44 open and 41
  closed. 39 open issues have a parent issue, and each of these parents is
  open.
- ai-ready-future/jag has 13 labels and 3 milestones. No milestone has a due
  date. The wiki has no page.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Form of the public history | The new history keeps each commit of `main`, with its date, its author, and its message. The clean step changes only the private data in it. It is not one new first commit: with one new first commit, a scan of each old commit has no use | Inferred: roadmap PR #12 (Q-o: remove the private data one time, after the STORY-47 history scan finds each piece of it); STORY-47 ("so that no old commit shows it"); CRITERION-86 |
| 2 | What goes to the new repository | Only `main`. No other branch, no tag, and no pull request. The owner of an open branch makes it again on the new `main` (step B10) | Inferred: DECISION-51 (a new repository with a clean history); CRITERION-87 (each ref "that it will have") |
| 3 | Where the scans and the clean step run | In GitHub Actions, in the new workflow `.github/workflows/go-public.yml`. A person with write access starts it by hand (`workflow_dispatch`). It reads the secret `PRIVATE_PATTERNS`. Its log holds only places, rule ids, and counts. No agent sees the list | FEATURE-17 spec decision 1; orchestrator TASK for FEATURE-19 (never read `PRIVATE_PATTERNS`); this spec |
| 4 | History mode of `check.sh` | The new input `HISTORY` (`main` or `all`) turns it on. The check scans the patch of each commit, each commit message, the author and committer names and addresses, each file name that a commit touches, and each ref name. It lists each binary file, because git-filter-repo cannot change it. Each finding gives the commit (12 characters), its place (the file and the line, or the kind of text), and the rule id. A path with private data shows as "a file whose path holds private data". The check never shows a value | CRITERION-76; FEATURE-17 spec decisions 4, 5, 13, 14, and 17 |
| 5 | Refs of each scan | Scan 1 reads each commit of `main` of the private repository (`HISTORY=main`). The second scan reads each commit of each ref of the new repository: each branch, each tag, and each `refs/pull/*` ref (`HISTORY=all`). Both read a mirror clone (`git clone --mirror`), which has each ref | CRITERION-76; CRITERION-87 |
| 6 | Clean tool | git-filter-repo v2.47.0, the latest release on 2026-10-14. The workflow downloads `git-filter-repo-2.47.0.tar.xz` and checks it with `sha256sum -c` against the fixed SHA-256 `4662cbe5918196a9f1b5b3e1211a32e61cff1812419c21df4f47c5439f09e902`. It takes only the file `git-filter-repo-2.47.0/git-filter-repo`. The tool needs git 2.24 or later and Python 3 | This spec, as FEATURE-17 spec decision 10; the git-filter-repo 2.47.0 release and its source |
| 7 | What the clean step changes | For each pattern line N, each match becomes `REDACTED` in the content of each file (`--replace-text`), in each commit message (`--replace-message`), and in each file name (`--filename-callback`). The expression is `regex:(?i)<line N>==>REDACTED`. Each `*.pyc` file goes out of the history (`--invert-paths --path-glob '*.pyc'`). Each `#N` in a commit message becomes `<OLD_REPO>#N` (`ai-ready-future/jag-private#N`), so that it does not point to another item of the new repository. git-filter-repo changes the old commit hashes in messages to the new hashes by default. The author and committer addresses change only by the mailmap of decision 13 | CRITERION-86; this spec |
| 8 | Order and stop rules of the clean step | One run does these steps in this order: scan 1 of `main` (it lists each finding), the clean, scan 2 of the clean `main`, and the check of the last tree. The run stops with exit 1 and makes no bundle when scan 2 finds a finding or a binary file, or when the last tree of the clean `main` is not the last tree of the old `main` | CRITERION-86 ("The new history is not made before the scan"); this spec |
| 9 | Output of the clean step | A `git bundle` of the clean `main`: the artifact `clean-history` of the run, kept for 1 day. `actions/upload-artifact` v7.0.2 (the latest release on 2026-10-14), pinned by its commit `cf430e030ddbb5b0abf93d22962f4752f3646cd9`. The job has `permissions: contents: read`, so it cannot push. Chores pushes the bundle to the new repository (step B5) | CRITERION-102; this spec |
| 10 | A key in the history | The clean step has no rule for a key that the default gitleaks rules find, because no pattern line names it. When scan 1 lists a key, the user revokes the key and adds a pattern line that matches it to `PRIVATE_PATTERNS` (step A3). Until then, scan 2 finds the key, and the clean step stops | This spec, from CONSTRAINT-01 |
| 11 | Name of the new repository | The new public repository is `ai-ready-future/jag`. Before step B3 creates it, the private repository gets the name `ai-ready-future/jag-private` (step B2). Before the rename, each copy of the private history changes its remote to `ai-ready-future/jag-private` (step B1). From step B10, the team works in new clones of the new repository. A copy of the old history never pushes to `ai-ready-future/jag` | ANSWER human:404pilo, 2026-10-14, hitl (the name `jag`; the private repository gets a new name first); `docs/adr/0019-give-the-new-public-repository-the-name-ai-ready-future-jag.md`. The name `jag-private`: ANSWER human:404pilo, 2026-10-14, hitl (the recommended option) |
| 12 | Guard against guesses | No guard. The user: "Do not check on this. Not a concern atm." The check of a fork pull request runs on each event, as FEATURE-17 builds it. `private-data.yml` does not change. STORY-59 (CRITERION-98) held only this guard. It is removed from the roadmap | ANSWER human:404pilo, 2026-10-14, hitl (neither option of the card); STORY-59: ANSWER human:404pilo, 2026-10-14, hitl |
| 13 | Addresses in the public history | One address for all commits. The user writes one mailmap line into the secret `CLEAN_MAILMAP` of the private repository (step A6). The clean step gives it to git-filter-repo with `--mailmap`. The mailmap is never a file of the repository. The repository, this spec, and the pull request hold no address | ANSWER human:404pilo, 2026-10-14, hitl (the recommended option) |
| 14 | Text mode of `check.sh` | The new inputs `TEXT_FILE` and `TEXT_INDEX` turn it on. The check scans the file. For each finding, it prints the place on line `StartLine` of the index, then the rule id | This spec |
| 15 | One mode at a time | `HISTORY`, `TEXT_FILE`, and `BASE_SHA` select the mode. When more than one is set, the check exits 2 | This spec |
| 16 | The issues scan | The new script `scripts/private-data/issues.sh` reads each issue of the repository, open and closed (`STATE=all`), with the job token (`issues: read`), but no pull request. It reads the title, each old title, the body, each comment, and each revision in the edit history of the body and of each comment. It also reads the name and the description of each label, and the title and the description of each milestone. It writes the text and its index, then runs the text mode. A place holds only numbers | CRITERION-101; this spec |
| 17 | The board text | The board has no text of its own that the issues scan does not read. On 2026-10-14: the title "jag roadmap", no short description, no README, no status update, one view "View 1" (table), the 15 fields of "The board", and only issue items. Chores compares the board with this list before the board becomes public (step B11) | CRITERION-101; `gh project view 6` and `gh project field-list 6` on 2026-10-14 |
| 18 | Move of the issues | Chores moves each issue of the private repository, open and closed, with `gh issue transfer`, in number order (step B9). Before the move, the issues scan of the private repository finds nothing (step B9). Before the move, the new repository also gets the same labels (`gh label clone`) and milestones (step B6). A milestone gets the same title, description, and state, and there is no due date. GitHub keeps the comments and the assignees of an issue. It keeps a label or a milestone when the new repository has one with the same name, and for a milestone also the same due date. The GitHub docs name only open issues, so step B9 checks the state of each moved issue | CRITERION-103; the ANSWER of decision 30 (all issues move); GitHub docs, "Transferring an issue to another repository" |
| 19 | Order of the go-public steps | Parts A, B, and C of "The go-public steps". Steps B1 to B3 change the remotes and the names first. Then the steps keep the order of DECISION-51: the new repository is private; the issues scan of the private repository finds nothing; all issues move; the board items and the issues are checked; the second scan runs; the repository becomes public; then the board | DECISION-51; CRITERION-100; the ANSWER of decision 11 |
| 20 | Freeze | From the snapshot of step A7, no pull request merges into `main` of the private repository. From step B1 to the end of step B10, no copy of the private history pushes (the hold of the orchestrator). No session opens a pull request from such a copy. The hold does not stop the pushes of the clean history (B5 and B10) or the test fork pull request (B8). The team works in the new repository from step B10. From step B12 to step C1, no one pushes, opens a pull request, or writes an issue or a comment in the new repository | This spec, from CRITERION-86 (one new history) and CRITERION-87; the ANSWER of decision 11 (the orchestrator stops the pushes of all sessions during the switch) |
| 21 | Owner of each step | The user: change the name of the private repository, create the new repository, set each secret, set the Actions event policy, revoke a key, remove private text from an issue, and change the visibility of the repository and of the board. The chores session gives the exact command for each user step, one step at a time, and checks the result. Chores alone: start the workflows, push, the settings, the labels, the milestones, the ruleset, the move of the issues, the test fork pull requests, and each check. Other roles: see each step | Orchestrator TASK for FEATURE-19; `docs/adlc.md`, role chores |
| 22 | Actions event policy | The user adds an Actions event policy that targets the new repository. Its rule `restrict_action_events` allows only the events in its list, so the list holds each event of the workflows: `pull_request`, `pull_request_target`, `push`, and `workflow_dispatch`. A policy is an organization setting, and the token of 404pilo has no `admin:org` scope, so this is a user step | CRITERION-99 and its source (GitHub changelog, 2026-09-17); FEATURE-17 spec decision 8; GitHub REST docs, Actions policy rule `restrict_action_events` |
| 23 | Fork approval setting | After the new repository is public (step C2), it holds the `pull_request` workflows of each outside contributor until a maintainer approves them (`all_external_contributors`). GitHub uses this setting only for public repositories. It does not hold `pull_request_target`: such workflows "will always run, regardless of approval settings" | This spec, from GitHub docs (context7 `/github/docs`), the setting "Approval for running fork pull request workflows from contributors" |
| 24 | Wiki | The new repository has no wiki. The wiki of the private repository has no page, and the docs are in `docs/` | Inferred: `git ls-remote` of the wiki on 2026-10-14 (not found) |
| 25 | When each story is done | STORY-47: after step A1. STORY-52: after steps B13 and C6. STORY-60: after steps B7 and B8, and after step C5 with a test fork pull request on or after 2026-11-02. STORY-61: after step C4. A code merge alone does not finish STORY-52, STORY-60, or STORY-61 | CRITERION-76, CRITERION-86, CRITERION-87, and CRITERION-99 to CRITERION-103 |
| 26 | Story state | STORY-47, STORY-52, STORY-60, and STORY-61 are `ready`. The spec is in this pull request, and FEATURE-19 has no open blocker (FEATURE-17 is `done`). The ASKs of decisions 11, 12, 13, and 30 have their ANSWERs (human:404pilo, 2026-10-14). STORY-59 is removed from the roadmap | `docs/adlc.md` rule 11; the ANSWERs of decisions 11, 12, 13, and 30 |
| 27 | Tests | Bash tests in `scripts/private-data/test.sh` with invented patterns, made at test time. The `private-data-tests` workflow downloads git-filter-repo as decision 6 says | FEATURE-17 spec decision 7 |
| 28 | Pattern check for the clean step | Before the clean, the step checks each pattern line with Python `re` on bytes. A line with `==>`, or a line that Python cannot compile, gives exit 2 and the line number only. git-filter-repo cuts a line at the last `==>`, and compiles it with `re.compile` and no flags | This spec, from git-filter-repo 2.47.0 (`get_replace_text`) |
| 29 | ADR | ADR-0019 (`status: accepted`): the name `ai-ready-future/jag` of the new public repository, and the rename of the private repository first. The scripts and the workflow need no ADR: they are cheap to change, and each go-public step happens one time | `docs/adr/CLAUDE.md`; the ANSWER of decision 11 (it accepts the ADR) |
| 30 | The roadmap tool after the move | All issues move, open and closed (decision 18). So after step B10, the repository of the team holds each issue, and the roadmap tool works with no change. The prd plugin 4.4.0 reads the issues of the repository of the remote of the folder. It finds the issue of a page by a marker in the issue body that names the page (`ghsnapshot.issues_by_page`). Else it uses the number in the `github` block of the page. It makes a new issue for each page whose issue it does not find (`ghplan.build`). It writes the number, the node id, and the URL of an issue to its page only when it makes the issue (`ghapply._write_ids`). So the pages need the values of step B9. Also, after B9 an old copy has a remote with no issue, so a sync there makes a new issue for each page. So from step B1, no one runs `prd.py github apply` or `prd.py github pull` in any copy. This hold ends in step B10, after the projecter writes the list of B9 into the pages. It ends only in the clones of the new repository. In a copy of the private repository, it never ends. `prd.py github plan` writes nothing, so it is the check | ANSWER human:404pilo, 2026-10-14, hitl (the card "The roadmap pages of the closed issues after the move": the option "Move the closed issues too", not the recommended option); the prd plugin 4.4.0 (`scripts/prdlib/ghsnapshot.py`, `scripts/prdlib/ghplan.py`, `scripts/prdlib/ghapply.py`) |

## Design

### Files

```
.github/workflows/go-public.yml            new: the history scan, the clean step, and the issues scan
.github/workflows/private-data-tests.yml   changed: git-filter-repo for the tests
scripts/private-data/check.sh              changed: history mode and text mode
scripts/private-data/clean-history.sh      new: the clean step
scripts/private-data/issues.sh             new: the text of the issues, the labels, and the milestones
scripts/private-data/test.sh               changed: the new cases
scripts/private-data/README.md             changed: the new modes
```

`.github/workflows/private-data.yml` does not change (decision 12).

The implementer can move the shared steps of `check.sh` (steps 1 to 4, step
7, and the hidden-path rule of step 8) into `scripts/private-data/lib.sh`.
Each script then sources it from its own folder, never from `REPO_DIR`. Each
FEATURE-17 case of `test.sh` must still pass.

All new scripts keep the rules of `check.sh`: `set -euo pipefail`, no
`set -x`, a private folder (`mktemp -d`, mode `0700`, removed on exit), and
no pattern, no found value, and no output of gitleaks, Python, or
git-filter-repo in the log. A pattern goes to a tool only through a file in
the private folder, never through a command line.

### The workflow `go-public.yml`

- Trigger: `workflow_dispatch` with these inputs:
  - `mode`: `type: choice`, with the options `history`, `clean`, and
    `issues`. Required.
  - `refs`: `type: choice`, with the options `main` and `all`. Default
    `main`. For `history`.
  - `state`: `type: choice`, with the options `open` and `all`. Default
    `all`. For `issues`.
  - `old_repo`: `type: string`. Default `ai-ready-future/jag-private`, the
    name of the private repository from step B2 (decision 11). For `clean`.
- `permissions: contents: read` and `issues: read`. The workflow uses no
  cache.
- Job `go-public`, with `name: go-public`, on `ubuntu-latest`. It is not a
  required check.
  1. `actions/checkout`, pinned as in `private-data.yml` (v7.0.1), with
     `path: base` and `persist-credentials: false`. The scripts come from this
     folder only.
  2. For `history` and `clean`: a mirror clone of the repository of the run
     into `$RUNNER_TEMP/repo.git`, with the job token:
     `gh repo clone "$GITHUB_REPOSITORY" "$RUNNER_TEMP/repo.git" -- --mirror`,
     with `GH_TOKEN: ${{ github.token }}`. The mirror has each ref, also each
     `refs/pull/*` ref. The token is not in the config of the mirror.
  3. Download gitleaks as `private-data.yml` does (FEATURE-17 spec
     decision 10).
  4. For `clean`: download git-filter-repo as decision 6 says, to a folder
     outside `base` and outside the mirror.
  5. Run the script of the mode. Give each input through `env:`, never through
     `${{ }}` in the `run:` text:
     - `history`: `base/scripts/private-data/check.sh` with `REPO_DIR`,
       `HISTORY: ${{ inputs.refs }}`,
       `PRIVATE_PATTERNS: ${{ secrets.PRIVATE_PATTERNS }}`, and `GITLEAKS`.
     - `clean`: `base/scripts/private-data/clean-history.sh` with `REPO_DIR`,
       `OLD_REPO: ${{ inputs.old_repo }}`, `OUT_DIR: ${{ runner.temp }}/out`,
       `PRIVATE_PATTERNS`, `CLEAN_MAILMAP: ${{ secrets.CLEAN_MAILMAP }}`,
       `GITLEAKS`, and `FILTER_REPO`.
     - `issues`: `base/scripts/private-data/issues.sh` with
       `REPO: ${{ github.repository }}`, `STATE: ${{ inputs.state }}`,
       `GH_TOKEN: ${{ github.token }}`, `PRIVATE_PATTERNS`, and `GITLEAKS`.
  6. For `clean`, after step 5 passes: `actions/upload-artifact`
     (decision 9) with `name: clean-history`,
     `path: ${{ runner.temp }}/out/clean.bundle`, `retention-days: 1`, and
     `if-no-files-found: error`.
- Only a person with write access can start the workflow. GitHub runs the
  file of the branch that the person names. The go-public steps use the
  default, `main`.

`.github/workflows/private-data-tests.yml` downloads git-filter-repo as
decision 6 says and gives its path to `test.sh` as `FILTER_REPO`.

### History mode of `check.sh`

`HISTORY=main` scans `refs/heads/main`. `HISTORY=all` scans each ref
(`--all`). Another value gives exit 2. Steps 1 to 4 run as today. Step 0
does not run, because `HEAD_SHA` is not set. Step 5 does not run, because a
mirror clone has no work tree. The tree scan, the path scan, and the link
scan do not run. `REPO_DIR` can be a mirror clone or a
clone with a work tree.

Each scan runs with both configs, as step 6 says:

- Patches: `gitleaks git --log-opts="--text --no-textconv --no-ext-diff --diff-merges=first-parent <revisions>" "$REPO_DIR"`.
  The options have the reasons of FEATURE-17 spec step 6. Each line that a
  commit adds is in its patch, so the scan reads each line of each file that
  a commit of the refs holds. Output:
  `<Commit 12 chars> <File>:<StartLine>: <RuleID>`.
- Messages: the message of each commit (`git log --format=%B`), in
  `git rev-list` order. An index in the private folder gives the commit and
  the line in the message for each line. Output:
  `<commit 12 chars> commit message, line <n>: <RuleID>`.
- People: four lines for each commit: the author name, the author address,
  the committer name, and the committer address. Output:
  `<commit 12 chars> <author name|author email|committer name|committer email>: <RuleID>`.
- File names: each name that a commit touches
  (`git log --name-only --no-renames --diff-merges=first-parent -z`), with an
  index of the commit and the number of the name in the commit. A name with a
  control character gives "private-data: commit <12 chars> has a file name
  with a control character; the check does not accept it" and exit 1. Output:
  `<commit 12 chars> file name number <k> of the commit: <RuleID>`.
- Refs: `git for-each-ref --format='%(refname)'`, one name on each line.
  Output: `ref number <n> in git for-each-ref order: <RuleID>`.

Binary files: a file version with a zero byte in its first 8192 bytes is a
binary file. This is the rule of git-filter-repo 2.47.0, and it does not
change such a file. The check reads the object of each file that a commit
adds or changes (`git log --raw --no-renames --diff-merges=first-parent <revisions>`
and `git cat-file --batch`). It does not use the counts of `--numstat`,
because a `.gitattributes` file changes them. After the findings, when there
is one binary file or more, the check prints
"private-data: binary files: <n>" and one line for each:
`<commit 12 chars> <path>`. A binary file is not a finding, and it does not
change the exit code.

Hidden paths: as step 8 of FEATURE-17. The check puts each distinct `File` of
the patch findings and each binary path through `gitleaks stdin` with both
configs. A path with a finding shows as "a file whose path holds private
data".

The result is as step 9: exit 0 with no finding, exit 1 with one finding or
more. `FORK_PR` has no effect in this mode.

### Text mode of `check.sh`

`TEXT_FILE` and `TEXT_INDEX` must both be set, else exit 2. `TEXT_INDEX` must
have the same number of lines as `TEXT_FILE`, else exit 2. Steps 1 to 4 run;
step 0 and step 5 do not run. The check puts `TEXT_FILE` through
`gitleaks stdin` with both configs. Output: `<line StartLine of TEXT_INDEX>: <RuleID>`.
The result is as step 9.

### The issues scan (`issues.sh`)

Inputs: `REPO` (`owner/name`), `STATE` (`open` or `all`), `GH_TOKEN`, and the
inputs of the text mode (`PRIVATE_PATTERNS` and `GITLEAKS`). For the tests,
`ISSUES_JSON` names a file with the same data in one document
(`{"issues": [...], "labels": [...], "milestones": [...]}`), so that the tests
need no network. With `ISSUES_JSON` and `STATE=open`, the script keeps only
the issues whose field `state` is `open`.

The script reads with `gh api` and `gh api graphql`:

- each issue of the state that is not a pull request: the number, the title,
  the body, and each revision of the body (GraphQL `userContentEdits`, field
  `diff`);
- each old title of each issue: the field `previousTitle` of each
  `RenamedTitleEvent` in the timeline. The timeline of a public issue shows
  each old title;
- each comment of each issue, in order: the body and each revision;
- each label: the name and the description;
- each milestone, open and closed: the title and the description.

It writes the text and the index to the private folder. The places are:

```
issue <n>, title
issue <n>, old title <t>
issue <n>, body, line <k>
issue <n>, body revision <r>, line <k>
issue <n>, comment <c>, line <k>
issue <n>, comment <c> revision <r>, line <k>
label <i>, name
label <i>, description
milestone <m>, title
milestone <m>, description, line <k>
```

`<n>` is the number of the issue in `REPO`. `<t>`, `<c>`, and `<r>` count
from 1 in the order of GitHub. `<i>` counts the labels from 1 in name order. `<m>` is the
number of the milestone. Then the script runs `check.sh` in text mode and
exits with its exit code. It never prints the text.

### The clean step (`clean-history.sh`)

Inputs: `REPO_DIR` (a git folder with `main`), `OLD_REPO` (it must match
`^[A-Za-z0-9-]+/[A-Za-z0-9._-]+$`, else exit 2), `OUT_DIR`,
`PRIVATE_PATTERNS`, `GITLEAKS`, `FILTER_REPO` (the path of the file
`git-filter-repo`), and `CLEAN_MAILMAP` (optional: the text of a mailmap).

1. Check the inputs: exit 2 with a message when one is missing. Make the
   private folder.
2. Read the snapshot: `git -C "$REPO_DIR" rev-parse refs/heads/main`. Print
   "private-data: source main <12 chars>".
3. Scan 1: run `check.sh` with `HISTORY=main` and `REPO_DIR`. Print its
   output. Exit 2 when it exits 2. Go on after exit 0 or exit 1.
4. Pattern check (decision 28). Read the list with the rules of `check.sh`
   step 1, with the same line numbers. For each line N:
   - When the line holds `==>`, write "private-data: pattern line N cannot be
     used" and exit 2.
   - Write `(?i)<line N>` to `<tmp>/re-N.txt`. Compile its bytes with
     `python3 -I` and `re.compile`. Send the output of Python to
     `<tmp>/re-N.log`. When Python fails, write "private-data: pattern line N
     is not a valid Python regular expression" and exit 2.
5. Write these files to the private folder:
   - `<tmp>/text.txt`: one line `regex:(?i)<line N>==>REDACTED` for each
     line N.
   - `<tmp>/messages.txt`: the lines of `<tmp>/text.txt`, then the line
     `regex:(?<![A-Za-z0-9_/&])#([0-9]+)\b==><OLD_REPO>#\1`.
   - `<tmp>/names.py`: the body of the file-name callback. On the first call,
     it compiles each `<tmp>/re-N.txt`. It reads the folder from the
     environment variable `CLEAN_TMP`. It gives back the name with each match
     changed to `REDACTED`.
   - `<tmp>/mailmap`: the text of `CLEAN_MAILMAP`, when it is set.
6. Clone: `git clone --no-local --no-tags --single-branch --branch main "$REPO_DIR" <tmp>/work`.
   git-filter-repo accepts a fresh clone with no `--force`. The script never
   writes to `REPO_DIR`.
7. Clean: in `<tmp>/work`, run `python3 -I "$FILTER_REPO"` with
   `--replace-text <tmp>/text.txt`, `--replace-message <tmp>/messages.txt`,
   `--filename-callback <tmp>/names.py`, `--invert-paths`,
   `--path-glob '*.pyc'`, and, with a mailmap, `--mailmap <tmp>/mailmap`.
   Send all its output to `<tmp>/filter-repo.log`. When it does not exit 0,
   write "private-data: the clean failed (exit N)" and exit 2. Never print the
   log, because it can show an expression. Then print "private-data: clean".
8. Scan 2: run `check.sh` with `HISTORY=main` and `REPO_DIR=<tmp>/work`. Print
   its output. When it exits 1, or its output has the line
   "private-data: binary files: <n>", write "private-data:
   findings remain after the clean. Revoke each key and add a pattern line
   for it. For a binary file, send an ASK. Then run again." and exit 1.
9. Last tree: when `main^{tree}` of `<tmp>/work` is not `main^{tree}` of
   `REPO_DIR`, write "private-data: the clean changed the last tree of main.
   The last tree holds private data or a .pyc file, or a pattern matches more
   text in Python than in RE2." and exit 1. Print no path. The `private-data`
   push run on `main` keeps the last tree clean, so the clean must change
   only old commits. This step also finds a pattern that matches more text in
   Python than in RE2.
10. Bundle: `git -C <tmp>/work bundle create "$OUT_DIR/clean.bundle" refs/heads/main`,
    then `git bundle verify`. Print "private-data: clean history ready: <count>
    commits, last commit <12 chars>, source main <12 chars>" and exit 0.

git-filter-repo gives the same commits for the same source and the same
inputs. So a second run gives the same history.

### Limits

- Binary files: gitleaks does not read binary files (FEATURE-17 spec,
  Limits). git-filter-repo does not change a file with a zero byte in its
  first 8192 bytes. The clean step removes each `.pyc` file and stops on each
  other binary file (decision 8).
- Allowlist: the `keys.toml` runs keep the global allowlist of the default
  gitleaks config (FEATURE-17 spec, Limits). So a key in, for example,
  `package-lock.json` or an `.svg` file is not found.
- Letter case: in Python, `(?i)` on bytes ignores the case of ASCII letters
  only. In RE2, it also ignores the case of other letters. So a private word
  with other letters, in another case, stays after the clean. Scan 2 finds it,
  and the clean step stops. Then add a pattern line for that form.
- Tag messages: the history mode does not read the message of an annotated
  tag. The new repository has no tag before it is public (decision 2).
- Signatures: the new commits have no signature. GitHub shows no "Verified"
  mark on them.
- Pull request text: no pull request moves. In the new repository, the
  `private-data` check reads the title and the body of each pull request. It
  does not read its comments, its reviews, or the edit history of its body.
- Commits that no ref reaches: GitHub can show a commit by its hash after a
  force push or after the deletion of a branch. The scans read the refs only.
  So in the new repository, no one pushes private data, also not to a branch
  that is deleted later.
- Short references in comments: GitHub moves the comments of an issue with no
  change. A `#N` in a comment of a moved issue then opens item N of the new
  repository, which is another item. No step changes the comments.
- Old links: from step B3, a full link to an issue or a pull request of
  `ai-ready-future/jag` in an old text (a comment, a message of the private
  history, or a page outside this repository) opens an item of the new
  repository, or a 404 (ADR-0019).

### The board

On 2026-10-14, PROJECT-01 has this text of its own:

- the title "jag roadmap", no short description, no README, and no status
  update;
- one view, "View 1", with the table layout;
- 15 fields: Title, Assignees, Status, Labels, Linked pull requests,
  Milestone, Repository, Reviewers, Parent issue, Sub-issues progress,
  Created, Updated, Closed, Planned end, and Planned start;
- the options of Status: Backlog, Ready, Claimed, In progress, In review,
  Blocked, Done, and Cancelled;
- 85 items, and each item is an issue (no draft item).

The fields hold no free text. So the issues scan of step B12 reads each text
of each item that becomes public. All issues, open and closed, move to the new
repository (step B9), so the public board holds no item of the private
repository.

### What the name `jag` changes

GitHub sends the old name of a renamed repository to its new name, until a
new repository takes the old name (GitHub docs, "Renaming a repository"). So
from step B3, each old remote and each old link to `ai-ready-future/jag` opens
the new public repository. These items change:

- Remotes. Before the rename, each copy of the private history changes its
  remote to `ai-ready-future/jag-private` (step B1). The worktrees of a folder
  share the remote of that folder. Each other clone needs its own change.
  From step B10, the team works in a new clone of the new repository. A copy
  of the old history never pushes to `ai-ready-future/jag`.
- The roadmap tool. `prd.py` reads the issues and the board through the
  repository of the remote of the folder (`gh repo view`). After step B9, all
  issues of the pages are in the new repository. After step B10, the remote
  of the team is the new repository. So the tool works with no change
  (decision 30). From step B1 until the pages hold the new values (step B10),
  no one runs `prd.py github apply` or `prd.py github pull`.
- The roadmap pages. On 2026-10-14, 85 pages have a `github` block whose
  `url` names `ai-ready-future/jag`. Step B9 records the new number, node id,
  and URL of each moved issue. In step B10, the projecter writes them to the
  pages.
- The board. Step C3 links the board to `ai-ready-future/jag` and removes the
  link to `ai-ready-future/jag-private`. The planner changes the text of
  PROJECT-01 after the switch.
- Secrets and settings. The rename keeps the secrets and the settings in the
  private repository. Step B4 sets `PRIVATE_PATTERNS` in the new repository.
  `CLEAN_MAILMAP` stays only in the private repository. Step B6 makes the
  labels, the milestones, the ruleset, and the Actions settings again.
- Short references. The clean step changes each `#N` in a commit message to
  `ai-ready-future/jag-private#N` (decision 7). Pull requests do not move.

These items do not change. They name `ai-ready-future/jag`, and from step B3
that is the public repository: `repository` in `Cargo.toml`, the line of the
STORY-01 spec that names it, the row "Repository" of `docs/adlc.md`, and the
package names of FEATURE-18. The row "Main folder" of `docs/adlc.md` also
stays, because step B10 puts the new clone at the same path.
The row "Extra context files" changes. `docs/handoff.md` is not in git, and
it points to the source of the earlier prototype. So it stays in the old
folder of step B10, and no step copies it into a clone of the new repository.
The README names no repository address.

### The go-public steps

Steps A1 and A2 can run as soon as their code merges. Steps A3 to A5 run as
soon as A1 or A2 lists a finding. The steps from A6 on start when these are
true: the code of STORY-47, STORY-52, and STORY-61 is on `main`, and the
orchestrator says that the user wants to start.

Each chores step records its result (run URL, exit code, counts, hashes) and
sends it to the orchestrator. Never copy the place lines of a scan into an
issue, a pull request, or the board.

**Part A: in the private repository, before the rename (`ai-ready-future/jag`)**

- **A1. chores:** start the history scan of `main`:

  ```
  $ gh workflow run go-public.yml -R ai-ready-future/jag -f mode=history -f refs=main
  ```

  Read the log with `gh run view <run id> -R ai-ready-future/jag --log`.
  Report the number of findings of each kind (patch, message, people, file
  name, and ref) and the number of binary files. Check: exit 0 or exit 1, not
  exit 2. This step proves CRITERION-76 on the real history.
- **A2. chores:** start the issues scan of each issue, open and closed:

  ```
  $ gh workflow run go-public.yml -R ai-ready-future/jag -f mode=issues -f state=all
  ```

  Report the number of findings. All issues move in step B9, so this scan
  also reads the old comments of the closed issues.
- **A3. user, only when A1 lists a key** (a rule id that is not
  `private-pattern-N`): revoke the key where it was made. Then add a pattern
  line that matches the key to the private list, and set the secret again:

  ```
  $ gh secret set PRIVATE_PATTERNS -R ai-ready-future/jag < /path/to/private-list
  ```

  Then chores does A1 again.
- **A4. orchestrator, only when A1 lists a binary file that is not a `.pyc`
  file:** send an ASK. The clean step stops on such a file (decision 8).
- **A5. user, only when A2 lists a finding:** remove the private text from
  the issue or the comment, also in a closed issue. Then delete each revision
  with the private text from its edit history. Then chores does A2 again.
- **A6. user, with the recommended option of decision 13:** set the mailmap.
  The line is the address to keep, then a space, then the address to change.
  Each address is in angle brackets, for example
  `<keep@example.com> <change@example.com>`. The user types the line when
  gh asks for the value:

  ```
  $ gh secret set CLEAN_MAILMAP -R ai-ready-future/jag
  ```

  Check (chores): `gh secret list -R ai-ready-future/jag` shows
  `CLEAN_MAILMAP`. Case `clean-d` and scan 2 of A8 show its effect.
- **A7. orchestrator:** start the freeze of decision 20. Chores records the
  snapshot:

  ```
  $ gh api repos/ai-ready-future/jag/commits/main --jq .sha
  ```

  When a change must go to `main` after the snapshot (for example, the last
  tree holds private data), the orchestrator lets that one change merge.
  Then chores does A7 and A8 again.
- **A8. chores:** start the clean step:

  ```
  $ gh workflow run go-public.yml -R ai-ready-future/jag -f mode=clean
  ```

  Check: the run passes. Its log shows scan 1, then "private-data: clean",
  then scan 2 with "private-data: no findings". The source is the snapshot of
  A7. The run has the artifact:

  ```
  $ gh api repos/ai-ready-future/jag/actions/runs/<run id>/artifacts --jq '.artifacts[].name'
  ```

  The output is `clean-history`. This step proves CRITERION-86.

**Part B: the names, and the new repository while it is private**

- **B1. orchestrator:** stop each push and each new pull request from a copy
  of the private history (the hold of decision 20). Then change the remote of
  each copy of the private history:
  the main folder of the team, and each other clone of the team and of the
  user. The worktrees of a folder share its remote, so one change in each
  folder is enough:

  ```
  $ git remote set-url origin https://github.com/ai-ready-future/jag-private.git
  ```

  Check: in each folder and in each worktree, `git remote -v` shows only
  `https://github.com/ai-ready-future/jag-private.git`. Until step B2, a fetch
  or a push with this remote fails. This is correct, and it is a part of the
  hold.
- **B2. user:** change the name of the private repository:

  ```
  $ gh repo rename jag-private -R ai-ready-future/jag --yes
  ```

  Check (chores): `gh api repos/ai-ready-future/jag-private --jq '.name, .private, .id'`
  gives `jag-private`, `true`, and an id. Record the id. In the main folder,
  `git ls-remote origin refs/heads/main` gives the snapshot of A7.
- **B3. user:** create the new repository, private and empty:

  ```
  $ gh repo create ai-ready-future/jag --private --description "Grep for agents: rank the matches of a term by a question"
  ```

  Check (chores): `gh api repos/ai-ready-future/jag --jq '.full_name, .private, .size, .id'`
  gives `ai-ready-future/jag`, `true`, `0`, and an id that is not the id of
  B2. Record the id. From this step, `ai-ready-future/jag` is the new
  repository, and GitHub no longer sends the old name to the private
  repository.
- **B4. user:** set the list in the new repository:

  ```
  $ gh secret set PRIVATE_PATTERNS -R ai-ready-future/jag < /path/to/private-list
  ```

  Check (chores): `gh secret list -R ai-ready-future/jag` shows
  `PRIVATE_PATTERNS`.
- **B5. chores:** push the clean history. The run of A8 is now in
  `ai-ready-future/jag-private`:

  ```
  $ gh run download <run id> -R ai-ready-future/jag-private -n clean-history -D <folder>
  $ git clone --branch main <folder>/clean.bundle <folder>/clean
  $ cd <folder>/clean
  $ git push https://github.com/ai-ready-future/jag.git main
  ```

  Check: `git ls-remote https://github.com/ai-ready-future/jag.git` shows
  only `HEAD` and `refs/heads/main`, at the last commit of the bundle. The
  `private-data` run of this push passes.
- **B6. chores:** the settings, in this order:
  1. The features and the merge settings of the private repository, with no
     wiki (decision 24), and forks allowed (for B8):
     `gh repo edit ai-ready-future/jag --enable-wiki=false --enable-issues --enable-projects --enable-merge-commit --enable-squash-merge --enable-rebase-merge --delete-branch-on-merge=false --allow-forking`.
  2. The labels, with the same colors and descriptions:
     `gh label clone ai-ready-future/jag-private --repo ai-ready-future/jag --force`.
     Then delete each label of the new repository that the private repository
     does not have: `gh label delete <name> -R ai-ready-future/jag --yes`.
  3. The milestones, in number order, with the values of
     `gh api 'repos/ai-ready-future/jag-private/milestones?state=all'`:
     `gh api -X POST repos/ai-ready-future/jag/milestones -f title=<title> -f description=<description> -f state=<state>`.
  4. The ruleset of `main`, after B5: the ruleset of the private repository
     has `do_not_enforce_on_create: false`, so it refuses the push that
     creates `main`. Read ruleset 25030130 of `ai-ready-future/jag-private`.
     Make one with the same `name`, `target`, `enforcement`, `conditions`,
     `rules`, and `bypass_actors`:
     `gh api -X POST repos/ai-ready-future/jag/rulesets --input <file>`.
  5. The Actions settings: read
     `repos/ai-ready-future/jag-private/actions/permissions` and
     `repos/ai-ready-future/jag-private/actions/permissions/workflow`, and set
     the same values on `ai-ready-future/jag` with `gh api -X PUT`.

  Check: read each value again with `gh api`, and compare it with the private
  repository.
- **B7. user:** add the Actions event policy of decision 22 in the settings
  of the organization. Chores gives the steps from the GitHub docs on Actions
  policies. For the REST API, the user first gives the token the `admin:org`
  scope:

  ```
  $ gh auth refresh -h github.com -s admin:org
  ```

  Check: chores reads the policy when the token can. Else the user confirms
  it. B8 shows that the check starts.
- **B8. chores:** the test fork pull request:
  1. Fork: `gh repo fork ai-ready-future/jag --clone=false`.
  2. In the fork, on a new branch, add the file `docs/test-fork.md` with one
     invented line. Open a pull request to `main` of the new repository.
  3. Check: the `private-data` check runs on `opened` and passes with
     "private-data: no findings".
  4. Close the pull request with no merge.

  Record the URL of the run. This step proves CRITERION-99.
- **B9. chores:** move all issues, open and closed (decision 18).
  1. Just before the move, start the issues scan of the private repository:

     ```
     $ gh workflow run go-public.yml -R ai-ready-future/jag-private -f mode=issues -f state=all
     ```

     Check: the run passes with "private-data: no findings". With a finding,
     stop, and move no issue. The user does A5 in the private repository, and
     then chores does this scan again.
  2. Record the number, the state, the close reason, and the number of
     comments of each issue:

     ```
     $ gh api --paginate 'repos/ai-ready-future/jag-private/issues?state=all&per_page=100' --jq '.[] | select(.pull_request == null) | [.number, .state, .state_reason, .comments] | @tsv'
     ```

  3. Move each issue, in number order:

     ```
     $ gh issue transfer <number> ai-ready-future/jag -R ai-ready-future/jag-private
     ```

     The command prints the URL of the moved issue, with its new number. For
     each moved issue, record the old number and these values:
     `gh api repos/ai-ready-future/jag/issues/<new number> --jq '[.number, .node_id, .html_url, .state, .state_reason, .comments] | @tsv'`.
     Step B10 writes this list into the roadmap pages. After the first closed
     issue moves, compare its state and its close reason with step 2. When a
     value is not the same, set it as the check below says, before the next
     move.

  Check: `gh issue list -R ai-ready-future/jag-private --state all` shows no
  issue. Each moved issue has the state, the close reason, the labels, the
  milestone, and the number of comments of step 2. The board has each moved
  issue. When a state or a close reason is not the same, chores sets the
  values of step 2 again with
  `gh api -X PATCH repos/ai-ready-future/jag/issues/<new number>`. For an
  issue that must be closed, it sends
  `-f state=closed -f state_reason=<close reason>`. For an issue that must be
  open, it sends `-f state=open`. GitHub changes a close reason only with a
  change of the state. So a closed issue with another close reason first gets
  `-f state=open`. This step proves CRITERION-103.
- **B10. orchestrator:** move the team to the new repository. The
  orchestrator stops each session for this step:
  1. Move the main folder of the team to another path, for example
     `<main folder>-private`. Its remote stays
     `https://github.com/ai-ready-future/jag-private.git`. Its
     worktrees moved with it. So in it, run
     `git worktree repair .claude/worktrees/*`. With no path, the command does
     not repair a worktree that moved with its folder. Check: `git worktree
     list` shows each worktree at its new path, and no line shows `prunable`.
  2. Clone the new repository into the path of the main folder:
     `git clone https://github.com/ai-ready-future/jag.git <main folder>`.
  3. For each open branch, take only its changes, never its commits. In the
     old folder: `git fetch origin`, then
     `git diff --binary origin/main...<branch> > <patch file>`. In
     the new clone: `git switch -c <branch> origin/main`, then
     `git apply --index <patch file>`, then a commit. The new clone never
     fetches from the old folder, so no old commit goes into it.
  4. Each session works in a worktree of the new clone. It opens its pull
     request again in the new repository. The owner closes the old pull
     request with a link to the new one. Each new pull request gets the
     `private-data` check.

  Check: in the new clone, `git remote -v` shows only
  `https://github.com/ai-ready-future/jag.git`, and `git rev-parse origin/main`
  gives the last commit of the bundle. In the old folder, `git remote -v`
  shows only `https://github.com/ai-ready-future/jag-private.git`. Then the
  orchestrator ends the hold of B1.

  After the hold of B1 ends, the projecter writes the list of B9 into the
  `github` block of each page: `number`, `node_id`, and `url` (decision 30). It does this in a pull
  request in the new repository. Before the merge, `prd.py github plan` in its
  worktree shows no `create_issue`. After the merge, the projecter syncs with
  `prd.py github apply`. The sync makes the issue bodies, the parent links,
  and the board cards agree with the pages again. Then the hold of decision 30
  ends.
- **B11. chores:** compare the board with "The board":

  ```
  $ gh project view 6 --owner ai-ready-future --format json
  $ gh project field-list 6 --owner ai-ready-future --format json
  ```

  Read the views and the status updates with GraphQL
  (`projectV2(number: 6) { views(first: 20) { nodes { name layout } } statusUpdates(first: 20) { totalCount } }`).
  When a value is new or changed, chores shows it to the user, and the user
  checks it.
- **B12. chores:** the issues scan of the new repository:

  ```
  $ gh workflow run go-public.yml -R ai-ready-future/jag -f mode=issues -f state=all
  ```

  Check: "private-data: no findings". With a finding: A5 in the new
  repository, then B12 again. B11 and B12 prove CRITERION-101.
- **B13. chores:** the second scan:

  ```
  $ gh workflow run go-public.yml -R ai-ready-future/jag -f mode=history -f refs=all
  ```

  Check: "private-data: no findings", and no binary file. This step proves
  CRITERION-87. From B12 to C1, the freeze of decision 20 holds. When an
  event breaks it, do B12 and B13 again.

**Part C: public**

- **C1. user:** make the new repository public. First, chores checks that
  `gh api repos/ai-ready-future/jag --jq .id` gives the id of B3. With
  another id, stop. Do not do this step, and the orchestrator sends an ASK.

  ```
  $ gh repo edit ai-ready-future/jag --visibility public
  ```

  A newer gh also needs `--accept-visibility-change-consequences`. Check
  (chores): `gh api repos/ai-ready-future/jag --jq '.private, .id'` gives
  `false` and the id of B3.
- **C2. chores:** the fork approval setting (decision 23):

  ```
  $ gh api -X PUT repos/ai-ready-future/jag/actions/permissions/fork-pr-contributor-approval -f approval_policy=all_external_contributors
  ```

  Check: a `GET` of the same path gives `all_external_contributors`.
- **C3. chores:** link the board to the new repository, and remove the link
  to the private repository:

  ```
  $ gh project link 6 --owner ai-ready-future --repo jag
  $ gh project unlink 6 --owner ai-ready-future --repo jag-private
  ```

  Check: the GraphQL field `repositories` of `projectV2(number: 6)` gives
  only `ai-ready-future/jag`.
- **C4. user:** make the board public:

  ```
  $ gh project edit 6 --owner ai-ready-future --visibility PUBLIC
  ```

  Check (chores): `gh project view 6 --owner ai-ready-future --format json --jq .public`
  gives `true`. C1, C3, and C4, in this order, prove CRITERION-100.
- **C5. chores:** a second test fork pull request, as B8 says, now in the
  public repository. The check must start and pass. This shows that the
  event policy works in a public repository, where GitHub blocks
  `pull_request_target` by default from 2026-11-02. Before 2026-11-02, this
  run does not test the policy, because GitHub does not block the event yet.
  Then chores does one more test fork pull request on or after 2026-11-02.
- **C6. chores:** check the private repository:

  ```
  $ gh api repos/ai-ready-future/jag-private --jq '.private, .id'
  $ gh api repos/ai-ready-future/jag-private/commits/main --jq .sha
  ```

  The first gives `true` and the id of B2. The second gives the snapshot of
  A7. This step proves CRITERION-102.

### After the switch

These changes are not in the criteria. The orchestrator sends each task after
C6.

- planner: the PROJECT-01 page. The line "The Project is linked to the
  repository `ai-ready-future/jag`." stays. The lines "The Project is private
  while the repository is private." and "The Project becomes public when the
  new public repository goes public. See FEATURE-19, STORY-61 and
  CRITERION-100." change to: "The Project is public from <date of C4>
  (FEATURE-19, STORY-61, CRITERION-100)." and "Until <date of B2>,
  `ai-ready-future/jag` was the private repository. It is now
  `ai-ready-future/jag-private`, and all its issues moved to
  `ai-ready-future/jag`."
- The orchestrator: in a pull request in the new repository, the row "Extra
  context files" of `docs/adlc.md` names `docs/handoff.md` in the old folder
  of step B10. The user can choose another place for the file.
- No change: `Cargo.toml`, the STORY-01 spec, and the rows "Repository" and
  "Main folder" of `docs/adlc.md` (see "What the name `jag` changes").

### Other features

- FEATURE-17 (done): this spec adds two modes to its check. Its pull request
  mode and `private-data.yml` do not change, and each of its cases in
  `test.sh` still passes.
- FEATURE-18 (blocked by this feature): its release workflow must use an
  event that the policy of decision 22 allows. Its package links name
  `ai-ready-future/jag`, the name of decision 11, so they do not change.
- FEATURE-01 (done): `repository` in `Cargo.toml` does not change
  (decision 11).

### README

No change. The README names no repository address.

### Acceptance criteria and their tests

`test.sh` makes a new temporary repository for each case. As in FEATURE-17,
it makes each test value at run time, and the test pattern is invented:
`jagtest-private-word-[0-9]{4}`. "The word" below is a value that matches it.
`REPO_DIR` is a mirror clone of the test repository, except where a case says
another value. Each output line with a commit gives its first 12 characters.
No output holds the word, a key value, or a pattern text.

| Criterion | Test |
|---|---|
| CRITERION-76 The history scan lists each finding and never shows the value | Cases `hist-a` to `hist-n`, and go-public step A1 |
| CRITERION-86 The new public repository gets one clean history, after the scan | Cases `clean-a` to `clean-k`, and step A8 |
| CRITERION-87 A second scan is clean before the new repository becomes public | Cases `hist-f`, `hist-g`, `hist-h`, and `hist-m`, and step B13 |
| CRITERION-99 The Actions event policy lets the check run on fork pull requests | Step B7 (the policy, read again by chores or confirmed by the user) proves that the policy allows the event. Step B8 proves that the check runs. Step C5, on or after 2026-11-02, proves both in the public repository |
| CRITERION-100 The project board becomes public after the new repository | Steps C1, C3, and C4, in this order. Chores records the time of each |
| CRITERION-101 The project board holds no private data when it becomes public | Cases `text-a` to `text-f`, and steps B11 and B12 |
| CRITERION-102 The private repository stays private | Case `clean-e`, and step C6 |
| CRITERION-103 All issues move to the new repository while it is private | Case `text-f` (the scan reads the closed issues), step B6 (labels and milestones first), and step B9 (the issues scan first, then the move) |

History cases (`HISTORY=main` unless a case says another value):

- `hist-a`: the word on line 2 of `a.txt` in commit 1; commit 2 removes it.
  Exit 1, and the output holds `<commit 1> a.txt:2: private-pattern-1`.
- `hist-b`: the word on line 3 of a commit message. The output holds
  `<commit> commit message, line 3: private-pattern-1`.
- `hist-c`: the word in the author name of one commit and in the committer
  address of another. The output holds `<commit> author name: private-pattern-1`
  and `<commit> committer email: private-pattern-1`.
- `hist-d`: a commit adds a file whose name and content hold the word; the
  next commit renames it. The output holds
  `<commit> file name number 1 of the commit: private-pattern-1`, and the
  patch finding shows "a file whose path holds private data".
- `hist-e`: a key-shaped value in an old commit, made at run time as in
  FEATURE-17 case (b). Exit 1, with the rule id of a default rule.
- `hist-f`: the word only in a commit of the branch `other`. `HISTORY=main`
  gives exit 0. `HISTORY=all` gives exit 1.
- `hist-g`: the word only in a commit that only `refs/pull/1/head` reaches
  (made with `git update-ref` in the test repository). `HISTORY=all` gives
  exit 1.
- `hist-h`: the word in a branch name. `HISTORY=all` gives
  `ref number <n> in git for-each-ref order: private-pattern-1`.
- `hist-i`: a binary file (it holds a zero byte) in an old commit. Exit 0, and
  the output holds "private-data: binary files: 1" and the commit and the
  path. With the word in its path, the line shows "a file whose path holds
  private data".
- `hist-j`: a `.gitattributes` file with `* -diff`, and the word in a later
  commit that a third commit removes. Exit 1, and the output lists no binary
  file.
- `hist-k`: a merge commit that adds the word, which no parent has. Exit 1.
- `hist-l`: `HISTORY` and `BASE_SHA` both set: exit 2. `HISTORY=other`: exit
  2.
- `hist-m`: case `hist-a` with `REPO_DIR` set to a clone with a work tree:
  the same output.
- `hist-n`: a history with no private data. Exit 0 and "private-data: no
  findings".

Text cases:

- `text-a`: a text file with the word on line 3, and an index whose line 3 is
  `issue 7, comment 2, line 1`. Exit 1, and the output holds
  `issue 7, comment 2, line 1: private-pattern-1`.
- `text-b`: an index with another number of lines, or `TEXT_FILE` with no
  `TEXT_INDEX`: exit 2.
- `text-c`: `issues.sh` with `ISSUES_JSON`. The word in revision 1 of comment
  1 of issue 4 gives `issue 4, comment 1 revision 1, line 1: private-pattern-1`.
  The word in the first old title of issue 5 gives
  `issue 5, old title 1: private-pattern-1`. The word in the description of
  the second label gives `label 2, description: private-pattern-1`. The word
  in the title of milestone 3 gives `milestone 3, title: private-pattern-1`.
- `text-d`: `ISSUES_JSON` with a pull request that holds the word: exit 0,
  because the script reads no pull request.
- `text-e`: `ISSUES_JSON` with no private data: exit 0.
- `text-f`: `ISSUES_JSON` with the word only in comment 1 of the closed issue
  6. `STATE=all` gives exit 1 and `issue 6, comment 1, line 1: private-pattern-1`.
  `STATE=open` gives exit 0.

Clean cases (`OLD_REPO=ai-ready-future/test`, and `FILTER_REPO` is the pinned
file):

- `clean-a`: the word in an old line of `a.txt`, in an old commit message, and
  in an old file name. Two `.pyc` files (with a zero byte) that a commit adds
  and a later commit removes. The last tree holds no word. Exit 0. The output
  shows the lines of scan 1 before the line "private-data: clean".
  `clean.bundle` exists. In a clone of the bundle: `HISTORY=main` gives exit 0
  and no binary file; the old version of `a.txt` holds `REDACTED`; no commit
  has a `.pyc` path; `main^{tree}` is the same as in the source.
- `clean-b`: a commit title that ends with `(#12)`, and a body with
  `Refs #5 #6`. They become `(ai-ready-future/test#12)` and
  `Refs ai-ready-future/test#5 ai-ready-future/test#6`. `&#39;` and `a#1` do
  not change.
- `clean-c`: a message that names an older commit by its first 10
  characters. In the clean history, the message names the new commit.
- `clean-d`: `CLEAN_MAILMAP` with one line that changes an invented address to
  another invented address. Each commit of the clean history has the new
  address. With no `CLEAN_MAILMAP`, each address stays.
- `clean-e`: after the run, the source repository has the same refs, and
  `main` is the same commit.
- `clean-f`: a key-shaped value in an old commit, and no pattern for it. Exit
  1. The output names the place of the finding after "findings remain after
  the clean". There is no `clean.bundle`.
- `clean-g`: a pattern line with `==>`. Exit 2, with the line number only.
- `clean-h`: the pattern line `jagtest-\pL+`, which RE2 accepts and Python
  does not. Exit 2 and "pattern line 1 is not a valid Python regular
  expression".
- `clean-i`: the last tree of `main` holds the word. Exit 1 and "the clean
  changed the last tree of main".
- `clean-j`: a binary file that is not a `.pyc` file, in an old commit. Exit 1
  and "findings remain after the clean", and no `clean.bundle`.
- `clean-k`: two runs on the same source give the same `main` commit.

Each FEATURE-17 case of `test.sh` still passes.

## Build order

1. STORY-47: the history mode of `check.sh`, `go-public.yml` with
   `mode=history`, and the cases `hist-a` to `hist-n`. After the merge,
   chores does step A1.
2. STORY-61: the text mode, `issues.sh`, `mode=issues`, and the cases
   `text-a` to `text-f`. After the merge, chores does step A2.
3. STORY-52: `clean-history.sh`, `mode=clean`, the download of git-filter-repo
   in both workflows, and the cases `clean-a` to `clean-k`.
4. The go-public steps from A3 on, when the conditions of "The go-public
   steps" are true. Then "After the switch".

One implementer pull request can hold steps 1 to 3.

## Assumptions

- Decision 1 (each commit stays): from roadmap PR #12 (Q-o) and STORY-47.
- Decision 2 (only `main` goes): from DECISION-51 and CRITERION-87.
- Decision 24 (no wiki): from `git ls-remote` of the wiki on 2026-10-14.
- The GraphQL field `UserContentEdit.diff` gives the full text of each
  revision. Check in step A2: for one issue that the projecter changed, the
  scan reads as many revisions as the issue page shows.
- `gitleaks git` reads a mirror clone. Cases `hist-a` to `hist-n` show it.
- An Actions event policy can target one repository of the organization
  (decision 22).
- GitHub keeps a moved issue on the board, also a closed one. All issues
  move, so it keeps each parent link and each sub-issue link.
- `gh issue transfer` moves a closed issue, and the issue keeps its state and
  its close reason. The GitHub docs name only open issues. A report in the
  issues of another project says that open and closed issues can move. Check
  in B9: chores compares each moved issue with the values of B9 step 2. It
  checks the first closed issue before it moves the next issue.
- `gh repo clone -- --mirror` with `GH_TOKEN` clones a private repository in
  a job, and it writes no token into the config of the mirror.
- Decision 11 (the name `ai-ready-future/jag-private`): not inferred. The
  user chose this name (ANSWER human:404pilo, 2026-10-14, hitl). A later
  change of the name changes each `jag-private` of this spec, DECISION-51,
  CRITERION-100, CRITERION-102, and FEATURE-19, and `prd.py regen` writes the
  index pages again.
- A renamed repository keeps its Actions runs, their artifacts, its secrets,
  its issues, and its ruleset. So B5 reads the artifact of A8 from
  `ai-ready-future/jag-private`. Check in B5: `gh run download` finds it.
- The board link follows the private repository through the rename. Check in
  C3: before the commands, the field `repositories` of the board gives
  `ai-ready-future/jag-private`.
- `docs/handoff.md` stays in the old folder of step B10 (the row "Extra
  context files"): from the orchestrator, inferred from CONSTRAINT-01. The
  user can choose another place for the file.

## Risks

- A pattern matches more text in Python than in RE2. Signal: step 9 of the
  clean step stops with exit 1. Mitigation: change the pattern line, so that
  both match the same text.
- A key is in the history. Signal: A1 lists a rule id that is not
  `private-pattern-N`. Mitigation: step A3.
- `gh repo fork` of B8 fails, because the organization does not allow forks
  of private repositories. Signal: B8 step 1 fails. Mitigation: the user
  allows such forks in the settings of the organization for the time of B8.
  Else the orchestrator sends an ASK: CRITERION-99 then cannot be proven
  before the repository is public.
- `pull_request_target` does not start for a pull request from a private
  fork while the repository is private. Signal: B8 shows no `private-data`
  run. Mitigation: the user turns on "Run workflows from fork pull requests"
  in the Actions settings of the new repository, then chores does B8 again.
  When the check still does not start, the orchestrator sends an ASK:
  CRITERION-99 then cannot be proven before the repository is public.
- The artifact `clean-history` is deleted after 1 day, before B5. Signal:
  `gh run download` finds no artifact. Mitigation: do A8 again, now with
  `-R ai-ready-future/jag-private`. The same source gives the same history.
- The repository becomes public before 2026-11-02. Then B8 and C5 do not
  test the policy, because GitHub does not block `pull_request_target` yet
  (and never in a private repository). Signal: the date of C5. Mitigation:
  the read of the policy in B7 is the proof of the policy, and chores does
  one more test fork pull request on or after 2026-11-02.
- The event policy blocks an event that a later workflow needs, for example
  the release workflow of FEATURE-18. Signal: the workflow does not start.
  Mitigation: add the event to the policy.
- A closed issue holds private data in an old comment or in an old revision.
  Signal: A2 or the scan of B9 step 1 lists a finding. Mitigation: A5, then
  the scan again. No issue moves before the scan finds nothing.
- A closed issue opens again after the move, or it gets another close reason.
  Signal: the check of B9. Mitigation: chores sets the values of B9 step 2
  again, as B9 says.
- `gh issue transfer` does not move a closed issue. Signal: the command
  gives an error for the first closed issue of B9 step 3. Mitigation: for
  each closed issue, chores opens it in the private repository
  (`gh api -X PATCH repos/ai-ready-future/jag-private/issues/<number> -f state=open`),
  moves it, and then sets the state and the close reason of B9 step 2 in
  the new repository.
- A moved issue loses its parent link or its place on the board. Signal: the
  check of B9, or an empty "Parent issue" field. Mitigation: the projecter
  sets it again in the sync of step B10.
- A `#N` in the body of a moved issue points to another item of the new
  repository. Signal: a link opens another item. Mitigation: the sync of step
  B10 makes the issue bodies again from the pages.
- Someone runs `prd.py github apply` or `prd.py github pull` after B1 and
  before the sync of step B10. Or someone runs one of them after B10 in a
  copy of the private repository. Signal: new issues for the pages in
  `ai-ready-future/jag-private`, from a sync in an old copy after B9. Or an
  issue or a card has the values of another page. Mitigation: close each new
  issue as "not planned", and the projecter sets each card again with the
  list of B9 (decision 30).
- A commit goes to `main` of the private repository after the snapshot.
  Signal: B2 or C6 gives another hash. Mitigation: make the change again as a
  pull request in the new repository.
- The freeze between B12 and C1 breaks. Signal: a new event in the new
  repository. Mitigation: do B12 and B13 again.
- Two old file names become the same name after the clean, because each
  match becomes `REDACTED`. Signal: git-filter-repo fails, and the clean step
  exits 2. Mitigation: send an ASK.
- A copy of the old history pushes to `ai-ready-future/jag` after B3. Then
  old commits are in the new repository. Signal: `git ls-remote
  https://github.com/ai-ready-future/jag.git` shows a ref that the team did
  not make, or B13 finds a finding. Mitigation: B1 and B10 change each
  remote, and B13 runs before C1. A deleted ref leaves its commits readable
  by their hash (see "Limits"). So before C1, the orchestrator sends an ASK,
  with the recommendation to delete the new repository and to do B3 again.
  After C1, the orchestrator sends an ASK at once.
- A step after B2 goes to the private repository through the old name. This
  can happen only when B3 did not create the new repository, because then
  GitHub still sends `ai-ready-future/jag` to `ai-ready-future/jag-private`.
  Signal: the id check of B3 or C1 gives the id of B2. Mitigation: stop, and
  do not do C1. The orchestrator sends an ASK.
- An old link to an issue or a pull request of the private repository opens
  another item, or a 404, from B3 (ADR-0019). Signal: a link in a page or a
  comment opens an item with another title. Mitigation: in step B10, the
  projecter changes the links of the pages. The clean step changes
  each `#N` of the public commit messages. The links in comments stay (see
  "Limits").

## Out of scope

- The archive of `ai-ready-future/jag-private`. The user can do it later. The
  repository stays private in both cases.
- The deletion of the test fork. The user can do it later. It holds only
  invented text.
- A guard against guesses through the result of the check on fork pull
  requests (decision 12). The user answered "Do not check on this. Not a
  concern atm." STORY-59 (CRITERION-98) is removed from the roadmap.
- The changes of "After the switch": separate pull requests of their owners.
- The install from GitHub Releases, crates.io, and npm: FEATURE-18.
- A check of the text that people write in issues and comments after the
  board is public.
- A scan of the other refs of the private repository (its other branches and
  its `refs/pull/*` refs): they do not go public (decision 2).
