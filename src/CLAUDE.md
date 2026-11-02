# src

This folder holds the Rust code of jag: grep with a model that ranks the matches. Each module starts with a `//!` comment that tells what it does.

- `main.rs`: the `jag` binary. It calls `cli::run`.
- `lib.rs`: the library. It lists the modules.
- `cli.rs`, `cli/`: option parsing, the search flow, and the exit code.
- `config.rs`, `config/`: the API key, the model settings, and the limits.
- `search.rs`, `search/`: the search of files, folders, or piped text.
- `prerank.rs`: the BM25 rank that keeps the best results under the cost limit.
- `scorer.rs`, `scorer/`: the model calls that score each result.
- `output.rs`, `output/`: the text form of the results.
- `check.rs`: `jag --check`, the readiness check.
- `failure.rs`: the cause, the fix, and the exit code of each failure.
- `bench.rs`, `bench_main.rs`: the `jag-bench` tool. Only the Cargo feature `bench` builds it.

Unit tests: `cargo test --lib`. The end-to-end tests are in `tests/`.
