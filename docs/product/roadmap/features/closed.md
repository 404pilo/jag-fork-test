---
type: ClosedItems
title: Closed features
description: The features that are done or cancelled.
status: stable
generated:
  by: prd.py regen
---

# Closed features

## MILESTONE-01: Make it work

* [FEATURE-01: Store a key one time](FEATURE-01.md) - The user stores a TypeSafe key one time; jag finds it on each search, never shows it, and tells a new user where to get one.
* [FEATURE-04: Rank matching lines by a question](FEATURE-04.md) - The user runs `jag TERM "QUESTION" [PATH...]`, and jag ranks the lines that match the term by how well they answer the question, and labels weak results.
* [FEATURE-05: Search a folder as grep -r does](FEATURE-05.md) - jag searches each subfolder of a folder, skips binary, hidden, and ignored files, and says how many files it skipped.
* [FEATURE-17: Block private data in the repository](FEATURE-17.md) - A check on each pull request fails when the change adds a key, case content, private evaluation data, or an internal project name.
