---
type: Feature
title: Size limit on the text that jag returns
description: jag never writes more characters than the size limit, leaves out whole
  results with the lowest rank first, and says how many it left out and how to raise
  the limit.
status: draft
state: in_progress
actual_start_date: '2026-10-14'
github:
  number: 50
  node_id: I_kwDOVDCdD88AAAABXB-Blw
  url: https://github.com/ai-ready-future/jag/issues/50
  pushed_hash: 0fa67b
  synced:
    state: in_progress
parent: /epics/EPIC-03.md
blocked_by:
- /features/FEATURE-08.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-04.md
  hash: d24158
- to: prd:/decisions/DECISION-19.md
  hash: bd8726
- to: prd:/decisions/DECISION-20.md
  hash: f87d35
- to: prd:/decisions/DECISION-21.md
  hash: f3ae8f
- to: prd:/decisions/DECISION-22.md
  hash: 5ee453
- to: prd:/decisions/DECISION-23.md
  hash: 44a92a
- to: prd:/decisions/DECISION-24.md
  hash: '887425'
- to: prd:/decisions/DECISION-25.md
  hash: c2b298
- to: prd:/decisions/DECISION-26.md
  hash: 76a6a2
- to: prd:/decisions/DECISION-47.md
  hash: 2ebb9b
---

# FEATURE-09: Size limit on the text that jag returns

jag never writes more characters than the size limit, leaves out whole results with the lowest rank first, and says how many it left out and how to raise the limit.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-25: Stay inside a default size limit](../stories/STORY-25.md) | ready | - | - | - | - |
| [STORY-26: Leave out the lowest-ranked results](../stories/STORY-26.md) | ready | - | - | - | - |
| [STORY-27: Cut a top result that is larger than the limit](../stories/STORY-27.md) | ready | - | - | - | - |
| [STORY-28: Reject a size limit that is not valid](../stories/STORY-28.md) | ready | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-03: Output that fits an agent](../epics/EPIC-03.md)
- blocked by: [FEATURE-08: File, line, and snippet for each result](FEATURE-08.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-04: Limit on the text that jag returns](../../prd/capabilities/CAPABILITY-04.md)
- traces: [DECISION-19: The size limit counts characters](../../prd/decisions/DECISION-19.md)
- traces: [DECISION-20: A default size limit always applies](../../prd/decisions/DECISION-20.md)
- traces: [DECISION-21: jag leaves out whole results with the lowest rank first](../../prd/decisions/DECISION-21.md)
- traces: [DECISION-22: All text that jag writes to standard output counts toward the size limit](../../prd/decisions/DECISION-22.md)
- traces: [DECISION-23: jag cuts a top result that is larger than the limit](../../prd/decisions/DECISION-23.md)
- traces: [DECISION-24: The notice gives the count and how to raise the limit](../../prd/decisions/DECISION-24.md)
- traces: [DECISION-25: A size limit that is not valid is a usage error](../../prd/decisions/DECISION-25.md)
- traces: [DECISION-26: Leaving out results has its own exit code](../../prd/decisions/DECISION-26.md)
- traces: [DECISION-47: The cost-limit exit code wins over the left-out exit code](../../prd/decisions/DECISION-47.md)
- child: [STORY-25: Stay inside a default size limit](../stories/STORY-25.md)
- child: [STORY-26: Leave out the lowest-ranked results](../stories/STORY-26.md)
- child: [STORY-27: Cut a top result that is larger than the limit](../stories/STORY-27.md)
- child: [STORY-28: Reject a size limit that is not valid](../stories/STORY-28.md)
<!-- LINKS:END -->
