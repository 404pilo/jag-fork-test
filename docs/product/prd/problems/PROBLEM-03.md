---
type: Problem
title: Current alternatives waste tokens or need setup
description: Agents cope today with raw keyword search, a sub-agent that reads the
  files, or vector search, and each one has a cost.
status: stable
---

# PROBLEM-03: Current alternatives waste tokens or need setup

Agents cope today in three ways:

- **Raw keyword search** with `grep`, `rg`, or `ag`. The output is large, and it can still miss the answer when the answer uses other words.
- **A sub-agent that reads the files** and returns a summary. The tokens are still spent, only in another context.
- **Vector search**, also called RAG. It needs setup, a store, and a new index when the files change.
