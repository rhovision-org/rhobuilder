# Design workflow

`/rhobuilder-build <idea or design.json>` runs this loop. The CLI equivalent is the
sequence `design check`, `new`, `deploy plan`, then `deploy execute` in the app directory.

1. **Probe.** Resolve the network, compare networkId, shard, genesis block hash and node
   build with the registry, and confirm registry freshness. Public testnet uses
   status/capabilities/shards/blocks, never an exploratory loop.
2. **Design.** Interview for the unique value, actors, capabilities, resources, lifecycle,
   value paths, failure/remedy and acceptance. Write `design.json` (`rhobuilder.design/v1`)
   and `spec/design.md`. The dashboard composer can build, validate and save the same
   document; model the unique interaction protocol, not a CRUD table.
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

`rhobuilder check --project <dir> --profile ...` runs the project's Makefile targets; it
does not hide a second gate implementation.

| Profile | Contents |
|---|---|
| `static` | `make check`: spec, graph, flows, docs |
| `formal` | `make formal-check`: Lean, vendored hash, per-arm obligations |
| `full` | static + formal + `make unit-all` (devnet harness; `not_run` when off) |

`parse-check` remains a separate Makefile target because it needs the pinned node parser;
`static` does not claim it ran.

`proved-model` is a theorem about the shallow Lean model. It is not a refinement of the
deployed Rholang, and no command may call it "verified".
