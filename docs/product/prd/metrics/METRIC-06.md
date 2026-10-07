---
type: Metric
title: Answer still found
description: jag must not save context by cutting the line that answers the question.
status: stable
role: counter
target: recall of the answer line within the size limit is no lower than grep
measures:
- /capabilities/CAPABILITY-04.md
guards:
- /metrics/METRIC-02.md
---

# METRIC-06: Answer still found

Within the output size limit, jag must keep the line that answers the question at least as often as grep finds it. jag must not save context by cutting the answer.

<!-- LINKS:START (generated, do not edit) -->

## Links

- measures: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- guards: [METRIC-02: Context saved](METRIC-02.md)
- subject of: [DECISION-21: jag leaves out whole results with the lowest rank first](../decisions/DECISION-21.md)
- subject of: [DECISION-24: The notice gives the count and how to raise the limit](../decisions/DECISION-24.md)
<!-- LINKS:END -->
