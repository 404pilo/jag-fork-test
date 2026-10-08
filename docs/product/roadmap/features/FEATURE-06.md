---
type: Feature
title: Cost limit on scoring
description: When a term matches too many lines, jag scores only the best part up
  to a cost limit that the user can change, and says how many matches it skipped.
status: draft
state: backlog
parent: /epics/EPIC-02.md
blocked_by:
- /features/FEATURE-04.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-02.md
  hash: 11db8b
- to: prd:/decisions/DECISION-12.md
  hash: b7b943
- to: prd:/decisions/DECISION-13.md
  hash: 3ddee2
- to: prd:/decisions/DECISION-47.md
  hash: 2ebb9b
---

# FEATURE-06: Cost limit on scoring

When a term matches too many lines, jag scores only the best part up to a cost limit that the user can change, and says how many matches it skipped.

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-02: Ranked search with file and line](../epics/EPIC-02.md)
- blocked by: [FEATURE-04: Rank matching lines by a question](FEATURE-04.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-02: Term and question search](../../prd/capabilities/CAPABILITY-02.md)
- traces: [DECISION-12: The cost limit has a default that the user can change](../../prd/decisions/DECISION-12.md)
- traces: [DECISION-13: Reaching the cost limit has its own exit code](../../prd/decisions/DECISION-13.md)
- traces: [DECISION-47: The cost-limit exit code wins over the left-out exit code](../../prd/decisions/DECISION-47.md)
- criterion or run of: [CRITERION-25: jag scores only the best matches](../criteria/CRITERION-25.md)
- criterion or run of: [CRITERION-26: The cost limit has a default that the user can change](../criteria/CRITERION-26.md)
- criterion or run of: [CRITERION-27: jag says how many matches it skipped](../criteria/CRITERION-27.md)
- criterion or run of: [CRITERION-28: Reaching the cost limit has its own exit code](../criteria/CRITERION-28.md)
- criterion or run of: [CRITERION-66: Cost-limit code wins when both limits apply](../criteria/CRITERION-66.md)
- blocks: [FEATURE-16: Fast enough to replace grep](FEATURE-16.md)
<!-- LINKS:END -->
