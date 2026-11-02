# scripts/parity

This folder holds the set-up of the rg parity tests.

- `install-rg.sh`: downloads the pinned ripgrep, checks it against `tests/parity/rg.sha256`, and prints the path of `rg`.

Set `JAG_PARITY_RG` to that path before you run the parity tests. See `tests/parity/CLAUDE.md`.

ADR: `docs/adr/0021-check-rg-parity-with-a-cargo-test-that-runs-a-pinned-rg-and.md`
