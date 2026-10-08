---
type: Feature
title: Fast enough to replace grep
description: A typical search returns in under 5 seconds, so that a shell user does
  not go back to grep.
status: draft
state: blocked
parent: /epics/EPIC-04.md
blocked_by:
- /features/FEATURE-04.md
- /features/FEATURE-06.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-06.md
  hash: 79d5d3
- to: prd:/constraints/CONSTRAINT-06.md
  hash: ff3f01
- to: prd:/metrics/METRIC-03.md
  hash: 05d8fb
---

# FEATURE-16: Fast enough to replace grep

A typical search returns in under 5 seconds, so that a shell user does not go back to grep.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-44: Return a typical search in under 5 seconds](../stories/STORY-44.md) | ready | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-04: A shell user keeps their habits](../epics/EPIC-04.md)
- blocked by: [FEATURE-04: Rank matching lines by a question](FEATURE-04.md)
- blocked by: [FEATURE-06: Cost limit on scoring](FEATURE-06.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-06: Feels like grep](../../prd/capabilities/CAPABILITY-06.md)
- traces: [CONSTRAINT-06: A typical search is one search on a medium code repository](../../prd/constraints/CONSTRAINT-06.md)
- traces: [METRIC-03: Search speed](../../prd/metrics/METRIC-03.md)
- child: [STORY-44: Return a typical search in under 5 seconds](../stories/STORY-44.md)
<!-- LINKS:END -->
