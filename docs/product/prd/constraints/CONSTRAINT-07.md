---
type: Constraint
title: jag fails within 10 seconds when the model service is down
description: When the model service does not answer, jag fails the search within 10
  seconds, retry included.
status: draft
constrains:
- /capabilities/CAPABILITY-01.md
- /capabilities/CAPABILITY-02.md
---

# CONSTRAINT-07: jag fails within 10 seconds when the model service is down

When the model service does not answer during a search, jag fails the full search within 10 seconds of the first call. The 10 seconds include the one retry of CONSTRAINT-04 and its wait. An agent that calls jag then waits at most 10 seconds for the cause and the exit code (DECISION-01, DECISION-06).

<!-- LINKS:START (generated, do not edit) -->

## Links

- constrains: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- constrains: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
<!-- LINKS:END -->
