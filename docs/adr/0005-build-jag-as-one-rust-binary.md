---
type: adr
title: Build jag as one Rust binary
description: Open this when you change the language, the runtime, or the form of the jag program.
area: building-blocks
status: accepted
date: 2026-10-12
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: []
parent_spec: docs/product/system-design.md#building-blocks
model:
  status: modeled
  element: C-cli
---

# ADR-0005: Build jag as one Rust binary

## Context

jag is a command-line tool that a person or an agent runs many times. It must start fast, search a folder fast (QS-3), and install in one step (METRIC-04). The ripgrep libraries are Rust crates.

## Decision

jag is one Rust program with two commands, `jag` and `jag-bench`. Each is one static binary with no runtime.

## Why

Rust links the ripgrep libraries directly, so jag starts no other program. A static binary starts fast and needs no runtime on the machine of the user.

## Alternatives considered

- TypeScript on Node: earlier code exists, but it needs a runtime and starts `rg` as a child process.
- Go: one static binary, but it must start `rg` as a child process, and it gives no gain for a model-bound search.

## Consequences

- The earlier TypeScript code does not move over. Its ideas and settings do.
- Release builds need a binary for each platform (see the install ADR).
