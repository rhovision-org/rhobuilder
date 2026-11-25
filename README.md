# rhobuilder

A DeepSeek Harness (DSH) app builder for rhovision dApps on the rholo blockchain.

`rhobuilder` makes the existing rhovision primitive graph the default. It guides a dApp
author through designing only the unique part, reusing r-cash/r-sure/r-gen/r-share/r-print/
r-bridge primitives by `rho:id:`, scaffolding the standard repo skeleton, proving and
testing the result, and deploying only new contracts against primitives that already exist
on the target network.

## Status

The build is at P5-measured / P6-code-complete. A full five-primitive graph is installed
and verified on the local devnet, and the generated `bookable-service` example passes nine
behavioural suites. The all-five public-testnet baseline is implemented and dry-run green;
no signed testnet deploy has been executed. Mainnet remains refused.

| Area | Status |
|---|---|
| Repo + DSH plugin/skills/launcher | implemented; DSH 0.2.2-rc.5 `--dump-config` validates web and headless overlays; no live model session (`DEEPSEEK_API_KEY` unset) |
| Rhobuilder studio UI | branded title/favicon/wordmark/hero + dashboard; design composer with validated `design.json` save, project-status badges, evidence renderer, testnet preflight and devnet handoff behind the DSH token fence |
| Non-technical fitness | adversarial review committed at `docs/src/review/non-technical-fitness.md`; the deferred items are addressed by the composer, devnet lifecycle, prose errors, project status and evidence renderer |
| Registry + network identity probe | implemented; pinned-commit drift clean; devnet roots verified |
| Design gate + scaffolder | design gate and full/headless scaffold implemented; `bookable-service` is hand-authored; other designs get generic root/resource/adapter contracts and a `.rhobuilder-contracts.json` install manifest; canonical gates are vendored and hash-checked |
| Lean `RhoApp` + obligations | shared RhoApp, vendoring, hash check and per-arm obligation gate implemented; `bookable-service` has a shallow `proved-model` (`voteOnce`, provider exclusion, settlement conservation); generic scaffolds remain `owed` |
| Devnet end-to-end | P5 measured: r-cash core, r-gen, r-share, r-sure and r-print verified; app root verified; nine suites pass — see `docs/src/evidence/p5-devnet.json` |
| Public-testnet baseline | P6 code-complete: all-five plan/dry-run, executor, keys, funding and Root verification implemented; `baseline preflight` reports pins/roots/funding blockers; live campaign gated and not run; r-cash account-factory is the shared blocker |
| MCP server | read-only/plan server implemented with both protocol lanes |

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
./target/debug/rhobuilder --json registry drift --source-root "$HOME" --strict

# all-five plans: read-only; testnet execution stays gated
./target/debug/rhobuilder --json baseline preflight --network testnet
./target/debug/rhobuilder --json baseline plan --network testnet
./target/debug/rhobuilder --json baseline run  --network testnet --dry-run

# read-only project status and safe devnet lifecycle
./target/debug/rhobuilder --json project status --project <dir>
python3 tools/devnet.py status --json
python3 tools/devnet.py start --json
python3 tools/devnet.py onboard --json          # dry run; --execute is devnet-only

# local devnet install (after keygen/fund), then a generated app
./target/debug/rhobuilder --json baseline run --network devnet \
  --keys .ops/devnet/keys.json --execute
./target/debug/rhobuilder new /tmp/app --design fixtures/designs/booking.json \
  --template headless --apply --force
./target/debug/rhobuilder --json deploy plan --project /tmp/app --network devnet

python3 tools/launch_dsh.py --check --json
python3 tools/launch_dsh.py --with-mcp --port 3081 --no-open web
python3 tools/launch_dsh.py --print-config headless

# complete release tarball (binary + registry + templates + UI + launcher)
cargo build --release -p rhobuilder-cli
python3 tools/package_release.py --check --build --self-test --json
```

A real DSH launch exports the skill catalog to `.dsh/skills/`, writes a mode-specific
overlay to `.dsh/patches/rhobuilder-<mode>.yml`, and starts the shipped `web` or `headless`
profile with `--patch`.

## Layout

- `crates/` — Rust core, registry, chain, formal, gates and CLI.
- `agents/deepseek/` — DSH plugin, branded studio UI, harness.toml, presets, procedure.
- `skills/` — canonical skill sources (`skills.toml` is the metadata source).
- `tools/` — launcher, packaged launcher, release packager, skill export, docs/tests.
- `docs/` — mdBook source, including the measured P5 evidence.
- `gates/` — canonical checkers and their hash manifest; vendored into the scaffold.
- `registry/`, `templates/`, `formal/` — catalog, scaffold and Lean.

## Maintainer

Originated and maintained by
[@PatrickMockridge](https://github.com/PatrickMockridge).

## Licence

AGPL-3.0-only. DeepSeek Harness is MIT and is not vendored.
