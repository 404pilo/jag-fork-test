---
type: Decision
title: A term with no question works as plain grep
description: With no question, jag makes no model call, has no cost, and works as
  grep.
status: stable
decided_at: '2026-10-07'
alternatives:
- A usage error with exit code 2 that asks for a question
- Rank by word match only, and say that no question was given
about:
- /capabilities/CAPABILITY-06.md
- /capabilities/CAPABILITY-02.md
---

# DECISION-32: A term with no question works as plain grep

When the user gives a term and no question, jag works as plain grep. It makes no model call and has no cost. The user can then use jag in place of grep for each search.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- about: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- subject of: [DECISION-43: A search with no model call reports a cost of zero and the reason](DECISION-43.md)
<!-- LINKS:END -->
