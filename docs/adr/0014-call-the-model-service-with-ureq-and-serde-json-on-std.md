---
type: adr
title: Call the model service with ureq and serde_json on std threads
description: Open this when you add or replace the HTTP or JSON crate, add an async runtime, or change how jag runs model calls in parallel.
area: building-blocks
status: accepted
date: 2026-10-13
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: [0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md, 0005-build-jag-as-one-rust-binary.md, 0010-ship-jag-on-github-releases-crates-io-and-npm.md, 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md]
parent_spec: docs/product/system-design.md#building-blocks
model:
  status: modeled
  element: C-scorer
---

# ADR-0014: Call the model service with ureq and serde_json on std threads

## Context

C-scorer calls the TypeSafe API, or an AI gateway, over HTTPS with JSON (IF-1, IF-2). jag runs 6 calls in parallel (0002-score-the-top-200-results-in-calls-of-32-six-calls-in.md). Each call times out after 4 seconds, and jag tries a failed call one more time (CV-5). C-check and C-bench use the same calls, so the crates bind three components. jag is one static binary with no runtime (0005-build-jag-as-one-rust-binary.md).

## Decision

- jag calls the model service with `ureq` 3. `ureq` is a blocking HTTP client. It uses rustls for TLS.
- jag reads and writes the JSON of the calls with `serde_json`.
- jag runs the parallel calls on 6 standard threads (`std::thread::scope`). jag has no async runtime.
- C-scorer, C-check and C-bench use one shared `ureq` agent with the CV-5 timeout.
- jag uses the certificate store of the machine to check the server certificate (`ureq` feature `platform-verifier`, `RootCerts::PlatformVerifier`).

## Why

6 calls in parallel need only 6 threads, so an async runtime gives no gain. A blocking client keeps the code short, and the tests do not need a runtime. `ureq` has few dependencies, so the binary stays small and builds fast. rustls needs no OpenSSL on the machine, so the static binary of 0010-ship-jag-on-github-releases-crates-io-and-npm.md works on each platform. `ureq` reads the proxy settings from the environment by default, which a gateway setup can need. The certificate store of the machine holds the company certificates, so a company gateway (IF-2) or a company proxy that checks TLS works with no jag setting.

## Alternatives considered

- `reqwest` with `tokio`: the most used HTTP client and async runtime, but they add many dependencies, a bigger binary and a longer build for 6 parallel calls.
- The built-in WebPki root certificates (the `ureq` default): the same on each machine, but a company gateway or proxy with a company certificate fails, and jag would need a setting for a certificate file.

## Consequences

- A change to many more parallel calls, or to a streaming API, can need an async runtime and a new ADR.
- The CV-5 timeout and retry are jag code around `ureq`, not a feature of the crate.
- A new HTTP or JSON crate needs a new ADR.
- A machine with no certificate store, for example a minimal container, must install the CA certificates of the system before jag can connect.
- `Cargo.lock` pins the rustls crypto provider, so a change of provider shows in a pull request.
