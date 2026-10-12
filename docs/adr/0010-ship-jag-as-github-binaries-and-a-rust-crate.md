---
type: adr
title: Ship jag as GitHub binaries and a Rust crate
description: Open this when you change how users install jag, or when you add a package channel.
area: deployment
status: proposed
date: 2026-10-12
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: []
parent_spec: docs/product/system-design.md#deployment
model:
  status: modeled
  element: C-cli
---

# ADR-0010: Ship jag as GitHub binaries and a Rust crate

## Context

jag is a Rust binary (see the building-blocks ADR). A new user must get a first good result in under 5 minutes (METRIC-04). Agent builders and shell users run Linux and macOS.

## Decision

Each release puts one prebuilt binary archive for each platform, with a checksum, on GitHub Releases. Rust users can run `cargo install` from crates.io. jag has no npm package.

## Why

A prebuilt binary installs in seconds with no runtime. The crate serves users who build from source. One language keeps one build and one release path.

## Alternatives considered

- An npm package that pulls the right binary: gives `npx` for agent setups, but adds a second ecosystem and an npm org.
- `cargo install` only: needs a Rust toolchain and a compile of minutes, which breaks METRIC-04.

## Consequences

- No `npx` route. The install docs give the download command for each platform.
- Each release builds four binaries in CI.
