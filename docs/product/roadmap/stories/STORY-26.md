---
type: Story
title: Leave out the lowest-ranked results
description: As an agent builder (PERSONA-01), I want jag to leave out the lowest-ranked
  results and tell me how many, so that I know the list is not complete and how to
  see more.
status: draft
state: done
actual_end_date: '2026-10-14'
github:
  number: 87
  node_id: I_kwDOVDCdD88AAAABXCAdog
  url: https://github.com/ai-ready-future/jag/issues/87
  pushed_hash: 31996f
  synced:
    state: ready
spec: docs/superpowers/specs/2026-10-13-STORY-25-size-limit-on-the-text-that-jag-returns-design.md
parent: /features/FEATURE-09.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/decisions/DECISION-21.md
  hash: f3ae8f
- to: prd:/decisions/DECISION-24.md
  hash: '887425'
- to: prd:/decisions/DECISION-26.md
  hash: 76a6a2
- to: prd:/decisions/DECISION-47.md
  hash: 2ebb9b
---

# STORY-26: Leave out the lowest-ranked results

As an agent builder (PERSONA-01), I want jag to leave out whole results with the lowest rank first, give a notice with their number and how to raise the limit, and exit with its own code, so that I know the list is not complete and how to see more.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [CRITERION-104: The size-limit notice is named fields in structured output](../criteria/CRITERION-104.md) | - | - | - | - | - |
| [CRITERION-39: jag leaves out whole results with the lowest rank first](../criteria/CRITERION-39.md) | - | - | - | - | - |
| [CRITERION-41: The notice gives the count and how to raise the limit](../criteria/CRITERION-41.md) | - | - | - | - | - |
| [CRITERION-42: Leaving out results has its own exit code](../criteria/CRITERION-42.md) | - | - | - | - | - |
| [CRITERION-66: Cost-limit code wins when both limits apply](../criteria/CRITERION-66.md) | - | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [FEATURE-09: Size limit on the text that jag returns](../features/FEATURE-09.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [DECISION-21: jag leaves out whole results with the lowest rank first](../../prd/decisions/DECISION-21.md)
- traces: [DECISION-24: The notice gives the count and how to raise the limit](../../prd/decisions/DECISION-24.md)
- traces: [DECISION-26: Leaving out results has its own exit code](../../prd/decisions/DECISION-26.md)
- traces: [DECISION-47: The cost-limit exit code wins over the left-out exit code](../../prd/decisions/DECISION-47.md)
- criterion or run of: [CRITERION-104: The size-limit notice is named fields in structured output](../criteria/CRITERION-104.md)
- criterion or run of: [CRITERION-39: jag leaves out whole results with the lowest rank first](../criteria/CRITERION-39.md)
- criterion or run of: [CRITERION-41: The notice gives the count and how to raise the limit](../criteria/CRITERION-41.md)
- criterion or run of: [CRITERION-42: Leaving out results has its own exit code](../criteria/CRITERION-42.md)
- criterion or run of: [CRITERION-66: Cost-limit code wins when both limits apply](../criteria/CRITERION-66.md)
<!-- LINKS:END -->
