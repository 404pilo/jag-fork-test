---
type: Feature
title: Go public with a clean history
description: A scan lists the private data in the history. A new public repository
  gets a clean history, and a second scan finds none. This repository stays private.
status: draft
state: backlog
github:
  number: 59
  node_id: I_kwDOVDCdD88AAAABXB-m8A
  url: https://github.com/ai-ready-future/jag/issues/59
  pushed_hash: f68441
  synced:
    state: backlog
parent: /epics/EPIC-05.md
blocked_by:
- /features/FEATURE-17.md
milestone: /milestones/MILESTONE-02.md
traces:
- to: prd:/constraints/CONSTRAINT-01.md
  hash: 1970e3
- to: prd:/decisions/DECISION-51.md
  hash: 53ecd1
---

# FEATURE-19: Go public with a clean history

Before jag goes public, a scan lists the private data in the history. A new public repository gets a clean history, and a second scan finds none. This repository stays private.

Decision (DECISION-51): jag goes public as a new public repository with a clean history. The repository ai-ready-future/jag stays private. Its history is not rewritten. The new repository is created as a private repository. While it is private, the open issues move to it, the board items and the issues are checked, and the second scan runs. Then the new repository becomes public, and after it the board. GitHub cannot move an issue from a private repository to a public one, so the issues move before the new repository becomes public. The board is linked to the new repository, so that outside contributors can see the board and the issues.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-47: Check the full history before jag goes public](../stories/STORY-47.md) | ready | - | - | - | - |
| [STORY-52: Create the new public repository with a clean history](../stories/STORY-52.md) | backlog | - | - | - | - |
| [STORY-59: Limit guesses through the check results of fork pull requests](../stories/STORY-59.md) | backlog | - | - | - | - |
| [STORY-60: Run the private-data check on fork pull requests in the new public repository](../stories/STORY-60.md) | backlog | - | - | - | - |
| [STORY-61: Make the project board and the open issues public with the new repository](../stories/STORY-61.md) | backlog | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-05: Safe public release](../epics/EPIC-05.md)
- blocked by: [FEATURE-17: Block private data in the repository](FEATURE-17.md)
- milestone: [MILESTONE-02: Make it good](../milestones/MILESTONE-02.md)
- traces: [CONSTRAINT-01: The public release holds no private data](../../prd/constraints/CONSTRAINT-01.md)
- traces: [DECISION-51: jag goes public as a new repository with a clean history](../../prd/decisions/DECISION-51.md)
- blocks: [FEATURE-18: Install jag from GitHub Releases, crates.io, or npm](FEATURE-18.md)
- child: [STORY-47: Check the full history before jag goes public](../stories/STORY-47.md)
- child: [STORY-52: Create the new public repository with a clean history](../stories/STORY-52.md)
- child: [STORY-59: Limit guesses through the check results of fork pull requests](../stories/STORY-59.md)
- child: [STORY-60: Run the private-data check on fork pull requests in the new public repository](../stories/STORY-60.md)
- child: [STORY-61: Make the project board and the open issues public with the new repository](../stories/STORY-61.md)
<!-- LINKS:END -->
