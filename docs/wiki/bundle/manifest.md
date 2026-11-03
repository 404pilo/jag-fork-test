---
type: Manifest
title: The knowledge brain
description: 'The generic graph: sources, entities, concepts, synthesis, and open questions.'
status: stable
brain: knowledge
plugin: knowledge
plugin_version: 1.1.3
ask_skill:
  plugin: knowledge:ask
  skills: knowledge-ask
install: claude plugin install knowledge@cc-plugins-library
save_code: scripts/knowledge_graph.py
id_pattern: ^[a-z0-9]+(-[a-z0-9]+)*\.md$
id_folders:
- sources
- entities
- concepts
- synthesis
- open-questions
answers:
- what do the sources say about X
- which sources support a claim
- which questions are still open
required_links: {}
fixed_pages: []
retired_ids: []
generated:
  by: knowledge/1.1.3
  at: '2026-11-02T23:07:17Z'
---

# The knowledge brain

Ask with `/knowledge:ask`. Add with `/knowledge:learn`. Remove with `/knowledge:forget`.
