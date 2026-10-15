---
type: Feature
title: Cost limit on scoring
description: When a term matches too many lines, jag scores only the best part up
  to a cost limit that the user can change, and says how many matches it skipped.
status: draft
state: done
actual_start_date: '2026-10-14'
actual_end_date: '2026-10-14'
github:
  number: 12
  node_id: I_kwDOVKMnL88AAAABXTLONw
  url: https://github.com/ai-ready-future/jag/issues/12
  pushed_hash: 0d4f5f
  synced:
    state: done
parent: /epics/EPIC-02.md
blocked_by:
- /features/FEATURE-04.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-02.md
  hash: 11db8b
- to: prd:/decisions/DECISION-12.md
  hash: b7b943
- to: prd:/decisions/DECISION-13.md
  hash: 3ddee2
- to: prd:/decisions/DECISION-47.md
  hash: 2ebb9b
---

# FEATURE-06: Cost limit on scoring

When a term matches too many lines, jag scores only the best part up to a cost limit that the user can change, and says how many matches it skipped.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-18: Score only the best matches](../stories/STORY-18.md) | done | - | - | - | - |
| [STORY-19: Change the cost limit](../stories/STORY-19.md) | done | - | - | - | - |
| [STORY-20: Tell a reached cost limit by its exit code](../stories/STORY-20.md) | done | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-02: Ranked search with file and line](../epics/EPIC-02.md)
- blocked by: [FEATURE-04: Rank matching lines by a question](FEATURE-04.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-02: Term and question search](../../prd/capabilities/CAPABILITY-02.md)
- traces: [DECISION-12: The cost limit has a default that the user can change](../../prd/decisions/DECISION-12.md)
- traces: [DECISION-13: Reaching the cost limit has its own exit code](../../prd/decisions/DECISION-13.md)
- traces: [DECISION-47: The cost-limit exit code wins over the left-out exit code](../../prd/decisions/DECISION-47.md)
- blocks: [FEATURE-16: Fast enough to replace grep](FEATURE-16.md)
- child: [STORY-18: Score only the best matches](../stories/STORY-18.md)
- child: [STORY-19: Change the cost limit](../stories/STORY-19.md)
- child: [STORY-20: Tell a reached cost limit by its exit code](../stories/STORY-20.md)
<!-- LINKS:END -->
