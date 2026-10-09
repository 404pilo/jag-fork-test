---
type: Milestone
title: Make it good
description: 'The nice-to-have capabilities: grep habits, the readiness check, clear
  failures, agent instructions, cost, data flow, and the release gate.'
status: draft
tags:
- make-it-good
traces:
- to: prd:/scope/SCOPE-02.md
  hash: 8e5942
- to: prd:/scope/SCOPE-03.md
  hash: e57d07
- to: prd:/capabilities/CAPABILITY-11.md
  hash: 0ab6ec
- to: prd:/capabilities/CAPABILITY-06.md
  hash: 79d5d3
- to: prd:/scope/SCOPE-10.md
  hash: 1a7d4e
---

# MILESTONE-02: Make it good

This milestone holds the capabilities that make jag better to use and easier to trust. It holds the grep habits of CAPABILITY-06, the readiness check, clear failures, and the release gate (the held-out score). The items stay in backlog until the next `/prd:plan` run.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [EPIC-04: A shell user keeps their habits](../epics/EPIC-04.md) | backlog | 0/5 (0%) | - | - | - |
| [FEATURE-02: Readiness check](../features/FEATURE-02.md) | backlog | 0/3 (0%) | - | FEATURE-01 | - |
| [FEATURE-03: Clear failure when the model is not available](../features/FEATURE-03.md) | backlog | 0/4 (0%) | - | FEATURE-01 | - |
| [FEATURE-12: Plain grep with no question](../features/FEATURE-12.md) | backlog | 0/2 (0%) | - | - | - |
| [FEATURE-13: Common grep options and exit codes](../features/FEATURE-13.md) | backlog | 0/4 (0%) | - | - | - |
| [FEATURE-14: Files, folders, and piped text](../features/FEATURE-14.md) | backlog | 0/2 (0%) | - | FEATURE-05 | - |
| [FEATURE-15: Good behavior in a terminal and in a pipe](../features/FEATURE-15.md) | backlog | 0/3 (0%) | - | FEATURE-04 | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- traces: [SCOPE-02: Release 2: trust and cost](../../prd/scope/SCOPE-02.md)
- traces: [SCOPE-03: Release 3: question without a term](../../prd/scope/SCOPE-03.md)
- traces: [CAPABILITY-11: Agent instructions to copy](../../prd/capabilities/CAPABILITY-11.md)
- traces: [CAPABILITY-06: Feels like grep](../../prd/capabilities/CAPABILITY-06.md)
- traces: [SCOPE-10: The held-out score confirms the gain over grep](../../prd/scope/SCOPE-10.md)
- item of: [EPIC-04: A shell user keeps their habits](../epics/EPIC-04.md)
- item of: [FEATURE-02: Readiness check](../features/FEATURE-02.md)
- item of: [FEATURE-03: Clear failure when the model is not available](../features/FEATURE-03.md)
- item of: [FEATURE-12: Plain grep with no question](../features/FEATURE-12.md)
- item of: [FEATURE-13: Common grep options and exit codes](../features/FEATURE-13.md)
- item of: [FEATURE-14: Files, folders, and piped text](../features/FEATURE-14.md)
- item of: [FEATURE-15: Good behavior in a terminal and in a pipe](../features/FEATURE-15.md)
- item of: [STORY-06: Check the five items of readiness](../stories/STORY-06.md)
- item of: [STORY-07: Use the check result as an exit code](../stories/STORY-07.md)
- item of: [STORY-08: Run the check for almost no cost](../stories/STORY-08.md)
- item of: [STORY-09: Name the cause and the fix of a model failure](../stories/STORY-09.md)
- item of: [STORY-10: Tell each model failure by its exit code](../stories/STORY-10.md)
- item of: [STORY-11: Try again one time before reporting the service as down](../stories/STORY-11.md)
- item of: [STORY-12: Fail the full search when the model stops during it](../stories/STORY-12.md)
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
- item of: [STORY-45: Run the held-out evaluation before release](../stories/STORY-45.md)
<!-- LINKS:END -->
