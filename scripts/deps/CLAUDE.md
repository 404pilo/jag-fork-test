# scripts/deps

This folder holds the dependency checks of jag (FEATURE-22, STORY-63).

- `check.sh`: runs cargo-deny (the policy is `deny.toml`), `npm audit` of the npm packages, and `release.py check-npm-deps`. It reads no secret.

Run it from the repository root: `scripts/deps/check.sh`. It needs cargo, npm, curl, tar, and sha256sum.
