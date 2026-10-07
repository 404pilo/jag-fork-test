---
type: Decision
title: jag shows progress only in a terminal
description: While jag ranks, it shows a progress line on the error stream only when
  that stream is a terminal.
status: stable
decided_at: '2026-10-07'
alternatives:
- Show nothing until the results are ready
- Show the term matches at once, then show them again in ranked order
about:
- /capabilities/CAPABILITY-06.md
- /decisions/DECISION-29.md
---

# DECISION-37: jag shows progress only in a terminal

While jag ranks, it shows a progress line on the error stream, but only when the error stream is a terminal. jag removes the line when the results come. In a pipe, jag shows no progress.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- about: [DECISION-29: In text mode, notices go to the error stream](DECISION-29.md)
<!-- LINKS:END -->
