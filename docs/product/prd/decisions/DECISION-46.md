---
type: Decision
title: jag scores each search again and does not reuse scores
description: Release 1 keeps no scores between searches; a repeated search pays the
  model service again.
status: stable
decided_at: '2026-10-07'
alternatives:
- Reuse the score of each unchanged result between searches
about:
- /capabilities/CAPABILITY-08.md
- /scope/SCOPE-01.md
---

# DECISION-46: jag scores each search again and does not reuse scores

jag scores each search again, also when the user asks the same question of the same text. It keeps no scores between searches. Score reuse is an early optimization, so CAPABILITY-08 is out of release 1.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-08: A repeated search costs nothing](../capabilities/CAPABILITY-08.md)
- about: [SCOPE-01: Release 1: first public release](../scope/SCOPE-01.md)
<!-- LINKS:END -->
