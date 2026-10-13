---
type: adr
title: Write structured output as one JSON document
description: Open this when you change the form of structured output, or when agent code that reads jag output breaks.
area: contracts
status: accepted
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
  element: C-output
---

# ADR-0008: Write structured output as one JSON document

## Context

Agents parse structured output (CAPABILITY-05). It must show the form version (CONSTRAINT-05), and an agent must tell "nothing found" from "jag stopped" (DECISION-30).

## Decision

Structured output is one JSON document on standard output, with `version`, `prompt_version`, `results`, and the summary fields of DECISION-28. An error is a JSON document of the same form. In one major version, fields are only added.

## Why

A cut document does not parse, so an agent sees that jag stopped. One document is the easiest form for agent code to read.

## Alternatives considered

- JSON Lines: an agent can read while jag runs, but it must check for a summary line to know that the output is complete.

## Consequences

- jag writes the document only after the search ends.
- The size limit counts the whole document, the wrapper included (DECISION-22).
