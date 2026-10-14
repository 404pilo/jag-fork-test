---
type: Decision
title: The command takes the term and the paths, and the question only from --ask
description: The form is jag [OPTIONS] TERM [PATH...], with the term first as in grep.
  The question comes only from the option --ask "QUESTION".
status: stable
decided_at: '2026-10-14'
alternatives:
- 'The term, then the question, then the paths: jag TERM "QUESTION" [PATH...]'
- 'The question comes first: jag "QUESTION" TERM [PATH...]'
- The word after the term is the question when it has a space and names no file
- Each word after the term that names no file is the question
- The option --ask and a question after the term, both
about:
- /capabilities/CAPABILITY-06.md
- /capabilities/CAPABILITY-02.md
---

# DECISION-31: The command takes the term and the paths, and the question only from --ask

The user types `jag [OPTIONS] TERM [PATH...]`. The term comes first, as in grep, then the paths. The question comes only from the option `--ask "QUESTION"`, also written `--ask=QUESTION`. The option has no short form. Each word after the term is a path, so jag never has to guess if a word is a question or a path. With no `--ask`, jag works as plain grep (DECISION-32).

Source: the user, hitl answer of 2026-10-14 (FEATURE-12). It replaces the form `jag TERM "QUESTION" [PATH...]` of 2026-10-07. ADR-0018 records it.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- about: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- traced by: [FEATURE-04: Rank matching lines by a question](../../roadmap/features/FEATURE-04.md)
- traced by: [STORY-13: Rank matching lines by a question](../../roadmap/stories/STORY-13.md)
<!-- LINKS:END -->
