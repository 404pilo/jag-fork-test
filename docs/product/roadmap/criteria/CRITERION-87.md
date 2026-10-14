---
type: AcceptanceCriterion
title: A second scan is clean before the new repository becomes public
description: Before the new repository becomes public, a second scan of each of its
  refs finds no key, no case content, no private evaluation data, and no internal
  project name.
status: draft
item: /stories/STORY-52.md
---

# CRITERION-87: A second scan is clean before the new repository becomes public

Before the new repository becomes public, a second scan of each ref that it will have (each branch, each tag, and each `refs/pull/*` ref) finds no key, no case content, no private evaluation data, and no internal project name.

<!-- LINKS:START (generated, do not edit) -->

## Links

- item: [STORY-52: Create the new public repository with a clean history](../stories/STORY-52.md)
<!-- LINKS:END -->
