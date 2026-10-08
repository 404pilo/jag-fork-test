---
type: Decision
title: Structured mode always writes a complete output
description: With no results, structured mode writes an empty result list and the
  summary fields.
status: stable
decided_at: '2026-10-07'
alternatives:
- Write nothing, as grep does, and rely on the exit code
about:
- /capabilities/CAPABILITY-05.md
---

# DECISION-30: Structured mode always writes a complete output

When there are no results, structured mode writes a complete output with an empty result list and the summary fields. An agent can then tell "nothing found" from "jag stopped before it finished".

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-05: Text output for people and structured output for agents](../capabilities/CAPABILITY-05.md)
- traced by: [FEATURE-11: Text output for a person and structured output for an agent](../../roadmap/features/FEATURE-11.md)
<!-- LINKS:END -->
