# tests/common

This folder holds the helpers that the Rust integration tests share. A test file uses them with `mod common;`.

- `mod.rs`: the `jag` command with no key and no real key file, a made-up test key, and the proxy variables to clear.
- `fake_server.rs`: a fake model server on 127.0.0.1. It keeps each request for the test and replies as the test says.

Do not put a real key here. The private-data check scans this folder.
