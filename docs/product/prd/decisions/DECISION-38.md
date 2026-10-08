---
type: Decision
title: Color works as grep --color=auto
description: jag colors the matching text only in a terminal, and --color=never or
  always changes that.
status: stable
decided_at: '2026-10-07'
alternatives:
- No color in release 1
about:
- /capabilities/CAPABILITY-06.md
---

# DECISION-38: Color works as grep --color=auto

jag colors the matching text only when the output is a terminal, as `grep --color=auto` does. The `--color=never` and `--color=always` options change this.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- traced by: [FEATURE-15: Good behavior in a terminal and in a pipe](../../roadmap/features/FEATURE-15.md)
- traced by: [STORY-42: Color the matches as grep does](../../roadmap/stories/STORY-42.md)
<!-- LINKS:END -->
