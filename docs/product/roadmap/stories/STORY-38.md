---
type: Story
title: Keep the grep exit codes
description: As a shell power user (PERSONA-04), I want the grep exit codes 0, 1,
  and 2, so that my scripts that test grep exit codes keep working.
status: draft
state: ready
github:
  number: 99
  node_id: I_kwDOVDCdD88AAAABXCBYAw
  url: https://github.com/ai-ready-future/jag/issues/99
  pushed_hash: b3558b
  synced:
    state: ready
spec: docs/superpowers/specs/2026-10-14-STORY-35-common-grep-options-and-exit-codes-design.md
parent: /features/FEATURE-13.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/decisions/DECISION-05.md
  hash: 2503eb
---

# STORY-38: Keep the grep exit codes

As a shell power user (PERSONA-04), I want jag to exit with 0 when it returns a result, 1 when it finds no match, and 2 for a usage error, so that my scripts that test the grep exit codes keep working.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [CRITERION-58: jag keeps the grep exit codes](../criteria/CRITERION-58.md) | - | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [FEATURE-13: Common grep options and exit codes](../features/FEATURE-13.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](../../prd/decisions/DECISION-05.md)
- criterion or run of: [CRITERION-58: jag keeps the grep exit codes](../criteria/CRITERION-58.md)
<!-- LINKS:END -->
