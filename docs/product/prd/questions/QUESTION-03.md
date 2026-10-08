---
type: Question
title: Which exit code wins when both limits apply?
description: A search can reach the cost limit and also leave out results; each has
  its own exit code.
status: stable
about:
- /decisions/DECISION-13.md
- /capabilities/CAPABILITY-04.md
---

# QUESTION-03: Which exit code wins when both limits apply?

One search can reach the cost limit and also leave out results to stay in the size limit. Each case has its own exit code, but jag gives only one. Exit 1 already wins when nothing answers well.

Answered on 2026-10-08 by [DECISION-47](../decisions/DECISION-47.md): the cost-limit code wins.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [DECISION-13: Reaching the cost limit has its own exit code](../decisions/DECISION-13.md)
- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- subject of: [DECISION-47: The cost-limit exit code wins over the left-out exit code](../decisions/DECISION-47.md)
<!-- LINKS:END -->
