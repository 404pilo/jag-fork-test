---
type: Feature
title: Show the cost of a search
description: After each search, jag reports how many results it scored and skipped,
  and the billed cost in US dollars, as a text line or a structured field.
status: draft
state: done
actual_end_date: '2026-10-14'
github:
  number: 60
  node_id: I_kwDOVDCdD88AAAABXB-rBA
  url: https://github.com/ai-ready-future/jag/issues/60
  pushed_hash: e6f99d
  synced:
    state: backlog
parent: /epics/EPIC-03.md
blocked_by:
- /features/FEATURE-04.md
- /features/FEATURE-11.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/capabilities/CAPABILITY-07.md
  hash: b22a25
- to: prd:/decisions/DECISION-40.md
  hash: e87c28
- to: prd:/decisions/DECISION-41.md
  hash: 06e65a
- to: prd:/decisions/DECISION-42.md
  hash: 2b7875
- to: prd:/decisions/DECISION-43.md
  hash: 5d45be
- to: prd:/decisions/DECISION-44.md
  hash: 50c0de
- to: prd:/decisions/DECISION-45.md
  hash: 5c6b1f
---

# FEATURE-20: Show the cost of a search

After each search, jag reports how many results it scored and skipped, and the billed cost in US dollars, as a text line or a structured field.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-53: See the cost after each search](../stories/STORY-53.md) | done | - | - | - | - |
| [STORY-54: Read the cost in structured output](../stories/STORY-54.md) | done | - | - | - | - |
| [STORY-55: Know the cost when no figure is available](../stories/STORY-55.md) | done | - | - | - | - |
| [STORY-56: Know the cost of a failed or stopped search](../stories/STORY-56.md) | done | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-03: Output that fits an agent](../epics/EPIC-03.md)
- blocked by: [FEATURE-04: Rank matching lines by a question](FEATURE-04.md)
- blocked by: [FEATURE-11: Text output for a person and structured output for an agent](FEATURE-11.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [CAPABILITY-07: Show the cost of a search](../../prd/capabilities/CAPABILITY-07.md)
- traces: [DECISION-40: The cost is the billed amount in US dollars](../../prd/decisions/DECISION-40.md)
- traces: [DECISION-41: In text mode, the cost line goes to the error stream after each search](../../prd/decisions/DECISION-41.md)
- traces: [DECISION-42: The report counts results scored and results skipped](../../prd/decisions/DECISION-42.md)
- traces: [DECISION-43: A search with no model call reports a cost of zero and the reason](../../prd/decisions/DECISION-43.md)
- traces: [DECISION-44: A failed or stopped search reports the cost up to that point](../../prd/decisions/DECISION-44.md)
- traces: [DECISION-45: A missing cost figure never fails the search](../../prd/decisions/DECISION-45.md)
- child: [STORY-53: See the cost after each search](../stories/STORY-53.md)
- child: [STORY-54: Read the cost in structured output](../stories/STORY-54.md)
- child: [STORY-55: Know the cost when no figure is available](../stories/STORY-55.md)
- child: [STORY-56: Know the cost of a failed or stopped search](../stories/STORY-56.md)
<!-- LINKS:END -->
