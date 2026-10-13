---
type: Decision
title: jag reads the key from two variables and a config file, in a fixed order
description: jag reads JAG_TYPESAFE_API_KEY, then TYPESAFE_API_KEY, then ~/.config/jag/config.yml;
  the first key found wins.
status: stable
decided_at: '2026-10-06'
alternatives:
- A stored key only
- A key from the environment only
- TYPESAFE_API_KEY first
- The config file first
about:
- /capabilities/CAPABILITY-01.md
---

# DECISION-03: jag reads the key from two variables and a config file, in a fixed order

jag reads the key from these sources, in this order. The first key found wins.

1. The `JAG_TYPESAFE_API_KEY` environment variable.
2. The `TYPESAFE_API_KEY` environment variable.
3. The file `~/.config/jag/config.yml`, at `config.api-key`.

Agent runs and automated jobs, with no person at the terminal, use a variable. The health check names the source that jag uses.

The code of the earlier prototype reads other names. The public release uses this order.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- traced by: [FEATURE-01: Store a key one time](../../roadmap/features/FEATURE-01.md)
- traced by: [STORY-02: Find the key in a fixed order](../../roadmap/stories/STORY-02.md)
<!-- LINKS:END -->
