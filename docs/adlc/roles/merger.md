# merger (repository changes)

This file changes only 1 step of the skill role file `<skill folder>/references/roles/merger.md`. All other lines of the skill role file stay. Where the two files differ, this file wins (section 3, "Role files").

## Step 4 — Merge

Replaces step 4 of the skill role file: "Merge with squash: `gh pr merge <n> --squash --delete-branch`. The squash message is the PR title and the Proofs block."

4. Merge with squash and the author address (section 1, repository rule "Merge command", decisions D-002 and D-006):

   ```
   gh pr merge <n> --squash --delete-branch --author-email "$(<address source>)"
   ```

   - `<address source>` is the "Address source" of the rule "Merge command" in section 1. Read it there for each merge. Do not copy it into this file, because only the source can change later.
   - Before the merge, read the address into a shell variable: `a="$(<address source>)"`. If git gives an error, or `a` is empty, do not merge. Send `BLOCKED · decision`. Then run the command above with `"$a"` in place of `"$(<address source>)"`. An empty `--author-email` makes gh merge with the account address.
   - Never print the address. Never run the merge with `set -x` or with a command log.
   - If the merge with `--author-email` fails, stop and send `BLOCKED · decision`. Never merge without `--author-email`.
   - Never use the REST merge API, or another API, to merge.
   - When GitHub limits the GraphQL rate, wait for the reset, send `STATUS` to the orchestrator, and run the merge command again.
   - After the merge, do the author check (section 1, repository rule "Squash merge author check"). If the author or the committer is wrong, stop and send `BLOCKED · decision`.
   - The squash message is the PR title and the Proofs block.
