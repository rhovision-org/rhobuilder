# Scaffolding and project status

`rhobuilder new` renders the standard skeleton only after the design passes the reuse gate.
A plan writes nothing; an apply writes once.

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
- Applying is a user decision; the studio shows a read-only plan and the chat requires an
  explicit confirmation.

## What is generated

- `contracts/src/` — for a design with a worked example (`bookable-service`), the
  hand-authored contracts; otherwise a generic root, one linear cell per resource, one cell
  per `new` component, and a resolver adapter per `reuse`/`extend` component.
- `.rhobuilder-contracts.json` — generic designs only: install order, URI captures and
  required Root keys consumed by `deploy plan|execute`.
- `spec/graph/{vertices,edges}.yaml` and `spec/flows/core.yaml` — generated from the design;
  every arm starts `lean_status: owed`.
- `spec/lean/` — a self-contained Lean 4.12 stub (`lake build` passes; only `True` is
  proved, so it is a scaffold, not a model of the app).
- `harness/` — standard-library smoke and a `not_run` behavioural runner.
- `crates/` — Rust core, SDK and CLI stubs; `web/` in the full template.
- `docs/` — mdBook source; `audit/registers/` — generated arm and finding registers.
- `design.json`, `reuse-report.json`, `.rhobuilder.json`.

## Project status

```sh
rhobuilder --json project status --project /tmp/app
```

The read-only report reads `design.json`, `.rhobuilder.json`,
`.rhobuilder-contracts.json`, `spec/flows`, `spec/lean` and `deployments/`. It never runs
`make`, Docker or the network.

| Badge | Meaning |
|---|---|
| `Draft` | A validated design saved by the composer; no project files implied. |
| `Scaffold` | Generated files exist; the unique arms/model are still the author’s work. |
| `Implemented` | A verified deployment record or a recorded passing suite supports the claim. |
| `Tested` | Recorded suites pass. |
| `Proved model` | A shallow Lean theorem; never a Rholang refinement. |
| `Owed` | A proof obligation is missing. |
| `Not run` | The check did not execute; not a pass. |
| `Refused` | The design or reuse gate rejected the project. |

`project status` uses recorded evidence, not a fresh gate run. To run the gates, use
`rhobuilder check` or `make check`; see [Testing and gates](testing.md).

## Next

- [Testing and gates](testing.md) — static, formal, full and devnet profiles.
- [Lean and RhoApp](formal.md) — the model obligations.
- [Command reference](command-reference.md) — `new`, `check`, `project status`.
