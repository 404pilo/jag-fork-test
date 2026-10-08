---
type: Feature
title: Rank matching lines by a question
description: The user runs `jag TERM "QUESTION" [PATH...]`, and jag ranks the lines
  that match the term by how well they answer the question, and labels weak results.
status: draft
aliases:
- FEATURE-07
state: blocked
parent: /epics/EPIC-02.md
blocked_by:
- /features/FEATURE-01.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-02.md
  hash: 11db8b
- to: prd:/decisions/DECISION-10.md
  hash: 089ac2
- to: prd:/decisions/DECISION-31.md
  hash: 8d2856
- to: prd:/decisions/DECISION-33.md
  hash: 8ed9c9
- to: prd:/decisions/DECISION-11.md
  hash: 8e1add
---

# FEATURE-04: Rank matching lines by a question

The user runs `jag TERM "QUESTION" [PATH...]`, and jag ranks the lines that match the term by how well they answer the question.

When no result answers the question well, jag shows the best results with a clear no-good-answer label and exits with code 1, so that an agent does not trust noise. This part was FEATURE-07.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-13: Rank matching lines by a question](../stories/STORY-13.md) | ready | - | - | - | - |
| [STORY-14: See the lines around a match](../stories/STORY-14.md) | ready | - | - | - | - |
| [STORY-15: Know when no result answers well](../stories/STORY-15.md) | ready | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-02: Ranked search with file and line](../epics/EPIC-02.md)
- blocked by: [FEATURE-01: Store a key one time](FEATURE-01.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-02: Term and question search](../../prd/capabilities/CAPABILITY-02.md)
- traces: [DECISION-10: A result is a matching line with the lines around it](../../prd/decisions/DECISION-10.md)
- traces: [DECISION-31: The command takes the term, then the question, then the paths](../../prd/decisions/DECISION-31.md)
- traces: [DECISION-33: The term is a regular expression, and -F makes it fixed text](../../prd/decisions/DECISION-33.md)
- traces: [DECISION-11: When nothing answers well, jag labels the best results and exits 1](../../prd/decisions/DECISION-11.md)
- blocks: [FEATURE-06: Cost limit on scoring](FEATURE-06.md)
- blocks: [FEATURE-08: File, line, and snippet for each result](FEATURE-08.md)
- blocks: [FEATURE-15: Good behavior in a terminal and in a pipe](FEATURE-15.md)
- blocks: [FEATURE-16: Fast enough to replace grep](FEATURE-16.md)
- child: [STORY-13: Rank matching lines by a question](../stories/STORY-13.md)
- child: [STORY-14: See the lines around a match](../stories/STORY-14.md)
- child: [STORY-15: Know when no result answers well](../stories/STORY-15.md)
<!-- LINKS:END -->
