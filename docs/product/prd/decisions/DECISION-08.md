---
type: Decision
title: Setup asks before it replaces a key, and one command removes it
description: Setup asks the user before it replaces a stored key, and one command
  removes the stored key.
status: stable
decided_at: '2026-10-06'
alternatives:
- Setup replaces the key and does not ask
about:
- /capabilities/CAPABILITY-01.md
---

# DECISION-08: Setup asks before it replaces a key, and one command removes it

When a stored key exists, setup asks the user before it replaces the key. One command removes the stored key.

<!-- LINKS:START (generated, do not edit) -->

## Links

- about: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- traced by: [FEATURE-01: Store a key one time](../../roadmap/features/FEATURE-01.md)
<!-- LINKS:END -->
