---
type: Decision
title: Errors are structured in structured output mode
description: In structured output mode, an error is structured, with the cause, the
  fix, and the exit code; in text mode, it is readable text.
status: stable
decided_at: '2026-10-06'
alternatives:
- Errors are always plain text, and only results are structured
about:
- /capabilities/CAPABILITY-01.md
- /capabilities/CAPABILITY-05.md
---

# DECISION-07: Errors are structured in structured output mode

In structured output mode, jag writes each error in the same structured form as the results, with the cause, the fix, and the exit code. An agent can read the error without a second parser. In text mode, the error is readable text.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- about: [CAPABILITY-05: Text output for people and structured output for agents](../capabilities/CAPABILITY-05.md)
<!-- LINKS:END -->
