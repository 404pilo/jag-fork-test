---
type: Decision
title: In text mode, notices go to the error stream
description: Text mode writes results to standard output and notices to the error
  stream; structured mode writes all of it in the structure.
status: stable
decided_at: '2026-10-07'
alternatives:
- All text goes to standard output in both modes
- Notices go to the error stream in both modes
about:
- /capabilities/CAPABILITY-05.md
- /capabilities/CAPABILITY-04.md
- /decisions/DECISION-22.md
---

# DECISION-29: In text mode, notices go to the error stream

In text mode, jag writes the results to standard output and the notices to the error stream, as grep does with its warnings. The results in a pipe then stay clean. In structured mode, jag writes everything to standard output, inside the structure.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-05: Text output for people and structured output for agents](../capabilities/CAPABILITY-05.md)
- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- about: [DECISION-22: All text that jag writes to standard output counts toward the size limit](DECISION-22.md)
- subject of: [DECISION-37: jag shows progress only in a terminal](DECISION-37.md)
- subject of: [DECISION-41: In text mode, the cost line goes to the error stream after each search](DECISION-41.md)
- traced by: [FEATURE-11: Text output for a person and structured output for an agent](../../roadmap/features/FEATURE-11.md)
<!-- LINKS:END -->
