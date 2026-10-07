---
type: Decision
title: With no path, jag searches the current folder or reads the pipe
description: With no path, jag searches the current folder, or reads standard input
  when text comes in through a pipe.
status: stable
decided_at: '2026-10-07'
alternatives:
- Always read standard input, as grep does
- A usage error with exit code 2 when there is no path and no piped text
about:
- /capabilities/CAPABILITY-06.md
- /decisions/DECISION-18.md
---

# DECISION-35: With no path, jag searches the current folder or reads the pipe

When the user gives no path, jag searches the current folder, as rg does. When text comes in through a pipe, jag reads that text.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- about: [DECISION-18: Piped text has the label (standard input)](DECISION-18.md)
<!-- LINKS:END -->
