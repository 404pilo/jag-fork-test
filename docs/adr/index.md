---
okf_version: "0.2"
---

# Architecture decisions

## Areas

* [building-blocks](#building-blocks) - 1 decision: 1 proposed
* [context](#context) - 1 decision: 1 proposed
* [contracts](#contracts) - 4 decisions: 4 proposed
* [deployment](#deployment) - 1 decision: 1 proposed
* [strategy](#strategy) - 3 decisions: 3 proposed

## building-blocks

### proposed

* [0005: Build jag as one Rust binary](./0005-build-jag-as-one-rust-binary.md) - proposed. Open this when you change the language, the runtime, or the form of the jag program.

## context

### proposed

* [0001: Find matches with the ripgrep libraries](./0001-find-matches-with-the-ripgrep-libraries.md) - proposed. Open this when you change how jag finds the lines that match the term, or when grep and jag match different lines.

## contracts

### proposed

* [0006: Run setup, the readiness check, and key removal as options](./0006-run-setup-the-readiness-check-and-key-removal-as-options.md) - proposed. Open this when you add a command to jag that is not a search, or change the setup and check commands.
* [0007: Number the exit codes above 2 from 3 to 8](./0007-number-the-exit-codes-above-2-from-3-to-8.md) - proposed. Open this when you add an exit code, or when a script or an agent acts on the exit code of jag.
* [0008: Write structured output as one JSON document](./0008-write-structured-output-as-one-json-document.md) - proposed. Open this when you change the form of structured output, or when agent code that reads jag output breaks.
* [0009: Pin the Jev model version in each release](./0009-pin-the-jev-model-version-in-each-release.md) - proposed. Open this when you change the Jev model that jag calls, or when scores change with no jag release.

## deployment

### proposed

* [0010: Ship jag as GitHub binaries and a Rust crate](./0010-ship-jag-as-github-binaries-and-a-rust-crate.md) - proposed. Open this when you change how users install jag, or when you add a package channel.

## strategy

### proposed

* [0002: Score the top 200 results in calls of 32, six calls in parallel](./0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md) - proposed. Open this when you change the default cost limit, the batch size, or the count of parallel Jev calls.
* [0003: Decide no good answer with a fixed threshold on the top probability](./0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md) - proposed. Open this when you change how jag decides that no result answers the question, or when the decoy AUC drops.
* [0004: Ship one scoring prompt with a version id in each release](./0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md) - proposed. Open this when you change the scoring prompt, or when you trace a change in scores after an upgrade.
