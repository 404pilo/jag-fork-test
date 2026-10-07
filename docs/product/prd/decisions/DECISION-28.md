---
type: Decision
title: Structured output has a field for each item that the text shows
description: Each label, count, mark, and notice of the text is a named field; each
  result has its rank and no score.
status: stable
decided_at: '2026-10-07'
alternatives:
- The same fields, and a score for each result
- Only the file, the line, and the snippet
about:
- /capabilities/CAPABILITY-05.md
---

# DECISION-28: Structured output has a field for each item that the text shows

Each item that the text output shows is a named field in structured output: the no-good-answer label, the counts of skipped matches and skipped files, the line range and the marks on matching lines, the marks that show a cut, and the notice about results that jag left out. Each result has its rank. A result has no score.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-05: Text output for people and structured output for agents](../capabilities/CAPABILITY-05.md)
<!-- LINKS:END -->
