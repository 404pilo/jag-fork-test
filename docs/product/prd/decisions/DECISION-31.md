---
type: Decision
title: The command takes the term, then the question, then the paths
description: The form is jag TERM "QUESTION" [PATH...], with the term first as in
  grep.
status: stable
decided_at: '2026-10-07'
alternatives:
- 'The question is an option: jag TERM [PATH...] -q "QUESTION"'
- 'The question comes first: jag "QUESTION" TERM [PATH...]'
about:
- /capabilities/CAPABILITY-06.md
- /capabilities/CAPABILITY-02.md
---

# DECISION-31: The command takes the term, then the question, then the paths

The user types `jag TERM "QUESTION" [PATH...]`. The term comes first, as in grep, then the question, then the paths.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- about: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- traced by: [FEATURE-04: Rank matching lines by a question](../../roadmap/features/FEATURE-04.md)
- traced by: [STORY-13: Rank matching lines by a question](../../roadmap/stories/STORY-13.md)
<!-- LINKS:END -->
