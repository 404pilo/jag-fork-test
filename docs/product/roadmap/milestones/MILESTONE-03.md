---
type: Milestone
title: Make it fast
description: 'The performance and cost improvements: a repeated search costs nothing.'
status: draft
tags:
- make-it-fast
traces:
- to: prd:/capabilities/CAPABILITY-08.md
  hash: 4c9732
- to: prd:/metrics/METRIC-03.md
  hash: 05d8fb
---

# MILESTONE-03: Make it fast

This milestone holds the improvements to speed and cost. It holds FEATURE-16, the 5-second search target. Release 1 does not reuse scores (DECISION-46). The items stay in backlog until the next `/prd:plan` run.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [FEATURE-16: Fast enough to replace grep](../features/FEATURE-16.md) | backlog | 0/1 (0%) | - | FEATURE-04, FEATURE-06 | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- traces: [CAPABILITY-08: A repeated search costs nothing](../../prd/capabilities/CAPABILITY-08.md)
- traces: [METRIC-03: Search speed](../../prd/metrics/METRIC-03.md)
- item of: [FEATURE-16: Fast enough to replace grep](../features/FEATURE-16.md)
- item of: [STORY-44: Return a typical search in under 5 seconds](../stories/STORY-44.md)
<!-- LINKS:END -->
