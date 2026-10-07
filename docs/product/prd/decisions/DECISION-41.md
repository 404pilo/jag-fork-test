---
type: Decision
title: In text mode, the cost line goes to the error stream after each search
description: Text mode writes a cost line to the error stream after each search, also
  in a pipe; structured mode has a cost field.
status: stable
decided_at: '2026-10-07'
alternatives:
- Show the cost line only in a terminal, or when the user gives an option
- Show the cost only when the user gives an option, in both modes
about:
- /capabilities/CAPABILITY-07.md
- /decisions/DECISION-29.md
---

# DECISION-41: In text mode, the cost line goes to the error stream after each search

In text mode, jag writes a cost line to the error stream after each search, also when the output goes to a pipe. In structured mode, the cost is always a field.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-07: Show the cost of a search](../capabilities/CAPABILITY-07.md)
- about: [DECISION-29: In text mode, notices go to the error stream](DECISION-29.md)
<!-- LINKS:END -->
