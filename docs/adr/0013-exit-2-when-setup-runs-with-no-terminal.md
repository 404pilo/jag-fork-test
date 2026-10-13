---
type: adr
title: Exit 2 when setup runs with no terminal
description: Open this when you change what jag --setup does with no terminal, or when a script acts on the exit code of setup.
area: contracts
status: accepted
date: 2026-10-13
supersedes: null
superseded_by: null
amends: null
refines: 0007-number-the-exit-codes-above-2-from-3-to-8.md
deviates_from: null
related: [0006-run-setup-the-readiness-check-and-key-removal-as-options.md]
parent_spec: docs/product/system-design.md#contracts
model:
  status: modeled
  element: C-cli
---

# ADR-0013: Exit 2 when setup runs with no terminal

## Context

`jag --setup` asks for the key on the terminal with no echo (CV-6, 0006-run-setup-the-readiness-check-and-key-removal-as-options.md). With no terminal, setup cannot ask. Then it must store nothing, say why on the error stream, and exit with a code that is not 0 (CRITERION-72). jag keeps the grep exit codes 0, 1, and 2 (DECISION-05). 0007-number-the-exit-codes-above-2-from-3-to-8.md gives the codes above 2.

## Decision

When `jag --setup` runs with no terminal, it stores nothing, writes the cause on the error stream, and exits with 2.

## Why

The person or the agent used setup in a way that cannot work. That is a usage error, and grep and clap give 2 for a usage error. The codes 3 to 8 tell a model failure or a limit, so a script that acts on them sees no new meaning.

## Alternatives considered

- A new code 9 for "no terminal": the code shows the cause, but it adds a code for a case that only the setup step has.
- Code 3 (key missing): the key is missing after this step, but the cause is not the key, and a script would show the wrong fix.

## Consequences

- The exit code table of 0007-number-the-exit-codes-above-2-from-3-to-8.md stays the same.
- The README gives 2 for this case in the setup section.
- The message tells the user to run `jag --setup` in a terminal.
