---
type: Constraint
title: A typical search is one search on a medium code repository
description: The 5-second target applies to one search at the default cost limit,
  on a medium code repository, with a working service.
status: stable
constrains:
- /capabilities/CAPABILITY-06.md
- /metrics/METRIC-03.md
---

# CONSTRAINT-06: A typical search is one search on a medium code repository

The 5-second target of METRIC-03 applies to one search at the default cost limit, on a medium code repository, while the model service works. The wait before a retry (CONSTRAINT-04) does not count toward the target.

<!-- LINKS:START (generated, do not edit) -->

## Links

- constrains: [CAPABILITY-06: Feels like grep](../capabilities/CAPABILITY-06.md)
- constrains: [METRIC-03: Search speed](../metrics/METRIC-03.md)
- traced by: [FEATURE-16: Fast enough to replace grep](../../roadmap/features/FEATURE-16.md)
<!-- LINKS:END -->
