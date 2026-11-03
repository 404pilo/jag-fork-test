---
type: Component
title: Match search
description: Uses the ripgrep libraries to find the matches of the term in the files or the piped text, skips noise files, and joins near matches into results with their lines around them.
status: draft
slug: search
kind: library
areas:
- building-blocks
- contracts
governed_by:
- adr:/0001-find-matches-with-the-ripgrep-libraries.md
realizes:
- prd:/capabilities/CAPABILITY-02.md
- prd:/capabilities/CAPABILITY-03.md
- prd:/capabilities/CAPABILITY-06.md
aliases:
- C-search
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
---

# COMPONENT-03: Match search

Uses the ripgrep libraries to find the matches of the term in the files or the piped text, skips noise files, and joins near matches into results with their lines around them.

<!-- LINKS:START (generated, do not edit) -->

## Links

- governed by: [adr:/0001-find-matches-with-the-ripgrep-libraries.md](../../adr/0001-find-matches-with-the-ripgrep-libraries.md)
- realizes: [prd:/capabilities/CAPABILITY-02.md](../../product/prd/capabilities/CAPABILITY-02.md)
- realizes: [prd:/capabilities/CAPABILITY-03.md](../../product/prd/capabilities/CAPABILITY-03.md)
- realizes: [prd:/capabilities/CAPABILITY-06.md](../../product/prd/capabilities/CAPABILITY-06.md)
- area: [building-blocks: Building blocks](../areas/building-blocks.md)
- area: [contracts: Contracts](../areas/contracts.md)
- provides: [INTERFACE-06: IF-6](../interfaces/INTERFACE-06.md)
- used by: [COMPONENT-01: Command line](COMPONENT-01.md)
- used by: [COMPONENT-08: jag-bench](COMPONENT-08.md)
<!-- LINKS:END -->
