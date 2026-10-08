---
type: Decision
title: Only an explicit option turns on structured output
description: Text is the default, also in a pipe; an option turns on structured output.
status: stable
decided_at: '2026-10-07'
alternatives:
- Give structured output when the output does not go to a terminal
- An option, plus a setting that makes structured output the default for an agent
about:
- /capabilities/CAPABILITY-05.md
- /capabilities/CAPABILITY-06.md
---

# DECISION-27: Only an explicit option turns on structured output

Text output is the default, also when the output goes to a pipe, as in grep. Only an explicit option turns on structured output.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-05: Text output for people and structured output for agents](../capabilities/CAPABILITY-05.md)
- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- traced by: [FEATURE-11: Text output for a person and structured output for an agent](../../roadmap/features/FEATURE-11.md)
<!-- LINKS:END -->
