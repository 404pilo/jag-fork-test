# src/output

This folder holds the parts of the C-output module. `../output.rs` holds the text form of the results and the no-good-answer label.

- `progress.rs`: the progress line on the error stream (STORY-41, DECISION-37).

Unit tests: `cargo test --lib output::`. End-to-end: `cargo test --test output`, `--test terminal`.
