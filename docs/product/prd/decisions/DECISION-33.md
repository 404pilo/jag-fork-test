---
type: Decision
title: The term is a regular expression, and -F makes it fixed text
description: The term is a regular expression, as in grep; the -F option makes it
  fixed text.
status: stable
decided_at: '2026-10-07'
alternatives:
- The term is always fixed text
- Fixed text by default, and an option makes it a regular expression
about:
- /capabilities/CAPABILITY-06.md
- /capabilities/CAPABILITY-02.md
---

# DECISION-33: The term is a regular expression, and -F makes it fixed text

The term is a regular expression, as in grep. The `-F` option makes the term fixed text.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- about: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- traced by: [FEATURE-04: Rank matching lines by a question](../../roadmap/features/FEATURE-04.md)
- traced by: [FEATURE-13: Common grep options and exit codes](../../roadmap/features/FEATURE-13.md)
<!-- LINKS:END -->
