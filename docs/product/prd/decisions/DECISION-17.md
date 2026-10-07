---
type: Decision
title: jag cuts a very long line and marks the cut
description: A result shows the exact source text, and jag cuts each line that is
  longer than a fixed length.
status: stable
decided_at: '2026-10-06'
alternatives:
- Always show the full line, and let the output size limit drop the whole result
- Show a shorter summary of the line in place of the source text
about:
- /capabilities/CAPABILITY-03.md
- /decisions/DECISION-10.md
- /capabilities/CAPABILITY-04.md
---

# DECISION-17: jag cuts a very long line and marks the cut

A result shows the exact source text. When a line is longer than a fixed length, jag cuts it and marks the cut, so the reader knows that text is missing. One long line, such as a line of a minified file or a long log line, can't use most of the output size limit.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-03: File and line for each result](../capabilities/CAPABILITY-03.md)
- about: [DECISION-10: A result is a matching line with the lines around it](DECISION-10.md)
- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- subject of: [DECISION-23: jag cuts a top result that is larger than the limit](DECISION-23.md)
<!-- LINKS:END -->
