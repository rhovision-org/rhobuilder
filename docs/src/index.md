# rhobuilder

Design the unique part of a rholo dApp, reuse the rest by `rho:id`, and deploy only what is
new. `rhobuilder` is a DeepSeek Harness (DSH) app builder over the existing rhovision
primitive graph: r-cash, r-sure, r-gen, r-share, r-print and r-bridge.

It guides the author through the design and reuse gate, scaffolds the standard skeleton,
runs static/formal/devnet checks, and produces a deploy plan. Signing stays in the CLI with
explicit gates.

## Status

This is the canonical status table. Other pages summarize and link here.

| Area | Status |
|---|---|
| Repo + DSH plugin/skills/launcher | implemented; DSH 0.2.2-rc.5 overlays validate; no live model session (`DEEPSEEK_API_KEY` unset) |
| Rhobuilder studio UI | branded shell; composer with validated `design.json` save; project status, evidence and testnet preflight; behind the DSH token fence |
| Non-technical fitness | adversarial review committed; composer, devnet lifecycle, prose errors, project status and evidence renderer address the deferred items |
| Registry + network identity probe | implemented; pinned-commit drift clean; devnet roots verified |
| Design gate + scaffolder | design gate and full/headless scaffold implemented; `bookable-service` is hand-authored; other designs get generic contracts and an install manifest; canonical gates are vendored and hash-checked |
| Lean `RhoApp` + obligations | shared RhoApp, hash check and per-arm obligation gate implemented; `bookable-service` has a shallow `proved-model`; generic scaffolds remain `owed` |
| Devnet end-to-end | P5 measured: r-cash core, r-gen, r-share, r-sure and r-print verified; app root verified; nine suites pass |
| Public-testnet baseline | P6 code-complete: preflight/plan/dry-run/executor/keys/Root verification; live gated and not run; r-cash account-factory is the shared blocker |
| MCP server | read-only/plan server implemented with both protocol lanes |

P5 evidence is in [Evidence](evidence.md); the P6 blockers are in
[Public testnet](baseline.md). Mainnet is refused.

## Choose your path

- **Non-technical author** — [Quickstart](start.md), then [Studio tour](ui.md) and
  [Core concepts](concepts.md).
- **Developer** — [Workflow](workflow.md), [Design and reuse](design.md), then
  [Command reference](command-reference.md).
- **Operator** — [Install](install.md), [Local devnet](devnet.md), then
  [Public testnet](baseline.md).
- **Maintainer** — [Architecture](architecture.md), [Security model](security.md), then
  [Contributing](contributing.md).

## First five minutes

```sh
python3 tools/launch_dsh.py --check --json
cargo build -p rhobuilder-cli
python3 tools/launch_dsh.py --with-mcp --port 3081 web
```

Open the printed token URL. The dashboard shows readiness, networks, the primitive graph,
the design composer, project status, evidence and testnet blockers. Read the setup hints;
a `setup` item is not a pass. Continue with [Quickstart](start.md).

## Docs map

| Part | Start here | Also |
|---|---|---|
| Front | [Install](install.md) | [Quickstart](start.md), [Studio tour](ui.md), [Core concepts](concepts.md), [Glossary](glossary.md) |
| Build loop | [Workflow](workflow.md) | [Design](design.md), [Scaffold](scaffold.md), [Formal](formal.md), [Registry](registry.md), [Testing](testing.md) |
| Operate | [DeepSeek Harness](dsh.md) | [Devnet](devnet.md), [Testnet](baseline.md), [MCP](mcp.md), [Command reference](command-reference.md), [Troubleshooting](troubleshooting.md), [FAQ](faq.md) |
| Understand | [Architecture](architecture.md) | [Security](security.md), [Gates](gates.md), [Evidence](evidence.md) |
| Appendix | [Contributing](contributing.md) | [Fitness review](review/non-technical-fitness.md) |

## What it targets

- **Runtime:** DeepSeek Harness 0.2.x, native plugin, skills and presets.
- **Chain:** rholo devnet and `https://testnet.rhobot.net`; the mainnet profile refuses.
- **Primitives:** r-cash, r-sure, r-gen, r-share, r-print, r-bridge and rholo genesis.
- **Formal:** Lean 4 shallow models; `proved-model` is the ceiling, never a refinement.

## Licence

AGPL-3.0-only. DeepSeek Harness is MIT and is not vendored.
