---
type: Feature
title: Go public with a clean history
description: Before the repository goes public, a scan of each commit in the history
  finds no private data, so that no old commit shows it.
status: draft
state: backlog
parent: /epics/EPIC-05.md
blocked_by:
- /features/FEATURE-17.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/constraints/CONSTRAINT-01.md
  hash: 1970e3
---

# FEATURE-19: Go public with a clean history

Before the repository goes public, a scan of each commit in the history finds no private data, so that no old commit shows it.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-47: Check the full history before the repository goes public](../stories/STORY-47.md) | ready | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-05: Safe public release](../epics/EPIC-05.md)
- blocked by: [FEATURE-17: Block private data in the repository](FEATURE-17.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [CONSTRAINT-01: The public release holds no private data](../../prd/constraints/CONSTRAINT-01.md)
- blocks: [FEATURE-18: Install jag from GitHub Releases, crates.io, or npm](FEATURE-18.md)
- child: [STORY-47: Check the full history before the repository goes public](../stories/STORY-47.md)
<!-- LINKS:END -->
