---
type: Decision
title: The cost-limit exit code wins over the left-out exit code
description: When a search reaches the cost limit and also leaves out results, jag
  exits with the cost-limit code.
status: stable
decided_at: '2026-10-08'
alternatives:
- The left-out exit code wins
- A separate exit code for both limits
about:
- /questions/QUESTION-03.md
- /decisions/DECISION-13.md
- /decisions/DECISION-26.md
---

# DECISION-47: The cost-limit exit code wins over the left-out exit code

When one search reaches the cost limit and also leaves out results to stay in the size limit, jag exits with the cost-limit code. Results that jag did not score are the larger loss. The notice still says that jag left out results. Exit 1 still wins when nothing answers well (DECISION-13).

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [QUESTION-03: Which exit code wins when both limits apply?](../questions/QUESTION-03.md)
- about: [DECISION-13: Reaching the cost limit has its own exit code](DECISION-13.md)
- about: [DECISION-26: Leaving out results has its own exit code](DECISION-26.md)
- traced by: [FEATURE-06: Cost limit on scoring](../../roadmap/features/FEATURE-06.md)
- traced by: [FEATURE-09: Size limit on the text that jag returns](../../roadmap/features/FEATURE-09.md)
- traced by: [STORY-20: Tell a reached cost limit by its exit code](../../roadmap/stories/STORY-20.md)
- traced by: [STORY-26: Leave out the lowest-ranked results](../../roadmap/stories/STORY-26.md)
<!-- LINKS:END -->
