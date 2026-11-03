---
type: Area
title: Building blocks
description: The building blocks area of the design.
status: draft
area: building-blocks
area_state: decided
anchor: building-blocks
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
decisions:
- key: building-blocks-1
  binds: The language and the runtime of jag
  adr: adr:/0005-build-jag-as-one-rust-binary.md
  source: adr
- key: building-blocks-2
  binds: The components inside the jag program
  adr: adr:/0011-build-jag-as-one-cargo-package-with-one-module-per-component.md
  source: adr
- key: building-blocks-3
  binds: Where the tool lives that measures jag on your own questions
  source: human:404pilo, 2026-10-12
- key: building-blocks-4
  binds: The outside crates of setup and their test crates
  adr: adr:/0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md
  source: adr
- key: building-blocks-5
  binds: The HTTP and JSON crates of the model calls, and how jag runs them in parallel
  adr: adr:/0014-call-the-model-service-with-ureq-and-serde-json-on-std.md
  source: adr
- key: building-blocks-6
  binds: 'The crates.io name of the one package: `jag-cli` (`cargo install --locked jag-cli`); the command stays `jag`'
  adr: adr:/0026-name-the-crates-io-package-jag-cli.md
  source: adr
---

# Building blocks

<!-- LINKS:START (generated, do not edit) -->

## Links

- decisions adr: [adr:/0005-build-jag-as-one-rust-binary.md](../../adr/0005-build-jag-as-one-rust-binary.md)
- decisions adr: [adr:/0011-build-jag-as-one-cargo-package-with-one-module-per-component.md](../../adr/0011-build-jag-as-one-cargo-package-with-one-module-per-component.md)
- decisions adr: [adr:/0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md](../../adr/0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md)
- decisions adr: [adr:/0014-call-the-model-service-with-ureq-and-serde-json-on-std.md](../../adr/0014-call-the-model-service-with-ureq-and-serde-json-on-std.md)
- decisions adr: [adr:/0026-name-the-crates-io-package-jag-cli.md](../../adr/0026-name-the-crates-io-package-jag-cli.md)
- area: [building-blocks: Building blocks](building-blocks.md)
- named by: [COMPONENT-01: Command line](../components/COMPONENT-01.md)
- named by: [COMPONENT-02: Key and settings](../components/COMPONENT-02.md)
- named by: [COMPONENT-03: Match search](../components/COMPONENT-03.md)
- named by: [COMPONENT-04: Pre-rank](../components/COMPONENT-04.md)
- named by: [COMPONENT-05: Jev scorer](../components/COMPONENT-05.md)
- named by: [COMPONENT-06: Output](../components/COMPONENT-06.md)
- named by: [COMPONENT-07: Readiness check](../components/COMPONENT-07.md)
- named by: [COMPONENT-08: jag-bench](../components/COMPONENT-08.md)
- named by: [COMPONENT-11: Agent skill](../components/COMPONENT-11.md)
- named by: [building-blocks: Building blocks](building-blocks.md)
<!-- LINKS:END -->
