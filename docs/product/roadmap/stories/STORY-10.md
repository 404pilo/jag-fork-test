---
type: Story
title: Tell each model failure by its exit code
description: As an agent builder (PERSONA-01), I want each cause of a model failure
  to have its own exit code above 2, so that my agent can tell the causes apart and
  react to each one.
status: draft
state: done
actual_end_date: '2026-10-14'
spec: docs/superpowers/specs/2026-10-13-STORY-09-clear-failure-when-the-model-is-not-available-design.md
parent: /features/FEATURE-03.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/decisions/DECISION-05.md
  hash: 2503eb
---

# STORY-10: Tell each model failure by its exit code

As an agent builder (PERSONA-01), I want each cause of a model failure to have its own exit code above 2, so that my agent can tell the causes apart and react to each one.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [CRITERION-14: Each model failure has its own exit code](../criteria/CRITERION-14.md) | - | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [FEATURE-03: Clear failure when the model is not available](../features/FEATURE-03.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](../../prd/decisions/DECISION-05.md)
- criterion or run of: [CRITERION-14: Each model failure has its own exit code](../criteria/CRITERION-14.md)
<!-- LINKS:END -->
