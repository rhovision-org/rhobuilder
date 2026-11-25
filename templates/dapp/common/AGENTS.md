# __app_name__ operating notes

Authority order: `contracts/src` and the gates > `spec/` > `docs/` > this file.

## Platform rules

- One linear state cell per entity: take, validate, repost exactly once, reply.
- `select` is not committed choice. Do not use it.
- Deploy term cap 64 KiB inclusive; keep the client term under 56 KiB.
- Peeks are hints, not locks. Never carry a peeked record across an external await.
- Unmatched `for` is silent; match reply shapes exhaustively.
- `*x` sends a name; `@x` sends data. Capabilities are passed by name.
- After value moves, an unexpected branch calls `1/0` so the deploy rolls back.
- No timers. Deadlines are block heights; the block timestamp is the proposer's clock.
- `explore-deploy` has no block context. A read that needs height is a signed deploy.
- Deploy lifespan is 50 blocks. Anchor `validAfterBlockNumber` at the tip.
- Finality = finalized + processedWithSuccess + `(true, …)`.

## Reuse

The design's component tags are binding. Reuse a registered primitive by `rho:id:` through
an adapter. `extend` is adapter-only or a new-version proposal; never edit a sibling
repository. A `new` commodity primitive needs an approved override in `design.json`.

## Evidence

`make check` covers spec structure, term budgets, docs and JSON. `make parse-check` is
separate and reports `not_run` without `RHOLO_DIR`. `make formal-check` is Lean. A suite
that cannot reach a healthy devnet is `not_run`, never a pass. `proved-model` is the
ceiling, not a refinement of the deployed Rholang.

## Commands

```sh
make check
make parse-check RHOLO_DIR=/path/to/rholo-fixed
make formal-check
make unit-all RHOBUILDER_DEVNET=1
```
