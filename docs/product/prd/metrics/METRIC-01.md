---
type: Metric
title: Answer quality
description: jag's ndcg@10 on a held-out evaluation set, compared with blind grep;
  the target is at least 0.70 at release 1.
status: stable
role: north-star
target: ndcg@10 >= 0.70 on the held-out set at release 1
measures:
- /problems/PROBLEM-01.md
- /capabilities/CAPABILITY-02.md
- /journeys/JOURNEY-02.md
---

# METRIC-01: Answer quality

Answer quality measures the core promise of jag: the top lines answer the question. The score is ndcg@10 on a held-out evaluation set, next to blind grep on the same set.

On the private development set, jag scores about 0.72 and blind grep scores about 0.06. Public text gives these numbers as a summary only.

<!-- LINKS:START (generated, do not edit) -->

## Links

- measures: [PROBLEM-01: Keyword search returns too many matches to judge by hand](../problems/PROBLEM-01.md)
- measures: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- measures: [JOURNEY-02: A person asks a question at the shell](../journeys/JOURNEY-02.md)
- guarded by: [METRIC-05: Cost per search](METRIC-05.md)
- guarded by: [METRIC-07: False confidence](METRIC-07.md)
<!-- LINKS:END -->
