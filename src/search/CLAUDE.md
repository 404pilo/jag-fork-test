# src/search

This folder holds the parts of the search module. `../search.rs` ties them together.

- `walk.rs`: the parallel folder walk, the search of one file, and the skip count.
- `sequence.rs`: the files of a plain search, in the order of rg (FEATURE-23).
- `input.rs`: the input of a search with no path: piped text or the current folder.
- `filter.rs`: the globs of `-g` and `--glob`, with the rules of rg.
- `snippet.rs`: the lines around each match (STORY-14).

Unit tests: `cargo test --lib search::`. End-to-end: `cargo test --test walk`, `--test input`, `--test search`.
