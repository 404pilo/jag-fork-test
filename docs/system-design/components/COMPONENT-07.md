---
type: Component
title: Readiness check
description: Runs the five checks of DECISION-04 with one free call, and gives the exit code of the first cause that fails.
status: draft
slug: check
kind: library
areas:
- building-blocks
- conventions
uses:
- to: /components/COMPONENT-02.md
  style: sync
- to: /components/COMPONENT-05.md
  style: sync
governed_by:
- /conventions/CONVENTION-01.md
- /conventions/CONVENTION-05.md
realizes:
- prd:/capabilities/CAPABILITY-01.md
aliases:
- C-check
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
---

# COMPONENT-07: Readiness check

Runs the five checks of DECISION-04 with one free call, and gives the exit code of the first cause that fails.

<!-- LINKS:START (generated, do not edit) -->

## Links

- uses to: [COMPONENT-02: Key and settings](COMPONENT-02.md)
- uses to: [COMPONENT-05: Jev scorer](COMPONENT-05.md)
- governed by: [CONVENTION-01: Each failure gives its cause, its fix, and](../conventions/CONVENTION-01.md)
- governed by: [CONVENTION-05: Each Jev call times out after 4 seconds.](../conventions/CONVENTION-05.md)
- realizes: [prd:/capabilities/CAPABILITY-01.md](../../product/prd/capabilities/CAPABILITY-01.md)
- area: [building-blocks: Building blocks](../areas/building-blocks.md)
- area: [conventions: Crosscutting conventions](../areas/conventions.md)
- used by: [COMPONENT-01: Command line](COMPONENT-01.md)
<!-- LINKS:END -->
