---
type: Story
title: Name the cause and the fix of a model failure
description: As an agent builder (PERSONA-01), I want jag to stop with the cause and
  the fix when the model is not available, so that I know what to do next.
status: draft
state: done
actual_end_date: '2026-10-14'
github:
  number: 34
  node_id: I_kwDOVKMnL88AAAABXTLt9A
  url: https://github.com/ai-ready-future/jag/issues/34
  pushed_hash: 6db699
  synced:
    state: done
spec: docs/superpowers/specs/2026-10-13-STORY-09-clear-failure-when-the-model-is-not-available-design.md
parent: /features/FEATURE-03.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/decisions/DECISION-01.md
  hash: 303d3c
---

# STORY-09: Name the cause and the fix of a model failure

As an agent builder (PERSONA-01), I want jag to stop with a message that names the cause and the fix when the key is missing or not valid, the credit is gone, or the service is down, so that I know what to do next.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [CRITERION-13: Each model failure names its cause and fix](../criteria/CRITERION-13.md) | - | - | - | - | - |
| [CRITERION-15: jag never ranks without the model](../criteria/CRITERION-15.md) | - | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [FEATURE-03: Clear failure when the model is not available](../features/FEATURE-03.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [DECISION-01: jag fails clearly when the model is not available](../../prd/decisions/DECISION-01.md)
- criterion or run of: [CRITERION-13: Each model failure names its cause and fix](../criteria/CRITERION-13.md)
- criterion or run of: [CRITERION-15: jag never ranks without the model](../criteria/CRITERION-15.md)
<!-- LINKS:END -->
