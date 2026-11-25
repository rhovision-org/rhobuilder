---
name: rhobuilder-standards
description: Apply the rhovision repo standard: contracts/spec/harness/crates/web/docs/audit layout, Makefile targets, graph and flow manifests, status labels, pins and secrets.
---

# The rhovision repo standard

A generated dApp must be indistinguishable in shape from r-share, r-sure, r-print and
r-gen. A reviewer should find every artifact where they expect it.

## Layout

```text
AGENTS.md  audit/  book.toml  Cargo.toml  clippy.toml  configs/
contracts/{src/deploy,src/lib,build}
crates/<app>-{core,keystore,sdk,cli,wasm}
deployments/  .devnet/  docs/  harness/
spec/{lean,flows,vectors,graph,audit,guards,hazop}
tools/  web/  Makefile  rust-toolchain.toml
```

Contracts: numbered `00_root`, `01_kernel`, `02_admin`, `03_terms`, domain files,
`40_payments`; `lib/*.rho` inlined with `// @@INLINE:lib/X.rho@@`; Root
`resolve(key,minVersion,ret)` (3-arg) for new apps.

## Make targets

`spec-check`, `parse-check`, `docs-check`, `lean-check`/`formal-check`, `unit-all`,
`audit-check`, `client-check`, `check`, `ci`, plus `devnet-up`, `install`, `keys`.
`check` is static + docs + formal; `ci` adds the behavioural suites.

## Spec and graph

- `spec/graph/{vertices,edges}.yaml` — named integration vertices and edges.
- `spec/flows/core.yaml` — arm, guard, effect, `lean`, `lean_status`, suite.
- `spec/audit/{findings,capabilities,closeout}.yaml` and `audit/registers/*.yaml`.
- Status vocabulary: `implemented`, `surface`, `design`, `owed`, `proved-model`,
  `proved-conformance`, `proved-but-narrower`, `modeled`, `mechanism`, `assumption`,
  `open`. Never write `proved` or `verified` about deployed Rholang.

## Docs

Stock mdBook; `docs/` is the source, `docs/book/` is generated and ignored. Every chapter
except `SUMMARY.md` appears once. Use ASCII diagrams. State the status near the top:
Scaffold, Implemented (source + static/parser gates), Tested (named suite passed on a named
devnet).

## Pins and boundaries

- `rcash-core/chain/sdk` git `rhovision-org/r-cash` `49b441d…`; `rchain-crypto` git
  `rhovision-org/rholo` `841cb84…`; node `rchain-community/rchain-rust` `e2d073d…`;
  toolchain 1.95.0.
- r-share/r-print pins are `TBD-review-required` until a consumer pin is recorded.
- Do not modify a sibling repo. Do not path-depend on a sibling checkout; git-pin it.
- Secrets: no `.devnet/`, `.testnet/`, `.dsh-home/` or key material in the tree. Testnet
  deploys need `--allow-live`.

## Commands

```sh
make spec-check parse-check docs-check
make formal-check
make unit-all
make check
make ci
```
