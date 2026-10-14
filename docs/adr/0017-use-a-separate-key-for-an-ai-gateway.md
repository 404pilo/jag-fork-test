---
type: adr
title: Use a separate key for an AI gateway
description: Open this when you change which key jag sends to a base URL, add a key source, or change how --setup, --forget-key, and --check handle keys.
area: contracts
status: accepted
date: 2026-10-14
supersedes: null
superseded_by: null
amends: null
refines: null
deviates_from: null
related: [0015-read-the-base-url-and-the-model-name-from-config-base-url.md, 0016-add-v1-to-a-base-url-whose-path-does-not-end-with-v1.md]
parent_spec: docs/superpowers/specs/2026-10-13-STORY-57-send-the-model-calls-through-an-ai-gateway-design.md
model:
  status: modeled
  element: C-config
  reason: C-config finds the key for the route of each call
---

# ADR-0017: Use a separate key for an AI gateway

## Context

jag finds the TypeSafe key in `JAG_TYPESAFE_API_KEY`, then in `TYPESAFE_API_KEY`, then in `config.api-key` of `~/.config/jag/config.yml` (DECISION-03). It sends the key as a Bearer key to the base URL. An option, a variable, or the config file can set the base URL to an AI gateway (IF-2). The parent spec gives the route: the direct route for the TypeSafe API, and the gateway route for each other base URL. Each gateway takes its own key (TypeSafe docs). If jag sent the key that it finds in the order of DECISION-03, a TypeSafe key could go to another host. For example, a CI job sets `JAG_BASE_URL` and also holds `TYPESAFE_API_KEY` for other TypeSafe tools. The key must stay inside the trust boundary of the user (QS-5, CV-6). The names of the key sources are a contract with the user.

## Decision

On the gateway route, jag sends only a gateway key. It reads the key from `JAG_GATEWAY_API_KEY`, then from `config.gateway-api-key` in the config file. The first key that it finds wins. When jag finds no gateway key, the search stops before the call, with exit 3, as with no TypeSafe key. jag never sends a key from `JAG_TYPESAFE_API_KEY`, `TYPESAFE_API_KEY`, or `config.api-key` to a gateway. On the direct route, jag uses the order of DECISION-03 and does not read the gateway key.

## Why

- A TypeSafe key never goes to a host other than the TypeSafe API, also when a variable of a shell or of a CI job sets a base URL by mistake.
- Each name tells what it holds. `JAG_TYPESAFE_API_KEY` holds only a TypeSafe key, and `JAG_GATEWAY_API_KEY` holds only a gateway key.
- With no gateway key, the user gets a clear error before the call, not `HTTP 401` from the gateway for a TypeSafe key.

## Alternatives considered

- The same key sources for each route (DECISION-03): no new setting and short docs. But a TypeSafe key can go to a gateway by mistake, and `JAG_TYPESAFE_API_KEY` then holds a key of another service.
- The same sources, but never `TYPESAFE_API_KEY` for a gateway: a key that the user set for other TypeSafe tools does not go to a gateway. But it is one more rule to learn, and a key in `JAG_TYPESAFE_API_KEY` or in `config.api-key` can still go to a gateway by mistake.

## Consequences

- jag must know the route before it looks for the key. So a search reads the settings before the key. When a setting and the key are both bad, the user sees the error of the setting first.
- `--setup`, `--forget-key`, and `--check` must handle the gateway key. The parent spec gives how.
- The config file can hold `config.gateway-api-key`. The mode check of the file protects it, as it protects `config.api-key` (CV-6).
- The fake server of the tests is on the gateway route, so the end-to-end tests set the gateway key.
- A user who moves from the direct route to a gateway must set the gateway key. The error text names `JAG_GATEWAY_API_KEY`.
- To rename or remove a key source breaks the setup of users. Such a change needs a new major version of jag and a new ADR.
