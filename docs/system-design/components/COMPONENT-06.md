---
type: Component
title: Output
description: Writes the ranked results as text or as structured output, applies the size limit, and writes the notices and the cost line.
status: draft
slug: output
kind: library
areas:
- building-blocks
- contracts
- conventions
governed_by:
- adr:/0008-write-structured-output-as-one-json-document.md
- /conventions/CONVENTION-01.md
- /conventions/CONVENTION-03.md
realizes:
- prd:/capabilities/CAPABILITY-03.md
- prd:/capabilities/CAPABILITY-04.md
- prd:/capabilities/CAPABILITY-05.md
- prd:/capabilities/CAPABILITY-07.md
aliases:
- C-output
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
---

# COMPONENT-06: Output

Writes the ranked results as text or as structured output, applies the size limit, and writes the notices and the cost line.

<!-- LINKS:START (generated, do not edit) -->

## Links

- governed by: [adr:/0008-write-structured-output-as-one-json-document.md](../../adr/0008-write-structured-output-as-one-json-document.md)
- governed by: [CONVENTION-01: Each failure gives its cause, its fix, and](../conventions/CONVENTION-01.md)
- governed by: [CONVENTION-03: In text mode, results go to standard output,](../conventions/CONVENTION-03.md)
- realizes: [prd:/capabilities/CAPABILITY-03.md](../../product/prd/capabilities/CAPABILITY-03.md)
- realizes: [prd:/capabilities/CAPABILITY-04.md](../../product/prd/capabilities/CAPABILITY-04.md)
- realizes: [prd:/capabilities/CAPABILITY-05.md](../../product/prd/capabilities/CAPABILITY-05.md)
- realizes: [prd:/capabilities/CAPABILITY-07.md](../../product/prd/capabilities/CAPABILITY-07.md)
- area: [building-blocks: Building blocks](../areas/building-blocks.md)
- area: [contracts: Contracts](../areas/contracts.md)
- area: [conventions: Crosscutting conventions](../areas/conventions.md)
- consumes: [INTERFACE-06: IF-6](../interfaces/INTERFACE-06.md)
- used by: [COMPONENT-01: Command line](COMPONENT-01.md)
<!-- LINKS:END -->
