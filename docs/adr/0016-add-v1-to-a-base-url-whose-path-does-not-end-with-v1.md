---
type: adr
title: Add /v1 to a base URL whose path does not end with /v1
description: Open this when you change how jag builds the call address from a base URL, or when a gateway needs another path.
area: contracts
status: accepted
date: 2026-10-13
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: [0014-call-the-model-service-with-ureq-and-serde-json-on-std.md, 0015-read-the-base-url-and-the-model-name-from-config-base-url.md]
parent_spec: docs/superpowers/specs/2026-10-13-STORY-57-send-the-model-calls-through-an-ai-gateway-design.md
model:
  status: modeled
  element: C-config
  reason: C-config applies the address rule to each source of the base URL
---

# ADR-0016: Add /v1 to a base URL whose path does not end with /v1

## Context

jag sends each model call to `<base URL>/systemone`. The default base URL is `https://api.typesafe.ai/v1`, and `JAG_BASE_URL` takes values in that form. The TypeSafe docs give a root URL for TypeSafe and for each AI gateway, with no `/v1`: `https://api.typesafe.ai`, `https://openrouter.ai/api`, `https://ai-gateway.vercel.sh/typesafe`, and `https://gateway-us.pydantic.dev/proxy/typesafe`. The SDK calls `<root URL>/v1/systemone`. A user copies the address from these docs, so jag must accept both forms. Saved values in options, variables, and config files depend on this rule, so it is a contract with the user.

## Decision

jag removes the white space at the start and at the end of the base URL, and each `/` at the end. It refuses a base URL with `?` or `#`, because it adds a path at the end. Then the address checks of FEATURE-04 apply. When the path does not end with `/v1`, jag adds `/v1`. The compare is exact. The call goes to that address with `/systemone` at the end.

## Why

- The root URL of the docs and the `/v1` URL of the default give the same call, so the user can copy either form.
- Values that work today, with `/v1` at the end, do not change.
- One rule for each source keeps the option, the variable, and the config file the same.

## Alternatives considered

- Require the full `/v1` URL and use it as it is: no rule to learn, but the root URL of each gateway page gives `HTTP 404` with no clear cause.
- The user gives the full call URL with `/v1/systemone`: each form is possible, but the user must know the path of the call, and the default changes form.

## Consequences

- A gateway that needs no `/v1`, or `/v1` two times, does not work with this rule. The signal is `HTTP 404` with a base URL from the docs of the gateway.
- A path that ends with `/V1` or `/v1beta` does not end with `/v1`, so jag adds `/v1`.
- The direct route to TypeSafe needs the host `api.typesafe.ai` and the path `/v1` after this rule.
- To change the rule changes the call address of saved values. Such a change needs a new major version of jag and a new ADR.
