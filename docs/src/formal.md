# Lean and RhoApp

Generated apps keep a shallow Lean 4.12 model and vendor the shared `RhoApp` library. The
model is deliberately shallow: it does not formalise Rholang syntax, ERTP, phlo or
cryptography, and a `proved-model` result is not a refinement of deployed Rholang.

## RhoApp

`formal/lean/RhoApp` is extracted from r-share's proven `Basic.lean` and
`Linear/{Cell,Protocol,Deploy,Concurrency}.lean` (AGPL-3.0-or-later, commit `7114acba`).
It provides:

- the linear cell store (`peek`, `take`, `put`, `call`, `commit`), `WF`, and the D1/D2/D3
  rejection theorems;
- helper protocols H1, H2a, H2b and P2;
- deploy atomicity: `deploy_atomic_rollback`, `run_failure_no_leak`,
  `commit_preserves_wf`;
- concurrency: `commit_commute`, `run_disjoint_commute`,
  `disjoint_cells_linearizable`;
- shared quantities: `Amount`, `Height`, `Bps`, `floorDiv`, `ceilDiv`, `share`, `sumNat`.

Its axiom audit reports only `propext` and `Quot.sound`.

## Vendoring

A generated app contains:

```text
spec/lean/RhoApp.lean
spec/lean/RhoApp/Basic.lean
spec/lean/RhoApp/Linear/{Cell,Protocol,Deploy,Concurrency}.lean
spec/lean/RhoApp/Audit.lean
spec/lean/VENDORED.json
```

`VENDORED.json` records the source, version, source commit (or `working-tree`) and a
deterministic SHA-256 over the vendored paths and bytes. The app's `make formal-check`
runs `lake build` and `tools/check_vendored.py`; a tampered file fails.

```sh
rhobuilder --json formal check --project /path/to/app
rhobuilder --json formal sync  --project /path/to/app --source /path/to/rhobuilder
```

## Status vocabulary

`proved-model` means proved about the shallow model. `proved-conformance` additionally
needs a theorem, a corpus vector and a suite assertion. `owed` is first-class. Never write
`verified` or a claim of Rholang refinement.

The app keeps its own `Basic`, `Invariants`, `Emit`, `Audit` and `Model/*`; `Emit` and
`Audit` are app-specific names and vectors and are not moved into `RhoApp`.
