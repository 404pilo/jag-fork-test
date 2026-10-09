---
type: Feature
title: Readiness check
description: One command examines five items, shows pass or fail for each one, and
  names the fix for each failure, with one call that costs almost nothing.
status: draft
state: backlog
parent: /epics/EPIC-01.md
blocked_by:
- /features/FEATURE-01.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/capabilities/CAPABILITY-01.md
  hash: 295cfa
- to: prd:/decisions/DECISION-04.md
  hash: 7b0f54
- to: prd:/constraints/CONSTRAINT-02.md
  hash: fd39da
- to: prd:/decisions/DECISION-48.md
  hash: 05a25b
---

# FEATURE-02: Readiness check

One command examines five items, shows pass or fail for each one, and names the fix for each failure, with one call that costs almost nothing.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-06: Check the five items of readiness](../stories/STORY-06.md) | backlog | - | - | - | - |
| [STORY-07: Use the check result as an exit code](../stories/STORY-07.md) | backlog | - | - | - | - |
| [STORY-08: Run the check for almost no cost](../stories/STORY-08.md) | backlog | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-01: Setup and readiness](../epics/EPIC-01.md)
- blocked by: [FEATURE-01: Store a key one time](FEATURE-01.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [CAPABILITY-01: Guided key setup and health check](../../prd/capabilities/CAPABILITY-01.md)
- traces: [DECISION-04: The health check examines five items with one free call](../../prd/decisions/DECISION-04.md)
- traces: [CONSTRAINT-02: jag protects the API key of the user](../../prd/constraints/CONSTRAINT-02.md)
- traces: [DECISION-48: The readiness check exits 0 or with the code of the first cause](../../prd/decisions/DECISION-48.md)
- child: [STORY-06: Check the five items of readiness](../stories/STORY-06.md)
- child: [STORY-07: Use the check result as an exit code](../stories/STORY-07.md)
- child: [STORY-08: Run the check for almost no cost](../stories/STORY-08.md)
<!-- LINKS:END -->
