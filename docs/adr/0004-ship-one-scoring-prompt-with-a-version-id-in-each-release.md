---
type: adr
title: Ship one scoring prompt with a version id in each release
description: Open this when you change the scoring prompt, or when you trace a change in scores after an upgrade.
area: strategy
status: proposed
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

# ADR-0004: Ship one scoring prompt with a version id in each release

## Context

The scoring prompt decides the answer quality (QS-1), and the threshold of the no-good-answer label is tuned with it (QS-2). Agents read structured output, so a change in scores must be traceable.

## Decision

Each jag release holds one built-in scoring prompt with a version id. Structured output names the id in `prompt_version`. The user cannot change the prompt.

## Why

One prompt keeps one tuned threshold and one quality claim for each release. The version id lets a user trace a change in scores to a release.

## Alternatives considered

- Several built-in prompts that an option selects: useful while tuning, but each prompt needs its own threshold and its own quality numbers.
- A user prompt file: most freedom, but the quality targets hold only for the built-in prompt.

## Consequences

- A prompt change needs a jag release.
- Prompt experiments happen outside the released binary, for example with `jag-bench`.
