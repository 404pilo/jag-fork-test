---
type: adr
title: Take the question only from the --ask option
description: Open this when you change how jag gets the question on the command line, or when a script depends on the command form.
area: contracts
status: accepted
date: 2026-10-14
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: [0006-run-setup-the-readiness-check-and-key-removal-as-options.md, 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md]
parent_spec: docs/superpowers/specs/2026-10-14-STORY-33-plain-grep-with-no-question-design.md
model:
  status: modeled
  element: C-cli
---

# ADR-0018: Take the question only from the --ask option

## Context

DECISION-31 gave the command form `jag TERM ["QUESTION"] [PATH...]`. With no question, jag works as plain grep, with no model call and no cost (DECISION-32). In that form, jag must guess if the word after the term is a question or a path. Some words can be both: the name of a folder can have a space, and a question can be one word. The guess decides if a command makes a paid model search or a free grep. Scripts and agents depend on the command form (IF-3).

The user answered on 2026-10-14: the question is an option, and only an option.

## Decision

The command form is `jag [OPTIONS] TERM [PATH...]`. The question comes only from the option `--ask QUESTION`, also written `--ask=QUESTION`. `--ask` has no short form. Each word after the term is a path.

- With `--ask`, jag finds the matches and ranks them with the model.
- With no `--ask`, jag is plain grep: no model call, no key, and no cost.

jag has no positional question. This replaces the positional question of DECISION-31.

## Why

- jag does not guess. The command line alone tells a question from a path, and no file or folder on the disk changes what a command does.
- A paid model search happens only when the command says `--ask`. A mistyped path never becomes a model call.
- `jag TERM PATH...` reads as a grep command. A shell user keeps their habits (CAPABILITY-06).
- No short form, because grep already uses `-q`, `-a`, `-e`, `-i`, and `-w`, and jag takes grep options (DECISION-34).

## Alternatives considered

- The word after the term is the question when it has a space and names no file: no option to type, but a one-word question needs a space at the end, and a folder name with a space is a path only when it exists.
- Each word after the term that names no file is the question: a one-word question works, but a mistyped path becomes a paid model search, and a command changes when a file with that name is made or removed.
- `--ask` and the positional question together: the user refused it. Two forms keep the guess and make the help longer.

## Consequences

- Each question search is a little longer to type.
- The help, the README, and the usage line show `jag [OPTIONS] TERM [PATH...]` and `--ask QUESTION`.
- The system design changes IF-3 and contracts-1 to this form.
- After the first release, a change of the command form needs a new major version of jag and a new ADR.
