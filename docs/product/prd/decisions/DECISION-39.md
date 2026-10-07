---
type: Decision
title: When the user stops jag, it stops at once with no partial results
description: On Ctrl-C or a closed pipe, jag stops, starts no new model call, and
  shows no partial results.
status: stable
decided_at: '2026-10-07'
alternatives:
- On Ctrl-C, show the results that jag scored before the stop, marked as partial
about:
- /capabilities/CAPABILITY-06.md
- /decisions/DECISION-06.md
---

# DECISION-39: When the user stops jag, it stops at once with no partial results

When the user stops jag, or the pipe closes, jag stops at once. It starts no new model call and shows no partial results. It exits as grep does: with code 130 on Ctrl-C, and with no message when the pipe closes.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- about: [DECISION-06: A model failure during a search fails the full search](DECISION-06.md)
<!-- LINKS:END -->
