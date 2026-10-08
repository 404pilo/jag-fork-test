---
type: Decision
title: A size limit that is not valid is a usage error
description: A limit of zero, a negative limit, or a value that is not a number gives
  exit code 2.
status: stable
decided_at: '2026-10-06'
alternatives:
- Ignore the value and use the default limit
about:
- /capabilities/CAPABILITY-04.md
- /decisions/DECISION-05.md
---

# DECISION-25: A size limit that is not valid is a usage error

A limit of zero, a negative limit, or a value that is not a number is a usage error. jag says what is wrong and exits with code 2, as grep does.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- about: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](DECISION-05.md)
- traced by: [FEATURE-09: Size limit on the text that jag returns](../../roadmap/features/FEATURE-09.md)
<!-- LINKS:END -->
