---
type: Decision
title: All text that jag writes to standard output counts toward the size limit
description: Labels, the structured wrapper, and notices on standard output count;
  text on the error stream does not.
status: stable
decided_at: '2026-10-07'
alternatives:
- Only the source text of the results counts
- All text that jag writes counts, also on the error stream
about:
- /capabilities/CAPABILITY-04.md
---

# DECISION-22: All text that jag writes to standard output counts toward the size limit

Each character that jag writes to standard output counts toward the limit: the file and line labels, the wrapper of structured output, and the notices in structured output. The limit is then true for the text that the agent reads. In text mode, notices go to the error stream and do not count. The simulation of CAPABILITY-05 changed this decision on 2026-10-07.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-04: Limit on the text that jag returns](../capabilities/CAPABILITY-04.md)
- subject of: [DECISION-29: In text mode, notices go to the error stream](DECISION-29.md)
- traced by: [FEATURE-09: Size limit on the text that jag returns](../../roadmap/features/FEATURE-09.md)
<!-- LINKS:END -->
