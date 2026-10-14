---
type: Feature
title: Text output for a person and structured output for an agent
description: An option turns on structured output that an agent can parse, with the
  same results as the text, a named field for each item, structured errors, and a
  form version.
status: draft
aliases:
- FEATURE-10
state: done
actual_start_date: '2026-10-14'
actual_end_date: '2026-10-14'
github:
  number: 51
  node_id: I_kwDOVDCdD88AAAABXB-F-g
  url: https://github.com/ai-ready-future/jag/issues/51
  pushed_hash: e6e9b3
  synced:
    state: done
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

<!-- STATUS:START (generated, do not edit) -->

## Status

| Item | State | Progress | Owner | Blocked by | Drift |
| --- | --- | --- | --- | --- | --- |
| [STORY-29: Read text output by default](../stories/STORY-29.md) | done | - | - | - | - |
| [STORY-30: Turn on structured output](../stories/STORY-30.md) | done | - | - | - | - |
| [STORY-31: Get a complete structured output with notices and errors](../stories/STORY-31.md) | done | - | - | - | - |
| [STORY-32: Rely on a stable structured form](../stories/STORY-32.md) | done | - | - | - | - |
<!-- STATUS:END -->

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
- blocks: [FEATURE-20: Show the cost of a search](FEATURE-20.md)
- child: [STORY-29: Read text output by default](../stories/STORY-29.md)
- child: [STORY-30: Turn on structured output](../stories/STORY-30.md)
- child: [STORY-31: Get a complete structured output with notices and errors](../stories/STORY-31.md)
- child: [STORY-32: Rely on a stable structured form](../stories/STORY-32.md)
<!-- LINKS:END -->
