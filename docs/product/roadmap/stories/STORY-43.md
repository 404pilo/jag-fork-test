---
type: Story
title: Stop at once when stopped
description: As a shell power user (PERSONA-04), I want jag to stop at once on Ctrl-C
  or a closed pipe, so that a stopped search wastes no more credit.
status: draft
state: backlog
github:
  number: 104
  node_id: I_kwDOVDCdD88AAAABXCBtLw
  url: https://github.com/ai-ready-future/jag/issues/104
  pushed_hash: 37537d
  synced:
    state: backlog
parent: /features/FEATURE-15.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/decisions/DECISION-39.md
  hash: ea0aa5
---

# STORY-43: Stop at once when stopped

As a shell power user (PERSONA-04), I want jag to stop at once, start no new model call, and show no partial results when I press Ctrl-C or the pipe closes, so that a stopped search wastes no more credit.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [CRITERION-64: jag stops at once when the user stops it](../criteria/CRITERION-64.md) | - | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [FEATURE-15: Good behavior in a terminal and in a pipe](../features/FEATURE-15.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [DECISION-39: When the user stops jag, it stops at once with no partial results](../../prd/decisions/DECISION-39.md)
- criterion or run of: [CRITERION-64: jag stops at once when the user stops it](../criteria/CRITERION-64.md)
<!-- LINKS:END -->
