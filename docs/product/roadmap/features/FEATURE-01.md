---
type: Feature
title: Store a key one time
description: The user stores a TypeSafe key one time; jag finds it on each search,
  never shows it, and tells a new user where to get one.
status: draft
state: done
actual_end_date: '2026-10-14'
github:
  number: 7
  node_id: I_kwDOVKMnL88AAAABXTLGkQ
  url: https://github.com/ai-ready-future/jag/issues/7
  synced:
    state: done
  pushed_hash: bb2557
parent: /epics/EPIC-01.md
blocked_by:
- /features/FEATURE-17.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-01.md
  hash: 295cfa
- to: prd:/decisions/DECISION-03.md
  hash: '131646'
- to: prd:/decisions/DECISION-08.md
  hash: 497b8a
- to: prd:/decisions/DECISION-09.md
  hash: cdd875
- to: prd:/constraints/CONSTRAINT-02.md
  hash: fd39da
- to: prd:/constraints/CONSTRAINT-03.md
  hash: 7636cf
---

# FEATURE-01: Store a key one time

The user stores a TypeSafe key one time; jag finds it on each search, never shows it, and tells a new user where to get one.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-01: Store a key with setup](../stories/STORY-01.md) | done | - | - | - | - |
| [STORY-02: Find the key in a fixed order](../stories/STORY-02.md) | done | - | - | - | - |
| [STORY-03: Replace or remove the stored key](../stories/STORY-03.md) | done | - | - | - | - |
| [STORY-04: Keep the key private](../stories/STORY-04.md) | done | - | - | - | - |
| [STORY-05: Find where to get a key](../stories/STORY-05.md) | done | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-01: Setup and readiness](../epics/EPIC-01.md)
- blocked by: [FEATURE-17: Block private data in the repository](FEATURE-17.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-01: Guided key setup and health check](../../prd/capabilities/CAPABILITY-01.md)
- traces: [DECISION-03: jag reads the key from two variables and a config file, in a fixed order](../../prd/decisions/DECISION-03.md)
- traces: [DECISION-08: Setup asks before it replaces a key, and one command removes it](../../prd/decisions/DECISION-08.md)
- traces: [DECISION-09: The missing-key message tells a new user where to get a key](../../prd/decisions/DECISION-09.md)
- traces: [CONSTRAINT-02: jag protects the API key of the user](../../prd/constraints/CONSTRAINT-02.md)
- traces: [CONSTRAINT-03: A TypeSafe key is enough to start](../../prd/constraints/CONSTRAINT-03.md)
- blocks: [FEATURE-02: Readiness check](FEATURE-02.md)
- blocks: [FEATURE-03: Clear failure when the model is not available](FEATURE-03.md)
- blocks: [FEATURE-04: Rank matching lines by a question](FEATURE-04.md)
- blocks: [FEATURE-18: Install jag from GitHub Releases, crates.io, or npm](FEATURE-18.md)
- child: [STORY-01: Store a key with setup](../stories/STORY-01.md)
- child: [STORY-02: Find the key in a fixed order](../stories/STORY-02.md)
- child: [STORY-03: Replace or remove the stored key](../stories/STORY-03.md)
- child: [STORY-04: Keep the key private](../stories/STORY-04.md)
- child: [STORY-05: Find where to get a key](../stories/STORY-05.md)
<!-- LINKS:END -->
