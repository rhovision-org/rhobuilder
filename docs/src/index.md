# rhobuilder

`rhobuilder` is a DeepSeek Harness (DSH) app builder for rhovision dApps on the rholo
blockchain. It makes the existing primitive graph the default: design the unique part,
reuse r-cash/r-sure/r-gen/r-share/r-print/r-bridge by `rho:id:`, scaffold the standard
skeleton, prove and test, then deploy only new contracts against primitives that already
exist on the target network.

## Status

This is the P0 scaffold plus the P1 read-only identity slice. The registry profile, pins
and first-cut catalog are committed; sibling drift, the scaffolder, Lean library, devnet
harness and testnet baseline are not implemented yet. `make check` is the current gate.

| Area | Status |
|---|---|
| Repo + DSH plugin/skills/launcher | P0/P3 — implemented; DSH 0.2.2-rc.5 `--dump-config` validates web and headless overlays |
| Registry + network identity probe | P1 — profile/pins/catalog committed; pinned-commit drift clean |
| Design gate + scaffolder + vendored gates | P2 — design gate and full/headless scaffold implemented; canonical gate vendoring pending |
| Lean `RhoApp` + per-dApp models | P4 — shared RhoApp, vendoring and hash check implemented; per-app model is scaffold |
| Devnet end-to-end | P5 — installer/smoke implemented; execution pending a healthy devnet (local node throttled) |
| Public-testnet baseline | P6 — not implemented |
| Optional MCP server | P7 — read-only/plan server implemented with both protocol lanes |

## Quick start

```sh
cargo build -p rhobuilder-cli
./target/debug/rhobuilder version --json

python3 tools/launch_dsh.py --check --json
python3 tools/launch_dsh.py --print-config headless
```

## What it targets

- **Runtime:** DeepSeek Harness 0.2.x, native plugin and skills.
- **Chain:** rholo devnet and `https://testnet.rhobot.net`; mainnet is gated.
- **Formal:** Lean 4 shallow models; `proved-model` is the ceiling, never a refinement.

## Licence

AGPL-3.0-only. DeepSeek Harness is MIT and is not vendored.
