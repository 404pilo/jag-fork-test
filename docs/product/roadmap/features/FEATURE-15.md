---
type: Feature
title: Good behavior in a terminal and in a pipe
description: jag shows progress and color only in a terminal, and it stops at once
  with no partial results when the user stops it or the pipe closes.
status: draft
state: backlog
parent: /epics/EPIC-04.md
blocked_by:
- /features/FEATURE-04.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-06.md
  hash: 79d5d3
- to: prd:/decisions/DECISION-37.md
  hash: a45d0f
- to: prd:/decisions/DECISION-38.md
  hash: 20bf23
- to: prd:/decisions/DECISION-39.md
  hash: ea0aa5
---

# FEATURE-15: Good behavior in a terminal and in a pipe

jag shows progress and color only in a terminal, and it stops at once with no partial results when the user stops it or the pipe closes.

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-04: A shell user keeps their habits](../epics/EPIC-04.md)
- blocked by: [FEATURE-04: Rank matching lines by a question](FEATURE-04.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-06: Feels like grep](../../prd/capabilities/CAPABILITY-06.md)
- traces: [DECISION-37: jag shows progress only in a terminal](../../prd/decisions/DECISION-37.md)
- traces: [DECISION-38: Color works as grep --color=auto](../../prd/decisions/DECISION-38.md)
- traces: [DECISION-39: When the user stops jag, it stops at once with no partial results](../../prd/decisions/DECISION-39.md)
- criterion or run of: [CRITERION-62: jag shows progress only in a terminal](../criteria/CRITERION-62.md)
- criterion or run of: [CRITERION-63: Color works as grep --color=auto](../criteria/CRITERION-63.md)
- criterion or run of: [CRITERION-64: jag stops at once when the user stops it](../criteria/CRITERION-64.md)
<!-- LINKS:END -->
