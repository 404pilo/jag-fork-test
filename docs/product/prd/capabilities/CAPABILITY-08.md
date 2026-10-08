---
type: Capability
title: A repeated search costs nothing
description: When the user asks the same question of the same text again, jag does
  not pay for it again.
status: stable
kano: performance
addresses:
- /needs/NEED-02.md
- /personas/PERSONA-01.md
depends_on:
- /capabilities/CAPABILITY-02.md
---

# CAPABILITY-08: A repeated search costs nothing

When the user asks the same question of the same text again, jag gives the same result and does not pay the model service again.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addresses: [NEED-02: Search output fills the agent context with noise](../needs/NEED-02.md)
- addresses: [PERSONA-01: Agent builder](../personas/PERSONA-01.md)
- depends on: [CAPABILITY-02: Term and question search](CAPABILITY-02.md)
- subject of: [DECISION-46: jag scores each search again and does not reuse scores](../decisions/DECISION-46.md)
- included by: [SCOPE-01: Release 1: first public release](../scope/SCOPE-01.md) (wont)
- traced by: [MILESTONE-03: Make it fast](../../roadmap/milestones/MILESTONE-03.md)
<!-- LINKS:END -->
