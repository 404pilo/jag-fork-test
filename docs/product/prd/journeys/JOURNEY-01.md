---
type: Journey
title: Agent builder fixes a wrong answer
description: After the agent gives a wrong answer, the agent builder sets up jag,
  gives it to the agent, and gets correct answers over many runs.
status: stable
serves:
- /personas/PERSONA-01.md
---

# JOURNEY-01: Agent builder fixes a wrong answer

## Trigger

The agent gives a wrong answer. The trace shows that keyword search missed the right lines or buried them in noise.

## Today

1. The agent builder tunes the search instructions. The change fixes one case and breaks another.
2. The agent builder adds a reader sub-agent or a vector index. The cost or the setup goes up, and the miss can still happen.

## With jag

1. **First impression.** The agent builder sets up a TypeSafe key, then runs one command on their own folder and sees good lines.
2. The agent builder gives jag to the agent.
3. **Moment of value.** The agent builder runs the failed question again. The response is faster, the answer is right, and the context is small and clean.

## Moment of truth

The agent answers correctly over many runs, with no more tuning of the search instructions.

<!-- LINKS:START (generated, do not edit) -->

## Links

- serves: [PERSONA-01: Agent builder](../personas/PERSONA-01.md)
- addressed by: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- addressed by: [CAPABILITY-05: Text output for people and structured output for agents](../capabilities/CAPABILITY-05.md)
- addressed by: [CAPABILITY-11: Agent instructions to copy](../capabilities/CAPABILITY-11.md)
- addressed by: [CAPABILITY-12: Measure jag on your own questions](../capabilities/CAPABILITY-12.md)
- measured by: [METRIC-02: Context saved](../metrics/METRIC-02.md)
- measured by: [METRIC-04: Time to first result](../metrics/METRIC-04.md)
<!-- LINKS:END -->
