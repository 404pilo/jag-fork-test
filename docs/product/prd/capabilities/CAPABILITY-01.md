---
type: Capability
title: Guided key setup and health check
description: The user stores a TypeSafe key one time, and one command confirms that
  jag is ready or says what to fix.
status: stable
kano: must-be
addresses:
- /journeys/JOURNEY-01.md
- /needs/NEED-03.md
---

# CAPABILITY-01: Guided key setup and health check

The user stores a TypeSafe key one time. One command confirms that jag is ready, or names each problem and its fix.

When the key is missing or out of credit, or the model service is down, jag fails with a clear message. The message names the cause and the fix, and jag exits with a distinct exit code. jag does not return a result that ignores the question.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addresses: [JOURNEY-01: Agent builder fixes a wrong answer](../journeys/JOURNEY-01.md)
- addresses: [NEED-03: Drop-in search with no setup](../needs/NEED-03.md)
- depended on by: [CAPABILITY-02: Term and question search](CAPABILITY-02.md)
- constrained by: [CONSTRAINT-04: jag tries again one time before it reports the service as down](../constraints/CONSTRAINT-04.md)
- constrained by: [CONSTRAINT-07: jag fails within 2 × timeout + retry wait when the model service is down](../constraints/CONSTRAINT-07.md)
- subject of: [DECISION-01: jag fails clearly when the model is not available](../decisions/DECISION-01.md)
- subject of: [DECISION-03: jag reads the key from two variables and a config file, in a fixed order](../decisions/DECISION-03.md)
- subject of: [DECISION-04: The health check examines five items with one free call](../decisions/DECISION-04.md)
- subject of: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](../decisions/DECISION-05.md)
- subject of: [DECISION-06: A model failure during a search fails the full search](../decisions/DECISION-06.md)
- subject of: [DECISION-07: Errors are structured in structured output mode](../decisions/DECISION-07.md)
- subject of: [DECISION-08: Setup asks before it replaces a key, and one command removes it](../decisions/DECISION-08.md)
- subject of: [DECISION-09: The missing-key message tells a new user where to get a key](../decisions/DECISION-09.md)
- subject of: [DECISION-48: The readiness check exits 0 or with the code of the first cause](../decisions/DECISION-48.md)
- subject of: [DECISION-49: Plain grep mode needs no key](../decisions/DECISION-49.md)
- measured by: [METRIC-04: Time to first result](../metrics/METRIC-04.md)
- measured by: [METRIC-08: Setup failures](../metrics/METRIC-08.md)
- included by: [SCOPE-01: Release 1: first public release](../scope/SCOPE-01.md) (must)
- traced by: [EPIC-01: Setup and readiness](../../roadmap/epics/EPIC-01.md)
- traced by: [FEATURE-01: Store a key one time](../../roadmap/features/FEATURE-01.md)
- traced by: [FEATURE-02: Readiness check](../../roadmap/features/FEATURE-02.md)
- traced by: [FEATURE-03: Clear failure when the model is not available](../../roadmap/features/FEATURE-03.md)
<!-- LINKS:END -->
