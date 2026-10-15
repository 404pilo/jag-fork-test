---
type: Feature
title: Search a folder as grep -r does
description: jag searches each subfolder of a folder, skips binary, hidden, and ignored
  files, and says how many files it skipped.
status: draft
state: done
actual_end_date: '2026-10-14'
github:
  number: 11
  node_id: I_kwDOVKMnL88AAAABXTLMuw
  url: https://github.com/ai-ready-future/jag/issues/11
  pushed_hash: '836292'
  synced:
    state: done
parent: /epics/EPIC-02.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-02.md
  hash: 11db8b
- to: prd:/decisions/DECISION-14.md
  hash: 9fedf1
---

# FEATURE-05: Search a folder as grep -r does

jag searches each subfolder of a folder, skips binary, hidden, and ignored files, and says how many files it skipped.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-16: Search each subfolder of a folder](../stories/STORY-16.md) | done | - | - | - | - |
| [STORY-17: Skip noise files and count them](../stories/STORY-17.md) | done | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-02: Ranked search with file and line](../epics/EPIC-02.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-02: Term and question search](../../prd/capabilities/CAPABILITY-02.md)
- traces: [DECISION-14: A folder search works like grep -r and skips noise files](../../prd/decisions/DECISION-14.md)
- blocks: [FEATURE-04: Rank matching lines by a question](FEATURE-04.md)
- blocks: [FEATURE-14: Files, folders, and piped text](FEATURE-14.md)
- child: [STORY-16: Search each subfolder of a folder](../stories/STORY-16.md)
- child: [STORY-17: Skip noise files and count them](../stories/STORY-17.md)
<!-- LINKS:END -->
