---
type: Decision
title: A search with no model call reports a cost of zero and the reason
description: With no question, jag makes no model call, reports a cost of zero, and
  says why.
status: stable
decided_at: '2026-10-07'
alternatives:
- Report a cost of zero with no reason
- Report no cost
about:
- /capabilities/CAPABILITY-07.md
- /decisions/DECISION-32.md
---

# DECISION-43: A search with no model call reports a cost of zero and the reason

When a search makes no model call, jag reports a cost of zero and gives the reason, such as "no question". jag does not reuse scores in release 1 (DECISION-46).

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-07: Show the cost of a search](../capabilities/CAPABILITY-07.md)
- about: [DECISION-32: A term with no question works as plain grep](DECISION-32.md)
- traced by: [FEATURE-20: Show the cost of a search](../../roadmap/features/FEATURE-20.md)
<!-- LINKS:END -->
