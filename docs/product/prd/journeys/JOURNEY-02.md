---
type: Journey
title: A person asks a question at the shell
description: A person whose term search returns too many matches adds a plain-language
  question to the same command and reads the few lines that answer it.
status: stable
serves:
- /personas/PERSONA-02.md
- /personas/PERSONA-03.md
- /personas/PERSONA-04.md
---

# JOURNEY-02: A person asks a question at the shell

## Trigger

A search for a term, such as `error`, a topic, or a name, returns too many matches.

## Today

The person must guess the exact text and search again, or open each match and judge it by hand.

## With jag

1. The person runs the same search and adds a question in plain language to the same command.
2. **Moment of value.** The person reads the few top lines, with the file and the line number of each one.

## Moment of truth

The top lines answer the question, so the person does not go back to plain grep.

<!-- LINKS:START (generated, do not edit) -->

## Links

- serves: [PERSONA-02: Operator](../personas/PERSONA-02.md)
- serves: [PERSONA-03: Knowledge seeker](../personas/PERSONA-03.md)
- serves: [PERSONA-04: Shell power user](../personas/PERSONA-04.md)
- addressed by: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- addressed by: [CAPABILITY-03: File and line for each result](../capabilities/CAPABILITY-03.md)
- addressed by: [CAPABILITY-05: Text output for people and structured output for agents](../capabilities/CAPABILITY-05.md)
- measured by: [METRIC-01: Answer quality](../metrics/METRIC-01.md)
<!-- LINKS:END -->
