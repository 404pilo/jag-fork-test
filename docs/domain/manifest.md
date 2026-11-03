---
type: Manifest
title: The ddd brain
description: 'The domain model: bounded contexts, terms, aggregates, events, and relationships.'
status: stable
brain: ddd
plugin: business-domain
plugin_version: 1.0.1
ask_skill:
  plugin: business-domain:ask
  skills: business-domain-ask
install: claude plugin install business-domain@cc-plugins-library
save_code: scripts/ddd.py
id_pattern: ^[A-Z]+-\d{2,}\.md$
id_folders:
- contexts
- relationships
- terms
- aggregates
- events
- questions
- sources
answers:
- what does X mean in context Y
- which contexts use X
- what emits event X
required_links:
  Relationship:
  - between
fixed_pages: []
retired_ids: []
generated:
  by: ddd/1.0.1
  at: '2026-11-03T12:12:38Z'
---

# The ddd brain

Ask with `/business-domain:ask`. Learn with `/business-domain:learn`.
