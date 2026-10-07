---
type: Decision
title: A failed or stopped search reports the cost up to that point
description: The error message gives the cost that jag paid before the failure or
  the stop.
status: stable
decided_at: '2026-10-07'
alternatives:
- Report no cost when a search fails
about:
- /capabilities/CAPABILITY-07.md
- /decisions/DECISION-06.md
- /decisions/DECISION-39.md
---

# DECISION-44: A failed or stopped search reports the cost up to that point

When a search fails or the user stops it, the error message gives the cost that jag paid up to that point. In structured mode, the cost is a field of the error.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-07: Show the cost of a search](../capabilities/CAPABILITY-07.md)
- about: [DECISION-06: A model failure during a search fails the full search](DECISION-06.md)
- about: [DECISION-39: When the user stops jag, it stops at once with no partial results](DECISION-39.md)
<!-- LINKS:END -->
