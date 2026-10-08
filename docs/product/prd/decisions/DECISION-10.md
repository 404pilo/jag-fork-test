---
type: Decision
title: A result is a matching line with the lines around it
description: A result is the matching line with a few lines before and after it; matches
  near each other join into one result.
status: stable
decided_at: '2026-10-06'
alternatives:
- One matching line with no text around it
- The matching line plus a number of lines around it that the user sets
about:
- /capabilities/CAPABILITY-02.md
- /capabilities/CAPABILITY-03.md
- /capabilities/CAPABILITY-04.md
---

# DECISION-10: A result is a matching line with the lines around it

A result is the matching line with a few lines of text before and after it. When matches are near each other, they join into one result. jag scores each result, and the output size limit counts these results.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- about: [CAPABILITY-03: File and line for each result](../capabilities/CAPABILITY-03.md)
- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- subject of: [DECISION-16: A joined result shows its line range and marks each matching line](DECISION-16.md)
- subject of: [DECISION-17: jag cuts a very long line and marks the cut](DECISION-17.md)
- subject of: [DECISION-50: The -A, -B, and -C options set the context lines of a result](DECISION-50.md)
- traced by: [FEATURE-04: Rank matching lines by a question](../../roadmap/features/FEATURE-04.md)
- traced by: [STORY-14: See the lines around a match](../../roadmap/stories/STORY-14.md)
<!-- LINKS:END -->
