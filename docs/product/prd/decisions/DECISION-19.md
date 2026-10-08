---
type: Decision
title: The size limit counts characters
description: The limit on the text that jag returns is a count of characters.
status: stable
decided_at: '2026-10-06'
alternatives:
- Approximate tokens
- A number of results
- A number of lines
about:
- /capabilities/CAPABILITY-04.md
---

# DECISION-19: The size limit counts characters

The limit is a count of characters. The count is exact and the same on each machine, and it is close to the token cost for the agent.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- traced by: [FEATURE-09: Size limit on the text that jag returns](../../roadmap/features/FEATURE-09.md)
- traced by: [STORY-25: Stay inside a default size limit](../../roadmap/stories/STORY-25.md)
<!-- LINKS:END -->
