---
type: Capability
title: Show the cost of a search
description: jag reports how many results it scored and what the search cost.
status: stable
kano: performance
addresses:
- /personas/PERSONA-01.md
- /needs/NEED-02.md
depends_on:
- /capabilities/CAPABILITY-02.md
---

# CAPABILITY-07: Show the cost of a search

After each search, jag reports how many results it scored and what the search cost.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addresses: [PERSONA-01: Agent builder](../personas/PERSONA-01.md)
- addresses: [NEED-02: Search output fills the agent context with noise](../needs/NEED-02.md)
- depends on: [CAPABILITY-02: Term and question search](CAPABILITY-02.md)
- subject of: [DECISION-40: The cost is the billed amount in US dollars](../decisions/DECISION-40.md)
- subject of: [DECISION-41: In text mode, the cost line goes to the error stream after each search](../decisions/DECISION-41.md)
- subject of: [DECISION-42: The report counts results scored and results skipped](../decisions/DECISION-42.md)
- subject of: [DECISION-43: A search with no model call reports a cost of zero and the reason](../decisions/DECISION-43.md)
- subject of: [DECISION-44: A failed or stopped search reports the cost up to that point](../decisions/DECISION-44.md)
- subject of: [DECISION-45: A missing cost figure never fails the search](../decisions/DECISION-45.md)
- included by: [SCOPE-01: Release 1: first public release](../scope/SCOPE-01.md) (could)
- included by: [SCOPE-02: Release 2: trust and cost](../scope/SCOPE-02.md) (must)
<!-- LINKS:END -->
