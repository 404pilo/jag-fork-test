---
type: Decision
title: The health check examines five items with one free call
description: The health check examines key present, key valid, credit left, service
  reachable, and key file safe, with one call that costs almost nothing.
status: stable
decided_at: '2026-10-06'
alternatives:
- Only local items, with no network call
- A full sample search that shows its cost
about:
- /capabilities/CAPABILITY-01.md
- /constraints/CONSTRAINT-02.md
---

# DECISION-04: The health check examines five items with one free call

The health check examines five items and shows pass or fail for each one:

1. A key is present.
2. The service accepts the key.
3. The account has credit.
4. The service is reachable.
5. Other users cannot read the key file.

The check makes one live call that costs nothing or almost nothing.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- about: [CONSTRAINT-02: jag protects the API key of the user](../constraints/CONSTRAINT-02.md)
- subject of: [DECISION-48: The readiness check exits 0 or with the code of the first cause](DECISION-48.md)
- traced by: [FEATURE-02: Readiness check](../../roadmap/features/FEATURE-02.md)
- traced by: [STORY-06: Check the five items of readiness](../../roadmap/stories/STORY-06.md)
- traced by: [STORY-08: Run the check for almost no cost](../../roadmap/stories/STORY-08.md)
<!-- LINKS:END -->
