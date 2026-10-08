---
type: Decision
title: A result path has the same form as in grep
description: The path of each result follows the form of the folder that the user
  gave.
status: stable
decided_at: '2026-10-06'
alternatives:
- Always give absolute paths
- Always give paths relative to the folder that the user searched
about:
- /capabilities/CAPABILITY-03.md
---

# DECISION-15: A result path has the same form as in grep

The path of each result has the same form as in grep. A relative folder gives relative paths, and an absolute folder gives absolute paths.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-03: File and line for each result](../capabilities/CAPABILITY-03.md)
- traced by: [FEATURE-08: File, line, and snippet for each result](../../roadmap/features/FEATURE-08.md)
- traced by: [STORY-21: See the file, the line, and the snippet of a result](../../roadmap/stories/STORY-21.md)
<!-- LINKS:END -->
