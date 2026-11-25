# Architecture

```text
contracts/src/        Rholang deploy terms
spec/graph/           integration vertices and edges (generated)
spec/flows/           arm and Lean-status manifest (generated)
spec/lean/            shallow Lean model (scaffold)
harness/              behavioural suites (not wired yet)
crates/               Rust core, SDK and CLI
docs/                 this mdBook
audit/                arm and finding registers (generated)
configs/              devnet/testnet profiles and pins
deployments/          generated manifests (ignored)
```

## Invariants

- One linear state cell per entity; take and repost exactly once.
- No `select`; exclusivity is one consumable datum.
- Deploy cap 64 KiB; client budget 56 KiB.
- No timers; block heights only.
- After a value move, an unexpected branch rolls back with `1/0`.
- Finality is finalized + processedWithSuccess + `(true, …)`.

## Evidence labels

`Scaffold` means source and structure only. `Implemented` means the source and static
gates exist. `Tested` means a named suite passed on a named devnet. Lean `proved-model`
means proved about the shallow model, never a refinement of the deployed Rholang.

`make parse-check` is separate from `make check`; without `RHOLO_DIR` it reports
`not_run` (exit 2).
