---
type: Feature
title: Text output for a person and structured output for an agent
description: An option turns on structured output that an agent can parse, with the
  same results as the text, a named field for each item, structured errors, and a
  form version.
status: draft
aliases:
- FEATURE-10
state: backlog
parent: /epics/EPIC-03.md
blocked_by:
- /features/FEATURE-08.md
milestone: /milestones/MILESTONE-01.md
traces:
- to: prd:/capabilities/CAPABILITY-05.md
  hash: 66c8c5
- to: prd:/decisions/DECISION-07.md
  hash: b268a1
- to: prd:/decisions/DECISION-27.md
  hash: ac8204
- to: prd:/decisions/DECISION-28.md
  hash: dc4887
- to: prd:/decisions/DECISION-29.md
  hash: 9fd4ee
- to: prd:/decisions/DECISION-30.md
  hash: c7e96e
- to: prd:/constraints/CONSTRAINT-05.md
  hash: b88d63
---

# FEATURE-11: Text output for a person and structured output for an agent

An option turns on structured output that an agent can parse, with the same results as the text, a named field for each item, structured errors, and a form version.

Text is the default output, also in a pipe. In text mode, jag writes results to standard output and notices to the error stream, so a pipe receives results only. This part was FEATURE-10.

<!-- LINKS:START (generated, do not edit) -->

## Links

- parent: [EPIC-03: Output that fits an agent](../epics/EPIC-03.md)
- blocked by: [FEATURE-08: File, line, and snippet for each result](FEATURE-08.md)
- milestone: [MILESTONE-01: Make it work](../milestones/MILESTONE-01.md)
- traces: [CAPABILITY-05: Text output for people and structured output for agents](../../prd/capabilities/CAPABILITY-05.md)
- traces: [DECISION-07: Errors are structured in structured output mode](../../prd/decisions/DECISION-07.md)
- traces: [DECISION-27: Only an explicit option turns on structured output](../../prd/decisions/DECISION-27.md)
- traces: [DECISION-28: Structured output has a field for each item that the text shows](../../prd/decisions/DECISION-28.md)
- traces: [DECISION-29: In text mode, notices go to the error stream](../../prd/decisions/DECISION-29.md)
- traces: [DECISION-30: Structured mode always writes a complete output](../../prd/decisions/DECISION-30.md)
- traces: [CONSTRAINT-05: The structured form only adds fields in one major version](../../prd/constraints/CONSTRAINT-05.md)
- criterion or run of: [CRITERION-44: Text is the default output](../criteria/CRITERION-44.md)
- criterion or run of: [CRITERION-45: Notices go to the error stream](../criteria/CRITERION-45.md)
- criterion or run of: [CRITERION-46: Only an explicit option turns on structured output](../criteria/CRITERION-46.md)
- criterion or run of: [CRITERION-47: Each item of the text is a named field](../criteria/CRITERION-47.md)
- criterion or run of: [CRITERION-48: Each result has its rank and no score](../criteria/CRITERION-48.md)
- criterion or run of: [CRITERION-49: Structured mode always writes a complete output](../criteria/CRITERION-49.md)
- criterion or run of: [CRITERION-50: Notices are inside the structure](../criteria/CRITERION-50.md)
- criterion or run of: [CRITERION-51: Errors are structured](../criteria/CRITERION-51.md)
- criterion or run of: [CRITERION-52: The form only adds fields in one major version](../criteria/CRITERION-52.md)
<!-- LINKS:END -->
