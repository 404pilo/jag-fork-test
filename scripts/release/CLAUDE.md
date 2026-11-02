# scripts/release

This folder holds the release steps. One version tag publishes jag to GitHub Releases, crates.io, and npm. Read `README.md` first.

- `release.py`: the steps that `.github/workflows/release.yml` runs.
- `test_release.py`: tests of `release.py`.
- `first-publish.sh`: the first publish of version 0.0.0. The user runs it. A session runs it only with `--dry-run`.
- `about.toml`, `notices.hbs`: make the `THIRD-PARTY-LICENSES` file.

Test: `python3 -I -m unittest discover -s scripts/release -p 'test_*.py'`

Spec: `docs/superpowers/specs/2026-10-14-STORY-48-install-jag-from-github-releases-crates-io-or-npm-design.md`
ADR: `docs/adr/0020-name-the-release-archives-and-the-npm-platform-packages-by.md`
