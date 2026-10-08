---
type: Constraint
title: jag protects the API key of the user
description: jag never shows the API key and refuses a key file that other users on
  the machine can read.
status: stable
---

# CONSTRAINT-02: jag protects the API key of the user

jag must never print, log, or send the API key of the user to a place other than the model service. jag must refuse a key file that other users on the same machine can read.

<!-- LINKS:START (generated, do not edit) -->

## Links

- subject of: [DECISION-04: The health check examines five items with one free call](../decisions/DECISION-04.md)
- traced by: [FEATURE-01: Store a key one time](../../roadmap/features/FEATURE-01.md)
- traced by: [FEATURE-02: Readiness check](../../roadmap/features/FEATURE-02.md)
<!-- LINKS:END -->
