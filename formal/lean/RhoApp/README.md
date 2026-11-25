# RhoApp

Shared, namespace-generic Lean 4.12 library for rhovision dApp models.

Extracted from `r-share`'s proven `spec/lean/RShare/Basic.lean` and
`Linear/{Cell,Protocol,Deploy,Concurrency}.lean` (AGPL-3.0-or-later, commit
`7114acba`). Theorem statements are unchanged; only the namespace and imports moved.

It models a linear cell store and its protocol:

- `Op.peek`, `take`, `put`, `call`, `commit`; `WF` and the D1/D2/D3 rejection theorems.
- Helper protocols H1, H2a, H2b and P2.
- Deploy atomicity: `deploy_atomic_rollback`, `run_failure_no_leak`,
  `commit_preserves_wf`, `held_across_external_call_unsafe`.
- Concurrency: `commit_commute`, `run_disjoint_commute`,
  `disjoint_cells_linearizable`.

`proved-model` is the ceiling: these are theorems about the shallow model, not a
refinement of deployed Rholang. Generated apps vendor this library under
`spec/lean/RhoApp` with a `spec/lean/VENDORED.json` hash and keep their own
`Basic`, `Invariants`, `Emit`, `Audit` and `Model/*`.

Build:

```sh
cd formal/lean/RhoApp && lake build
```
