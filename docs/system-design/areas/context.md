---
type: Area
title: Context
description: The context area of the design.
status: draft
area: context
area_state: decided
anchor: context
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
decisions:
- key: context-1
  binds: 'The outside service that scores the matches: TypeSafe API, Jev model'
  source: human:404pilo, 2026-10-12
- key: context-2
  binds: 'The route to the model service: direct API, or an AI gateway too'
  source: human:404pilo, 2026-10-12
- key: context-3
  binds: 'How jag finds the matching lines: its own search code, or an outside program such as `ag`'
  adr: adr:/0001-find-matches-with-the-ripgrep-libraries.md
  source: adr
---

# Context

<!-- LINKS:START (generated, do not edit) -->

## Links

- decisions adr: [adr:/0001-find-matches-with-the-ripgrep-libraries.md](../../adr/0001-find-matches-with-the-ripgrep-libraries.md)
- area: [context: Context](context.md)
- named by: [COMPONENT-09: TypeSafe API](../components/COMPONENT-09.md)
- named by: [COMPONENT-10: AI gateway](../components/COMPONENT-10.md)
- named by: [context: Context](context.md)
<!-- LINKS:END -->
