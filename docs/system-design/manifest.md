---
type: Manifest
title: The system-design brain
description: 'The architecture of the system: components, interfaces, data, quality, and deployment.'
status: stable
brain: system-design
plugin: system-design
plugin_version: 0.2.3
ask_skill:
  plugin: system-design:ask
  skills: system-design-ask
install: claude plugin install system-design@cc-plugins-library
save_code: scripts/system_design.py
id_pattern: ^[A-Z]+-\d{2,}\.md$
id_folders:
- components
- interfaces
- data
- scenarios
- conventions
- risks
- deferred
- assumptions
- nodes
- glossary
answers:
- which component realizes capability X
- which interface do A and B use
- which scenarios constrain this component
- why is the design this way
required_links:
  Interface:
  - provided_by
  DataEntity:
  - owned_by
  QualityScenario:
  - cites
fixed_pages:
- system.md
- areas/*.md
retired_ids: []
generated:
  by: system-design/0.2.3
  at: '2026-11-03T11:37:02Z'
---

# The system-design brain

Ask with `/system-design:ask`. Learn with `/system-design:learn`. Forget with `/knowledge:forget`.
