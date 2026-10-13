---
type: adr
title: Use clap, serde-saphyr, and rpassword as the outside crates of setup
description: Open this when you add or replace an outside crate, or change how jag reads options, YAML settings, or a hidden key prompt.
area: building-blocks
status: accepted
date: 2026-10-13
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: [0011-build-jag-as-one-cargo-package-with-one-module-per-component.md]
parent_spec: docs/product/system-design.md#building-blocks
model:
  status: modeled
  element: C-config
  reason: C-config uses serde-saphyr and rpassword. C-cli uses clap, and its governed_by also lists this ADR
---

# ADR-0012: Use clap, serde-saphyr, and rpassword as the outside crates of setup

## Context

FEATURE-01 must read the command line, read and write the settings file in YAML, and read the key from the terminal with no echo (CV-6). Later features add more settings to the same file (CV-7). The tests must run the `jag` binary and check its output and its files.

## Decision

The outside crates are:

- `clap` 4 with the derive API, for the command line.
- `serde` and `serde-saphyr`, to read and write the YAML settings file.
- `rpassword`, to read the key from the terminal with no echo.

The test crates are `assert_cmd`, `predicates` and `tempfile`.

## Why

`clap` gives grep-like long options, help, and usage errors with exit code 2. `serde-saphyr` reads YAML into typed Rust structs and writes them back, and it gives clear errors. A typed struct holds new settings with no new parser code. `rpassword` reads a line with no echo on Linux and macOS. The test crates run the real binary in a temporary folder, so a test never touches the real settings or key of the user.

## Alternatives considered

- Hand-written YAML read and write: no new crate, but it breaks when later features add settings.

## Consequences

- A new setting is a new field in the settings struct.
- Each new outside crate needs a reason in the pull request. A replacement of one of these crates needs a new ADR.
