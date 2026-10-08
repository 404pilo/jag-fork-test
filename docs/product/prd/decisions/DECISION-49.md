---
type: Decision
title: Plain grep mode needs no key
description: A term with no question makes no model call, so jag works with no stored
  key.
status: stable
decided_at: '2026-10-08'
alternatives:
- jag asks for a key for every use
about:
- /decisions/DECISION-32.md
- /capabilities/CAPABILITY-01.md
---

# DECISION-49: Plain grep mode needs no key

When the user gives a term and no question, jag makes no model call. It works with no stored key and does not ask for one.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [DECISION-32: A term with no question works as plain grep](DECISION-32.md)
- about: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- traced by: [FEATURE-12: Plain grep with no question](../../roadmap/features/FEATURE-12.md)
- traced by: [STORY-34: Use plain grep with no key](../../roadmap/stories/STORY-34.md)
<!-- LINKS:END -->
