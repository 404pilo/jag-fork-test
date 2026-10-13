---
type: adr
title: Ship jag on GitHub Releases, crates.io, and npm
description: Open this when you change how users install jag, or when you add or remove a package channel.
area: deployment
status: accepted
date: 2026-10-13
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

# ADR-0010: Ship jag on GitHub Releases, crates.io, and npm

## Context

jag is a Rust binary (see the building-blocks ADR). A new user must get a first good result in under 5 minutes (METRIC-04). Agent builders and shell users run Linux and macOS. Many agent setups install tools with `npm` or run them with `npx`. The unscoped npm name `jag` belongs to another package.

## Decision

Each release puts one prebuilt binary archive for each platform, with a checksum, on GitHub Releases. Rust users can run `cargo install` from crates.io. The npm package `@ai-ready-future/jag` installs the same prebuilt binary: one npm package for each platform holds the binary, and the main package names them as optional dependencies, so npm installs only the package that matches the machine. A small launcher in the main package starts the binary. The release publishes all three channels from one tag.

## Why

A prebuilt binary installs in seconds with no runtime. The npm package gives agent setups `npx` and `npm install`, which they already use. Biome, esbuild, and SWC ship native binaries on npm in the same way. The crate serves users who build from source.

## Alternatives considered

- GitHub Releases and crates.io only: one package path less, but no `npx` route for agent setups.
- An npm package that downloads the binary from GitHub Releases at install time (the npm installer of `dist`): less to publish, but the install fails when a proxy blocks GitHub or when scripts are off.
- `cargo install` only: needs a Rust toolchain and a compile of minutes, which breaks METRIC-04.

## Consequences

- The npm route needs Node on the machine. The binary itself needs no runtime.
- Each release builds four binaries in CI and publishes five npm packages: the main package and one for each platform.
- The release needs the npm scope `@ai-ready-future`. It publishes to npm and to crates.io with trusted publishing from GitHub Actions, so the repository stores no publish token.
- All channels must carry the same version.
