---
type: Decision
title: jag leaves out whole results with the lowest rank first
description: To stay in the limit, jag removes whole results, starting with the lowest
  rank.
status: stable
decided_at: '2026-10-06'
alternatives:
- Keep the rank order and cut the last result to fit
- Skip a large result and keep a smaller result with a lower rank
about:
- /capabilities/CAPABILITY-04.md
- /metrics/METRIC-06.md
---

# DECISION-21: jag leaves out whole results with the lowest rank first

To stay in the limit, jag removes whole results, starting with the result with the lowest rank. It never cuts a result in the middle.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- about: [METRIC-06: Answer still found](../metrics/METRIC-06.md)
- traced by: [FEATURE-09: Size limit on the text that jag returns](../../roadmap/features/FEATURE-09.md)
<!-- LINKS:END -->
