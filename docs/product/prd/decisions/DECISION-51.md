---
type: Decision
title: jag goes public as a new repository with a clean history
description: A new public repository gets a clean history. The private repository
  stays private as ai-ready-future/jag-private, and its history is not rewritten.
status: draft
decided_at: '2026-10-13'
alternatives:
- Rewrite the history of the private repository and make it public
about:
- /constraints/CONSTRAINT-01.md
---

# DECISION-51: jag goes public as a new repository with a clean history

jag goes public as a new public repository with a clean history. The private repository stays private. Its history is not rewritten. It first gets the name ai-ready-future/jag-private, so that the new public repository can get the name ai-ready-future/jag (ADR-0019). The new repository is created as a private repository. A scan checks each issue of the private repository and its comments, and no issue moves while the scan finds private data. Then, while the new repository is private, all issues move to it, open and closed. Then the board items and the issues are checked, and the second scan runs. Then the new repository becomes public, and after it the board. GitHub cannot move an issue from a private repository to a public one, so the issues move before the new repository becomes public. The board is linked to the new repository, so that outside contributors can see the board and the issues.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CONSTRAINT-01: The public release holds no private data](../constraints/CONSTRAINT-01.md)
- traced by: [FEATURE-19: Go public with a clean history](../../roadmap/features/FEATURE-19.md)
<!-- LINKS:END -->
