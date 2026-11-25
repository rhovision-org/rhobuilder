# Scaffolding

`rhobuilder new` renders the standard skeleton only after the design passes the reuse
gate. A plan writes nothing; an apply writes once.

```sh
rhobuilder --json new /tmp/app --design fixtures/designs/booking.json \
  --template full --plan
rhobuilder --json new /tmp/app --design fixtures/designs/booking.json \
  --template full --apply
```

- `--template full` includes the React/Vite shell; `headless` omits `web/`.
- `--plan` is the default. `--apply` writes.
- A target outside the working directory or a non-empty target needs `--force`.
- The design gate runs first; a `refuse` verdict writes nothing.

## What is generated

- `contracts/src/` — root, kernel/admin/terms placeholders, the first resource cell and the
  r-cash payments adapter placeholder.
- `spec/graph/{vertices,edges}.yaml` and `spec/flows/core.yaml` — generated from the design;
  every arm starts `lean_status: owed`.
- `spec/lean/` — a self-contained Lean 4.12 stub (`lake build` passes; only `True` is
  proved, so it is a scaffold, not a model of the app).
- `harness/` — standard-library smoke and a `not_run` behavioural runner.
- `crates/` — Rust core, SDK and CLI stubs; `web/` in the full template.
- `docs/` — mdBook source; `audit/registers/` — generated arm and finding registers.
- `design.json`, `reuse-report.json`, `.rhobuilder.json`.

## Gates

```sh
make check              # spec structure, term budgets, JSON, docs
make parse-check RHOLO_DIR=/path/to/rholo-fixed
make formal-check       # lake build; exit 2 if lake is absent
make unit-all           # not_run until RHOBUILDER_DEVNET=1 and a devnet is reachable
cargo check --workspace
```

`make check` does not claim the parser or devnet ran. The generated Rholang parses with the
pinned node parser when `RHOLO_DIR` is set; the scaffold suite is not a behavioural test.

## Status

Generated repositories are `Scaffold`: structure and static gates only. The unique arms,
the real Lean invariants and the behavioural suites are the app author's work.
