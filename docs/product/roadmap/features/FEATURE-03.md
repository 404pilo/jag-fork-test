---
type: Feature
title: Clear failure when the model is not available
description: When the key is missing or out of credit, or the model service is down,
  jag stops with a message that names the cause and the fix, and exits with the code
  of that cause.
status: draft
state: backlog
parent: /epics/EPIC-01.md
blocked_by:
- /features/FEATURE-01.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-01.md
  hash: 295cfa
- to: prd:/decisions/DECISION-01.md
  hash: 303d3c
- to: prd:/decisions/DECISION-05.md
  hash: 2503eb
- to: prd:/decisions/DECISION-06.md
  hash: 09aef4
- to: prd:/constraints/CONSTRAINT-04.md
  hash: e31500
---

# FEATURE-03: Clear failure when the model is not available

When the key is missing or out of credit, or the model service is down, jag stops with a message that names the cause and the fix, and exits with the code of that cause.

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-01: Setup and readiness](../epics/EPIC-01.md)
- blocked by: [FEATURE-01: Store a key one time](FEATURE-01.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-01: Guided key setup and health check](../../prd/capabilities/CAPABILITY-01.md)
- traces: [DECISION-01: jag fails clearly when the model is not available](../../prd/decisions/DECISION-01.md)
- traces: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](../../prd/decisions/DECISION-05.md)
- traces: [DECISION-06: A model failure during a search fails the full search](../../prd/decisions/DECISION-06.md)
- traces: [CONSTRAINT-04: jag tries again one time before it reports the service as down](../../prd/constraints/CONSTRAINT-04.md)
- criterion or run of: [CRITERION-13: Each model failure names its cause and fix](../criteria/CRITERION-13.md)
- criterion or run of: [CRITERION-14: Each model failure has its own exit code](../criteria/CRITERION-14.md)
- criterion or run of: [CRITERION-15: jag never ranks without the model](../criteria/CRITERION-15.md)
- criterion or run of: [CRITERION-16: jag tries again one time before it reports the service as down](../criteria/CRITERION-16.md)
- criterion or run of: [CRITERION-17: A failure during a search fails the full search](../criteria/CRITERION-17.md)
<!-- LINKS:END -->
