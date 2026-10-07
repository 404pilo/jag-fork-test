---
type: Problem
title: Keyword search returns too many matches to judge by hand
description: A keyword search returns hundreds of matches, and the agent or person
  must read and judge each one; agents feel this most.
status: stable
---

# PROBLEM-01: Keyword search returns too many matches to judge by hand

> "When my agent searches a folder of text, it gets hundreds of matching lines. It reads them all, fills its context, and costs me tokens and time."

A keyword search returns each line that matches a term, not each line that answers the question. The searcher, an agent or a person, must read each match and judge whether it is relevant. Agents feel this most: an agent reads the full output, so most of its context holds noise.

jag adds that judgment. A term finds the matches, and a prompt in plain language ranks them. jag is a general tool, like grep: it serves agents first, and also operators, knowledge seekers, and shell users.

<!-- LINKS:START (generated, do not edit) -->

## Links

- addressed by: [CAPABILITY-02: Term and question search](../capabilities/CAPABILITY-02.md)
- measured by: [METRIC-01: Answer quality](../metrics/METRIC-01.md)
<!-- LINKS:END -->
