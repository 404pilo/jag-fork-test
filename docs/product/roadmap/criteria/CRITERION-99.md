---
type: AcceptanceCriterion
title: The Actions event policy lets the check run on fork pull requests
description: Before the new repository becomes public, its Actions event policy allows
  pull_request_target for the private-data check, and a test fork pull request shows
  that the check runs.
status: draft
item: /stories/STORY-60.md
---

# CRITERION-99: The Actions event policy lets the check run on fork pull requests

Before the new repository becomes public, its Actions event policy allows pull_request_target for the private-data check, and a test fork pull request shows that the check runs.

Why: from 2026-11-02, GitHub turns off `pull_request_target` by default in a public repository that has no Actions event policy. Private repositories do not get this default. To let a workflow run, allow `pull_request_target` in an event policy, or allow one workflow file. Source: GitHub changelog, 2026-09-17, "Workflow execution protections in GitHub Actions generally available".

<!-- LINKS:START (generated, do not edit) -->

## Links

- item: [STORY-60: Run the private-data check on fork pull requests in the new public repository](../stories/STORY-60.md)
<!-- LINKS:END -->
