---
type: adr
title: Run setup, the readiness check, and key removal as options
description: Open this when you add a command to jag that is not a search, or change the setup and check commands.
area: contracts
status: accepted
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

# ADR-0006: Run setup, the readiness check, and key removal as options

## Context

The command form is `jag TERM "QUESTION" [PATH...]` (DECISION-31). jag also needs a setup command, a readiness check (DECISION-48), and a key removal (DECISION-08). A subcommand such as `jag check` is the same as a search for "check".

## Decision

The options `--setup`, `--check`, and `--forget-key` run the setup, the readiness check, and the key removal.

## Why

Options never clash with a term, and grep users know long options.

## Alternatives considered

- A second binary, `jag-setup`: no clash, but one more command to find.
- Subcommands such as `jag setup`: short, but a search for the word "setup" then needs `jag -e setup`.

## Consequences

- Every new non-search action is an option, not a subcommand.
- These options are part of the public command line (IF-3). A change needs a new major version.
