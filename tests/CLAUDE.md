# tests

This folder holds the integration tests of jag. Each `<name>.rs` file is one Cargo test target. Its `//!` comment names the feature and the criteria that it tests. Most tests run jag against a fake model server.

- `common/`: the helpers that the Rust tests share, with the fake server.
- `parity/`, `rg_parity.rs`, `rg_parity_fs.rs`: jag must give the same output as the pinned rg.
- `adr/`: the Python tests of `scripts/adr/`.
- `agent_skill/`, `agent_skill.rs`: tests of the agent skill.
- `agent_use/`, `agent_use.rs`: the agent test and its record (FEATURE-26).
- `bench/`, `bench.rs`: the cases and the tests of `jag-bench`.
- `readme/`, `readme_examples.rs`: the README examples.
- `speed.rs`: only a release build runs it.

Tests:
- `cargo test --locked --all-features` (all Rust tests; CI runs this)
- `cargo test --test <name>` (one file, for example `--test search`)
- `cargo test --locked --release --test speed`
