---
type: AcceptanceCriterion
title: All issues move to the new repository while it is private
description: While the new repository is private, each issue, open or closed, moves
  to it with gh issue transfer. A scan of the issues and the same labels and milestones
  come first.
status: draft
item: /stories/STORY-61.md
---

# CRITERION-103: All issues move to the new repository while it is private

While the new repository is still private, each issue of the private repository, open or closed, moves to it with gh issue transfer. Before the issues move, a scan checks each issue and its comments. It finds no key, no case content, no private evaluation data, and no internal project name. When the scan finds one, no issue moves. Before the issues move, the new repository has the same labels and milestones, so that each issue keeps them.

Why: GitHub cannot move an issue from a private repository to a public one. All issues move, so that the board and the roadmap pages find each issue in one repository.

<!-- LINKS:START (generated, do not edit) -->

## Links

- item: [STORY-61: Make the project board and all issues public with the new repository](../stories/STORY-61.md)
<!-- LINKS:END -->
