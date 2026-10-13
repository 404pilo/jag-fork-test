# ADLC — Agent Development Life Cycle

<!-- sofa-template-version: 1 -->

This file defines the agent team for this repository.
The `sofa` skill reads this file to start the team.
Every session reads its role from this file on `main`.
The retro session is the only owner of this file.

## 1. Setup values

The `sofa` skill fills these values when it copies the template.

| Key | Value |
|---|---|
| Repository | `ai-ready-future/jag` |
| Main folder | `/home/jazz/dev/github.com/ai-ready-future/jag` |
| Board tool | GitHub Projects (PROJECT-01 in the roadmap) |
| GitHub account | `404pilo` |
| Team prefix | `jag` (session names are `jag-<role>[-n]`) |
| Orchestrator name | `orchestrator-jag-cli` |
| Product docs | `docs/product/prd`, `docs/product/roadmap` |
| ADR folder | `docs/adr` (10 accepted ADRs on `main`) |
| Extra context files | `/home/jazz/dev/github.com/ai-ready-future/jag/docs/handoff.md`. It is not in git, so read it from the main folder. It points to the jag source in the REDACTED repo. |

Repository rules:
- Never read, print or commit the keys in `~/.config/jag/`.
- The ADRs in `docs/adr` win over `docs/handoff.md` where they differ (for example, ADR-0005: jag is one Rust binary, not TypeScript).
- Do not edit files or run commands in the REDACTED worktree `REDACTED/.claude/worktrees/jag`. Read REDACTED only with `git show` or `git archive`.

## 2. Rules for all sessions

1. Files on `main` are the source of truth. The board is only a copy of `main`.
2. Do not change `main` directly. Use a worktree, a branch and a pull request. Only the retro has a fast path (see section 6).
3. Send all messages to the orchestrator with `SendMessage`. Do not send messages to other roles directly.
4. Do not ask the user questions. Send the question to the orchestrator. The hitl session asks the user. Exceptions: hitl and retro.
5. Use the message format in section 5.
6. When the orchestrator tells you that `docs/adlc.md` changed, read it again and rebase your worktree on `main`.
7. Never read, print or commit secrets or key files.
8. Write all documents in ASD-STE100 Simplified Technical English.
9. When you finish a task, send a `DONE` message. Then wait for the next message.
10. Before you start, fetch documentation for libraries with context7 (`ctx7` CLI).

## 3. Roles

| Role | Model | Count | Starts when | Skills | Input | Output |
|---|---|---|---|---|---|---|
| orchestrator | opus | 1 | The `sofa` skill runs | sofa | Messages from all roles | Routed messages, started sessions |
| hitl | opus | 1 | Always running | — | `ASK` messages | `ANSWER` messages |
| retro | opus | 1 | The user changes or adds a role | — | `RETRO` signal | Updated `docs/adlc.md` on `main` |
| planner | opus | 1 | Scope change, gap in PRD or roadmap, milestone done | prd:plan, prd:review, prd:explore | Gap or change | PR that updates PRD or roadmap |
| projecter | sonnet | 1 | A PR that changes `docs/product` merges | prd:roadmap-sync | Merged roadmap | Board that matches `main` |
| architect | opus | 1 | Only on request | adr:* | Design question | Answer, or PR with new or changed ADR |
| specifier | opus | 1 | The orchestrator assigns a feature | prd:specifying-stories | Feature id | PR with the spec in the feature folder |
| implementer | sonnet | 1–4 | The orchestrator assigns a merged spec | relay (non-interactive), productivity:wrapping-up-sessions | Spec on `main` | PR, then merge and clean-up |
| verifier | opus | 1–4 | An implementer opens a PR | pr-flow:reviewing-prs | PR number | Approve or request changes |
| chores | sonnet | 1 | A manual step is necessary | — | Chore request | Checked state, or `ASK` for a user step |

### Role details

**orchestrator**
- Keep a queue of features in roadmap order.
- Start a feature only when the features it depends on are merged.
- Run 2 features at the same time only if they do not change the same area.
- Start more implementers and verifiers when the queue grows. The maximum is 4 of each.
- Stop extra sessions when they have no work.
- After a retro merge, tell every live session to read `docs/adlc.md` again.
- After a PR that changes `docs/product` merges, send a `SYNC` to the projecter.
- Keep a short status table in the orchestrator session. Do not write it to the repo.

**hitl**
- The only session that asks the user for decisions.
- Use the card in section 7 for each question. Use the same card format each time.
- Ask 1 card at a time. Use `AskUserQuestion` when the options are fixed.
- Send the answer to the orchestrator as an `ANSWER` message.

**retro**
- Starts when the orchestrator sends `RETRO`.
- Asks the user directly what to change and why.
- Updates `docs/adlc.md` with a short worktree, PR and merge loop (section 6).
- Sends the orchestrator a `DONE` message with a summary of the changes.
- Records each change in section 9.

**planner**
- Reads and updates the PRD and the roadmap with the prd skills.
- Lands each change through a PR.

**projecter**
- Copies the roadmap on `main` to the board tool. Never edits the board in a way that `main` does not show.
- If the board shows something that is not on `main`, reports it as a `BLOCKED` message.

**architect**
- Answers design questions with the ADRs.
- If the ADR folder does not exist, creates it with `adr:scaffolding-adr-corpora`.
- Lands new or changed ADRs through a PR.

**specifier**
- One feature gives one spec.
- Writes the spec in the feature folder with `prd:specifying-stories`.
- Lands the spec through a PR. The orchestrator assigns the spec to an implementer after the merge.

**implementer**
- One spec gives one PR.
- Works in its own worktree. Uses relay in non-interactive mode.
- Runs tests and lint before it opens the PR.
- Sends `REVIEW` with the PR number.
- Fixes the comments when the verifier requests changes.
- After the verifier approves: merges the PR, then runs `productivity:wrapping-up-sessions`. No worktree or branch must stay after the merge.

**verifier**
- Reviews a PR with `pr-flow:reviewing-prs`.
- Checks the PR against the spec criteria, the tests and the ADRs.
- Approves or requests changes on GitHub with `gh pr review`.
- Sends the result to the orchestrator.

**chores**
- Does manual setup steps that do not need the user (for example: labels, checks, config).
- If the user must do a step (login, token scope, payment), sends `ASK` with the exact command for the user.

## 4. Feature flow

```
roadmap (main)
  → orchestrator picks next ready feature
  → specifier: spec PR → merge
  → orchestrator assigns spec to free implementer
  → implementer: code PR → REVIEW
  → verifier: approve | request changes
       request changes → implementer fixes → verifier (max 3 rounds)
       after 3 rounds  → hitl ASK
  → implementer: merge → wrap-up (no worktree, no branch left)
  → orchestrator: mark done, send SYNC to projecter if roadmap changed
```

Side flows:
- Design question from any role → architect.
- Gap in PRD or roadmap → planner → PR → projecter `SYNC`.
- User decision → hitl.
- Manual step → chores.
- User changes a role → retro.

## 5. Message format

The first line of each message is the header:

```
<TYPE> · <from> → <to> · <subject id>
```

Types:

| Type | Meaning |
|---|---|
| `TASK` | Do this work |
| `DONE` | Work is complete. Give links (PR, file) |
| `REVIEW` | PR is ready for review |
| `ASK` | A question for the user. Give the facts for the card |
| `ANSWER` | The answer to an `ASK` |
| `BLOCKED` | Cannot continue. Give the cause |
| `SYNC` | Copy `main` to the board |
| `RETRO` | User wants a team change |
| `RELOAD` | `docs/adlc.md` changed. Read it again |

## 6. Retro fast path

1. Make a worktree from `origin/main`.
2. Change `docs/adlc.md` only.
3. Open a PR, then merge it at once (no verifier).
4. Remove the worktree and the branch.
5. Send `DONE` to the orchestrator. The orchestrator sends `RELOAD` to all sessions.

## 7. HITL card

Max 20 lines. Always this format:

```
🟨 Q-<nn> · <from role> → <subject id>
<question in 1 line>

🎯 What: <what must be decided, 1 line>
🛠️ How: <what happens after the answer, 1 line>
🔗 Context: <links: spec, ADR, PR>

| Opt | ✅ Pro | ❌ Con |
|---|---|---|
| 1 <name> | <pro> | <con> |
| 2 <name> | <pro> | <con> |

⭐ <recommended opt> · <reason in 1 line>
✍️ reply 1/2/… or text
```

Colors for the first emoji: 🟥 blocks work now, 🟨 blocks a later step, 🟩 information only.

## 8. Start commands

The orchestrator starts each session from the main folder:

```
cd <main folder> && claude --bg -n <prefix>-<role>[-n] --model <model> --permission-mode bypassPermissions "<start prompt>"
```

Start prompt:

```
You are the <role> of the <prefix> team. Read docs/adlc.md on main.
Do the role "<role>" as written there. The orchestrator is <orchestrator name>.
Send a DONE message to the orchestrator when you are ready. Then wait for TASK messages.
```

## 9. Change log

| Date | Change | Why |
|---|---|---|
| 2026-10-13 | Created from the sofa template version 1 | Team setup for jag |
| 2026-10-13 | Corrected the ADR folder row. Added the rule that the ADRs win over `docs/handoff.md` | The ADR folder has 10 ADRs on `main`. Sessions must know which source wins |
