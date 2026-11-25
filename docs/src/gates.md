# Gates

There is one gate implementation per check, not one in the builder and another in the
generated app.

## Canonical source

`gates/` holds the canonical checkers:

| File | Check |
|---|---|
| `spec_check.py` | design, term budgets, graph/flows/audit JSON, docs links |
| `parse_check.sh` | node parser when `RHOLO_DIR` is present |
| `check_vendored.py` | vendored `RhoApp` tree hash |
| `check_obligations.py` | per-arm Lean obligation and forbidden escapes |
| `check_gates.py` | the generated project's `GATES.json` |

`GATES.json` records a SHA-256 per file. `tools/gate_sync.py --check` verifies both the
canonical tree and the vendored `templates/dapp/common/tools/` copy; CI and `make check`
run it. `make gates-check` is the same check.

## Generated project

The scaffold copies the gates and `GATES.json` into the app's `tools/`. Its `make check`
runs `check_gates.py` first, then `spec_check.py` and docs. `make formal-check` runs
`check_vendored.py` and `check_obligations.py`.

This is a drift record, not a signature. It makes an unintended edit visible; a generated
project can still be edited deliberately.
