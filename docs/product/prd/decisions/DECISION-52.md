---
type: Decision
title: The failure limit comes from the timeout and the retry wait
description: When the model service does not answer, the time limit is 2 × the timeout
  + the retry wait, not a fixed number.
status: draft
decided_at: '2026-10-13'
alternatives:
- A fixed limit of 10 seconds
about:
- /constraints/CONSTRAINT-07.md
- /constraints/CONSTRAINT-04.md
---

# DECISION-52: The failure limit comes from the timeout and the retry wait

The time limit of CONSTRAINT-07 is not a fixed number. It is the worst case that the timeout of one call and the retry wait give: 2 × the timeout + the retry wait, from the first call of the search. This time covers all calls of the search, the one retry of CONSTRAINT-04, and the wait before it. With the default values (a 4-second timeout and a 1-second wait), the limit is 9 seconds. The user can set both values in whole seconds from 1 to 60, and the limit is computed the same way. Source: the user, question Q-as. The earlier limit of 10 seconds was not based on these values.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CONSTRAINT-07: jag fails within 2 × timeout + retry wait when the model service is down](../constraints/CONSTRAINT-07.md)
- about: [CONSTRAINT-04: jag tries again one time before it reports the service as down](../constraints/CONSTRAINT-04.md)
<!-- LINKS:END -->
