---
type: Component
title: Jev scorer
description: Sends the results to Jev in calls of 32, with 6 calls in parallel, under the cost limit. Owns the scoring prompt, the retry, the no-good-answer threshold, and the cost count.
status: draft
slug: scorer
kind: library
areas:
- building-blocks
- strategy
- contracts
- conventions
uses:
- to: /components/COMPONENT-02.md
  style: sync
- to: /components/COMPONENT-09.md
  style: sync
  via: /interfaces/INTERFACE-01.md
- to: /components/COMPONENT-10.md
  style: sync
  via: /interfaces/INTERFACE-02.md
governed_by:
- adr:/0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md
- adr:/0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md
- adr:/0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md
- adr:/0009-pin-the-jev-model-version-in-each-release.md
- adr:/0014-call-the-model-service-with-ureq-and-serde-json-on-std.md
- /conventions/CONVENTION-02.md
- /conventions/CONVENTION-05.md
- /conventions/CONVENTION-06.md
realizes:
- prd:/capabilities/CAPABILITY-02.md
- prd:/capabilities/CAPABILITY-07.md
aliases:
- C-scorer
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
---

# COMPONENT-05: Jev scorer

Sends the results to Jev in calls of 32, with 6 calls in parallel, under the cost limit. Owns the scoring prompt, the retry, the no-good-answer threshold, and the cost count.

<!-- LINKS:START (generated, do not edit) -->

## Links

- uses to: [COMPONENT-02: Key and settings](COMPONENT-02.md)
- uses to: [COMPONENT-09: TypeSafe API](COMPONENT-09.md)
- uses to: [COMPONENT-10: AI gateway](COMPONENT-10.md)
- uses via: [INTERFACE-01: IF-1](../interfaces/INTERFACE-01.md)
- uses via: [INTERFACE-02: IF-2](../interfaces/INTERFACE-02.md)
- governed by: [adr:/0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md](../../adr/0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md)
- governed by: [adr:/0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md](../../adr/0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md)
- governed by: [adr:/0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md](../../adr/0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md)
- governed by: [adr:/0009-pin-the-jev-model-version-in-each-release.md](../../adr/0009-pin-the-jev-model-version-in-each-release.md)
- governed by: [adr:/0014-call-the-model-service-with-ureq-and-serde-json-on-std.md](../../adr/0014-call-the-model-service-with-ureq-and-serde-json-on-std.md)
- governed by: [CONVENTION-02: A model failure fails the full search. jag](../conventions/CONVENTION-02.md)
- governed by: [CONVENTION-05: Each Jev call times out after 4 seconds.](../conventions/CONVENTION-05.md)
- governed by: [CONVENTION-06: Only C-config reads the key. The key lives](../conventions/CONVENTION-06.md)
- realizes: [prd:/capabilities/CAPABILITY-02.md](../../product/prd/capabilities/CAPABILITY-02.md)
- realizes: [prd:/capabilities/CAPABILITY-07.md](../../product/prd/capabilities/CAPABILITY-07.md)
- area: [building-blocks: Building blocks](../areas/building-blocks.md)
- area: [strategy: Solution strategy](../areas/strategy.md)
- area: [contracts: Contracts](../areas/contracts.md)
- area: [conventions: Crosscutting conventions](../areas/conventions.md)
- consumes: [INTERFACE-01: IF-1](../interfaces/INTERFACE-01.md)
- consumes: [INTERFACE-02: IF-2](../interfaces/INTERFACE-02.md)
- consumes: [INTERFACE-06: IF-6](../interfaces/INTERFACE-06.md)
- used by: [COMPONENT-01: Command line](COMPONENT-01.md)
- used by: [COMPONENT-07: Readiness check](COMPONENT-07.md)
- used by: [COMPONENT-08: jag-bench](COMPONENT-08.md)
<!-- LINKS:END -->
