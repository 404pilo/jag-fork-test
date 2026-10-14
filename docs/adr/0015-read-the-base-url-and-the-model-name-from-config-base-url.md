---
type: adr
title: Read the base URL and the model name from config.base-url and config.model
description: Open this when you add a setting to the config file, change its keys, or change what --setup and --forget-key keep.
area: contracts
status: accepted
date: 2026-10-13
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: [0006-run-setup-the-readiness-check-and-key-removal-as-options.md, 0009-pin-the-jev-model-version-in-each-release.md, 0017-use-a-separate-key-for-an-ai-gateway.md]
parent_spec: docs/superpowers/specs/2026-10-13-STORY-57-send-the-model-calls-through-an-ai-gateway-design.md
model:
  status: modeled
  element: C-config
  reason: C-config reads the config file and gives each setting its source
---

# ADR-0015: Read the base URL and the model name from config.base-url and config.model

## Context

jag finds its key in `JAG_TYPESAFE_API_KEY`, then in `TYPESAFE_API_KEY`, then in `config.api-key` of `~/.config/jag/config.yml` (DECISION-03). The system design gives the other settings as options and variables only, and DF-1 keeps them out of the config file. To send the model calls through an AI gateway, the user sets a base URL and a model name (IF-2). CRITERION-96 gives the order of each of the two settings: the option, then the variable, then the config file, then the default. So the config file must hold the two settings. The user writes this file, and `--setup` and `--forget-key` write it again, so its keys are a contract with the user.

## Decision

The config file holds the base URL as `config.base-url` and the model name as `config.model`, next to `config.api-key`. For each of the two settings, the option wins over the variable, the variable wins over the file, and the file wins over the default. `--setup` and `--forget-key` keep the two keys when they write the file again. jag writes no other setting to the file, except the gateway key of ADR-0017.

## Why

- CRITERION-96 needs the config file as a source of the two settings. The trigger of DF-1 fires for these two settings only.
- `config.api-key` uses the nested `config.` form (FEATURE-01 spec decision 8). One form for each key of the file is easier to learn and to parse.
- jag never deletes a setting that the user wrote (FEATURE-01 spec decision 18). If `--setup` removed the two keys, the next search would call TypeSafe directly and not the gateway that the user chose.

## Alternatives considered

- Options and variables only, as the Settings row of the system design says: no change to the file, but CRITERION-96 fails, and the user must set a variable in each shell and in each job.
- Top-level keys `base-url` and `model`: shorter keys, but the file has two forms, and the parse of the `config` section must change.

## Consequences

- A search reads the file when a setting has no option and no variable, also when the key comes from `JAG_TYPESAFE_API_KEY` or `TYPESAFE_API_KEY`. The mode check and the YAML check of the file apply to that read. So a file that other users can read, or that is not valid YAML, gives exit 3.
- The Settings row and DF-1 of the system design need a change for the two settings. The times and the limits stay out of the file (DF-1).
- `--setup` and `--forget-key` still refuse a file with another key (exit 2). The message names each key that jag knows.
- To remove or rename one of the two keys breaks the files of users. Such a change needs a new major version of jag and a new ADR.
