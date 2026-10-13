---
type: adr
title: Score the top 200 results in calls of 32, six calls in parallel
description: Open this when you change the default cost limit, the batch size, or the count of parallel Jev calls.
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

# ADR-0002: Score the top 200 results in calls of 32, six calls in parallel

## Context

Jev scores each result, and each scored result costs money and time. A search must cost $0.02 or less (METRIC-05) and return in under 5 seconds (QS-3). One Jev call can score many snippets, and more snippets in one call add little time. The price is $0.042 for each 1M input tokens, and the rate limits are 100K tokens and 80 requests each second.

## Decision

The default cost limit is 200 results (DECISION-12). jag sends 32 results in each Jev call and runs 6 calls in parallel. The user can change the count with an option.

## Why

200 results cost about $0.009 in the worst case and about $0.002 in a typical case. A typical search needs two rounds of calls, about 1 second of model time, which leaves time for the file search inside 5 seconds.

## Alternatives considered

- 100 results, 4 in parallel: half the cost, but fewer matches scored on a big folder.
- 400 results, 8 in parallel: near the cost cap in the worst case, and more risk of HTTP 429 and of a slow search.

## Consequences

- The time of one call with 32 snippets is an estimate. Measure it before release 1.
- A folder with more than 200 good matches depends on the pre-rank to put the answer in the top 200.
