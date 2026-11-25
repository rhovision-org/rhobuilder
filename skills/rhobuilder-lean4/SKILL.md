---
name: rhobuilder-lean4
description: Build the shallow Lean 4 model for a dApp: Basic/Model/Invariants/Emit/Audit, flow obligations, no Mathlib or axioms, proved-model status.
---

# Lean 4 models for rhovision dApps

Each app carries a self-contained Lean 4.12 model under `spec/lean/<Pkg>/`. It is a
shallow model of the contract's state machine, not a Rholang semantics and not a
refinement.

## Layout

```
spec/lean/
  lakefile.lean
  <Pkg>.lean                 -- imports every module
  <Pkg>/Basic.lean           -- Amount/Height/Id/Bps and pure helpers
  <Pkg>/Model/*.lean         -- one file per resource or protocol
  <Pkg>/Invariants.lean      -- the named invariants
  <Pkg>/Emit.lean            -- machine-readable corpus/vectors
  <Pkg>/Audit.lean           -- axiom and coverage audit
```

A generated app imports the vendored `RhoApp` library for the generic linear-cell,
capability, deploy-atomicity and concurrency lemmas. Keep domain semantics in
`Model/*`; do not fork the generic library.

## Rules

- Lean 4.12, no Mathlib, no `sorry`, `admit`, `axiom`, `opaque`, `unsafe`, `partial`,
  `extern`, `implemented_by`; `native_decide` is counted and capped.
- Every mutating arm names a theorem in `spec/flows/core.yaml`, or is explicitly `owed`.
  `owed` is first-class; do not hide it.
- Statuses: `proved-model`, `proved-tied`, `proved-conformance`, `modeled`, `assumption`,
  `owed`, `design`, `mechanism`. `proved-model` means proved about the shallow model.
- Value is integer minor units; time is a block height; no wall-clock.
- The flow manifest ties each arm to its guard, effect, corpus and suite. The model does not
  replace the behavioural suite.

## What to prove first

1. Single owner / all-or-none for the central cell.
2. A refusal preserves state.
3. A terminal settlement conserves value.
4. A capability gate cannot be bypassed.
5. A lazy expiry is permissionless and cannot fire early.
6. Commits on distinct cells commute (use `RhoApp.Concurrency`).

## Commands

```sh
cd spec/lean && lake build
make formal-check   # build, axiom audit, vectors, flow manifest
```

A model that compiles is not "verified". Report the exact status and the theorem name.

## Read

- `RShare/spec/lean/RShare/Linear/{Cell,Protocol,Deploy,Concurrency}.lean`.
- `RPrint/spec/flows/core.yaml` and `RPrint/spec/lean/RPrint/{Emit,Audit}.lean`.
- `rholo-fixed/spec/TYPE-SYSTEM.md` for the CoC boundary.
