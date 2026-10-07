---
type: Decision
title: A default size limit always applies
description: jag always applies a default size limit for an agent; the user can raise
  it or turn it off.
status: stable
decided_at: '2026-10-06'
alternatives:
- No limit until the user sets one, as in grep
- A default limit only in structured output
about:
- /capabilities/CAPABILITY-04.md
---

# DECISION-20: A default size limit always applies

When the user sets no limit, jag applies a default limit that is sized for the context of an agent. The user can raise the limit or turn it off.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
<!-- LINKS:END -->
