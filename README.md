# rhobuilder

A DeepSeek Harness (DSH) app builder for rhovision dApps on the rholo blockchain.

`rhobuilder` makes the existing rhovision primitive graph the default. It guides a dApp
author through designing only the unique part, reusing r-cash/r-sure/r-gen/r-share/r-print/
r-bridge primitives by `rho:id:`, scaffolding the standard repo skeleton, proving and
testing the result, and deploying only new contracts against primitives that already exist
on the target network.

## Status

P0 scaffold plus the P1 read-only identity slice: repository, Rust workspace, DSH
plugin/skills/launcher, docs and CI, a committed registry/profile/pins first cut, and a
live testnet identity probe. Sibling drift, the design gate, scaffolder, Lean library,
devnet harness and testnet baseline are not implemented yet. `make check` is the current
gate.

| Area | Status |
|---|---|
| Repo + DSH plugin/skills/launcher | P0/P3 — implemented; DSH 0.2.2-rc.5 `--dump-config` validates web and headless overlays |
| Registry + network identity probe | P1 — profile/pins/catalog committed; pinned-commit drift clean |
| Design gate + scaffolder + vendored gates | P2 — design gate and full/headless scaffold implemented; canonical gate vendoring pending |
| Lean `RhoApp` + per-dApp models | P4 — shared RhoApp, vendoring and hash check implemented; per-app model is scaffold |
| Devnet end-to-end | P5 — installer/smoke implemented; execution pending a healthy devnet (local node throttled) |
| Public-testnet baseline | P6 — not implemented |
| Optional MCP server | P7 — read-only/plan server implemented with both protocol lanes |

## What it targets

- **Runtime:** DeepSeek Harness 0.2.x (native plugin, skills, presets, profile overlay).
- **Chain:** rholo devnet and `https://testnet.rhobot.net`; a mainnet profile exists but
  refuses deploys until a live rhovision mainnet exists.
- **Primitives:** r-cash, r-sure, r-gen, r-share, r-print, r-bridge, plus the rholo genesis
  system contracts.
- **Formal:** Lean 4 models in the sibling style; `proved-model` is the ceiling, never a
  Rholang refinement.

## Quick start

```sh
cargo build -p rhobuilder-cli
./target/debug/rhobuilder version --json
./target/debug/rhobuilder --json chain probe --network testnet
./target/debug/rhobuilder --json registry drift --source-root /home/patrick --strict

python3 tools/launch_dsh.py --check --json
python3 tools/launch_dsh.py --print-config headless
```

A real DSH launch exports the skill catalog to `.dsh/skills/`, writes a mode-specific
overlay to `.dsh/patches/rhobuilder-<mode>.yml`, and starts the shipped `web` or `headless`
profile with `--patch`.

## Layout

- `crates/` — Rust core, registry, chain, formal, gates and CLI.
- `agents/deepseek/` — DSH plugin, harness.toml, presets, harness-only procedure.
- `skills/` — canonical skill sources (`skills.toml` is the metadata source).
- `tools/` — launcher, skill export, docs check, tests.
- `docs/` — mdBook source.
- `registry/`, `templates/`, `formal/`, `gates/` — P1–P4.

## Maintainer

Originated and maintained by
[@PatrickMockridge](https://github.com/PatrickMockridge).

## Licence

AGPL-3.0-only. DeepSeek Harness is MIT and is not vendored.
