# src/config

This folder holds the parts of the C-config module. `../config.rs` holds the API key, its lookup, its store, and its removal. Only this module reads the key.

- `change.rs`: reads and writes the key file.
- `fs.rs`: a small trait over the file system, so that tests can use fakes.
- `limit.rs`: the cost limit and the size limit.
- `model.rs`: the address, the model name, and the times of the model service.
- `setup.rs`: `--setup` and `--forget-key`.

No message here shows a key, an address, or a limit value.

Unit tests: `cargo test --lib config::`. End-to-end: `cargo test --test setup`, `--test secrecy`, `--test gateway`.
