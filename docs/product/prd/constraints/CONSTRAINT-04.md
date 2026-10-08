---
type: Constraint
title: jag tries again one time before it reports the service as down
description: Before jag reports the model service as down, it tries again one time
  and waits a fixed time that the user can change.
status: stable
constrains:
- /capabilities/CAPABILITY-01.md
---

# CONSTRAINT-04: jag tries again one time before it reports the service as down

Before jag reports the model service as down, it must try again one time. It must fail after a fixed wait. The user can change the wait.

<!-- LINKS:START (generated, do not edit) -->

## Links

- constrains: [CAPABILITY-01: Guided key setup and health check](../capabilities/CAPABILITY-01.md)
- traced by: [FEATURE-03: Clear failure when the model is not available](../../roadmap/features/FEATURE-03.md)
- traced by: [STORY-11: Try again one time before reporting the service as down](../../roadmap/stories/STORY-11.md)
<!-- LINKS:END -->
