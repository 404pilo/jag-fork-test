---
type: Problem
title: 'Why now: cheap scoring models and CLI-first agents'
description: Cheap yes/no scoring models, agents that call shell tools, first evidence,
  and a TypeSafe showcase make now the right time.
status: stable
---

# PROBLEM-04: Why now: cheap scoring models and CLI-first agents

Four changes make now the right time:

- **Cheap yes/no models.** Small scoring models, such as TypeSafe Jev, can score each snippet fast and at low cost. Before, only a large model could judge relevance.
- **Agents now use command-line tools.** Coding agents call shell tools all day. A command-line tool works in each of them with no plugin work.
- **First evidence.** On a private evaluation set, ndcg@10 rose from 0.06 with blind keyword search to about 0.72 with jag. The held-out run is not done, so these numbers are not final.
- **A TypeSafe showcase.** A public tool shows what TypeSafe models can do.
