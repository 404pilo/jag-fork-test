---
type: Decision
title: A model failure during a search fails the full search
description: When the credit or the service stops during a search, jag returns no
  partial result and exits with the code of the cause.
status: stable
decided_at: '2026-10-06'
alternatives:
- Return the snippets that have a score, mark the result incomplete, and use a separate
  exit code
about:
- /capabilities/CAPABILITY-01.md
- /decisions/DECISION-01.md
---

# DECISION-06: A model failure during a search fails the full search

When the credit or the service stops during a search, jag fails the full search. It returns no partial result and exits with the code of the cause. A partial ranking can look complete to an agent. This decision extends DECISION-01.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- about: [DECISION-01: jag fails clearly when the model is not available](DECISION-01.md)
- subject of: [DECISION-39: When the user stops jag, it stops at once with no partial results](DECISION-39.md)
- subject of: [DECISION-44: A failed or stopped search reports the cost up to that point](DECISION-44.md)
- traced by: [FEATURE-03: Clear failure when the model is not available](../../roadmap/features/FEATURE-03.md)
<!-- LINKS:END -->
