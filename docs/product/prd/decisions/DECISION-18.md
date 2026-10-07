---
type: Decision
title: Piped text has the label (standard input)
description: A result from piped text uses the grep label (standard input) and the
  line numbers of the piped text.
status: stable
decided_at: '2026-10-06'
alternatives:
- Show no file name, only the line number
- Do not accept piped text for a question search
about:
- /capabilities/CAPABILITY-03.md
---

# DECISION-18: Piped text has the label (standard input)

Text from a pipe has no file. A result from that text uses the same label as grep, `(standard input)`, and it gives the line numbers in the piped text.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-03: File and line for each result](../capabilities/CAPABILITY-03.md)
- subject of: [DECISION-35: With no path, jag searches the current folder or reads the pipe](DECISION-35.md)
<!-- LINKS:END -->
