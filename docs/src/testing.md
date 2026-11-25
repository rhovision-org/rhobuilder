# Testing and gates

There is one gate implementation per check. The builder does not reimplement a gate that
the generated project also runs.

## Test layers

| Layer | Command | What it proves |
|---|---|---|
| Rust | `cargo test --workspace` | core, registry, chain, formal, gates and CLI behavior |
| Plugin | `node --test agents/deepseek/plugin/test/*.test.mjs` | argv mapping, JSON envelopes, renderers and prose errors |
| Studio UI | `make ui-test` | host routes, save containment, slots and vm-rendered components |
| Launcher/tools | `make launcher-test` | overlay, readiness, devnet tool and docs checker |
| Docs | `make docs-check` | mdBook build, SUMMARY coverage, links, anchors and canonical status |
| Scaffold | `make scaffold-e2e` | generated project static gates, cargo check and formal check |
| Formal | `make formal-check` | shared RhoApp builds; exit 2 when `lake` is absent |
| Registry | `make registry-check` | catalog shape and measured network identity |
| Vendoring | `make vendor-install-check` | pinned vendored terms and hashes |
| Gates | `make gates-check` | canonical checkers and vendored copies match |
| Package | `make package-check` | complete release layout and self-test |
| MCP | `make mcp-test` | both protocol lanes and a real bridge call |

`make check` runs the developer/CI set; it does not require a live devnet or Lean.

## Generated project profiles

`rhobuilder check --project <dir> --profile ...` runs the project’s own Makefile targets:

| Profile | Contents |
|---|---|
| `static` | `make check`: spec, graph, flows, docs |
| `formal` | `make formal-check`: Lean, vendored hash, per-arm obligations |
| `full` | static + formal + `make unit-all` (devnet harness; `not_run` when off) |

`parse-check` is separate because it needs the pinned node parser. `static` does not claim
it ran.

## Not run is a result

A suite or gate can return `not_run` when the node is absent, throttled or degraded, or
when `lake`/Docker is missing. It is never converted to a pass. `make unit-all` and the
generated `run_suites.py` check node health first.

`deployments/suite-results.json` records `pass`, `fail` or `not_run`. The P5 records in
[Evidence](evidence.md) are the measured examples.

## What the statuses mean

- **Tested** — recorded suites pass.
- **Not run** — the check did not execute; inspect the reason.
- **Proved model** — a shallow Lean theorem; never call it Rholang `verified`.
- **Owed** — a proof obligation is still missing.

See [Glossary](glossary.md) for the vocabulary and [Command reference](command-reference.md)
for flags and exit codes.

## CI

`.github/workflows/ci.yml` has jobs for Rust, plugin/UI, launcher, Lean, scaffold, package
and docs. A release package is built and self-tested on every run. The docs job runs
`mdbook build` and `tools/check_docs.py`; the checker rejects unlisted pages, broken links,
bad anchors and a second status table.

## Evidence is not authority

A stored manifest or suite record is evidence of what ran. The live authority is a signed
Root resolve (`baseline verify --deep`) and the finalized chain. Testnet resets invalidate
prior roots.
