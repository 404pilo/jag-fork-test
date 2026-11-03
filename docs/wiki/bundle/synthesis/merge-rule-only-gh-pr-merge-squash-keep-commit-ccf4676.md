---
type: Synthesis
title: 'Merge rule: only gh pr merge --squash, keep commit ccf4676'
description: 'Human answer of 2026-11-03: keep the wrong author on ccf4676, allow only gh pr merge --squash, and keep the author check.'
status: draft
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T14:39:59Z'
---

# Merge rule: only gh pr merge --squash, keep commit ccf4676

Human answer of 2026-11-03: keep the wrong author on ccf4676, allow only gh pr merge --squash, and keep the author check.

## Question

What to do about a wrong author address on the public `main` branch? It is on the merge commit `ccf4676` of PR #279.

## Facts

- GitHub GraphQL was rate limited. The merger squash merged with the REST API (`PUT /pulls/<n>/merge`).
- That merge wrote an author address that is not the GitHub noreply address of `78aa3f5` and the earlier commits.
- The committer was correct.
- The author check of `docs/adlc.md` section 1 found the error after the merge.

## Answer

Keep the commit and fix the merge rule.

- The history of `main` does not change. This is the same choice as on 2026-10-15 (commit `9634a17`, FEATURE-19 spec decision 13).
- The rules permit only `gh pr merge --squash`. The merger waits when GitHub limits the rate.
- The author check after each merge stays.
- The user checks later that the GitHub setting "Keep my email addresses private" is on. The user thinks about the setting "Block command line pushes that expose my email".

## Provenance

- Decided by: human `404pilo`, through hitl, on 2026-11-03 (veracity 90).
- Card from decision-judge: level middle, confidence medium, hard to reverse, hidden, band strict.
- This page holds no email address.
