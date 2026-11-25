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
runs `lake build`, `tools/check_vendored.py` and `tools/check_obligations.py`; a tampered
file or a forbidden Lean escape fails.

```sh
rhobuilder --json formal check --project /path/to/app
rhobuilder --json formal check --project /path/to/app --strict
rhobuilder --json formal sync  --project /path/to/app --source /path/to/rhobuilder
```

`check_obligations.py` reads `spec/flows/core.yaml`. Every component with mutating arms
must declare `lean_status`; `owed` is accepted by default and rejected by `--strict`; a
non-`owed` status must name a theorem. `sorry`, `admit`, `axiom` and `native_decide` are
forbidden. This checks the model obligation, not a refinement of deployed Rholang.

## Status vocabulary

`proved-model` means proved about the shallow model. `proved-conformance` additionally
needs a theorem, a corpus vector and a suite assertion. `owed` is first-class. Never write
`verified` or a claim of Rholang refinement.

The `bookable-service` example carries a shallow dispute model: `voteOnce` proves the
second vote precondition fails, provider exclusion is explicit, and `settleConserves`
proves `settled + refund = amount`. Its `spec/flows/core.yaml` names `voteOnce` as
`proved-model`, so `formal check --strict` passes for that example. Generic scaffolds keep
their arms `owed`.

The app keeps its own `Basic`, `Invariants`, `Emit`, `Audit` and `Model/*`; `Emit` and
`Audit` are app-specific names and vectors and are not moved into `RhoApp`.
