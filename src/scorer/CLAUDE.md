# src/scorer

This folder holds the parts of the C-scorer module. `../scorer.rs` sends the results to the model service and gives one score for each result.

- `client.rs`: the `ureq` agent, one call, and the map from a status to a failure. Only this code calls `ApiKey::expose`.
- `prompt.rs`: the request of the prompt `jag-p1`, and the split of the results into calls.
- `cost.rs`: the cost of a search (FEATURE-20).

Unit tests: `cargo test --lib scorer::`. End-to-end: `cargo test --test score`, `--test cost`, `--test failure`.
