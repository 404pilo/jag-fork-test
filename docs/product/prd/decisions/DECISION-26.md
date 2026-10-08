---
type: Decision
title: Leaving out results has its own exit code
description: When jag leaves out results to stay in the limit, it exits with its own
  code above 2.
status: stable
decided_at: '2026-10-06'
alternatives:
- Keep exit code 0 and tell the user only in the notice
about:
- /capabilities/CAPABILITY-04.md
- /decisions/DECISION-05.md
---

# DECISION-26: Leaving out results has its own exit code

When jag leaves out results to stay in the limit, it exits with its own exit code above 2. An agent can then see from the exit code alone that it did not get all the results.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- about: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](DECISION-05.md)
- subject of: [DECISION-47: The cost-limit exit code wins over the left-out exit code](DECISION-47.md)
- traced by: [FEATURE-09: Size limit on the text that jag returns](../../roadmap/features/FEATURE-09.md)
<!-- LINKS:END -->
