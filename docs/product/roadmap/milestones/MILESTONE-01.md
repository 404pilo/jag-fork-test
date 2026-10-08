---
type: Milestone
title: Make it work
description: 'The bare minimum to see jag work: setup, term and question search, file
  and line, a size limit, two output forms, and a feel like grep.'
status: draft
tags:
- make-it-work
traces:
- to: prd:/scope/SCOPE-01.md
  hash: 4be1a2
- to: prd:/scope/SCOPE-10.md
  hash: 1a7d4e
---

# MILESTONE-01: Make it work

The first public release holds the six must-be capabilities. It goes public only after the held-out score confirms the gain over grep.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [EPIC-01: Setup and readiness](../epics/EPIC-01.md) | ready | 0/3 (0%) | - | - | - |
| [EPIC-02: Ranked search with file and line](../epics/EPIC-02.md) | ready | 0/4 (0%) | - | - | - |
| [EPIC-03: Output that fits an agent](../epics/EPIC-03.md) | ready | 0/2 (0%) | - | - | - |
| [EPIC-04: A shell user keeps their habits](../epics/EPIC-04.md) | ready | 0/5 (0%) | - | - | - |
| [FEATURE-01: Store a key one time](../features/FEATURE-01.md) | ready | 0/5 (0%) | - | - | - |
| [FEATURE-02: Readiness check](../features/FEATURE-02.md) | blocked | 0/3 (0%) | - | FEATURE-01 | - |
| [FEATURE-03: Clear failure when the model is not available](../features/FEATURE-03.md) | blocked | 0/4 (0%) | - | FEATURE-01 | - |
| [FEATURE-04: Rank matching lines by a question](../features/FEATURE-04.md) | blocked | 0/3 (0%) | - | FEATURE-01 | - |
| [FEATURE-05: Search a folder as grep -r does](../features/FEATURE-05.md) | ready | 0/2 (0%) | - | - | - |
| [FEATURE-06: Cost limit on scoring](../features/FEATURE-06.md) | blocked | 0/3 (0%) | - | FEATURE-04 | - |
| [FEATURE-08: File, line, and snippet for each result](../features/FEATURE-08.md) | blocked | 0/4 (0%) | - | FEATURE-04 | - |
| [FEATURE-09: Size limit on the text that jag returns](../features/FEATURE-09.md) | blocked | 0/4 (0%) | - | FEATURE-08 | - |
| [FEATURE-11: Text output for a person and structured output for an agent](../features/FEATURE-11.md) | blocked | 0/4 (0%) | - | FEATURE-08 | - |
| [FEATURE-12: Plain grep with no question](../features/FEATURE-12.md) | ready | 0/2 (0%) | - | - | - |
| [FEATURE-13: Common grep options and exit codes](../features/FEATURE-13.md) | ready | 0/4 (0%) | - | - | - |
| [FEATURE-14: Files, folders, and piped text](../features/FEATURE-14.md) | blocked | 0/2 (0%) | - | FEATURE-05 | - |
| [FEATURE-15: Good behavior in a terminal and in a pipe](../features/FEATURE-15.md) | blocked | 0/3 (0%) | - | FEATURE-04 | - |
| [FEATURE-16: Fast enough to replace grep](../features/FEATURE-16.md) | blocked | 0/1 (0%) | - | FEATURE-04, FEATURE-06 | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- traces: [SCOPE-01: Release 1: first public release](../../prd/scope/SCOPE-01.md)
- traces: [SCOPE-10: The held-out score confirms the gain over grep](../../prd/scope/SCOPE-10.md)
- item of: [EPIC-01: Setup and readiness](../epics/EPIC-01.md)
- item of: [EPIC-02: Ranked search with file and line](../epics/EPIC-02.md)
- item of: [EPIC-03: Output that fits an agent](../epics/EPIC-03.md)
- item of: [EPIC-04: A shell user keeps their habits](../epics/EPIC-04.md)
- item of: [FEATURE-01: Store a key one time](../features/FEATURE-01.md)
- item of: [FEATURE-02: Readiness check](../features/FEATURE-02.md)
- item of: [FEATURE-03: Clear failure when the model is not available](../features/FEATURE-03.md)
- item of: [FEATURE-04: Rank matching lines by a question](../features/FEATURE-04.md)
- item of: [FEATURE-05: Search a folder as grep -r does](../features/FEATURE-05.md)
- item of: [FEATURE-06: Cost limit on scoring](../features/FEATURE-06.md)
- item of: [FEATURE-08: File, line, and snippet for each result](../features/FEATURE-08.md)
- item of: [FEATURE-09: Size limit on the text that jag returns](../features/FEATURE-09.md)
- item of: [FEATURE-11: Text output for a person and structured output for an agent](../features/FEATURE-11.md)
- item of: [FEATURE-12: Plain grep with no question](../features/FEATURE-12.md)
- item of: [FEATURE-13: Common grep options and exit codes](../features/FEATURE-13.md)
- item of: [FEATURE-14: Files, folders, and piped text](../features/FEATURE-14.md)
- item of: [FEATURE-15: Good behavior in a terminal and in a pipe](../features/FEATURE-15.md)
- item of: [FEATURE-16: Fast enough to replace grep](../features/FEATURE-16.md)
- item of: [STORY-01: Store a key with setup](../stories/STORY-01.md)
- item of: [STORY-02: Find the key in a fixed order](../stories/STORY-02.md)
- item of: [STORY-03: Replace or remove the stored key](../stories/STORY-03.md)
- item of: [STORY-04: Keep the key private](../stories/STORY-04.md)
- item of: [STORY-05: Find where to get a key](../stories/STORY-05.md)
- item of: [STORY-06: Check the five items of readiness](../stories/STORY-06.md)
- item of: [STORY-07: Use the check result as an exit code](../stories/STORY-07.md)
- item of: [STORY-08: Run the check for almost no cost](../stories/STORY-08.md)
- item of: [STORY-09: Name the cause and the fix of a model failure](../stories/STORY-09.md)
- item of: [STORY-10: Tell each model failure by its exit code](../stories/STORY-10.md)
- item of: [STORY-11: Try again one time before reporting the service as down](../stories/STORY-11.md)
- item of: [STORY-12: Fail the full search when the model stops during it](../stories/STORY-12.md)
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
- item of: [STORY-33: Search with a term and no question](../stories/STORY-33.md)
- item of: [STORY-34: Use plain grep with no key](../stories/STORY-34.md)
- item of: [STORY-35: Use the common grep options](../stories/STORY-35.md)
- item of: [STORY-36: Set the lines around a match](../stories/STORY-36.md)
- item of: [STORY-37: Get a usage error for an unknown option](../stories/STORY-37.md)
- item of: [STORY-38: Keep the grep exit codes](../stories/STORY-38.md)
- item of: [STORY-39: Search with no path](../stories/STORY-39.md)
- item of: [STORY-40: Search a mix of files and folders](../stories/STORY-40.md)
- item of: [STORY-41: See progress only in a terminal](../stories/STORY-41.md)
- item of: [STORY-42: Color the matches as grep does](../stories/STORY-42.md)
- item of: [STORY-43: Stop at once when stopped](../stories/STORY-43.md)
- item of: [STORY-44: Return a typical search in under 5 seconds](../stories/STORY-44.md)
<!-- LINKS:END -->
