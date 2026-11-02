# tests/parity

This folder holds the data of the rg parity tests. With no `--ask`, jag must give the same output and exit code as the pinned rg.

- `cases.yaml`: one entry for each case. The file header tells the fields.
- `fixtures/`: the file trees of the cases.
- `stdin/`: the input files of the cases.
- `expected/<id>/`: the stored jag output of each recorded difference.
- `harness.rs`: the shared run and compare code. `../rg_parity.rs` and `../rg_parity_fs.rs` include it.
- `rg.sha256`: the pin of rg.

Test: set `JAG_PARITY_RG` to the path that `scripts/parity/install-rg.sh` prints, then run `cargo test --test rg_parity --test rg_parity_fs`. With no `JAG_PARITY_RG`, the tests skip the compare.

ADR: `docs/adr/0021-check-rg-parity-with-a-cargo-test-that-runs-a-pinned-rg-and.md`
