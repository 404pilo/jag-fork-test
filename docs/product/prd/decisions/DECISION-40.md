---
type: Decision
title: The cost is the billed amount in US dollars
description: jag shows the amount that the service billed, in US dollars; with no
  billed figure, it shows a labeled estimate.
status: stable
decided_at: '2026-10-07'
alternatives:
- Always show an estimate in US dollars
- Show service units, not money
about:
- /capabilities/CAPABILITY-07.md
- /metrics/METRIC-05.md
---

# DECISION-40: The cost is the billed amount in US dollars

jag shows the cost in US dollars. It uses the amount that the model service billed. When the service gives no billed amount, jag shows an estimate and labels it as an estimate.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-07: Show the cost of a search](../capabilities/CAPABILITY-07.md)
- about: [METRIC-05: Cost per search](../metrics/METRIC-05.md)
<!-- LINKS:END -->
