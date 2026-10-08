---
type: Decision
title: jag fails clearly when the model is not available
description: When the key is missing or out of credit, or the service is down, jag
  fails with a clear message and does not fall back to word-match ranking.
status: stable
decided_at: '2026-10-05'
alternatives:
- Fall back to word-match ranking and say that the question was not used
about:
- /capabilities/CAPABILITY-01.md
---

# DECISION-01: jag fails clearly when the model is not available

When jag cannot reach the model, because the key is missing or out of credit or the service is down, jag fails with a clear message, the cause, the fix, and a distinct exit code.

jag does not fall back to a ranking by word match. A result that ignores the question can look correct to an agent, and the agent can trust noise.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- subject of: [DECISION-06: A model failure during a search fails the full search](DECISION-06.md)
- traced by: [FEATURE-03: Clear failure when the model is not available](../../roadmap/features/FEATURE-03.md)
- traced by: [STORY-09: Name the cause and the fix of a model failure](../../roadmap/stories/STORY-09.md)
<!-- LINKS:END -->
