---
type: Story
title: Stay inside a default size limit
description: As an agent builder (PERSONA-01), I want jag to never write more characters
  than a default size limit, so that the output fits the context of my agent.
status: draft
state: done
actual_end_date: '2026-10-14'
github:
  number: 86
  node_id: I_kwDOVDCdD88AAAABXCAZMw
  url: https://github.com/ai-ready-future/jag/issues/86
  pushed_hash: '799827'
  synced:
    state: ready
spec: docs/superpowers/specs/2026-10-13-STORY-25-size-limit-on-the-text-that-jag-returns-design.md
parent: /features/FEATURE-09.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/decisions/DECISION-19.md
  hash: bd8726
- to: prd:/decisions/DECISION-20.md
  hash: f87d35
- to: prd:/decisions/DECISION-22.md
  hash: 5ee453
---

# STORY-25: Stay inside a default size limit

As an agent builder (PERSONA-01), I want jag to never write more characters to standard output than a size limit, with a default that always applies, so that the output of jag fits the context of my agent.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [CRITERION-36: jag never goes over the size limit](../criteria/CRITERION-36.md) | - | - | - | - | - |
| [CRITERION-37: A default size limit always applies](../criteria/CRITERION-37.md) | - | - | - | - | - |
| [CRITERION-38: All text on standard output counts](../criteria/CRITERION-38.md) | - | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [FEATURE-09: Size limit on the text that jag returns](../features/FEATURE-09.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [DECISION-19: The size limit counts characters](../../prd/decisions/DECISION-19.md)
- traces: [DECISION-20: A default size limit always applies](../../prd/decisions/DECISION-20.md)
- traces: [DECISION-22: All text that jag writes to standard output counts toward the size limit](../../prd/decisions/DECISION-22.md)
- criterion or run of: [CRITERION-36: jag never goes over the size limit](../criteria/CRITERION-36.md)
- criterion or run of: [CRITERION-37: A default size limit always applies](../criteria/CRITERION-37.md)
- criterion or run of: [CRITERION-38: All text on standard output counts](../criteria/CRITERION-38.md)
<!-- LINKS:END -->
