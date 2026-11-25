# Lean model

Scaffold: the model is deliberately shallow and proves only `True`. It imports the vendored
`RhoApp` library (`RhoApp.lean` and `RhoApp/**`, hash-pinned by `VENDORED.json`) for the
linear-cell, deploy-atomicity and concurrency lemmas. Replace the app model with its
resources and invariants, and mark each arm in `spec/flows/core.yaml` as `proved-model`,
`proved-conformance`, `modeled`, `assumption` or `owed`.

`proved-model` is a theorem about this shallow model, not a refinement of the deployed
Rholang. Do not add Mathlib, `sorry`, `admit`, `axiom`, `native_decide` or a new axiom.

Run:

```sh
cd spec/lean && lake build
make formal-check
```
