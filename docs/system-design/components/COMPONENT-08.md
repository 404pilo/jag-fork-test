---
type: Component
title: jag-bench
description: Runs a set of questions with their expected answers through jag and through plain grep, and reports which one finds the answer more often.
status: draft
slug: bench
kind: cli
areas:
- building-blocks
- deployment
uses:
- to: /components/COMPONENT-03.md
  style: sync
- to: /components/COMPONENT-04.md
  style: sync
- to: /components/COMPONENT-05.md
  style: sync
governed_by:
- adr:/0005-build-jag-as-one-rust-binary.md
realizes:
- prd:/capabilities/CAPABILITY-12.md
aliases:
- C-bench
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
---

# COMPONENT-08: jag-bench

Runs a set of questions with their expected answers through jag and through plain grep, and reports which one finds the answer more often.

<!-- LINKS:START (generated, do not edit) -->

## Links

- uses to: [COMPONENT-03: Match search](COMPONENT-03.md)
- uses to: [COMPONENT-04: Pre-rank](COMPONENT-04.md)
- uses to: [COMPONENT-05: Jev scorer](COMPONENT-05.md)
- governed by: [adr:/0005-build-jag-as-one-rust-binary.md](../../adr/0005-build-jag-as-one-rust-binary.md)
- realizes: [prd:/capabilities/CAPABILITY-12.md](../../product/prd/capabilities/CAPABILITY-12.md)
- area: [building-blocks: Building blocks](../areas/building-blocks.md)
- area: [deployment: Deployment](../areas/deployment.md)
<!-- LINKS:END -->
