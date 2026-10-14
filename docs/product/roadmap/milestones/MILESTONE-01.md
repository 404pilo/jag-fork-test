---
type: Milestone
title: Make it work
description: 'The bare minimum to see jag work: store a key, rank the matches of a
  term by a question, file and line, a cost limit, a size limit, and two output forms.'
status: draft
tags:
- make-it-work
traces:
- to: prd:/scope/SCOPE-01.md
  hash: 4be1a2
---

# MILESTONE-01: Make it work

This milestone holds the bare minimum to see jag work. The grep habits, the readiness check, and the release gate are in Make it good. The speed target is in Make it fast.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [EPIC-01: Setup and readiness](../epics/EPIC-01.md) | ready | 2/4 (50%) | - | - | - |
| [EPIC-02: Ranked search with file and line](../epics/EPIC-02.md) | ready | 2/5 (40%) | - | - | - |
| [EPIC-03: Output that fits an agent](../epics/EPIC-03.md) | ready | 0/3 (0%) | - | - | - |
| [FEATURE-01: Store a key one time](../features/FEATURE-01.md) | done | 5/5 (100%) | - | FEATURE-17 | - |
| [FEATURE-04: Rank matching lines by a question](../features/FEATURE-04.md) | done | 3/3 (100%) | - | FEATURE-01, FEATURE-05 | - |
| [FEATURE-05: Search a folder as grep -r does](../features/FEATURE-05.md) | done | 2/2 (100%) | - | - | - |
| [FEATURE-06: Cost limit on scoring](../features/FEATURE-06.md) | in_progress | 0/3 (0%) | - | FEATURE-04 | - |
| [FEATURE-08: File, line, and snippet for each result](../features/FEATURE-08.md) | ready | 0/4 (0%) | - | FEATURE-04 | - |
| [FEATURE-09: Size limit on the text that jag returns](../features/FEATURE-09.md) | blocked | 0/4 (0%) | - | FEATURE-08 | - |
| [FEATURE-11: Text output for a person and structured output for an agent](../features/FEATURE-11.md) | blocked | 0/4 (0%) | - | FEATURE-08 | - |
| [FEATURE-17: Block private data in the repository](../features/FEATURE-17.md) | done | 1/1 (100%) | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- traces: [SCOPE-01: Release 1: first public release](../../prd/scope/SCOPE-01.md)
- item of: [EPIC-01: Setup and readiness](../epics/EPIC-01.md)
- item of: [EPIC-02: Ranked search with file and line](../epics/EPIC-02.md)
- item of: [EPIC-03: Output that fits an agent](../epics/EPIC-03.md)
- item of: [FEATURE-01: Store a key one time](../features/FEATURE-01.md)
- item of: [FEATURE-04: Rank matching lines by a question](../features/FEATURE-04.md)
- item of: [FEATURE-05: Search a folder as grep -r does](../features/FEATURE-05.md)
- item of: [FEATURE-06: Cost limit on scoring](../features/FEATURE-06.md)
- item of: [FEATURE-08: File, line, and snippet for each result](../features/FEATURE-08.md)
- item of: [FEATURE-09: Size limit on the text that jag returns](../features/FEATURE-09.md)
- item of: [FEATURE-11: Text output for a person and structured output for an agent](../features/FEATURE-11.md)
- item of: [FEATURE-17: Block private data in the repository](../features/FEATURE-17.md)
- item of: [STORY-01: Store a key with setup](../stories/STORY-01.md)
- item of: [STORY-02: Find the key in a fixed order](../stories/STORY-02.md)
- item of: [STORY-03: Replace or remove the stored key](../stories/STORY-03.md)
- item of: [STORY-04: Keep the key private](../stories/STORY-04.md)
- item of: [STORY-05: Find where to get a key](../stories/STORY-05.md)
- item of: [STORY-13: Rank matching lines by a question](../stories/STORY-13.md)
- item of: [STORY-14: See the lines around a match](../stories/STORY-14.md)
- item of: [STORY-15: Know when no result answers well](../stories/STORY-15.md)
- item of: [STORY-16: Search each subfolder of a folder](../stories/STORY-16.md)
- item of: [STORY-17: Skip noise files and count them](../stories/STORY-17.md)
- item of: [STORY-18: Score only the best matches](../stories/STORY-18.md)
- item of: [STORY-19: Change the cost limit](../stories/STORY-19.md)
- item of: [STORY-20: Tell a reached cost limit by its exit code](../stories/STORY-20.md)
- item of: [STORY-21: See the file, the line, and the snippet of a result](../stories/STORY-21.md)
- item of: [STORY-22: See the line range of a joined result](../stories/STORY-22.md)
- item of: [STORY-23: Cut a very long line and mark the cut](../stories/STORY-23.md)
- item of: [STORY-24: Label piped text](../stories/STORY-24.md)
- item of: [STORY-25: Stay inside a default size limit](../stories/STORY-25.md)
- item of: [STORY-26: Leave out the lowest-ranked results](../stories/STORY-26.md)
- item of: [STORY-27: Cut a top result that is larger than the limit](../stories/STORY-27.md)
- item of: [STORY-28: Reject a size limit that is not valid](../stories/STORY-28.md)
- item of: [STORY-29: Read text output by default](../stories/STORY-29.md)
- item of: [STORY-30: Turn on structured output](../stories/STORY-30.md)
- item of: [STORY-31: Get a complete structured output with notices and errors](../stories/STORY-31.md)
- item of: [STORY-32: Rely on a stable structured form](../stories/STORY-32.md)
- item of: [STORY-46: Fail a pull request that adds private data](../stories/STORY-46.md)
<!-- LINKS:END -->
