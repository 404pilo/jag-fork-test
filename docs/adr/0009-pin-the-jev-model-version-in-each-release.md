---
type: adr
title: Pin the Jev model version in each release
description: Open this when you change the Jev model that jag calls, or when scores change with no jag release.
area: contracts
status: proposed
date: 2026-10-12
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: []
parent_spec: docs/product/system-design.md#contracts
model:
  status: modeled
  element: C-scorer
---

# ADR-0009: Pin the Jev model version in each release

## Context

The scoring prompt and the no-good-answer threshold are tuned on one Jev model (QS-1, QS-2). The service also offers a latest alias that can change at any time.

## Decision

Each jag release pins the Jev model version that its prompt and threshold were tuned on. A setting can name another model, for example for a gateway (IF-2).

## Why

A pinned model keeps the quality numbers of a release true. A model change then comes with a new threshold and a new test run.

## Alternatives considered

- Always call the latest model: users get model updates at once, but the threshold can drift with no jag release.

## Consequences

- A model update reaches users only with a jag release.
- When the service retires a pinned version, the old jag releases fail with the service-down or key error.
