---
type: Question
title: Does jag serve users whose text must not leave the machine?
description: jag sends text to an outside model service; some operators and knowledge
  seekers cannot send their text out.
status: stable
about:
- /constraints/CONSTRAINT-03.md
- /personas/PERSONA-02.md
- /personas/PERSONA-03.md
---

# QUESTION-01: Does jag serve users whose text must not leave the machine?

jag sends each candidate snippet to an outside model service to score it. Some users cannot send their text off the machine. The Scope lens must decide whether jag serves these users, and how.

**Answer (2026-10-06):** Not in release 1. [DECISION-02](../decisions/DECISION-02.md) records the limit. A later local model option can serve these users.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CONSTRAINT-03: A TypeSafe key is enough to start](../constraints/CONSTRAINT-03.md)
- about: [PERSONA-02: Operator](../personas/PERSONA-02.md)
- about: [PERSONA-03: Knowledge seeker](../personas/PERSONA-03.md)
- subject of: [DECISION-02: Release 1 does not serve users whose text must stay on the machine](../decisions/DECISION-02.md)
<!-- LINKS:END -->
