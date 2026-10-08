---
type: Decision
title: When nothing answers well, jag labels the best results and exits 1
description: When no result answers the question well, jag shows the best results
  with a clear no-good-answer label and exits with code 1.
status: stable
decided_at: '2026-10-06'
alternatives:
- Show no results, give a plain message, and exit 1
- Show the best results with a label and exit 0
about:
- /capabilities/CAPABILITY-02.md
- /metrics/METRIC-07.md
- /decisions/DECISION-05.md
---

# DECISION-11: When nothing answers well, jag labels the best results and exits 1

When no result answers the question well, jag shows the best results with a clear "no good answer" label. jag exits with code 1, so an agent sees this case from the exit code alone and does not trust noise.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- about: [METRIC-07: False confidence](../metrics/METRIC-07.md)
- about: [DECISION-05: jag keeps grep exit codes and adds one code for each model failure](DECISION-05.md)
- traced by: [FEATURE-04: Rank matching lines by a question](../../roadmap/features/FEATURE-04.md)
<!-- LINKS:END -->
