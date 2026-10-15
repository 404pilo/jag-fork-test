---
type: Story
title: Know when no result answers well
description: As an agent builder (PERSONA-01), I want jag to label the best results
  as weak and to exit with code 1 when no result answers the question well, so that
  my agent does not trust noise.
status: draft
state: done
actual_end_date: '2026-10-14'
github:
  number: 40
  node_id: I_kwDOVKMnL88AAAABXTL3Pg
  url: https://github.com/ai-ready-future/jag/issues/40
  pushed_hash: aba5f8
  synced:
    state: done
spec: docs/superpowers/specs/2026-10-13-STORY-13-rank-matching-lines-by-a-question-design.md
parent: /features/FEATURE-04.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/decisions/DECISION-11.md
  hash: 8e1add
---

# STORY-15: Know when no result answers well

As an agent builder (PERSONA-01), I want jag to label the best results as weak and to exit with code 1 when no result answers the question well, so that my agent does not trust noise.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [CRITERION-29: Weak results have a clear label](../criteria/CRITERION-29.md) | - | - | - | - | - |
| [CRITERION-30: No good answer exits with code 1](../criteria/CRITERION-30.md) | - | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [FEATURE-04: Rank matching lines by a question](../features/FEATURE-04.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [DECISION-11: When nothing answers well, jag labels the best results and exits 1](../../prd/decisions/DECISION-11.md)
- criterion or run of: [CRITERION-29: Weak results have a clear label](../criteria/CRITERION-29.md)
- criterion or run of: [CRITERION-30: No good answer exits with code 1](../criteria/CRITERION-30.md)
<!-- LINKS:END -->
