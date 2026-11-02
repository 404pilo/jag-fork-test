# ADLC — Agent Development Life Cycle

<!-- sofa-template-version: 8 -->

This file defines the agent team for this repository.
The `sofa:start` skill reads this file to start the team.
Every session loads the `sofa:station` skill, reads this file on `main`, then its role files (section 3, "Role files").
The retro session is the only owner of this file, of `docs/adlc/roles/` and of `docs/adlc/colours.md`.

## 1. Setup values

The `sofa:setup` skill fills these values. They are the same on each machine that starts the team.

| Key | Value |
|---|---|
| Repository | `ai-ready-future/jag` |
| Board tool | `GitHub Projects` (PROJECT-01 in the roadmap) |
| Team prefix | `jag` (session names are `jag-<role>[-n]`) |
| Orchestrator name | `jag-orchestrator` |
| Risk appetite | `early customers` (section 4, "Decision tables") |
| Product docs | `docs/product/prd`, `docs/product/roadmap` |
| System design | `docs/product/system-design.md` |
| ADR folder | `docs/adr`. The list of ADRs and their status is in `docs/adr/README.md` |
| Domain model | `none` (the `ddd` brain) |
| Proofs folder | `docs/proofs` (prototypes that support a proof of concept) |
| Extra context files | `/home/jazz/dev/github.com/ai-ready-future/jag-private/docs/handoff.md`. It is not in git. Since FEATURE-19 step B10, the main folder is a new clone of the public repo. `docs/handoff.md` stays only in the old folder `jag-private`; it was not copied. It points to the source of the earlier prototype. |
| Build | `cargo build` and `cargo test` on a machine that has `cargo`. On a machine without `cargo`, build and test jag in Docker with the image `rust:1.99` (the version in `rust-toolchain.toml`); the built binary runs on the host. |
| Run | `a terminal`: jag is a CLI. Build the binary as the Build row says, then run the jag binary on the host in a terminal |
| Environments | prod: the npm package `@airf/jag` (also GitHub Releases and the crate `jag-cli` on crates.io). Deploy: push a version tag; the workflow `.github/workflows/release.yml` runs `scripts/release/release.py` (see `scripts/release/README.md`). Logs: the GitHub Actions run of the tag. Metrics: none. |
| Healthy window | `24h` (how long a feature must run as expected in production before `monitoring` sets it `released`) |
| Reference program | `rg` (ripgrep), at the version of the pin file `tests/parity/rg.sha256` (ADR-0021) |
| Slack | `none` (the token is in the environment, never in this file, because this file is in the repository) |

Tools per role. `built-in` means the role uses only the skills of its row.

| Tool of | Tool |
|---|---|
| tracker | `gh` |
| security | `built-in` |
| code-quality | `built-in` |
| monitoring | `built-in` |
| feedback | `none` |
| ux-quality | `none` |

Machine values are not in this file, because they are different on each machine. The `sofa:start` skill finds them when it starts the team, and each start prompt gives them (section 8):
- Main folder: the absolute path of the main checkout on this machine.
- Skill folder: the absolute path of the installed `sofa` plugin on this machine.
- GitHub account: the active `gh` account on this machine. Sessions commit, push and open PRs as this account.

Repository rules:
- Never read, print or commit the keys in `~/.config/jag/`.
- The ADRs in `docs/adr` win over the `docs/handoff.md` of the "Extra context files" row where they differ (for example, ADR-0005: jag is one Rust binary, not TypeScript).
- Read the earlier prototype only as the `docs/handoff.md` of the "Extra context files" row says (`git show` or `git archive`). Do not edit files or run commands in its worktree.
- Squash merge author check. After each merge, the session that merges checks the new commit on `main`. The `sha256sum` of its author address must be the `sha256sum` of the author address of `78aa3f5`. Its committer must be the committer of `78aa3f5` (GitHub). The check never prints an address (FEATURE-19 spec decisions 13 and 31). A rebase merge wrote the account address as committer in `3ead9ae`.
- Chores and retro ask the user directly in their own session. While chores or retro wait for the user, they send `BLOCKED` with the tag `user step` to the orchestrator. The orchestrator records such a `BLOCKED` and does not send it on, because the user is already in that session.
- If the orchestrator session name changes, the retro updates section 1 and the orchestrator sends `RELOAD`.
- Planner: after a rule card, the planner sends no card for the PRD decision page of the rule. When a rule page changes, the planner asks a card only for a done feature that the new answer did not name.
- Release readiness check (FEATURE-22 spec, decisions 17 to 19): the orchestrator gives the security review to a reviewer session that wrote none of the code under review. The session does the review on the commit of `main` that the spec names. It fills in the section "Security review" of `docs/release-readiness.md` in a docs PR, and opens one issue with the label `security` for each open medium or low finding. The issue names the place and the fix, and gives no steps of an attack. A different reviewer session reviews that PR. After the approval, the merger merges it with squash, and the author session runs `wrapup`.

## 2. Rules for all sessions

The rules that are the same for all roles are in the `sofa:station` skill, which every session loads at start. Rules that are only in this file:

1. A story has the state `ready` only if all of these are true:
    - Its spec is merged on `main`.
    - Its feature is not blocked: each feature in its `blocked_by` list has the state `done` or `released`.
    - No open `ASK` blocks the story.

    If one of these is false, the story stays in `backlog`. A role that changes the state of a story or a feature checks this rule in the same PR.
    Exception: an `ANSWER` can set the stories of a blocked feature to `ready` (for example, "start now"). The `blocked_by` list stays. The `Plan` line in `log.md` names the `ANSWER`. FEATURE-18 used this exception (`ANSWER` of 2026-10-15, "start now") until FEATURE-19 was `done`.
    When a feature gets the state `done`, the same PR sets to `ready` each story that waited only for this feature and has a merged spec.
2. States of a story: `backlog` → `ready` → `in progress` → `done` → `released`. `monitoring` sets `released` (section 4, "Release flow"). A feature is `released` when each of its stories is.
3. Each role writes only the files of its row in section 3, "Who writes which file".

## 3. Roles

Tier: `always` sessions start with the team and stay live. `on demand` sessions start when an item needs the role, and stop after 1 hour free (orchestrator role file). `cron` is a system cron job, not a session.

| Role | Tier | Model | Effort | Compact at | Output style | Count | Starts when | Skills | Input | Output |
|---|---|---|---|---|---|---|---|---|---|---|
| orchestrator | `always` | `claude-opus-5-5` | `medium` | `400k` | `Proactive` | 1 | The `sofa:start` skill starts it | station, loop | Messages from all roles | Routed messages, started sessions |
| hitl | `always` | `claude-sonnet-5-5` | `medium` | `300k` | `Concise` | 1 | Always running | station, loop, slack:* | `ASK` messages, Slack threads | `ANSWER` messages, digest |
| requests | `always` | `claude-sonnet-5-5` | `medium` | `300k` | `Explanatory` | 1 | Always running | station, loop, knowledge:ask, slack:* | A request from a human | `TASK` to the orchestrator, answers to humans |
| decision-judge | `always` | `claude-opus-5-5` | `high` | `400k` | `Concise` | 1 | Always running | station, knowledge:ask | `BLOCKED · decision` cards | `ANSWER`, or `ASK` to hitl |
| scribe | `always` | `claude-sonnet-5-5` | `medium` | `300k` | `Concise` | 1 | Always running | station, knowledge:learn, knowledge:validate | Facts, answers, results | Pages in the `knowledge` brain, veracity updates |
| tracker | `always` | `claude-sonnet-5-5` | `medium` | `300k` | `Concise` | 1 | A PR that changes `docs/product` merges, or the orchestrator asks for a claim | station, product:roadmap-sync | Merged roadmap, or claim request | Board that matches `main`, and claims |
| plan-auditor | `always` | `claude-sonnet-5-5` | `high` | `300k` | `Concise` | 1 | Each merge on `main`, and every 30 minutes | station, loop, product:plan, wrapup | Merges, state store | PR with roadmap fixes and story metrics |
| merger | `always` | `claude-haiku-5-5` | `medium` | `200k` | `Concise` | 1 | All reviews of a PR are in | station, rebase | PR number and verdicts | Merged PR, or `BLOCKED` |
| monitoring | `always` | `claude-sonnet-5-5` | `medium` | `300k` | `Concise` | 1 | An environment exists | station, loop | Logs, metrics, traces, evals | `released` PRs, problem reports |
| scheduler | `cron` | `claude-haiku-5-5` | `low` | — | — | — | Every 5 minutes (system cron) | bg-watch | Session states | Restarted sessions |
| retro | `on demand` | `claude-opus-5-5` | `medium` | `400k` | `Concise` | 1 | The user changes or adds a role | station, wrapup | `RETRO` signal | Updated `docs/adlc.md` on `main` |
| planner | `on demand` | `claude-opus-5-5` | `medium` | `400k` | `Concise` | 1 | Scope change, gap in PRD or roadmap, new request, milestone done | station, product:plan, product:review, product:explore, wrapup | Gap, change or request | PR that updates PRD or roadmap |
| architect | `on demand` | `claude-opus-5-5` | `high` | `400k` | `Concise` | 1 | On request, or when a reviewer reports an ADR finding | station, architecture:*, wrapup | Design question or ADR finding | Answer, ADR verdict, or PR with new or changed ADR |
| designer | `on demand` | `claude-opus-5-5` | `high` | `400k` | `Concise` | 1 | A system design question, a new module, or a PR that adds or removes a folder | station, system-design:*, wrapup | Design question, folder change | PR with system design, `CLAUDE.md` routers, `.claude/rules` |
| business-domain | `on demand` | `claude-opus-5-5` | `high` | `400k` | `Concise` | 1 | A domain question, or a spec PR is open | station, business-domain:*, wrapup | Domain question, spec PR | Answer, domain review, PR with domain pages |
| specifier | `on demand` | `claude-opus-5-5` | `xhigh` | `400k` | `Concise` | 1–2 | The orchestrator assigns a feature | station, product:specifying-stories, wrapup | Feature id | PR with the spec in the feature folder |
| implementer | `on demand` | `claude-sonnet-5-5` | `high` | `300k` | `Concise` | 1–4 | The orchestrator assigns a merged spec | station, relay (non-interactive), wrapup | Spec on `main` | PR, then `wrapup` after the merge |
| reviewer | `on demand` | `claude-opus-5-5` | `high` | `400k` | `Concise` | 1–4 | A PR is open | station, code-review:reviewing-prs, architecture:reviewing-adr-adherence | PR number | Approve or request changes |
| verifier | `on demand` | `claude-opus-5-5` | `high` | `400k` | `Concise` | 1–4 | An implementer opens a code PR | station, verify, playwright | PR number and spec | Approve or request changes, the `product` proof line, and test scenarios |
| tester | `on demand` | `claude-sonnet-5-5` | `high` | `300k` | `Concise` | 1 | A verifier sends test scenarios | station, playwright, wrapup | Test scenarios | PR with new or changed e2e tests |
| security | `on demand` | `claude-sonnet-5-5` | `high` | `300k` | `Concise` | 1 | A code PR is open, or a security question | station, security-review | PR number, question | `REVIEW` with findings, answers |
| code-quality | `on demand` | `claude-sonnet-5-5` | `medium` | `300k` | `Concise` | 1 | A code PR is open | station, simplify | PR number | `REVIEW` with findings |
| researcher | `on demand` | `claude-opus-5-5` | `high` | `400k` | `Explanatory` | 1 | A study, a bench, a spike, a measurement, or a search for existing proofs | station, find-docs, wrapup | Question | Result with sources, no conclusions |
| troubleshooter | `on demand` | `claude-opus-5-5` | `high` | `400k` | `Explanatory` | 1 | A problem report | station, systematic-debugging | Problem | Root cause and proposed fix |
| operator | `on demand` | `claude-sonnet-5-5` | `medium` | `300k` | `Concise` | 1 | A command must run in an environment | station | Runbook task | Command results |
| feedback | `on demand` | `claude-haiku-5-5` | `medium` | `200k` | `Concise` | 1 | A feedback source exists (every 2 hours) | station, loop | Feedback sources | `TASK` suggestions |
| ux-quality | `on demand` | `claude-sonnet-5-5` | `medium` | `300k` | `Concise` | 1 | A ux tool exists (every 4 hours) | station, loop | Sentry or PostHog data | `TASK` suggestions |
| updater | `on demand` | `claude-haiku-5-5` | `low` | `200k` | `Concise` | 1 | Once a day | station, loop | Installed versions | Updated skills and plugins, reload requests |
| chores | `on demand` | `claude-sonnet-5-5` | `medium` | `300k` | `Concise` | 1 | A manual step is necessary | station | Chore request | Checked state |

Compact at: the size of the context window at which the session compacts its conversation by itself (`--autocompact`, section 8). The value comes from the model: `400k` for Opus, `300k` for Sonnet, `200k` for Haiku. A new row, or a row that changes its model, takes the value of its model.

Model: the tracker uses `claude-sonnet-5-5`, not the template value `claude-haiku-5-5`. The user chose Sonnet for the old projecter role on 2026-10-14, and the tracker does the work of the projecter.

Output style: a built-in style of Claude Code, passed with `--settings '{"outputStyle":"<style>"}'`. The names are `Proactive`, `Concise`, `Explanatory` and `Learning`, case-sensitive.

Colour: the session colour of each role is in `<skill folder>/references/colours.md`. A row in `docs/adlc/colours.md` (the retro writes it) wins for its role.

### Role files

The work of each role is in `<skill folder>/references/roles/<role>.md` and, if it exists, `docs/adlc/roles/<role>.md` (the retro writes it). Where the two files differ, the repository role file wins. A role that is only in this repository has only a repository role file.

### Who writes which file

| File or brain | Writer |
|---|---|
| PRD, roadmap stories and features | planner |
| Specs | specifier |
| Story and feature metrics (`docs/adlc/metrics/` until the `product` plugin takes them), missing roadmap stories | plan-auditor |
| Board | tracker |
| `adr` brain (`docs/adr`) | architect |
| `system-design` brain, nested `CLAUDE.md` routers, `.claude/rules` | designer |
| `ddd` brain | business-domain |
| `knowledge` brain, veracity of every brain | scribe |
| `docs/adlc.md`, `docs/adlc/roles/`, `docs/adlc/colours.md` | retro |
| Product code | implementer |
| e2e tests | tester |
| The section "Security review" of `docs/release-readiness.md` (section 1, "Repository rules", release readiness check) | reviewer |
| State store (`.sofa/state.db`) | the mod hooks and `sofa-state` |

## 4. Flows

### Feature flow

```
roadmap (main)
  → orchestrator picks next ready feature
  → tracker claims it on the board (if another team claimed it, the orchestrator picks the next one)
  → specifier: spec PR (business-domain reviews the domain language) → merge
       a spec with an unknown → specifier asks researcher for existing proofs
       no proof exists → orchestrator coordinates a proof of concept (research or a prototype in docs/proofs/)
       architect, designer or researcher learn the result into their brain
  → stories get the state ready (rule 1)
  → orchestrator at once assigns the spec to a free implementer (or starts one)
  → implementer: code PR with the Proofs block → REVIEW
  → reviewer (code, Proofs block, autonomous decisions), verifier (functions, product line), security and code-quality, at the same time
       request changes → implementer fixes → the same roles again (max 3 rounds)
       after 3 rounds  → decision-judge → hitl ASK
  → merger: rebase if needed, squash merge, author check (section 1, "Repository rules") → author runs `wrapup`
  → tester: e2e tests from the verifier scenarios → test PR → reviewer → merger (does not block done, unless the spec says so)
  → plan-auditor: story done, metrics in the state store
  → orchestrator: send SYNC to tracker if roadmap changed
```

### Release flow

```
feature done on main
  → operator deploys (or CI deploys), reports the environment and the time
  → monitoring watches the feature for the healthy window
       healthy → PR that sets the stories and the feature to released → plan-auditor writes the metrics in the frontmatter
       not healthy → troubleshooter → root cause → TASK to the orchestrator (implementer or operator)
       no metric to watch → 1 ASK to hitl
  → feedback and ux-quality propose a proof-of-intent score and improvements → hitl confirms → planner writes stories
```

### Decision flow

```
any role: BLOCKED · decision (with the result of knowledge:ask) → orchestrator
  → decision-judge: knowledge:ask in a fork, assessor subagent (level, confidence, reversibility, visibility), cross-check
       cell = decide → ANSWER to the orchestrator (line `decided by: decision-judge (...)`), decision in the state store, TASK to scribe
       cell = ask    → hitl: card → human → ANSWER → decision-judge validates the pages it used (scribe)
  → orchestrator sends the ANSWER to the role that asked, and to the planner or the architect when a rule or an ADR changes
  → hitl: daily digest of the autonomous decisions. A veto by number → ASK, and the pages lose veracity
```

### Decision tables

The assessor of `decision-judge` gives each question a **level** (`detail`, `middle`, `direction`) and a **confidence** (`high`, `medium`, `low`). A caution band selects the table. Points:

| Dimension | Points |
|---|---|
| Reversibility: reversible | 0 |
| Reversibility: hard to reverse (data shape, public API, migration, money, security, deletion) | 2 |
| Reversibility: irreversible | always ask, no table |
| Visibility: visible (a user or a tester sees the result at once) | 0 |
| Visibility: hidden (the result shows later, in some cases, or only in the data) | 1 |
| Stage of the feature: new | 0 |
| Stage of the feature: released | 1 |
| Risk appetite (section 1): `mvp` | 0 |
| Risk appetite: `early customers` | 1 |
| Risk appetite: `established` | 2 |

Band: 0–1 `relaxed`, 2–3 `normal`, 4 or more `strict`.

Relaxed:

| Level | High | Medium | Low |
|---|---|---|---|
| Detail | Decide | Decide | Decide |
| Middle | Decide | Decide | Ask |
| Direction | Decide | Ask | Ask |

Normal:

| Level | High | Medium | Low |
|---|---|---|---|
| Detail | Decide | Decide | Ask |
| Middle | Decide | Ask | Ask |
| Direction | Ask | Ask | Ask |

Strict:

| Level | High | Medium | Low |
|---|---|---|---|
| Detail | Decide | Ask | Ask |
| Middle | Ask | Ask | Ask |
| Direction | Ask | Ask | Ask |

A question with `priority: blocks later` at level `direction` is always `Ask`.

### Side flows

- Design question from any role → architect (ADR) or designer (system design).
- Domain question → business-domain.
- ADR finding from a reviewer → architect → decision flow (section 4, "ADR finding flow").
- Rule question from the specifier → decision flow. A `revisit:` → planner or architect (section 4, "Ask the Rule").
- Gap in PRD or roadmap, or a new request → planner → PR → tracker `SYNC`.
- Manual step → chores.
- Study, bench, spike, measurement, or a search for existing proofs → researcher.
- Problem in an environment → troubleshooter.
- Command in an environment → operator.
- Security question → security.
- A new folder in a PR → designer (router).
- A new fact (an answer, a result, a retro outcome) → scribe.
- User changes a role → retro.
- Blocker from any role → orchestrator → chores (`user step`), decision-judge (`decision`) or the role that must act (`team`). A `user step` from chores or retro stays with the orchestrator (section 1, "Repository rules").

### ADR finding flow

```
reviewer: ADR finding → VERDICT: CHANGES → orchestrator
  → orchestrator: finding → architect (starts the architect if it is not running)
  → architect: verdict → orchestrator → reviewer and implementer
       no ADR change          → reviewer: new VERDICT for the PR
       contradicts ADR-NNNN   → BLOCKED · decision (change the code | change the ADR | deviate)
       needs an ADR           → BLOCKED · decision (accept the decision | change the code)
  → decision flow → ANSWER
  → architect: ADR PR (status proposed) if the ANSWER changes an ADR, adds one, or deviates (`deviates_from`) → merge
  → implementer: code fix if the ANSWER says so
  → reviewer and verifier: new check of the code PR
```

- The orchestrator gives the question the priority `blocks code now`.
- The implementer does not change the code for an ADR finding before the `ANSWER`. An ADR finding and its new check are not one of the 3 review rounds.
- The code PR does not merge until the `ANSWER` is in, the ADR PR (if there is one) is merged, and the reviewer and the verifier approve the last code. In the new check, the ADR of the merged ADR PR is the decision for the finding.
- On the ADR PR of the architect, the reviewer checks that the ADR names the `ANSWER`. A change of an ADR that the `ANSWER` orders is not an ADR finding.
- An ADR finding from a review is the exception to "Write the ADR now ... Do not stop to ask" in `docs/adr/CLAUDE.md`. The user decides the direction first, because the review found a conflict.

### Ask the Rule

The specifier asks for the rule, not for the value. A spec meets these conditions:

1. A rule card asks about 1 rule. Its answer decides 2 or more open rows, changes a PRD decision or an ADR, or decides a thing that is hard to undo. Hard to undo: a done feature, a public option, released behavior, public data, money. The answer of a card does not depend on another card in the same batch. No card asks about a row that `main` decides at the time that you send the card.
2. Each rule card has the block `This answer decides:` with at most 5 lines, in plain words. 1 line can name a group of rows. The lines name each PRD decision and each ADR that changes, each done feature that changes, and each public option that goes away or changes.
3. Round 1: at most 2 rule cards. A priority question that changes the order of features or a `blocked_by` link goes in the same batch, with the line `owner: planner`. The batch has at most 4 cards. Round 2: at most 2 cards, only for rows that stay open. A card that you send again counts as a round 2 card. There is no round 3. After round 2, each open row gets the recommended option with the source `default after round cap`, and the PR body names the row. If that row is hard to undo, send `BLOCKED` with the cause.
4. A rule answer is 1 row with the source `human:<login>, <date>` or `decision-judge:<D-id>, <date>`. Each row that follows from it has the confidence `inferred`, the source `derived from row <n>`, and a place in the Assumptions list. If the answer is "other" or a different rule, the rows that followed from the old rule get derived again, and each card of the same batch whose premise changed goes out again.
5. Each other inferred row has the option that you would mark `[recommended]` and 1 source that a reviewer can open. The sources, from strong to weak: an accepted ADR, a PRD decision page (a page with the line `Principle:` holds a rule from an earlier answer), a `decided` area of the system design, a page of the `ddd` or `knowledge` brain with high veracity, a row of a built spec, the behavior of the reference program of section 1 at its pinned version, a line of the current code. A weak source does not count if a stronger source disagrees. A row with no source stays open. A hard-to-undo row is inferred only when a rule card named it, because the user cannot undo it in a review round.
6. The body of the spec PR lists the inferred rows with their sources, the stories that need no build, and the spec groups. The reviewer approval is the answer "reject none", so the ledger question of the skill is not sent. Send these as 1 card only if you send round 2, or if the reviewer rejects a row.
7. The owner of the page decides a change to an accepted ADR or a PRD decision that no card named. Send a `BLOCKED · decision` with the line `revisit: <page>`, the row, and the change that you recommend. A `revisit:` of an ADR goes to the architect, and the ADR finding flow continues from the verdict of the architect. A `revisit:` of a PRD decision goes to the planner, who sends a card only if the chosen option did not name the change. Until the owner decides, keep the current choice in that row, mark it `pending revisit`, keep only the dependent stories in `backlog`, and write the rest of the spec.

The ADR of a rule: `product:specifying-stories` writes 1 ADR with `status: proposed` for each rule row that binds code outside the stories of the feature, and no ADR for a derived row. The ADR holds only what the rule card showed: the chosen option and the lines of `This answer decides:`. The `ANSWER` of the rule card is the card that accepts that ADR (see the architect).

## 5. Message format

The first line of each message is the header:

```
<TYPE> · <from> → <to> · <subject id>
```

Types:

| Type | Meaning |
|---|---|
| `TASK` | Do this work. From a role to the orchestrator: a suggestion of work for another role |
| `DONE` | Work is complete. Give links (PR, file) |
| `REVIEW` | PR is ready for review, or a review result (`VERDICT: APPROVE` or `VERDICT: CHANGES`, then `Role: <role>`) |
| `ASK` | A question for a human. Give the facts for the card |
| `ANSWER` | The answer to an `ASK` or to a `BLOCKED · decision`. From decision-judge it has the line `decided by:` |
| `BLOCKED` | Cannot continue. Give the cause, where the work is, and 1 tag: `user step`, `decision` or `team` |
| `STATUS` | From the orchestrator: give your state. As a reply: 1 line with what you do now and the next step |
| `SYNC` | Copy `main` to the board |
| `RETRO` | User wants a team change |
| `RELOAD` | `docs/adlc.md` or a role file changed. Read them again |
| `PAUSE` | From the orchestrator: finish the turn, push the branch, send `STATUS`, then idle. With `now`: stop at once |
| `RESUME` | From the orchestrator: continue the open item from the branch |

## 6. Retro fast path

1. Make a worktree from `origin/main`.
2. Change only `docs/adlc.md`, the files in `docs/adlc/roles/` and `docs/adlc/colours.md`.
3. Open a PR. Send `REVIEW` to the orchestrator. A reviewer checks it as a document PR. After the approval, the merger merges it with squash and does the author check (section 1, "Repository rules").
4. Run `wrapup`.
5. Send `DONE` to the orchestrator. The orchestrator sends `RELOAD` to all sessions.

## 7. HITL card

Always use this layout, in this order:

1. The header: `from <role> → "<title>"`. The title tells the topic of the question in a few plain words.
2. The related line: `Related: <background in plain words>`. Write it only when there is useful background.
3. The question, in 1 sentence.
4. The decides block, for a rule card only (section 4, "Ask the Rule"): the line `This answer decides:` and at most 5 lines below it. Each line names 1 detail, or 1 group of details, that the recommended option decides.
5. The options list: 1 bullet for each option, with the pros and the cons indented below it.
6. For a card that `decision-judge` escalated: the line `Judge: <level>, <confidence>, <reversibility>, <visibility>, band <band>` and its recommended option.

Example (markdown source):

```markdown
from specifier → "The label for standard input"

Related: A search needs a path until the team adds search of standard input.

What do we build now for the "(standard input)" label?

- **Only the output code:** a result can come from a file or from standard input, and standard input prints "(standard input)". A unit test checks it. The end-to-end test comes later. [recommended]
  - **Pros:** ✅
    - This work stays small
    - The standard input code is not written 2 times
  - **Cons:** ❌
    - Only the later end-to-end test fully proves the label
- **The full standard input search now:** read standard input and print the label, with an end-to-end test.
  - **Pros:** ✅
    - The label is fully proven now
  - **Cons:** ❌
    - This work gets much larger
```

Rules:
- Plain words: Write the header, the related line, the question and the options in plain English (ASD-STE100 and the Google developer documentation style). Do not use IDs of features, stories, decisions, constraints or criteria, and do not use question IDs. Say what the item does in plain words.
- Question IDs: Hitl keeps the question ID for itself. It uses the ID in the `ANSWER` message, but not in the card.
- Options: Start each option with a short bold name and a colon, then the description, with no number in front. Do not use tables, because tables do not show correctly on a mobile screen or in Slack.
- Recommended option: Put the text `[recommended]` at the end of the option. This is the only place of the recommendation.
- Decides block: Write the lines in plain words, with no IDs. The user can veto 1 line with the answer `other: but <line>`.
- Pros and cons: Write `**Pros:** ✅` and `**Cons:** ❌`. Give each option 1 or more pros and 1 or more cons, each on its own indented bullet. Keep each bullet to a short phrase.
- Emojis: Use only ✅ and ❌, and only on the Pros and Cons lines.
- No repeated text: Give each fact 1 time only. For example, put the background in the related line, not in each option.
- Number of options: Use 2 or 3 options. If there are 4 or more options, keep this layout.
- `AskUserQuestion`: Use the short bold name of each option as its label. In Slack, the card is the top message of a thread, and the options are the bullets.

## 8. Start commands

The `sofa:start` skill starts the orchestrator. The orchestrator starts each other session with the dispatch script of the plugin, from the main folder of this machine:

```
<skill folder>/scripts/dispatch.sh --adlc docs/adlc.md --role <role> [--n <n>] --prompt-file <file> [--resume <session id>]
```

The script reads the row of the role in section 3 and runs:

```
claude --bg -n <prefix>-<role>[-n] --model <model> [--effort <effort>] --autocompact <compact at> --settings '{"outputStyle":"<output style>"}' --permission-mode bypassPermissions "<start prompt>"
```

It prints the session id and the name. The orchestrator gives each session a name that no other session of this repository has, live or stopped (`claude agents --json --all`). The first session of a role has no number. The next sessions take the lowest free number, from `-2`. A session that the orchestrator deleted (`claude rm`) is not in that list, so its number is free again.

A live session does not change its model, effort, compact value or output style after `RELOAD`. If section 3 changes one of them, or the name of a role, restart each session of that role when it has no work. The colour of each role is in `<skill folder>/references/colours.md`, not in this file. The orchestrator restarts a session that does not have the colour of its role (orchestrator role file, "Status check").

If a role gets a new name and a new role gets the old name, the orchestrator stops each live session of the old name before it sends `RELOAD`. Then it starts the session again with the new name. Else, after the `RELOAD`, the session reads the new role that has the old name.

If a session stops before its item is done, the `scheduler` (system cron, `bg-watch check`) restarts it and sends it `retry`. If the orchestrator finds a stopped session with an open item that the scheduler did not restart in 15 minutes, it resumes it with the dispatch script and `--resume <session id>` and the prompt `You were resumed. Continue your open item.` If the resume fails, it starts a new session.

Start prompt:

```
You are the <role> of the <prefix> team. Load the skill sofa:station and do what it says at start.
Your role files: <skill folder>/references/roles/<role>.md, and docs/adlc/roles/<role>.md if it exists.
The orchestrator is <orchestrator name>.
Machine values: main folder <main folder>, skill folder <skill folder>, GitHub account <gh account>.
```

The orchestrator start prompt is in the `sofa:start` skill. It gives the machine values, and the orchestrator copies them into each start prompt.

## 9. Change log

| Date | Change | Why |
|---|---|---|
| 2026-10-13 | Created from the sofa template version 1 | Team setup for jag |
| 2026-10-13 | Corrected the ADR folder row. Added the rule that the ADRs win over `docs/handoff.md` | The ADR folder has 10 ADRs on `main`. Sessions must know which source wins |
| 2026-10-13 | Removed an internal project name and its worktree path from section 1. Kept the rule | CONSTRAINT-01: the public repository contains no internal names. `docs/handoff.md` is not in git, so it keeps the details |
| 2026-10-13 | Changed the orchestrator name to `jag-orchestrator`. Added the rule for a name change to the orchestrator details | The orchestrator session name changed. Sessions send messages to this name |
| 2026-10-13 | Replaced the HITL card in section 7 with the layout "Option C": 2 context lines, the question, a bordered options table, and 1 recommendation line | The user chose this layout. It gives the background first, and the table borders stay in line |
| 2026-10-13 | The chores session can talk to the user directly and guide the user through manual steps. Decisions still go through hitl | The user asked for this change |
| 2026-10-13 | Replaced the options table in the HITL card (section 7) with a bullet list. Each option has its pros and cons as indented bullets | The user asked for this change. A table does not show correctly on a mobile screen |
| 2026-10-13 | Hitl asks the cards in priority order, up to 4 cards of 1 feature together. Up to 2 specifiers. The specifier uses effort `xhigh`, infers more from the ADRs and the system design, and asks guideline questions. For document PRs, an exact non-blocking fix merges with no new check | The user asked for these changes to make the team faster. Today, code work stopped for about 3 hours because specs waited for user answers |
| 2026-10-13 | Changed the HITL card (section 7): a header with the role and a plain title, plain words with no IDs, options with no numbers, ✅ and ❌ on the Pros and Cons lines, and `[recommended]` with no separate recommendation line | The user asked for this change. The cards must be easy to read with no knowledge of the IDs |
| 2026-10-13 | Added rule 11: a story is `ready` only if its spec is merged, its feature is not blocked, and no open `ASK` blocks it. The orchestrator assigns a ready story to an implementer at once. The specifier and each PR that sets a feature to `done` set the story state by this rule | The user asked for this change. 9 stories showed `ready` but were blocked or had no spec |
| 2026-10-14 | Added an Effort column to section 3. Orchestrator: `claude-opus-5-5`, `medium`. Hitl: `claude-sonnet-5-5`, `medium`. Planner: `claude-opus-5-5`, `medium`. Projecter: `claude-sonnet-5-5`, `medium`. Architect: `claude-opus-5-5`, `high`. Implementer: `claude-sonnet-5-5`, `high`. Verifier: `claude-opus-5-5`, `high`. Chores: `claude-sonnet-5-5`, `medium`. Retro: `claude-opus-5-5`, `medium`. Specifier: `claude-opus-5-5`, `xhigh`. Section 8 uses these values and restarts a session when its model or effort changes | The user asked for this change |
| 2026-10-14 | Removed the ADR count from the ADR folder row in section 1. The row points to `docs/adr/README.md` | The count (10) was out of date: `main` has 18 accepted ADRs. A count goes out of date again |
| 2026-10-15 | Rule 11: added the exception for a user `ANSWER` (FEATURE-18). Added rule 12: squash merge only, and a hash check of the author and the committer after each merge. The "Extra context files" row points to `docs/handoff.md` in the old folder `jag-private`. Section 4 points to the rule 11 exception. The repository rules point to the "Extra context files" row | Verifier comments on PR #94 and PR #95. A rebase merge wrote the account address as committer in `3ead9ae`. Since FEATURE-19 step B10, the main folder is a new clone and has no `docs/handoff.md` |
| 2026-10-15 | Rule 11: FEATURE-18 used the exception until FEATURE-19 was `done` | Verifier comment on PR #100. FEATURE-19 is `done` (`d023dd1`). STORY-48 to STORY-51 are now `ready` by the base rule |
| 2026-10-15 | The verifier checks ADR adherence on each PR (`adr:reviewing-adr-adherence`): a contradiction of an accepted ADR, or a change that needs an ADR. The architect decides each finding. Hitl asks the user. The architect writes the ADR after the `ANSWER`. Added the "ADR finding flow" to section 4. The retro fast path (section 6) gets a verifier check and a squash merge. The verifier posts a `VERDICT` comment and does not use `gh pr review` | The user asked for this change. The review must catch ADR problems, and the user decides ADR changes. Verifier comment on PR #103: section 6 and the verifier line did not agree with how the team works |
| 2026-10-15 | Changed the name of the verifier role to reviewer (it reviews PRs). Added a new verifier role: it runs the product from the PR branch (a terminal for a CLI, Playwright for a web app), checks the spec criteria, and sends test scenarios. Added the tester role: it writes and keeps the e2e tests, with no mocks and no unit tests. Added the Build row to section 1. Section 8 restarts a session when its role name changes, and stops a session of the old name before `RELOAD`. The verifier uses its own worktree and checks the last code after an ADR `ANSWER`. The tester sends design questions through the orchestrator. Defined a test PR | The user asked for this change. A review of the code does not show that the functions work for the user |
| 2026-10-15 | Reviewer: for a release readiness check, a reviewer session does the security review and writes `docs/release-readiness.md` in a docs PR. A different reviewer session reviews that PR | FEATURE-22 spec decision 18 (PR #112). Reviewer comment on PR #112: the reviewer role did not open PRs |
| 2026-10-15 | Added the method "Ask the Rule" to section 4: the specifier asks for the rule, not for the value; at most 2 rule cards, then at most 2 more, at most 4 cards in a batch; each rule card has a `This answer decides:` block; the specifier infers each detail from the rule with the recommended option and a source that a reviewer can open; a change to a PRD decision or an ADR goes as a `revisit:` line to the planner or the architect. The planner writes 1 PRD decision page for each rule. The architect accepts the ADR of a rule with the `ANSWER`. The reviewer opens the source of each inferred row. Section 7 gets the decides block | The user asked for this change. The FEATURE-23 spec sent 7 cards in 2 rounds, and the 5 detail cards all followed from the first rule answer. With this method, FEATURE-23 sends 2 cards in 1 batch |
| 2026-10-15 | Added rule 13 (each turn ends with 1 message to the orchestrator, and no role waits for the user) and rule 14 (`BLOCKED` with the tag `user step`, `decision` or `team`). The orchestrator sends each `BLOCKED` on by its tag: a user step to chores, a decision to hitl. Added a status check every 10 minutes to the orchestrator. It reads the job state of each session and sends `STATUS` only to a session that looks stuck. Added the `STATUS` message type and a side flow | The user asked for this change. Roles had no rule to send `BLOCKED`, and a blocker that needs the user had no path to the user. A session can end with a "needs input:" line that only the job list shows, and wait for a user who never sees it |
| 2026-10-27 | Rule 4: each question or decision for the user goes to hitl as an `ASK` card, through the orchestrator. The rule is also for the orchestrator. Only chores and retro ask the user directly, and while they wait they send `BLOCKED` with the tag `user step`. The orchestrator records such a `BLOCKED` and does not send it on. Changed rule 13, the hitl, specifier and chores details, and the side flow to agree | The user asked for this change (2026-10-27, in the orchestrator session): "all questions should go through hitl (except for chores and retro questions which are answered directly there)" |
| 2026-11-02 | Migrated to sofa template version 6 | Step 5 → 6 of `templates/MIGRATION.md`. The details are in the row for version 8 |
| 2026-11-02 | Migrated to sofa template version 7. No row in `docs/adlc/colours.md` is necessary, because the version 1 roles table had no `Colour` column | Step 6 → 7 of `templates/MIGRATION.md` |
| 2026-11-02 | Migrated to sofa template version 8 (from version 1; `templates/MIGRATION.md` has steps only from version 5). Section 1: removed the machine values Main folder and GitHub account; added Risk appetite, System design, Domain model, Proofs folder, Run, Environments, Healthy window, Reference program, Slack, and the table "Tools per role". Moved the team rules that are not in the `sofa:station` skill (squash merge author check, chores and retro ask directly, orchestrator name change, planner cards, release readiness check) to the repository rules. Section 2: rules 1 to 3 of the template; rule 1 keeps the FEATURE-18 note. Section 3: the template roles table (projecter is now tracker, and keeps `claude-sonnet-5-5`), "Role files" and "Who writes which file"; the role details are now in the role files of the plugin. Sections 4 to 8: the template sections, with the repository additions to the feature flow, the ADR finding flow, "Ask the Rule", the HITL card rules and the role rename rule kept | The sofa plugin 7.2.0 has template version 8 |
