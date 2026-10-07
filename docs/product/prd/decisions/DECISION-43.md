---
type: Decision
title: A search with no model call reports a cost of zero and the reason
description: With no question, or with reused scores, jag reports a cost of zero and
  says why.
status: stable
decided_at: '2026-10-07'
alternatives:
- Report a cost of zero with no reason
- Report no cost
about:
- /capabilities/CAPABILITY-07.md
- /decisions/DECISION-32.md
- /capabilities/CAPABILITY-08.md
---

# DECISION-43: A search with no model call reports a cost of zero and the reason

When a search makes no model call, jag reports a cost of zero and gives the reason: "no question" or "reused from an earlier search".

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-07: Show the cost of a search](../capabilities/CAPABILITY-07.md)
- about: [DECISION-32: A term with no question works as plain grep](DECISION-32.md)
- about: [CAPABILITY-08: A repeated search costs nothing](../capabilities/CAPABILITY-08.md)
<!-- LINKS:END -->
