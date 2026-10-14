---
type: Story
title: Name the cause and the fix of a model failure
description: As an agent builder (PERSONA-01), I want jag to stop with the cause and
  the fix when the model is not available, so that I know what to do next.
status: draft
state: ready
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
