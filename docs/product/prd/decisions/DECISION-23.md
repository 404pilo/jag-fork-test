---
type: Decision
title: jag cuts a top result that is larger than the limit
description: When the top result alone is larger than the limit, jag cuts it, marks
  the cut, and says so.
status: stable
decided_at: '2026-10-06'
alternatives:
- Show no results and say that the limit is too small
- Show the full top result and go over the limit
about:
- /capabilities/CAPABILITY-04.md
- /decisions/DECISION-17.md
---

# DECISION-23: jag cuts a top result that is larger than the limit

When the top result alone is larger than the limit, jag shows that result cut to the limit. It marks the cut and says that it cut the result.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- about: [DECISION-17: jag cuts a very long line and marks the cut](DECISION-17.md)
- traced by: [FEATURE-09: Size limit on the text that jag returns](../../roadmap/features/FEATURE-09.md)
<!-- LINKS:END -->
