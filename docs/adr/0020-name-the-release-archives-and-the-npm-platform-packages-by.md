---
type: adr
title: Name the release archives and the npm platform packages by platform
description: Open this when you change the name of a release archive, its checksum file, or an npm package of a platform, or when you add a platform.
area: deployment
status: accepted
date: 2026-10-14
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: [0010-ship-jag-on-github-releases-crates-io-and-npm.md]
parent_spec: docs/superpowers/specs/2026-10-14-STORY-48-install-jag-from-github-releases-crates-io-or-npm-design.md
model:
  status: modeled
  element: C-cli
---

# ADR-0020: Name the release archives and the npm platform packages by platform

## Context

Each release puts one binary for each of four platforms on GitHub Releases and on npm (0010-ship-jag-on-github-releases-crates-io-and-npm.md). The platforms are Linux x86_64, Linux arm64, macOS arm64, and macOS x86_64. The Linux binaries are static, so one Linux binary runs with each C library.

Users, install scripts, and agent setups find a binary by its name. A published npm name stays on npm, and each npm trusted publisher names one package. So a later change of a name is expensive.

## Decision

The name of each file or package that holds the binary of one platform shows that platform in the words of its channel. GitHub Releases uses the Rust target. npm uses the `os` and `cpu` names of Node.js.

## Why

- The Rust target names the CPU, the system, and the C library in one word. `rustup` and `cargo` use the same word, so the build, the archive, and the docs agree.
- ripgrep names its archives and its checksum files in the same form, for example `ripgrep-15.2.0-x86_64-unknown-linux-musl.tar.gz` and its `.sha256` file. jag finds matches with the ripgrep libraries, so its users know this form.
- npm compares the `os` and `cpu` fields of a package with the `process.platform` and `process.arch` values of Node.js. The same words in the package name let the launcher find the package of the machine with no second table. esbuild names its platform packages in the same form, for example `@esbuild/linux-x64`.
- The Linux binary is static, so npm needs no `libc` field and no second Linux package for each CPU.

## Alternatives considered

- Archive names with no version, such as `jag-linux-x64.tar.gz`: a link to the latest release stays the same, but two versions get the same file name in one folder.
- The words of Node.js in the archive names, such as `jag-1.2.3-linux-x64.tar.gz`: one set of words for each channel, but the name does not show the C library, and it differs from the target of the build.
- npm names with the C library, such as `@airf/jag-linux-x64-musl`, and a `libc` field: Biome uses this form. It is necessary only when a Linux binary needs the C library of the machine.
- One checksum file for all archives, such as `SHA256SUMS`: one file less, but a user must find the line of their archive before `sha256sum -c` checks it.

## Consequences

- Archive: `jag-<version>-<target>.tar.gz`, for example `jag-1.2.3-x86_64-unknown-linux-musl.tar.gz`. It holds the folder `jag-<version>-<target>/` with `jag`, `README.md`, and `LICENSE`.
- Checksum: `jag-<version>-<target>.tar.gz.sha256`. It holds one line: the SHA-256 in hex, two spaces, and the archive name. `sha256sum -c` and `shasum -a 256 -c` read this form.
- The four targets: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `aarch64-apple-darwin`, and `x86_64-apple-darwin`.
- npm: `@airf/jag-linux-x64`, `@airf/jag-linux-arm64`, `@airf/jag-darwin-arm64`, and `@airf/jag-darwin-x64`. Each has one `os` value, one `cpu` value, and no `libc` value. The main package `@airf/jag` keeps its name (0010-ship-jag-on-github-releases-crates-io-and-npm.md).
- A new platform adds one archive and one npm package in the same forms. The new npm package needs its own first publish and its own trusted publisher.
- A change of a form breaks the install scripts and the docs that use the old names. The old npm packages stay on npm.
