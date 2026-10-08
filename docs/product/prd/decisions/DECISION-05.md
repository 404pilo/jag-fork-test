---
type: Decision
title: jag keeps grep exit codes and adds one code for each model failure
description: jag keeps the grep exit codes 0, 1, and 2, and gives each model failure
  its own code above 2.
status: stable
decided_at: '2026-10-06'
alternatives:
- One code for every model failure, with the cause in the message
about:
- /capabilities/CAPABILITY-01.md
- /capabilities/CAPABILITY-06.md
---

# DECISION-05: jag keeps grep exit codes and adds one code for each model failure

jag keeps the grep exit codes 0, 1, and 2. Each model failure gets its own exit code above 2: key missing, key not accepted, no credit, and service down. A script or an agent can act on the cause without reading the message.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- subject of: [DECISION-11: When nothing answers well, jag labels the best results and exits 1](DECISION-11.md)
- subject of: [DECISION-13: Reaching the cost limit has its own exit code](DECISION-13.md)
- subject of: [DECISION-25: A size limit that is not valid is a usage error](DECISION-25.md)
- subject of: [DECISION-26: Leaving out results has its own exit code](DECISION-26.md)
- subject of: [DECISION-34: Release 1 keeps the grep options that people use most](DECISION-34.md)
- subject of: [DECISION-48: The readiness check exits 0 or with the code of the first cause](DECISION-48.md)
- traced by: [FEATURE-03: Clear failure when the model is not available](../../roadmap/features/FEATURE-03.md)
- traced by: [FEATURE-13: Common grep options and exit codes](../../roadmap/features/FEATURE-13.md)
<!-- LINKS:END -->
