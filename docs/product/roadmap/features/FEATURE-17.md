---
type: Feature
title: Block private data in the repository
description: A check on each pull request fails when the change adds a key, case content,
  private evaluation data, or an internal project name.
status: draft
state: done
actual_end_date: '2026-10-14'
github:
  number: 21
  node_id: I_kwDOVKMnL88AAAABXTLa3Q
  url: https://github.com/ai-ready-future/jag/issues/21
  pushed_hash: d4f1f1
  synced:
    state: done
parent: /epics/EPIC-05.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/constraints/CONSTRAINT-01.md
  hash: 1970e3
---

# FEATURE-17: Block private data in the repository

A check on each pull request fails when the change adds a key, case content, private evaluation data, or an internal project name.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-46: Fail a pull request that adds private data](../stories/STORY-46.md) | done | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-05: Safe public release](../epics/EPIC-05.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CONSTRAINT-01: The public release holds no private data](../../prd/constraints/CONSTRAINT-01.md)
- blocks: [FEATURE-01: Store a key one time](FEATURE-01.md)
- blocks: [FEATURE-18: Install jag from GitHub Releases, crates.io, or npm](FEATURE-18.md)
- blocks: [FEATURE-19: Go public with a clean history](FEATURE-19.md)
- child: [STORY-46: Fail a pull request that adds private data](../stories/STORY-46.md)
<!-- LINKS:END -->
