---
type: Story
title: Fail the full search when the model stops during it
description: As an agent builder (PERSONA-01), I want a search to show no partial
  result when the credit or the service stops during it, so that my agent never trusts
  a half-ranked list.
status: draft
state: ready
spec: docs/superpowers/specs/2026-10-13-STORY-09-clear-failure-when-the-model-is-not-available-design.md
parent: /features/FEATURE-03.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/decisions/DECISION-06.md
  hash: 09aef4
---

# STORY-12: Fail the full search when the model stops during it

As an agent builder (PERSONA-01), I want a search to show no partial result when the credit or the service stops during it, so that my agent never trusts a half-ranked list.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [CRITERION-17: A failure during a search fails the full search](../criteria/CRITERION-17.md) | - | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [FEATURE-03: Clear failure when the model is not available](../features/FEATURE-03.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [DECISION-06: A model failure during a search fails the full search](../../prd/decisions/DECISION-06.md)
- criterion or run of: [CRITERION-17: A failure during a search fails the full search](../criteria/CRITERION-17.md)
<!-- LINKS:END -->
