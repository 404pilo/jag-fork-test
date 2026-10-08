---
type: Capability
title: Limit on the text that jag returns
description: The user sets a limit on how much text jag returns to the agent; jag
  never goes over it and says what it left out.
status: stable
kano: must-be
addresses:
- /needs/NEED-02.md
- /needs/NEED-14.md
depends_on:
- /capabilities/CAPABILITY-02.md
---

# CAPABILITY-04: Limit on the text that jag returns

The user sets a limit on the amount of text that jag returns. jag never goes over the limit. When jag leaves out results, it says how many it left out.

The limit protects the context of the agent that reads the results. It is separate from the cost limit of [DECISION-12](../decisions/DECISION-12.md), which controls how many matches the model scores. The model scores text and writes no text, so the size of the results comes only from the matching text and the lines around it.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addresses: [NEED-02: Search output fills the agent context with noise](../needs/NEED-02.md)
- addresses: [NEED-14: A sub-agent or a vector index adds cost and can still miss](../needs/NEED-14.md)
- depends on: [CAPABILITY-02: Term and question search](CAPABILITY-02.md)
- subject of: [DECISION-10: A result is a matching line with the lines around it](../decisions/DECISION-10.md)
- subject of: [DECISION-17: jag cuts a very long line and marks the cut](../decisions/DECISION-17.md)
- subject of: [DECISION-19: The size limit counts characters](../decisions/DECISION-19.md)
- subject of: [DECISION-20: A default size limit always applies](../decisions/DECISION-20.md)
- subject of: [DECISION-21: jag leaves out whole results with the lowest rank first](../decisions/DECISION-21.md)
- subject of: [DECISION-22: All text that jag writes to standard output counts toward the size limit](../decisions/DECISION-22.md)
- subject of: [DECISION-23: jag cuts a top result that is larger than the limit](../decisions/DECISION-23.md)
- subject of: [DECISION-24: The notice gives the count and how to raise the limit](../decisions/DECISION-24.md)
- subject of: [DECISION-25: A size limit that is not valid is a usage error](../decisions/DECISION-25.md)
- subject of: [DECISION-26: Leaving out results has its own exit code](../decisions/DECISION-26.md)
- subject of: [DECISION-29: In text mode, notices go to the error stream](../decisions/DECISION-29.md)
- measured by: [METRIC-06: Answer still found](../metrics/METRIC-06.md)
- subject of: [QUESTION-03: Which exit code wins when both limits apply?](../questions/QUESTION-03.md)
- included by: [SCOPE-01: Release 1: first public release](../scope/SCOPE-01.md) (must)
- traced by: [EPIC-03: Output that fits an agent](../../roadmap/epics/EPIC-03.md)
<!-- LINKS:END -->
