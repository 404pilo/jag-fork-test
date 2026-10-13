---
type: adr
title: Build jag as one Cargo package with one module per component
description: Open this when you add a component, a crate, a binary, or a build check, or when you change the Rust edition or the toolchain.
area: building-blocks
status: accepted
date: 2026-10-13
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: [0005-build-jag-as-one-rust-binary.md]
parent_spec: docs/product/system-design.md#building-blocks
model:
  status: modeled
  element: C-cli
---

# ADR-0011: Build jag as one Cargo package with one module per component

## Context

jag is one Rust program with the commands `jag` and `jag-bench` (0005-build-jag-as-one-rust-binary.md). Each step of a search is one component with its own tests (building-blocks-2). jag ships on crates.io (0010-ship-jag-on-github-releases-crates-io-and-npm.md), but the crates.io name `jag` belongs to another crate. FEATURE-01 is the first code, so it sets the layout and the checks for all later features.

## Decision

- jag is one Cargo package with the name `jag-grep` on crates.io. The package has one library and the binary `jag` (`[[bin]] name = "jag"`). A second binary, `jag-bench`, comes later in the same package.
- The library has one module for each C- component of building-blocks-2 (for example `cli`, `config`, `search`, `prerank`, `scorer`, `output`, `check`). The binaries are thin and call the library.
- The package uses the Rust 2024 edition. `rust-toolchain.toml` pins the toolchain version.
- Each pull request must pass `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test`. CI runs these checks on Ubuntu and on macOS.

## Why

One package gives one version and one crate to publish. The name `jag-grep` is free and says what the tool is. The command stays `jag`. Modules keep the component borders that building-blocks-2 sets, and each module has its own tests. A pinned toolchain gives the same lint and format results on each machine and in CI. CI on Ubuntu and macOS finds platform errors before a release.

## Alternatives considered

- A Cargo workspace with one crate for each component: the borders are stronger, but there are many crates to version and to publish.
- The package names `jag-cli` and `jagrep`: both are free, but `jag-cli` says nothing about grep, and `jagrep` looks like a different command.

## Consequences

- A new component is a new module, not a new crate.
- A clippy warning stops a pull request.
- A change of the toolchain is a change to `rust-toolchain.toml` and goes through a pull request.
- The release builds of 0010-ship-jag-on-github-releases-crates-io-and-npm.md build this one package.
