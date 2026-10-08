---
type: Feature
title: Clear failure when the model is not available
description: When the key is missing or out of credit, or the model service is down,
  jag stops with a message that names the cause and the fix, and exits with the code
  of that cause.
status: draft
state: blocked
parent: /epics/EPIC-01.md
blocked_by:
- /features/FEATURE-01.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-01.md
  hash: 295cfa
- to: prd:/decisions/DECISION-01.md
  hash: 303d3c
- to: prd:/decisions/DECISION-05.md
  hash: 2503eb
- to: prd:/decisions/DECISION-06.md
  hash: 09aef4
- to: prd:/constraints/CONSTRAINT-04.md
  hash: e31500
---

# FEATURE-03: Clear failure when the model is not available

When the key is missing or out of credit, or the model service is down, jag stops with a message that names the cause and the fix, and exits with the code of that cause.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-09: Name the cause and the fix of a model failure](../stories/STORY-09.md) | ready | - | - | - | - |
| [STORY-10: Tell each model failure by its exit code](../stories/STORY-10.md) | ready | - | - | - | - |
| [STORY-11: Try again one time before reporting the service as down](../stories/STORY-11.md) | ready | - | - | - | - |
| [STORY-12: Fail the full search when the model stops during it](../stories/STORY-12.md) | ready | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-01: Setup and readiness](../epics/EPIC-01.md)
- blocked by: [FEATURE-01: Store a key one time](FEATURE-01.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-01: Guided key setup and health check](../../prd/capabilities/CAPABILITY-01.md)
- traces: [DECISION-01: jag fails clearly when the model is not available](../../prd/decisions/DECISION-01.md)
- traces: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](../../prd/decisions/DECISION-05.md)
- traces: [DECISION-06: A model failure during a search fails the full search](../../prd/decisions/DECISION-06.md)
- traces: [CONSTRAINT-04: jag tries again one time before it reports the service as down](../../prd/constraints/CONSTRAINT-04.md)
- child: [STORY-09: Name the cause and the fix of a model failure](../stories/STORY-09.md)
- child: [STORY-10: Tell each model failure by its exit code](../stories/STORY-10.md)
- child: [STORY-11: Try again one time before reporting the service as down](../stories/STORY-11.md)
- child: [STORY-12: Fail the full search when the model stops during it](../stories/STORY-12.md)
<!-- LINKS:END -->
