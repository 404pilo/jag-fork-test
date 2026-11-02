# src/cli

This folder holds the parts of the C-cli module. `../cli.rs` holds the parsing, the dispatch, and the search flow.

- `grep.rs`: the grep options (FEATURE-13, FEATURE-23), the matcher, the color, and the globs.
- `plain.rs`: a search with no `--ask` and no `--json`. It follows the flow of rg 15.2.0.
- `rg_text.rs`: the error texts of a plain search. They are the rg texts with the name `jag`.
- `stop.rs`: the stop of a search on Ctrl-C or on a closed pipe.

Unit tests: `cargo test --lib cli::`. End-to-end: `cargo test --test plain_grep`, `--test grep_options`, `--test stop`, `--test rg_parity`.

ADR: `docs/adr/0022-use-rg-as-the-reference-for-the-output-of-jag-with-no.md`
