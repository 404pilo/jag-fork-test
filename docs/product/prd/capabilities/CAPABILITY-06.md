---
type: Capability
title: Feels like grep
description: jag works in a pipe, uses grep-like options and exit codes, and is fast
  enough to replace grep.
status: stable
kano: must-be
addresses:
- /needs/NEED-12.md
- /personas/PERSONA-04.md
depends_on:
- /capabilities/CAPABILITY-02.md
---

# CAPABILITY-06: Feels like grep

jag works in a shell pipe, uses options and exit codes like grep, and returns fast enough that a user replaces grep with it. A shell user keeps their habits.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addresses: [NEED-12: grep, but it understands](../needs/NEED-12.md)
- addresses: [PERSONA-04: Shell power user](../personas/PERSONA-04.md)
- depends on: [CAPABILITY-02: Term and question search](CAPABILITY-02.md)
- constrained by: [CONSTRAINT-06: A typical search is one search on a medium code repository](../constraints/CONSTRAINT-06.md)
- subject of: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](../decisions/DECISION-05.md)
- subject of: [DECISION-27: Only an explicit option turns on structured output](../decisions/DECISION-27.md)
- subject of: [DECISION-31: The command takes the term, then the question, then the paths](../decisions/DECISION-31.md)
- subject of: [DECISION-32: A term with no question works as plain grep](../decisions/DECISION-32.md)
- subject of: [DECISION-33: The term is a regular expression, and -F makes it fixed text](../decisions/DECISION-33.md)
- subject of: [DECISION-34: Release 1 keeps the grep options that people use most](../decisions/DECISION-34.md)
- subject of: [DECISION-35: With no path, jag searches the current folder or reads the pipe](../decisions/DECISION-35.md)
- subject of: [DECISION-36: The user can give any mix of files and folders](../decisions/DECISION-36.md)
- subject of: [DECISION-37: jag shows progress only in a terminal](../decisions/DECISION-37.md)
- subject of: [DECISION-38: Color works as grep --color=auto](../decisions/DECISION-38.md)
- subject of: [DECISION-39: When the user stops jag, it stops at once with no partial results](../decisions/DECISION-39.md)
- measured by: [METRIC-03: Search speed](../metrics/METRIC-03.md)
- included by: [SCOPE-01: Release 1: first public release](../scope/SCOPE-01.md) (must)
- traced by: [EPIC-04: A shell user keeps their habits](../../roadmap/epics/EPIC-04.md)
<!-- LINKS:END -->
