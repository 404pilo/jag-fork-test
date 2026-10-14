---
type: Feature
title: File, line, and snippet for each result
description: Each result shows the file, the line range, and the exact source text,
  with each matching line marked, so the person or the agent can open the source at
  that line.
status: draft
state: ready
github:
  number: 49
  node_id: I_kwDOVDCdD88AAAABXB99aA
  url: https://github.com/ai-ready-future/jag/issues/49
  pushed_hash: 3f21d5
  synced:
    state: ready
parent: /epics/EPIC-02.md
blocked_by:
- /features/FEATURE-04.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-03.md
  hash: eb1f82
- to: prd:/decisions/DECISION-15.md
  hash: 10d26e
- to: prd:/decisions/DECISION-16.md
  hash: '646497'
- to: prd:/decisions/DECISION-17.md
  hash: c6c1de
- to: prd:/decisions/DECISION-18.md
  hash: 5fea32
---

# FEATURE-08: File, line, and snippet for each result

Each result shows the file, the line range, and the exact source text, with each matching line marked, so the person or the agent can open the source at that line.

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-21: See the file, the line, and the snippet of a result](../stories/STORY-21.md) | ready | - | - | - | - |
| [STORY-22: See the line range of a joined result](../stories/STORY-22.md) | ready | - | - | - | - |
| [STORY-23: Cut a very long line and mark the cut](../stories/STORY-23.md) | ready | - | - | - | - |
| [STORY-24: Label piped text](../stories/STORY-24.md) | ready | - | - | - | - |
<!-- STATUS:END -->

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-02: Ranked search with file and line](../epics/EPIC-02.md)
- blocked by: [FEATURE-04: Rank matching lines by a question](FEATURE-04.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-03: File and line for each result](../../prd/capabilities/CAPABILITY-03.md)
- traces: [DECISION-15: A result path has the same form as in grep](../../prd/decisions/DECISION-15.md)
- traces: [DECISION-16: A joined result shows its line range and marks each matching line](../../prd/decisions/DECISION-16.md)
- traces: [DECISION-17: jag cuts a very long line and marks the cut](../../prd/decisions/DECISION-17.md)
- traces: [DECISION-18: Piped text has the label (standard input)](../../prd/decisions/DECISION-18.md)
- blocks: [FEATURE-09: Size limit on the text that jag returns](FEATURE-09.md)
- blocks: [FEATURE-11: Text output for a person and structured output for an agent](FEATURE-11.md)
- child: [STORY-21: See the file, the line, and the snippet of a result](../stories/STORY-21.md)
- child: [STORY-22: See the line range of a joined result](../stories/STORY-22.md)
- child: [STORY-23: Cut a very long line and mark the cut](../stories/STORY-23.md)
- child: [STORY-24: Label piped text](../stories/STORY-24.md)
<!-- LINKS:END -->
