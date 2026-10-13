---
type: adr
title: Decide no good answer with a fixed threshold on the top probability
description: Open this when you change how jag decides that no result answers the question, or when the decoy AUC drops.
area: strategy
status: accepted
date: 2026-10-12
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: []
parent_spec: docs/product/system-design.md#solution-strategy
model:
  status: modeled
  element: C-scorer
---

# ADR-0003: Decide no good answer with a fixed threshold on the top probability

## Context

When no line answers the question, jag must say so and exit with code 1 (DECISION-11), with a decoy AUC of 0.85 or more (QS-2).

## Decision

jag labels the results "no good answer" when the top Jev probability is below a fixed threshold. The threshold is tuned with the scoring prompt on the development set, and it changes only with a new prompt version.

## Why

One value is easy to tune, to test, and to explain. Jev gives a probability, so one cut on the top score fits the model.

## Alternatives considered

- A threshold option: more control for the user, but one more option to explain, and a wrong value hides the label.
- A relative gap between the top score and the others: needs no calibration, but a folder with no answer can still show a gap.

## Consequences

- A new model version or a new prompt needs a new threshold and a new decoy run.
- The threshold is part of the scoring prompt version (see the prompt ADR).
