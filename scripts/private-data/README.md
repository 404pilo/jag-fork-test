# Private-data check

`check.sh` finds keys and private words in a repository. The workflow
`.github/workflows/private-data.yml` runs it on each pull request.

The list of private patterns is not in the repository. It is the GitHub
Actions secret `PRIVATE_PATTERNS`. It has one pattern on each line:

- Each pattern is a Go regular expression (RE2). The check ignores case.
- A line that starts with `#` is a comment. A blank line is skipped.
- The rule id in the output is `private-pattern-N`. N is the line number
  in the list.

## Run the check on your machine

1. Download gitleaks v8.30.1 and check its SHA-256 against the value in
   `.github/workflows/private-data.yml`.
2. Keep the pattern list in a private file outside the repository.
3. Run:

   ```
   $ PRIVATE_PATTERNS="$(cat /path/to/private-list)" \
     GITLEAKS=/path/to/gitleaks \
     REPO_DIR=. \
     scripts/private-data/check.sh
   ```

   To scan the commits of a branch, also set `BASE_SHA` and `HEAD_SHA`.
   `HEAD_SHA` must be the second parent of the commit of `REPO_DIR`. Set
   `PR_TITLE` and `PR_BODY` to scan the pull request text.

Exit code 0 means no finding. Exit code 1 means a finding. Exit code 2
means that the check cannot run. Do not share the output of a run that
used your private list.

## History mode

Set `HISTORY=main` to scan each commit of `main`. Set `HISTORY=all` to scan
each commit of each ref, also each `refs/pull/*` ref. Use a mirror clone
(`git clone --mirror`) to have each ref. The check reads the patch of each
commit, each commit message, the author and committer names and addresses,
each file name, and (for `all`) each ref name. It prints the commit (12
characters) and its place for each finding, never the value. It also lists
each binary file. A binary file is not a finding, but the clean step cannot
change it.

Set only one of `BASE_SHA`, `HISTORY`, and `TEXT_FILE`.

## Text mode

Set `TEXT_FILE` and `TEXT_INDEX`. Line N of `TEXT_INDEX` names the place of
line N of `TEXT_FILE`. For each finding, the check prints the place and the
rule id.

## The go-public workflow

`.github/workflows/go-public.yml` runs on `workflow_dispatch`. It has 3 modes:

- `history`: scans the history of the repository (`check.sh` in history
  mode).
- `issues`: scans each issue, its comments, the revisions of its text, the
  labels, and the milestones (`issues.sh`).
- `clean`: makes one clean history of `main` (`clean-history.sh`). Each match
  of a private pattern becomes `REDACTED`, each `.pyc` file goes out, and each
  `#N` in a commit message becomes `<old repo>#N`. The result is the artifact
  `clean-history` (a git bundle). The optional secret `CLEAN_MAILMAP` gives a
  mailmap that changes the addresses of the commits.

The steps of the move to a new public repository are in
`docs/superpowers/specs/2026-10-14-STORY-47-go-public-with-a-clean-history-design.md`.

## Run the tests

```
$ FILTER_REPO=/path/to/git-filter-repo GITLEAKS=/path/to/gitleaks scripts/private-data/test.sh
```

The tests use invented patterns and need no secret. Without `FILTER_REPO`
(git-filter-repo v2.47.0), the tests of the clean step do not run.
