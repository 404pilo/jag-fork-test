---
type: Constraint
title: jag fails within 2 × timeout + retry wait when the model service is down
description: When the model service does not answer, jag fails the search within 2
  × the timeout + the retry wait, from the first call of the search. With the defaults,
  this is 9 seconds.
status: draft
constrains:
- /capabilities/CAPABILITY-01.md
- /capabilities/CAPABILITY-02.md
---

# CONSTRAINT-07: jag fails within 2 × timeout + retry wait when the model service is down

When the model service does not answer during a search, jag fails the full search within the worst-case time of its settings: 2 × the timeout of one call + the retry wait, from the first call of the search. This time covers all calls of the search, the one retry of CONSTRAINT-04, and the wait before it. With the default values (a 4-second timeout and a 1-second wait), jag fails within 9 seconds. The user can set both values in whole seconds from 1 to 60, and the limit is computed the same way. An agent that calls jag then knows the longest time that it waits for the cause and the exit code (DECISION-01, DECISION-06, DECISION-52).

<!-- LINKS:START (generated, do not edit) -->

## Links

- constrains: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- constrains: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- subject of: [DECISION-52: The failure limit comes from the timeout and the retry wait](../decisions/DECISION-52.md)
<!-- LINKS:END -->
