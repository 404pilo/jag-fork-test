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

## Run the tests

```
$ GITLEAKS=/path/to/gitleaks scripts/private-data/test.sh
```

The tests use invented patterns and need no secret.
