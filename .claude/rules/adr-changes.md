---
paths:
  - "docs/adr/**"
---

# How to change an ADR

The router `docs/adr/CLAUDE.md` points here. Use one of these eight forms. A filename identifies each ADR, never a number.

| Change | Required action |
|---|---|
| Rewrite | Rewrite the ADR in place and set `amended:` when the decision remains the same. |
| Deprecate | Set `status: deprecated` when the decision is no longer recommended. |
| Full supersede | Write a new ADR, set the old record to `status: superseded`, and set both `superseded_by` and `supersedes`. |
| Partial supersede | Set both relationship entries with the same `# partial` comment; the old ADR stays accepted. |
| Amendment | Set both `amends` and `amended_by`. |
| Refine | Add `refines` to record a narrower decision without superseding it. |
| Deviate | Add `deviates_from` to explain a deliberate exception. |
| Archive | Move an obsolete record into `archive/` and repair every inbound reference. |

Edit frontmatter as text: change only the lines for the keys you touch. Keep all other key
order and inline comments. Never write the whole frontmatter block again. Do not append an
update block, because git holds the history.
