---
type: Metric
title: Cost per search
description: The model cost of one search stays at or below a fixed cap, so quality
  does not rise by scoring more snippets.
status: stable
role: counter
target: at most $0.02 per search (assumed; dev set shows about $0.014)
measures:
- /capabilities/CAPABILITY-02.md
guards:
- /metrics/METRIC-01.md
---

# METRIC-05: Cost per search

The model cost of one search must stay at or below the cap. jag must not raise answer quality by scoring many more snippets.

The cap of $0.02 is an assumption. The development set costs $1.13 per pass over 83 questions, which is about $0.014 for each question.

<!-- LINKS:START (generated, do not edit) -->

## Links

- measures: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- guards: [METRIC-01: Answer quality](METRIC-01.md)
- subject of: [DECISION-12: The cost limit has a default that the user can change](../decisions/DECISION-12.md)
<!-- LINKS:END -->
