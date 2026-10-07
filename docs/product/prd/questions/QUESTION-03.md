---
type: Question
title: Which exit code wins when both limits apply?
description: A search can reach the cost limit and also leave out results; each has
  its own exit code.
status: draft
about:
- /decisions/DECISION-13.md
- /capabilities/CAPABILITY-04.md
---

# QUESTION-03: Which exit code wins when both limits apply?

One search can reach the cost limit and also leave out results to stay in the size limit. Each case has its own exit code, but jag gives only one. Exit 1 already wins when nothing answers well.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [DECISION-13: Reaching the cost limit has its own exit code](../decisions/DECISION-13.md)
- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
<!-- LINKS:END -->
