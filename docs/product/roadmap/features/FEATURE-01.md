---
type: Feature
title: Store a key one time
description: The user stores a TypeSafe key one time; jag finds it on each search,
  never shows it, and tells a new user where to get one.
status: draft
state: backlog
parent: /epics/EPIC-01.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-01.md
  hash: 295cfa
- to: prd:/decisions/DECISION-03.md
  hash: '579917'
- to: prd:/decisions/DECISION-08.md
  hash: 497b8a
- to: prd:/decisions/DECISION-09.md
  hash: cdd875
- to: prd:/constraints/CONSTRAINT-02.md
  hash: fd39da
- to: prd:/constraints/CONSTRAINT-03.md
  hash: 7636cf
---

# FEATURE-01: Store a key one time

The user stores a TypeSafe key one time; jag finds it on each search, never shows it, and tells a new user where to get one.

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-01: Setup and readiness](../epics/EPIC-01.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-01: Guided key setup and health check](../../prd/capabilities/CAPABILITY-01.md)
- traces: [DECISION-03: jag reads the key from two variables and a config file, in a fixed order](../../prd/decisions/DECISION-03.md)
- traces: [DECISION-08: Setup asks before it replaces a key, and one command removes it](../../prd/decisions/DECISION-08.md)
- traces: [DECISION-09: The missing-key message tells a new user where to get a key](../../prd/decisions/DECISION-09.md)
- traces: [CONSTRAINT-02: jag protects the API key of the user](../../prd/constraints/CONSTRAINT-02.md)
- traces: [CONSTRAINT-03: A TypeSafe key is enough to start](../../prd/constraints/CONSTRAINT-03.md)
- criterion or run of: [CRITERION-01: Setup stores the key for later searches](../criteria/CRITERION-01.md)
- criterion or run of: [CRITERION-02: jag finds the key in a fixed order](../criteria/CRITERION-02.md)
- criterion or run of: [CRITERION-03: Setup asks before it replaces a key](../criteria/CRITERION-03.md)
- criterion or run of: [CRITERION-04: One command removes the stored key](../criteria/CRITERION-04.md)
- criterion or run of: [CRITERION-05: jag never shows the key](../criteria/CRITERION-05.md)
- criterion or run of: [CRITERION-06: jag refuses a key file that others can read](../criteria/CRITERION-06.md)
- criterion or run of: [CRITERION-07: A missing key points to where to get one](../criteria/CRITERION-07.md)
- criterion or run of: [CRITERION-08: A TypeSafe key is enough to start](../criteria/CRITERION-08.md)
- blocks: [FEATURE-02: Readiness check](FEATURE-02.md)
- blocks: [FEATURE-03: Clear failure when the model is not available](FEATURE-03.md)
- blocks: [FEATURE-04: Rank matching lines by a question](FEATURE-04.md)
<!-- LINKS:END -->
