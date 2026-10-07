---
type: Metric
title: False confidence
description: When nothing answers the question, jag says so; measured with decoy questions.
status: stable
role: counter
target: decoy AUC >= 0.85
measures:
- /capabilities/CAPABILITY-02.md
guards:
- /metrics/METRIC-01.md
---

# METRIC-07: False confidence

When no line answers the question, jag must say so. Decoy questions with no answer in the folder measure this. On the development set, the decoy AUC of jag is 0.85 to 0.87.

<!-- LINKS:START (generated, do not edit) -->

## Links

- measures: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- guards: [METRIC-01: Answer quality](METRIC-01.md)
- subject of: [DECISION-11: When nothing answers well, jag labels the best results and exits 1](../decisions/DECISION-11.md)
<!-- LINKS:END -->
