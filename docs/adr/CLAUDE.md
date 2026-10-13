# ADRs

One ADR records one decision that is expensive to undo.
If it is cheap to change your mind, do not write an ADR.

## Before you change an architectural boundary

Read `README.md`. It lists every ADR, one line each. Open only the ADRs that apply.

## When you decide something expensive to undo

Write the ADR now, with `status: proposed`. Do not stop to ask.
Run `/adr:new`, or write the file and then run `/adr:index`.

## When a decision changes

Use one of these eight forms; a filename, never a number, identifies each ADR.

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

Edit frontmatter textually: change only the lines for keys you touch, preserve all other key
order and inline comments, and never serialize the whole frontmatter block. Do not append an
update block — git holds the history.

## Rules

- One decision per ADR.
- Record what was decided and why. Do not record how the design was explored.
- Do not edit `README.md` between the generated markers. Run `/adr:index`.
- `index.md`, `links.tsv` and the archive `index.md` are generated. Do not edit them. Run `/adr:index`.
- Give each new ADR a `description` of 30 words or fewer, in the words of the task.
