---
type: Component
title: Key and settings
description: Finds the key for the route of each call and the settings, reads the config file at most one time in each run, checks the mode of the file at each read, and stores or removes the key with no loss of the other keys of the file. The one component that reads the key.
status: draft
slug: config
kind: library
areas:
- building-blocks
- data
- conventions
governed_by:
- adr:/0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md
- adr:/0015-read-the-base-url-and-the-model-name-from-config-base-url.md
- adr:/0016-add-v1-to-a-base-url-whose-path-does-not-end-with-v1.md
- adr:/0017-use-a-separate-key-for-an-ai-gateway.md
- /conventions/CONVENTION-06.md
- /conventions/CONVENTION-07.md
realizes:
- prd:/capabilities/CAPABILITY-01.md
aliases:
- C-config
generated:
  by: claude-code/opus-5.5
  at: '2026-11-03T11:37:50Z'
---

# COMPONENT-02: Key and settings

Finds the key for the route of each call and the settings, reads the config file at most one time in each run, checks the mode of the file at each read, and stores or removes the key with no loss of the other keys of the file. The one component that reads the key.

<!-- LINKS:START (generated, do not edit) -->

## Links

- governed by: [adr:/0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md](../../adr/0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md)
- governed by: [adr:/0015-read-the-base-url-and-the-model-name-from-config-base-url.md](../../adr/0015-read-the-base-url-and-the-model-name-from-config-base-url.md)
- governed by: [adr:/0016-add-v1-to-a-base-url-whose-path-does-not-end-with-v1.md](../../adr/0016-add-v1-to-a-base-url-whose-path-does-not-end-with-v1.md)
- governed by: [adr:/0017-use-a-separate-key-for-an-ai-gateway.md](../../adr/0017-use-a-separate-key-for-an-ai-gateway.md)
- governed by: [CONVENTION-06: Only C-config reads the key. The key lives](../conventions/CONVENTION-06.md)
- governed by: [CONVENTION-07: A setting takes its value from the first](../conventions/CONVENTION-07.md)
- realizes: [prd:/capabilities/CAPABILITY-01.md](../../product/prd/capabilities/CAPABILITY-01.md)
- area: [building-blocks: Building blocks](../areas/building-blocks.md)
- area: [data: Data and ownership](../areas/data.md)
- area: [conventions: Crosscutting conventions](../areas/conventions.md)
- used by: [COMPONENT-01: Command line](COMPONENT-01.md)
- used by: [COMPONENT-05: Jev scorer](COMPONENT-05.md)
- used by: [COMPONENT-07: Readiness check](COMPONENT-07.md)
<!-- LINKS:END -->
