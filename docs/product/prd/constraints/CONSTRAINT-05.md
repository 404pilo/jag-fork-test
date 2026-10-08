---
type: Constraint
title: The structured form only adds fields in one major version
description: In one major version, jag never removes or renames a field of structured
  output, and the output shows its form version.
status: stable
constrains:
- /capabilities/CAPABILITY-05.md
---

# CONSTRAINT-05: The structured form only adds fields in one major version

In one major version, jag can add a field to structured output. It must not remove or rename a field. The output must show the version of its form, so that agent code that reads it keeps working.

<!-- LINKS:START (generated, do not edit) -->

## Links

- constrains: [CAPABILITY-05: Text output for people and structured output for agents](../capabilities/CAPABILITY-05.md)
- traced by: [FEATURE-11: Text output for a person and structured output for an agent](../../roadmap/features/FEATURE-11.md)
<!-- LINKS:END -->
