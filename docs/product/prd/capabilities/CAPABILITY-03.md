---
type: Capability
title: File and line for each result
description: Each result shows the file, the line number, and the snippet, so the
  person or the agent can open it.
status: stable
kano: must-be
addresses:
- /needs/NEED-09.md
- /journeys/JOURNEY-02.md
depends_on:
- /capabilities/CAPABILITY-02.md
---

# CAPABILITY-03: File and line for each result

Each result shows the file, the line number, and the snippet. The person or the agent can open the source at that line.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addresses: [NEED-09: Point me to the passage](../needs/NEED-09.md)
- addresses: [JOURNEY-02: A person asks a question at the shell](../journeys/JOURNEY-02.md)
- depends on: [CAPABILITY-02: Term and question search](CAPABILITY-02.md)
- subject of: [DECISION-10: A result is a matching line with the lines around it](../decisions/DECISION-10.md)
- subject of: [DECISION-15: A result path has the same form as in grep](../decisions/DECISION-15.md)
- subject of: [DECISION-16: A joined result shows its line range and marks each matching line](../decisions/DECISION-16.md)
- subject of: [DECISION-17: jag cuts a very long line and marks the cut](../decisions/DECISION-17.md)
- subject of: [DECISION-18: Piped text has the label (standard input)](../decisions/DECISION-18.md)
- included by: [SCOPE-01: Release 1: first public release](../scope/SCOPE-01.md) (must)
- traced by: [EPIC-02: Ranked search with file and line](../../roadmap/epics/EPIC-02.md)
- traced by: [FEATURE-08: File, line, and snippet for each result](../../roadmap/features/FEATURE-08.md)
<!-- LINKS:END -->
