# Canonical gates

`gates/` is the source for the checkers vendored into a generated project's `tools/`.
`GATES.json` records a SHA-256 per file. The scaffold copies these files through
`templates/dapp/common/tools/`; `tools/gate_sync.py --check` verifies both trees in CI
and `make check`.

Files:

- `spec_check.py` — design, term budgets, graph/flows/audit JSON and docs links.
- `parse_check.sh` — node parser when `RHOLO_DIR` is present.
- `check_vendored.py` — vendored `RhoApp` tree hash.
- `check_obligations.py` — per-arm Lean obligations and forbidden escapes.
- `check_gates.py` — verifies the generated project's `GATES.json`.

`GATES.json` is a drift record, not a signature. A generated project can still be edited;
the hash makes an unintended edit visible, and CI catches divergence from the canonical
source.
