# scripts/private-data

This folder holds the check that finds keys and private words. Read `README.md` first. The pattern list is a secret. It is not in the repository.

- `check.sh`: scans files, commits, and pull request text with gitleaks.
- `clean-history.sh`: makes one clean history from main.
- `issues.sh`: scans the issues of a repository with `check.sh`.
- `test.sh`: tests of `check.sh` with invented values.

Test: `GITLEAKS=/path/to/gitleaks scripts/private-data/test.sh`

Specs:
- `docs/superpowers/specs/2026-10-13-STORY-46-block-private-data-in-pull-requests-design.md`
- `docs/superpowers/specs/2026-10-14-STORY-47-go-public-with-a-clean-history-design.md`
