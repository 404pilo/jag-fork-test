---
type: adr
title: Find matches with the ripgrep libraries
description: Open this when you change how jag finds the lines that match the term, or when grep and jag match different lines.
area: context
status: proposed
date: 2026-10-12
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: []
parent_spec: docs/product/system-design.md#context
model:
  status: modeled
  element: C-search
---

# ADR-0001: Find matches with the ripgrep libraries

## Context

jag must find the lines that match a term in a folder or in piped text, with the grep options of DECISION-34 and the skip rules of DECISION-14. The search must be fast (QS-3), and a new user must get a first result in under 5 minutes (METRIC-04).

## Decision

jag finds the matches with the ripgrep libraries (`grep-regex`, `grep-searcher`, and `ignore`), linked into the jag binary. jag starts no other program.

## Why

The ripgrep libraries are fast, and their skip rules for hidden, binary, and ignored files fit DECISION-14. As libraries, they need no second install and no child process.

## Alternatives considered

- Own search code: full control of grep pattern rules, but slow on a large folder and a lot of code to keep.
- Require the `ag` program: fast, but a second install, and its patterns and options differ from grep.
- Bundle the `rg` program: fast, but one more binary for each platform and a child process.

## Consequences

- The term follows the pattern rules of `rg`, not of grep (RK-1). DECISION-33 needs a change, or jag translates the grep form.
- jag must be a Rust program (see the building-blocks ADR).
