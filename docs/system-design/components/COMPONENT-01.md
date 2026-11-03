---
type: Component
title: Command line
description: Reads the command and its options, picks text or structured mode, runs the pipeline, and sets the exit code.
status: draft
slug: cli
kind: cli
areas:
- building-blocks
- contracts
- conventions
uses:
- to: /components/COMPONENT-02.md
  style: sync
- to: /components/COMPONENT-03.md
  style: sync
- to: /components/COMPONENT-04.md
  style: sync
- to: /components/COMPONENT-05.md
  style: sync
- to: /components/COMPONENT-06.md
  style: sync
- to: /components/COMPONENT-07.md
  style: sync
governed_by:
- adr:/0005-build-jag-as-one-rust-binary.md
- adr:/0006-run-setup-the-readiness-check-and-key-removal-as-options.md
- adr:/0007-number-the-exit-codes-above-2-from-3-to-8.md
- adr:/0010-ship-jag-on-github-releases-crates-io-and-npm.md
- adr:/0011-build-jag-as-one-cargo-package-with-one-module-per-component.md
- adr:/0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md
- adr:/0013-exit-2-when-setup-runs-with-no-terminal.md
- adr:/0018-take-the-question-only-from-the-ask-option.md
- adr:/0026-name-the-crates-io-package-jag-cli.md
- /conventions/CONVENTION-01.md
- /conventions/CONVENTION-03.md
- /conventions/CONVENTION-04.md
- /conventions/CONVENTION-07.md
realizes:
- prd:/capabilities/CAPABILITY-05.md
- prd:/capabilities/CAPABILITY-06.md
aliases:
- C-cli
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
---

# COMPONENT-01: Command line

Reads the command and its options, picks text or structured mode, runs the pipeline, and sets the exit code.

<!-- LINKS:START (generated, do not edit) -->

## Links

- uses to: [COMPONENT-02: Key and settings](COMPONENT-02.md)
- uses to: [COMPONENT-03: Match search](COMPONENT-03.md)
- uses to: [COMPONENT-04: Pre-rank](COMPONENT-04.md)
- uses to: [COMPONENT-05: Jev scorer](COMPONENT-05.md)
- uses to: [COMPONENT-06: Output](COMPONENT-06.md)
- uses to: [COMPONENT-07: Readiness check](COMPONENT-07.md)
- governed by: [adr:/0005-build-jag-as-one-rust-binary.md](../../adr/0005-build-jag-as-one-rust-binary.md)
- governed by: [adr:/0006-run-setup-the-readiness-check-and-key-removal-as-options.md](../../adr/0006-run-setup-the-readiness-check-and-key-removal-as-options.md)
- governed by: [adr:/0007-number-the-exit-codes-above-2-from-3-to-8.md](../../adr/0007-number-the-exit-codes-above-2-from-3-to-8.md)
- governed by: [adr:/0010-ship-jag-on-github-releases-crates-io-and-npm.md](../../adr/0010-ship-jag-on-github-releases-crates-io-and-npm.md)
- governed by: [adr:/0011-build-jag-as-one-cargo-package-with-one-module-per-component.md](../../adr/0011-build-jag-as-one-cargo-package-with-one-module-per-component.md)
- governed by: [adr:/0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md](../../adr/0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md)
- governed by: [adr:/0013-exit-2-when-setup-runs-with-no-terminal.md](../../adr/0013-exit-2-when-setup-runs-with-no-terminal.md)
- governed by: [adr:/0018-take-the-question-only-from-the-ask-option.md](../../adr/0018-take-the-question-only-from-the-ask-option.md)
- governed by: [adr:/0026-name-the-crates-io-package-jag-cli.md](../../adr/0026-name-the-crates-io-package-jag-cli.md)
- governed by: [CONVENTION-01: Each failure gives its cause, its fix, and](../conventions/CONVENTION-01.md)
- governed by: [CONVENTION-03: In text mode, results go to standard output,](../conventions/CONVENTION-03.md)
- governed by: [CONVENTION-04: On Ctrl-C, jag starts no new model call](../conventions/CONVENTION-04.md)
- governed by: [CONVENTION-07: A setting takes its value from the first](../conventions/CONVENTION-07.md)
- realizes: [prd:/capabilities/CAPABILITY-05.md](../../product/prd/capabilities/CAPABILITY-05.md)
- realizes: [prd:/capabilities/CAPABILITY-06.md](../../product/prd/capabilities/CAPABILITY-06.md)
- area: [building-blocks: Building blocks](../areas/building-blocks.md)
- area: [contracts: Contracts](../areas/contracts.md)
- area: [conventions: Crosscutting conventions](../areas/conventions.md)
<!-- LINKS:END -->
