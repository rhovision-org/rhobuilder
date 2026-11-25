---
name: rhobuilder-primitive-reuse
description: Map a design onto the existing primitive graph, tag every component reuse/extend/new, and refuse duplicate payments, escrow, claims, provenance, booking or bridge primitives.
---

# Reuse the graph

The rhovision repos are one graph, not six products. Before designing a component, check
whether a registered primitive already owns that job.

## Protected domains

| Domain | Owner |
|---|---|
| accounts, transfers, tokens, FX, stablecoins | r-cash |
| escrow, insurance, syndicates, policies, payouts | r-sure |
| knowledge claims, KAssets, provenance, identity, credentials | r-gen |
| booking, rental capacity, slot allocation | r-share |
| publishing, editions, commissions | r-print |
| cross-chain bridge/relayer | r-bridge |
| governance, external identity/DID | rgov / BYOID / FreeTrust |

`r-sure.claim` (insurance) and `r-gen.claim` (knowledge) are different primitives.
`r-share.cover` is booking-internal cover, not a replacement for r-sure's open insurance.
r-gen identity/credential is the on-chain primitive; BYOID/FreeTrust are external
references.

## Modes

- **reuse** — call the registered primitive through an adapter (`40_payments.rho`,
  `05_rgen.rho`, `06_rsure.rho`). Resolve the component key per arm.
- **extend** — adapter-only: a local contract wraps a registered primitive; or a
  new-version proposal to the sibling repo. Never edit deployed Rholang.
- **new** — a genuinely uncovered capability. Requires `whyUnique`, a checked
  `noRegisteredAlternative`, and a user-approved override.

An override records:

```json
{"component":"my-escrow","reason":"…","alternativesChecked":["r-sure.policy"],
 "reviewer":"…","date":"2026-11-25","scope":"local-app-only"}
```

## Graph discipline

Every mutating or cross-repo arm names:

- a vertex in `spec/graph/vertices.yaml` (owner, kind, component, state, description);
- an edge in `spec/graph/edges.yaml` (from, to, verb, kind, arms, source, status);
- an arm and a theorem/status in `spec/flows/core.yaml`.

Core-to-extension edges are optional and explicit. The extension never calls back into the
core primitive.

## Registry limits

- The registry is curated; there is no indexer and no enumeration.
- A `rho:id` lookup proves resolution, not code identity. Record
  `verified: lookup|manifest|missing|mismatch`.
- r-share and r-print have no authoritative consumer pin in their `configs/`; do not treat
  a local HEAD as a pin. Mark them `TBD-review-required` until a maintainer records one.
- Public testnet `explore-deploy` is rate-limited; verify roots opt-in and cached.

## Command

```sh
rhobuilder registry list --network testnet --json
rhobuilder design check --path design.json --json
```
