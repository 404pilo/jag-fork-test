---
type: adr
title: Number the exit codes above 2 from 3 to 8
description: Open this when you add an exit code, or when a script or an agent acts on the exit code of jag.
area: contracts
status: proposed
date: 2026-10-12
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: []
parent_spec: docs/product/system-design.md#contracts
model:
  status: modeled
  element: C-cli
---

# ADR-0007: Number the exit codes above 2 from 3 to 8

## Context

jag keeps the grep exit codes 0, 1, and 2 (DECISION-05). Each model failure and each limit has its own code above 2 (DECISION-05, DECISION-13, DECISION-26). Scripts and agents act on these codes.

## Decision

3 key missing, 4 key not accepted, 5 no credit, 6 service down, 7 cost limit reached, 8 results left out by the size limit. Ctrl-C gives 130. A code never changes its meaning in one major version.

## Why

A short, contiguous range is easy to remember and to test, and it stays clear of 126 and above, which shells use.

## Alternatives considered

- Two groups, 10 to 13 for model failures and 20 and 21 for the limits: the group shows the kind, but the numbers are less usual.

## Consequences

- A new cause takes 9 or a higher free code.
- The README and the structured error give the same table.
