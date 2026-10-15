---
type: Feature
title: Common grep options and exit codes
description: jag keeps the grep options that people use most and the grep exit codes
  0, 1, and 2; any other option is a usage error that names it.
status: draft
state: done
actual_end_date: '2026-10-14'
github:
  number: 17
  node_id: I_kwDOVKMnL88AAAABXTLVRA
  url: https://github.com/ai-ready-future/jag/issues/17
  pushed_hash: 7c1e93
  synced:
    state: done
parent: /epics/EPIC-04.md
blocked_by:
- /features/FEATURE-12.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/capabilities/CAPABILITY-06.md
  hash: 79d5d3
- to: prd:/decisions/DECISION-05.md
  hash: 2503eb
- to: prd:/decisions/DECISION-33.md
  hash: 8ed9c9
- to: prd:/decisions/DECISION-34.md
  hash: 47da48
- to: prd:/decisions/DECISION-50.md
  hash: a0c252
---

# FEATURE-13: Common grep options and exit codes

jag keeps the grep options that people use most and the grep exit codes 0, 1, and 2; any other option is a usage error that names it.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-35: Use the common grep options](../stories/STORY-35.md) | done | - | - | - | - |
| [STORY-36: Set the lines around a match](../stories/STORY-36.md) | done | - | - | - | - |
| [STORY-37: Get a usage error for an unknown option](../stories/STORY-37.md) | done | - | - | - | - |
| [STORY-38: Keep the grep exit codes](../stories/STORY-38.md) | done | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-04: A shell user keeps their habits](../epics/EPIC-04.md)
- blocked by: [FEATURE-12: Plain grep with no question](FEATURE-12.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [CAPABILITY-06: Feels like grep](../../prd/capabilities/CAPABILITY-06.md)
- traces: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](../../prd/decisions/DECISION-05.md)
- traces: [DECISION-33: The term is a regular expression, and -F makes it fixed text](../../prd/decisions/DECISION-33.md)
- traces: [DECISION-34: Release 1 keeps the grep options that people use most](../../prd/decisions/DECISION-34.md)
- traces: [DECISION-50: The -A, -B, and -C options set the context lines of a result](../../prd/decisions/DECISION-50.md)
- child: [STORY-35: Use the common grep options](../stories/STORY-35.md)
- child: [STORY-36: Set the lines around a match](../stories/STORY-36.md)
- child: [STORY-37: Get a usage error for an unknown option](../stories/STORY-37.md)
- child: [STORY-38: Keep the grep exit codes](../stories/STORY-38.md)
<!-- LINKS:END -->
