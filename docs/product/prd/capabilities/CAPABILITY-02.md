---
type: Capability
title: Term and question search
description: The user gives a folder, a term, and a plain-language question; jag returns
  the matching lines ranked by how well they answer the question.
status: stable
kano: must-be
addresses:
- /problems/PROBLEM-01.md
- /journeys/JOURNEY-02.md
- /needs/NEED-06.md
- /needs/NEED-08.md
- /needs/NEED-11.md
depends_on:
- /capabilities/CAPABILITY-01.md
---

# CAPABILITY-02: Term and question search

The user gives a folder, a term, and a question in plain language. jag finds the lines that match the term and ranks them by how well they answer the question.

- **Too many matches.** jag pre-ranks the matches by word match and scores only the best part, within a cost limit. jag reports how many matches it skipped.
- **No good answer.** When no match answers the question well, jag says so plainly, so that an agent does not trust noise.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addresses: [PROBLEM-01: Keyword search returns too many matches to judge by hand](../problems/PROBLEM-01.md)
- addresses: [JOURNEY-02: A person asks a question at the shell](../journeys/JOURNEY-02.md)
- addresses: [NEED-06: Ask the logs in plain words](../needs/NEED-06.md)
- addresses: [NEED-08: Each match must be judged by hand](../needs/NEED-08.md)
- addresses: [NEED-11: Too many matches unless the exact text is known](../needs/NEED-11.md)
- depends on: [CAPABILITY-01: Guided key setup and health check](CAPABILITY-01.md)
- depended on by: [CAPABILITY-03: File and line for each result](CAPABILITY-03.md)
- depended on by: [CAPABILITY-04: Limit on the text that jag returns](CAPABILITY-04.md)
- depended on by: [CAPABILITY-05: Text output for people and structured output for agents](CAPABILITY-05.md)
- depended on by: [CAPABILITY-06: Feels like grep](CAPABILITY-06.md)
- depended on by: [CAPABILITY-07: Show the cost of a search](CAPABILITY-07.md)
- depended on by: [CAPABILITY-08: A repeated search costs nothing](CAPABILITY-08.md)
- depended on by: [CAPABILITY-09: Say which text leaves the machine](CAPABILITY-09.md)
- depended on by: [CAPABILITY-10: Question without a term](CAPABILITY-10.md)
- depended on by: [CAPABILITY-11: Agent instructions to copy](CAPABILITY-11.md)
- depended on by: [CAPABILITY-12: Measure jag on your own questions](CAPABILITY-12.md)
- constrained by: [CONSTRAINT-07: jag fails within 2 × timeout + retry wait when the model service is down](../constraints/CONSTRAINT-07.md)
- subject of: [DECISION-10: A result is a matching line with the lines around it](../decisions/DECISION-10.md)
- subject of: [DECISION-11: When nothing answers well, jag labels the best results and exits 1](../decisions/DECISION-11.md)
- subject of: [DECISION-12: The cost limit has a default that the user can change](../decisions/DECISION-12.md)
- subject of: [DECISION-13: Reaching the cost limit has its own exit code](../decisions/DECISION-13.md)
- subject of: [DECISION-14: A folder search works like grep -r and skips noise files](../decisions/DECISION-14.md)
- subject of: [DECISION-31: The command takes the term, then the question, then the paths](../decisions/DECISION-31.md)
- subject of: [DECISION-32: A term with no question works as plain grep](../decisions/DECISION-32.md)
- subject of: [DECISION-33: The term is a regular expression, and -F makes it fixed text](../decisions/DECISION-33.md)
- measured by: [METRIC-01: Answer quality](../metrics/METRIC-01.md)
- measured by: [METRIC-05: Cost per search](../metrics/METRIC-05.md)
- measured by: [METRIC-07: False confidence](../metrics/METRIC-07.md)
- included by: [SCOPE-01: Release 1: first public release](../scope/SCOPE-01.md) (must)
- traced by: [EPIC-02: Ranked search with file and line](../../roadmap/epics/EPIC-02.md)
- traced by: [FEATURE-04: Rank matching lines by a question](../../roadmap/features/FEATURE-04.md)
- traced by: [FEATURE-05: Search a folder as grep -r does](../../roadmap/features/FEATURE-05.md)
- traced by: [FEATURE-06: Cost limit on scoring](../../roadmap/features/FEATURE-06.md)
<!-- LINKS:END -->
