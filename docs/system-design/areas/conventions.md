---
type: Area
title: Crosscutting conventions
description: The crosscutting conventions area of the design.
status: draft
area: conventions
area_state: decided
anchor: crosscutting-conventions
decisions:
- key: conventions-1
  binds: The error shape in each output mode
  source: human:404pilo, 2026-10-12
- key: conventions-2
  binds: A model failure fails the full search
  source: human:404pilo, 2026-10-12
- key: conventions-3
  binds: Results go to standard output, and notices go to the error stream
  source: human:404pilo, 2026-10-12
- key: conventions-4
  binds: Ctrl-C and a closed pipe stop jag at once
  source: human:404pilo, 2026-10-12
- key: conventions-5
  binds: The retry wait, and how you change it
  source: human:404pilo, 2026-10-13
- key: conventions-6
  binds: How jag keeps the key secret
  source: human:404pilo, 2026-10-12
- key: conventions-7
  binds: The order in which an option, a variable, and the config file set a value
  source: human:404pilo, 2026-10-12
- key: conventions-8
  binds: 'The one source of the agent instructions: the body of `skills/jag/SKILL.md`'
  adr: adr:/0025-use-the-body-of-the-jag-skill-file-as-the-one-source-of-the.md
  source: adr
open_rows:
- 'not migrated: DF-2 (no reason, trigger, or area)'
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:44:19Z'
---

# Crosscutting conventions

<!-- LINKS:START (generated, do not edit) -->

## Links

- decisions adr: [adr:/0025-use-the-body-of-the-jag-skill-file-as-the-one-source-of-the.md](../../adr/0025-use-the-body-of-the-jag-skill-file-as-the-one-source-of-the.md)
- area: [conventions: Crosscutting conventions](conventions.md)
- named by: [COMPONENT-01: Command line](../components/COMPONENT-01.md)
- named by: [COMPONENT-02: Key and settings](../components/COMPONENT-02.md)
- named by: [COMPONENT-05: Jev scorer](../components/COMPONENT-05.md)
- named by: [COMPONENT-06: Output](../components/COMPONENT-06.md)
- named by: [COMPONENT-07: Readiness check](../components/COMPONENT-07.md)
- named by: [conventions: Crosscutting conventions](conventions.md)
<!-- LINKS:END -->
