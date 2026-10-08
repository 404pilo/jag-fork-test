---
type: Decision
title: Reaching the cost limit has its own exit code
description: When jag reaches the cost limit, it returns the ranked results, says
  how many matches it skipped, and exits with its own code; exit 1 wins when nothing
  answers well.
status: stable
decided_at: '2026-10-06'
alternatives:
- Reaching the limit is a normal result with exit code 0
about:
- /capabilities/CAPABILITY-02.md
- /decisions/DECISION-05.md
---

# DECISION-13: Reaching the cost limit has its own exit code

When jag reaches the cost limit, it returns the ranked results and says how many matches it skipped. It exits with its own code above 2, so a script can see that jag did not score every match.

When jag reaches the limit and no result answers well, exit code 1 wins. The output still says how many matches jag skipped.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- about: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](DECISION-05.md)
- subject of: [DECISION-42: The report counts results scored and results skipped](DECISION-42.md)
- subject of: [DECISION-47: The cost-limit exit code wins over the left-out exit code](DECISION-47.md)
- subject of: [QUESTION-03: Which exit code wins when both limits apply?](../questions/QUESTION-03.md)
- traced by: [FEATURE-06: Cost limit on scoring](../../roadmap/features/FEATURE-06.md)
<!-- LINKS:END -->
