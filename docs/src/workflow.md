# Design workflow

`/rhobuilder-build <idea or design.json>` runs this loop. The CLI equivalent is
`rhobuilder build`, once P2 lands.

1. **Probe.** Resolve the network, compare networkId, shard, genesis block hash and node
   build with the registry, and confirm registry freshness. Public testnet uses
   status/capabilities/shards/blocks, never an exploratory loop.
2. **Design.** Interview for the unique value, actors, capabilities, resources, lifecycle,
   value paths, failure/remedy and acceptance. Write `design.json` (`rhobuilder.design/v1`)
   and `spec/design.md`. Model the unique interaction protocol, not a CRUD table.
3. **Reuse.** Query the registry and graph. Tag every component `reuse`, `extend` or
   `new`. `extend` is adapter-only or a sibling proposal; deployed Rholang is immutable.
   A duplicate commodity primitive without an approved override stops the loop.
4. **Scaffold.** `rhobuilder new --design design.json --template full|headless`. The
   target becomes a DSH project root with its own `.dsh/skills` and launcher overlay.
5. **Implement.** Rholang contracts and adapters. Every mutating arm needs a refusal and
   idempotency test; every value path needs conservation.
6. **Prove.** Lean model and flow obligations. `owed` is first-class; `proved-model` is the
   ceiling.
7. **Test.** Static gates, parser check when the pinned node is present, docs, formal,
   then devnet install and behavioural suites. A not-run suite is not a pass.
8. **Plan deploy.** Reuse registered primitives, deploy only new contracts, and refuse a
   duplicate, missing dependency, identity mismatch or mainnet.
9. **Execute (CLI only).** Signed deploy with the finality ladder and a manifest.
   Testnet requires `--allow-live` and a confirmation.
10. **Close.** Audit registers, docs/status labels, manifests and next steps.

## Gates

| Profile | Contents |
|---|---|
| `static` | build, spec, graph, flows, docs (not parser) |
| `parse` | node parser, only when `RHOLO_DIR`/`RCHAIN_RUST_DIR` is present |
| `formal` | Lean build, axiom audit, vectors, flow manifest |
| `full` | formal + devnet install + all behavioural suites |

`proved-model` is a theorem about the shallow Lean model. It is not a refinement of the
deployed Rholang, and no command may call it "verified".
