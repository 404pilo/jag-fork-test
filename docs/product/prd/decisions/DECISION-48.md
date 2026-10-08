---
type: Decision
title: The readiness check exits 0 or with the code of the first cause
description: The readiness check exits with code 0 when all five items pass, and otherwise
  with the code of the first cause that fails.
status: stable
decided_at: '2026-10-08'
alternatives:
- Exit 0 when all items pass, and exit 1 when one or more fails
about:
- /capabilities/CAPABILITY-01.md
- /decisions/DECISION-04.md
- /decisions/DECISION-05.md
---

# DECISION-48: The readiness check exits 0 or with the code of the first cause

The readiness check exits with code 0 when all five items pass. When an item fails, it exits with the code of the first cause that fails, as DECISION-05 gives it: key missing, key not accepted, no credit, or service down.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- about: [DECISION-04: The health check examines five items with one free call](DECISION-04.md)
- about: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](DECISION-05.md)
- traced by: [FEATURE-02: Readiness check](../../roadmap/features/FEATURE-02.md)
- traced by: [STORY-07: Use the check result as an exit code](../../roadmap/stories/STORY-07.md)
<!-- LINKS:END -->
