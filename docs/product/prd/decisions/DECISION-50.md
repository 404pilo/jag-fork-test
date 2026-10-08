---
type: Decision
title: The -A, -B, and -C options set the context lines of a result
description: The grep context options set the lines around each match; with no option,
  jag uses its default.
status: stable
decided_at: '2026-10-08'
alternatives:
- jag ignores -A, -B, and -C in a question search
about:
- /decisions/DECISION-10.md
- /decisions/DECISION-34.md
---

# DECISION-50: The -A, -B, and -C options set the context lines of a result

The `-A`, `-B`, and `-C` options set the number of lines around each match, as in grep. When the user gives none of them, a result has the default of a few lines (DECISION-10).

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [DECISION-10: A result is a matching line with the lines around it](DECISION-10.md)
- about: [DECISION-34: Release 1 keeps the grep options that people use most](DECISION-34.md)
- traced by: [FEATURE-13: Common grep options and exit codes](../../roadmap/features/FEATURE-13.md)
<!-- LINKS:END -->
