# rhobuilder

`rhobuilder` is a DeepSeek Harness (DSH) app builder for rhovision dApps on the rholo
blockchain. It makes the existing primitive graph the default: design the unique part,
reuse r-cash/r-sure/r-gen/r-share/r-print/r-bridge by `rho:id:`, scaffold the standard
skeleton, prove and test, then deploy only new contracts against primitives that already
exist on the target network.

## Status

P5 is measured on devnet and P6 is code-complete but live-gated. The full five-primitive
graph is installed and root-verified, and the generated `bookable-service` example passes
nine behavioural suites. No signed public-testnet deploy has been executed.

| Area | Status |
|---|---|
| Repo + DSH plugin/skills/launcher | implemented; plugin tools map to real CLI commands; no live model session |
| Rhobuilder studio UI | branded shell + dashboard; readiness, networks, primitives, examples and CLI handoff; read-only behind the DSH token fence |
| Non-technical fitness | adversarial review committed; readiness and release-package P0s addressed, design wizard and devnet provisioning deferred |
| Registry + network identity probe | implemented; pinned-commit drift clean; devnet roots verified |
| Design gate + scaffolder | design gate and full/headless scaffold implemented; `bookable-service` is hand-authored; other designs get generic root/resource/adapter contracts and an install manifest; canonical gates are vendored and hash-checked |
| Lean `RhoApp` + obligations | shared RhoApp, hash check and per-arm obligation gate implemented; `bookable-service` has a shallow `proved-model`; generic scaffolds remain `owed` |
| Devnet end-to-end | P5 measured: five roots verified, app root verified, nine suites pass |
| Public-testnet baseline | P6 code-complete: plan/dry-run/executor/keys/Root verification; live gated; r-cash account-factory blocker |
| MCP server | read-only/plan server with both protocol lanes |

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
