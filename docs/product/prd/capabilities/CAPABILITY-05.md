---
type: Capability
title: Text output for people and structured output for agents
description: jag gives readable text to a person and structured output that an agent
  can parse.
status: stable
kano: must-be
addresses:
- /journeys/JOURNEY-01.md
- /journeys/JOURNEY-02.md
depends_on:
- /capabilities/CAPABILITY-02.md
---

# CAPABILITY-05: Text output for people and structured output for agents

jag gives readable text to a person at the shell. On request, jag gives structured output that an agent can parse, with the same results.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addresses: [JOURNEY-01: Agent builder fixes a wrong answer](../journeys/JOURNEY-01.md)
- addresses: [JOURNEY-02: A person asks a question at the shell](../journeys/JOURNEY-02.md)
- depends on: [CAPABILITY-02: Term and question search](CAPABILITY-02.md)
- constrained by: [CONSTRAINT-05: The structured form only adds fields in one major version](../constraints/CONSTRAINT-05.md)
- subject of: [DECISION-07: Errors are structured in structured output mode](../decisions/DECISION-07.md)
- subject of: [DECISION-27: Only an explicit option turns on structured output](../decisions/DECISION-27.md)
- subject of: [DECISION-28: Structured output has a field for each item that the text shows](../decisions/DECISION-28.md)
- subject of: [DECISION-29: In text mode, notices go to the error stream](../decisions/DECISION-29.md)
- subject of: [DECISION-30: Structured mode always writes a complete output](../decisions/DECISION-30.md)
- included by: [SCOPE-01: Release 1: first public release](../scope/SCOPE-01.md) (must)
<!-- LINKS:END -->
