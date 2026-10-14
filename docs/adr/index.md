---
okf_version: "0.2"
---

# Architecture decisions

## Areas

* [building-blocks](#building-blocks) - 4 decisions: 4 accepted
* [context](#context) - 1 decision: 1 accepted
* [contracts](#contracts) - 6 decisions: 6 accepted
* [deployment](#deployment) - 1 decision: 1 accepted
* [strategy](#strategy) - 3 decisions: 3 accepted

## building-blocks

### accepted

* [0005: Build jag as one Rust binary](./0005-build-jag-as-one-rust-binary.md) - accepted. Open this when you change the language, the runtime, or the form of the jag program.
* [0011: Build jag as one Cargo package with one module per component](./0011-build-jag-as-one-cargo-package-with-one-module-per-component.md) - accepted. Open this when you add a component, a crate, a binary, or a build check, or when you change the Rust edition or the toolchain.
* [0012: Use clap, serde-saphyr, and rpassword as the outside crates of setup](./0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md) - accepted. Open this when you add or replace an outside crate, or change how jag reads options, YAML settings, or a hidden key prompt.
* [0014: Call the model service with ureq and serde_json on std threads](./0014-call-the-model-service-with-ureq-and-serde-json-on-std.md) - accepted. Open this when you add or replace the HTTP or JSON crate, add an async runtime, or change how jag runs model calls in parallel.

## context

### accepted

* [0001: Find matches with the ripgrep libraries](./0001-find-matches-with-the-ripgrep-libraries.md) - accepted. Open this when you change how jag finds the lines that match the term, or when grep and jag match different lines.

## contracts

### accepted

* [0006: Run setup, the readiness check, and key removal as options](./0006-run-setup-the-readiness-check-and-key-removal-as-options.md) - accepted. Open this when you add a command to jag that is not a search, or change the setup and check commands.
* [0007: Number the exit codes above 2 from 3 to 8](./0007-number-the-exit-codes-above-2-from-3-to-8.md) - accepted. Open this when you add an exit code, or when a script or an agent acts on the exit code of jag.
* [0008: Write structured output as one JSON document](./0008-write-structured-output-as-one-json-document.md) - accepted. Open this when you change the form of structured output, or when agent code that reads jag output breaks.
* [0009: Pin the Jev model version in each release](./0009-pin-the-jev-model-version-in-each-release.md) - accepted. Open this when you change the Jev model that jag calls, or when scores change with no jag release.
* [0013: Exit 2 when setup runs with no terminal](./0013-exit-2-when-setup-runs-with-no-terminal.md) - accepted. Open this when you change what jag --setup does with no terminal, or when a script acts on the exit code of setup.
* [0018: Take the question only from the --ask option](./0018-take-the-question-only-from-the-ask-option.md) - accepted. Open this when you change how jag gets the question on the command line, or when a script depends on the command form.

## deployment

### accepted

* [0010: Ship jag on GitHub Releases, crates.io, and npm](./0010-ship-jag-on-github-releases-crates-io-and-npm.md) - accepted. Open this when you change how users install jag, or when you add or remove a package channel.

## strategy

### accepted

* [0002: Score the top 200 results in calls of 32, six calls in parallel](./0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md) - accepted. Open this when you change the default cost limit, the batch size, or the count of parallel Jev calls.
* [0003: Decide no good answer with a fixed threshold on the top probability](./0003-decide-no-good-answer-with-a-fixed-threshold-on-the-top.md) - accepted. Open this when you change how jag decides that no result answers the question, or when the decoy AUC drops.
* [0004: Ship one scoring prompt with a version id in each release](./0004-ship-one-scoring-prompt-with-a-version-id-in-each-release.md) - accepted. Open this when you change the scoring prompt, or when you trace a change in scores after an upgrade.
