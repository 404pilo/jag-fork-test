---
type: adr
title: Give the new public repository the name ai-ready-future/jag
description: Open this when you change the name of the public or the private repository, the remote of a copy, or a link to an issue of the private repository.
area: deployment
status: accepted
date: 2026-10-14
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: [0010-ship-jag-on-github-releases-crates-io-and-npm.md]
parent_spec: docs/superpowers/specs/2026-10-14-STORY-47-go-public-with-a-clean-history-design.md
model:
  status: out-of-scope
  reason: the name of a repository is not an element of the system design
---

# ADR-0019: Give the new public repository the name ai-ready-future/jag

## Context

jag goes public as a new repository with a clean history, and the private repository `ai-ready-future/jag` stays private (DECISION-51). The name of the new repository is in each public link: `repository` in `Cargo.toml`, the packages of ADR-0010, the board, and each link to an issue. GitHub sends an old name of a renamed repository to the new name, until a new repository takes the old name. Then each old link and each old remote opens the new repository.

## Decision

The new public repository is `ai-ready-future/jag`. Before the new repository is created, the private repository gets another name, which the parent spec gives. Each copy of the private history changes its remote to the new private name before the rename, and never pushes to the new repository.

## Why

- The user chose the name jag (hitl answer, 2026-10-14).
- The tool and its repository have the same name. `Cargo.toml`, the package names of ADR-0010, and the docs already name `ai-ready-future/jag`, so they do not change.

## Alternatives considered

- A new name for the public repository, for example `ai-ready-future/jag-grep`, and no rename of the private repository: the old links stay correct, but `Cargo.toml`, each package link, and each public link must change, and the tool and its repository have different names.

## Consequences

- When the new repository takes the name, the redirect of the old name stops. Each old link to an issue or a pull request of the private repository then opens an item of the new repository, or a 404. So the roadmap pages change their links to the private repository.
- A copy of the old history that pushes to `ai-ready-future/jag` puts old commits in the public repository. So the team works in new clones of the new repository.
- DECISION-51 and CRITERION-100, CRITERION-102, and CRITERION-103 name `ai-ready-future/jag` as the private repository. Their text must change.
- To change the name of the public repository later breaks each public link and each package link. Such a change needs a new ADR.
