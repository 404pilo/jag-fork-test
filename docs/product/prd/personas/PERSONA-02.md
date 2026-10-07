---
type: Persona
title: Operator
description: A person who searches logs, configuration, and system output from the
  shell, for any question.
status: stable
---

# PERSONA-02: Operator

The operator searches logs, configuration, and command output from the shell. The questions vary: the cause of a failure, the state of a system, or the events of an audit trail.

Limits: some users cannot send their text to an outside model service, and each query costs model money. Choice criteria: the tool feels like grep, it gives better hits than grep, and it is open source and free.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addressed by: [CAPABILITY-09: Say which text leaves the machine](../capabilities/CAPABILITY-09.md)
- subject of: [DECISION-02: Release 1 does not serve users whose text must stay on the machine](../decisions/DECISION-02.md)
- served by: [JOURNEY-02: A person asks a question at the shell](../journeys/JOURNEY-02.md)
- served by: [NEED-04: Search logs and system files for any question](../needs/NEED-04.md)
- served by: [NEED-05: The operator must guess the exact keyword](../needs/NEED-05.md)
- served by: [NEED-06: Ask the logs in plain words](../needs/NEED-06.md)
- subject of: [QUESTION-01: Does jag serve users whose text must not leave the machine?](../questions/QUESTION-01.md)
<!-- LINKS:END -->
