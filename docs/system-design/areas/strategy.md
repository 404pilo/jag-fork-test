---
type: Area
title: Solution strategy
description: The solution strategy area of the design.
status: draft
area: strategy
area_state: decided
anchor: solution-strategy
decisions:
- key: strategy-1
  binds: The default count of results to score, and the count of model calls in parallel
  adr: adr:/0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md
  source: adr
- key: strategy-2
  binds: How jag decides that no result answers well
  adr: adr:/0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md
  source: adr
- key: strategy-3
  binds: How jag holds the scoring prompt, and how it gives a version to the prompt
  adr: adr:/0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md
  source: adr
- key: strategy-4
  binds: No fallback when the model is not available
  source: human:404pilo, 2026-10-12
- key: strategy-5
  binds: How jag pre-ranks the matches before it scores them
  source: human:404pilo, 2026-10-12
open_rows:
- 'not migrated: RK-1 (no mitigation and no acceptance)'
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:44:19Z'
---

# Solution strategy

<!-- LINKS:START (generated, do not edit) -->

## Links

- decisions adr: [adr:/0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md](../../adr/0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md)
- decisions adr: [adr:/0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md](../../adr/0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md)
- decisions adr: [adr:/0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md](../../adr/0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md)
- area: [strategy: Solution strategy](strategy.md)
- named by: [COMPONENT-04: Pre-rank](../components/COMPONENT-04.md)
- named by: [COMPONENT-05: Jev scorer](../components/COMPONENT-05.md)
- named by: [strategy: Solution strategy](strategy.md)
<!-- LINKS:END -->
