---
type: Synthesis
title: 'STORY-45: test the labeling rule on the development set first'
description: 'Human answer of 2026-11-03: test the rule on the development set, then freeze it, then convert the private set.'
status: draft
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T13:31:35Z'
sources:
- resource: prd:/decisions/DECISION-70.md
- resource: roadmap:/stories/STORY-45.md
- resource: prd:/metrics/METRIC-01.md
---

# STORY-45: test the labeling rule on the development set first

Human answer of 2026-11-03: test the rule on the development set, then freeze it, then convert the private set.

## Question

How to label the private test set of STORY-45? This is the 1 private held-out run of jag-bench before the first tag.

## Facts

- The old held-out set has no search term and no line numbers.
- jag-bench needs a term and answer lines (file, line, grade) for each case.

## Answer

Test the rule first.

1. Before the private set is opened, convert the development set with the labeling rule. Score it.
2. Accept the rule only if the scores are near the earlier development scores: jag about 0.72, grep about 0.06.
3. Freeze the rule. Convert the private set. Do the 1 run.

The rule under test:

- term = the first `checks.mustContain` entry.
- answer lines = the lines of the expected documents that hold the expected value or a `mustContain` entry.
- grade = 1.

## Provenance

- Decided by: human `404pilo`, through hitl, on 2026-11-03 (veracity 90).
- Card from decision-judge: level direction, confidence medium, irreversible, hidden.
- Related pages: `prd:/decisions/DECISION-70.md`, `roadmap:/stories/STORY-45.md`, and the STORY-45 spec.
- This page holds no held-out data, path, or result.

<!-- LINKS:START (generated, do not edit) -->

## Links

- sources: [prd:/decisions/DECISION-70.md](../../../product/prd/decisions/DECISION-70.md)
- sources: [roadmap:/stories/STORY-45.md](../../../product/roadmap/stories/STORY-45.md)
- sources: [prd:/metrics/METRIC-01.md](../../../product/prd/metrics/METRIC-01.md)
<!-- LINKS:END -->
