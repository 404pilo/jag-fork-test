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
milestone: /milestones/MILESTONE-01.md
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

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-01: Setup and readiness](../epics/EPIC-01.md)
- blocked by: [FEATURE-01: Store a key one time](FEATURE-01.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-01: Guided key setup and health check](../../prd/capabilities/CAPABILITY-01.md)
- traces: [DECISION-04: The health check examines five items with one free call](../../prd/decisions/DECISION-04.md)
- traces: [CONSTRAINT-02: jag protects the API key of the user](../../prd/constraints/CONSTRAINT-02.md)
- traces: [DECISION-48: The readiness check exits 0 or with the code of the first cause](../../prd/decisions/DECISION-48.md)
- criterion or run of: [CRITERION-09: The check shows each of the five items](../criteria/CRITERION-09.md)
- criterion or run of: [CRITERION-10: Each failure names its fix](../criteria/CRITERION-10.md)
- criterion or run of: [CRITERION-11: jag is ready when all items pass](../criteria/CRITERION-11.md)
- criterion or run of: [CRITERION-12: The check costs almost nothing](../criteria/CRITERION-12.md)
- criterion or run of: [CRITERION-67: A failed check exits with the code of the first cause](../criteria/CRITERION-67.md)
<!-- LINKS:END -->
