# Workflow

`/rhobuilder-build <idea or design.json>` runs this loop. The CLI equivalent is the sequence
`design check`, `new`, `deploy plan`, then `deploy execute` in the app directory.

1. **Probe.** Resolve the network, compare networkId, shard, genesis block hash and node
   build with the registry, and confirm registry freshness. Public testnet uses
   status/capabilities/shards/blocks, never an exploratory loop. See [Registry](registry.md).
2. **Design.** Interview for the unique value, actors, capabilities, resources, lifecycle,
   value paths, failure/remedy and acceptance. Write `design.json`
   (`rhobuilder.design/v1`) and `spec/design.md`. The dashboard composer can build, validate
   and save the same document; model the unique interaction protocol, not a CRUD table. See
   [Design and reuse](design.md).
3. **Reuse.** Query the registry and graph. Tag every component `reuse`, `extend` or `new`.
   `extend` is adapter-only or a sibling proposal; deployed Rholang is immutable. A
   duplicate commodity primitive without an approved override stops the loop.
4. **Scaffold.** `rhobuilder new --design design.json --template full|headless`. The target
   becomes a DSH project root with its own `.dsh/skills` and launcher overlay. Plan first;
   apply only after the user confirms. See [Scaffolding](scaffold.md).
5. **Implement.** Rholang contracts and adapters. Every mutating arm needs a refusal and
   idempotency test; every value path needs conservation.
6. **Prove.** Lean model and flow obligations. `owed` is first-class; `proved-model` is the
   ceiling, never a refinement. See [Lean and RhoApp](formal.md).
7. **Test.** Static gates, parser check when the pinned node is present, docs, formal, then
   devnet install and behavioural suites. A not-run suite is not a pass. See
   [Testing and gates](testing.md).
8. **Plan deploy.** Reuse registered primitives, deploy only new contracts, and refuse a
   duplicate, missing dependency, identity mismatch or mainnet. `rhobuilder deploy plan` is
   read-only.
9. **Execute (CLI only).** Signed deploy with the finality ladder and a manifest. Testnet
   requires `--allow-live` and a confirmation token. See [Security model](security.md).
10. **Close.** Audit registers, docs/status labels, manifests and next steps. Report
    `Scaffold`, `Implemented`, `Tested`, `Proved model`, `Owed` and `Not run` separately; do
    not collapse them into “done”.

## Where each step lives

| Step | Detail page |
|---|---|
| Probe / registry | [Registry and reuse](registry.md) |
| Design | [Design and reuse](design.md) |
| Scaffold | [Scaffolding and project status](scaffold.md) |
| Prove | [Lean and RhoApp](formal.md) |
| Test | [Testing and gates](testing.md) |
| Execute | [Public testnet](baseline.md), [Security model](security.md) |
| Commands | [Command reference](command-reference.md) |
