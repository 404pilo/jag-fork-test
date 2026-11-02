# ADRs

One ADR records one decision that is expensive to undo.
If it is cheap to change your mind, do not write an ADR.

## Before you change an architectural boundary

Read `README.md`. It lists every ADR, one line each. Open only the ADRs that apply.

## When you decide something expensive to undo

Write the ADR now, with `status: proposed`. Do not stop to ask.
Run `/adr:new`, or write the file and then run `/adr:index`.

## When a decision changes

Use one of the eight forms in `.claude/rules/adr-changes.md` (rewrite, deprecate, supersede,
amend, refine, deviate, archive). A filename, never a number, identifies each ADR.

## Rules

- One decision per ADR.
- Record what was decided and why. Do not record how the design was explored.
- Do not edit `README.md` between the generated markers. Run `/adr:index`.
- `index.md`, `links.tsv` and the archive `index.md` are generated. Do not edit them. Run `/adr:index`.
- Give each new ADR a `description` of 30 words or fewer, in the words of the task.
