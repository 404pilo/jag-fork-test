---
type: Area
title: Contracts
description: The contracts area of the design.
status: draft
area: contracts
area_state: decided
anchor: contracts
decisions:
- key: contracts-1
  binds: 'The command form of a search: `jag [OPTIONS] TERM [PATH...]`, with the question only in `--ask QUESTION`'
  adr: adr:/0018-take-the-question-only-from-the-ask-option.md
  source: adr
- key: contracts-2
  binds: The form of the setup, readiness, and key-removal commands
  adr: adr:/0006-run-setup-the-readiness-check-and-key-removal-as-options.md
  source: adr
- key: contracts-3
  binds: The numbers of the exit codes above 2
  adr: adr:/0007-number-the-exit-codes-above-2-from-3-to-8.md
  source: adr
- key: contracts-4
  binds: The format of structured output and its version field
  adr: adr:/0008-write-structured-output-as-one-json-document.md
  source: adr
- key: contracts-5
  binds: The Jev model version that jag calls
  adr: adr:/0009-pin-the-jev-model-version-in-each-release.md
  source: adr
- key: contracts-6
  binds: One result is a match with the lines around it
  source: human:404pilo, 2026-10-12
- key: contracts-7
  binds: The exit code when setup runs with no terminal
  adr: adr:/0013-exit-2-when-setup-runs-with-no-terminal.md
  source: adr
- key: contracts-8
  binds: The config file keys of the base URL and the model name
  adr: adr:/0015-read-the-base-url-and-the-model-name-from-config-base-url.md
  source: adr
- key: contracts-9
  binds: How jag makes the call address from a base URL
  adr: adr:/0016-add-v1-to-a-base-url-whose-path-does-not-end-with-v1.md
  source: adr
- key: contracts-10
  binds: The key that jag sends to an AI gateway, and its sources
  adr: adr:/0017-use-a-separate-key-for-an-ai-gateway.md
  source: adr
- key: contracts-11
  binds: 'The output of a search with no `--ask`: rg at the pinned version is the reference'
  adr: adr:/0022-use-rg-as-the-reference-for-the-output-of-jag-with-no.md
  source: adr
open_rows:
- 'not migrated: IF-3 (the provider or a consumer is not a catalogue component)'
- 'not migrated: IF-4 (the provider or a consumer is not a catalogue component)'
- 'not migrated: IF-5 (the provider or a consumer is not a catalogue component)'
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:44:19Z'
---

# Contracts

<!-- LINKS:START (generated, do not edit) -->

## Links

- decisions adr: [adr:/0018-take-the-question-only-from-the-ask-option.md](../../adr/0018-take-the-question-only-from-the-ask-option.md)
- decisions adr: [adr:/0006-run-setup-the-readiness-check-and-key-removal-as-options.md](../../adr/0006-run-setup-the-readiness-check-and-key-removal-as-options.md)
- decisions adr: [adr:/0007-number-the-exit-codes-above-2-from-3-to-8.md](../../adr/0007-number-the-exit-codes-above-2-from-3-to-8.md)
- decisions adr: [adr:/0008-write-structured-output-as-one-json-document.md](../../adr/0008-write-structured-output-as-one-json-document.md)
- decisions adr: [adr:/0009-pin-the-jev-model-version-in-each-release.md](../../adr/0009-pin-the-jev-model-version-in-each-release.md)
- decisions adr: [adr:/0013-exit-2-when-setup-runs-with-no-terminal.md](../../adr/0013-exit-2-when-setup-runs-with-no-terminal.md)
- decisions adr: [adr:/0015-read-the-base-url-and-the-model-name-from-config-base-url.md](../../adr/0015-read-the-base-url-and-the-model-name-from-config-base-url.md)
- decisions adr: [adr:/0016-add-v1-to-a-base-url-whose-path-does-not-end-with-v1.md](../../adr/0016-add-v1-to-a-base-url-whose-path-does-not-end-with-v1.md)
- decisions adr: [adr:/0017-use-a-separate-key-for-an-ai-gateway.md](../../adr/0017-use-a-separate-key-for-an-ai-gateway.md)
- decisions adr: [adr:/0022-use-rg-as-the-reference-for-the-output-of-jag-with-no.md](../../adr/0022-use-rg-as-the-reference-for-the-output-of-jag-with-no.md)
- area: [contracts: Contracts](contracts.md)
- named by: [COMPONENT-01: Command line](../components/COMPONENT-01.md)
- named by: [COMPONENT-03: Match search](../components/COMPONENT-03.md)
- named by: [COMPONENT-05: Jev scorer](../components/COMPONENT-05.md)
- named by: [COMPONENT-06: Output](../components/COMPONENT-06.md)
- named by: [COMPONENT-09: TypeSafe API](../components/COMPONENT-09.md)
- named by: [COMPONENT-10: AI gateway](../components/COMPONENT-10.md)
- named by: [contracts: Contracts](contracts.md)
<!-- LINKS:END -->
