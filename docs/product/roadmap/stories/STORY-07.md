---
type: Story
title: Use the check result as an exit code
description: As an agent builder (PERSONA-01), I want the readiness check to exit
  with 0 when all items pass and with the code of the first cause when one fails,
  so that a script or an agent can act on the result.
status: draft
state: backlog
parent: /features/FEATURE-02.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/decisions/DECISION-48.md
  hash: 05a25b
---

# STORY-07: Use the check result as an exit code

As an agent builder (PERSONA-01), I want the readiness check to exit with 0 when all items pass and with the code of the first cause when one fails, so that a script or an agent can act on the result.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [CRITERION-11: jag is ready when all items pass](../criteria/CRITERION-11.md) | - | - | - | - | - |
| [CRITERION-67: A failed check exits with the code of the first cause](../criteria/CRITERION-67.md) | - | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [FEATURE-02: Readiness check](../features/FEATURE-02.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [DECISION-48: The readiness check exits 0 or with the code of the first cause](../../prd/decisions/DECISION-48.md)
- criterion or run of: [CRITERION-11: jag is ready when all items pass](../criteria/CRITERION-11.md)
- criterion or run of: [CRITERION-67: A failed check exits with the code of the first cause](../criteria/CRITERION-67.md)
<!-- LINKS:END -->
