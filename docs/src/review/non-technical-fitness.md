# Non-technical fitness review

**Verdict:** rhobuilder is a strong, honest developer/CI engine and a weak non-technical
product. The technical gate is roughly **7.5/10**; the one-stop-shop experience is roughly
**2.5/10** until the UI, readiness and packaging work below lands.

This review is acceptance evidence, not a side report. Each finding has a disposition and a
traceability row to the UI or CLI change that addresses it.

## What is genuinely strong

- Mainnet is refused before HTTP; testnet live actions require `--allow-live` plus an
  identity+plan confirmation token.
- Keys stay mode 0600, gitignored and unprinted; devnet-named keys are refused on live
  networks.
- Adopt-before-deploy and a partial-root stop prevent duplicate primitives.
- `not_run` is a first-class result; a throttled devnet never reads as a pass.
- P5 is measured: five devnet roots, a generated app, and nine behavioural suites.

## P0 — blocks non-technical use

| ID | Finding | Evidence | Disposition |
|---|---|---|---|
| P0-1 | Preflight cannot fail on missing prerequisites; API key, npx DSH and cargo fallback are warnings, and Rust/Docker/Lean/mdBook/devnet are not probed. | `tools/launch_dsh.py:666-675`, `:764`, `:655-663` | **Fixed in this pass**: hard `setup` verdict + UI readiness panel |
| P0-2 | The release artifact is a bare binary plus docs; it cannot scaffold, check or plan because `registry/` and `templates/` are external. | `.github/workflows/release.yml:16-19`, `crates/rhobuilder-cli/src/main.rs:29-34` | **Fixed in this pass**: complete release tarball + extraction self-test |
| P0-3 | No design-authoring surface; `design.json` is hand-written or model-written and only validated after the fact. | `agents/deepseek/plugin/index.js:156-175`, `fixtures/designs/booking.json` | **Fixed**: dashboard composer validates in memory and saves a validated `design.json` for continuity |
| P0-4 | First launch is stock “DeepSeek Harness”: no rhobuilder title, onboarding, dashboard or quick actions. | generated `.dsh/patches/rhobuilder-web.yml`; shipped web profile | **Fixed in this pass**: branded UI package + dashboard |
| P0-5 | Devnet bring-up and funding are developer-machine work (Docker container, validator key, env, keys). | `crates/rhobuilder-chain/src/funding.rs:117-148`, `crates/rhobuilder-cli/src/main.rs:878-881` | **Fixed**: `tools/devnet.py status/start/wait/onboard`; onboarding is explicit `--execute` and devnet-only. Creating a container from scratch remains external |

## P1 — major friction

| ID | Finding | Evidence | Disposition |
|---|---|---|---|
| P1-1 | Curated skills omit the rho-calculus, CoC and Lean 4 skills that the build procedure tells the model to read. | `agents/deepseek/harness.toml:22-30`, `skills/rhobuilder-build.md:40-41` | **Fixed**: add the three skills to the curated set |
| P1-2 | Exit code 2 is overloaded across refusal, usage, identity and `not_run`. | `crates/rhobuilder-cli/src/main.rs:3-4`, `:2141`, `:2758-2763` | **Fixed**: error envelopes carry `status` and `hint`; exit codes stay compatible |
| P1-3 | Errors are developer jargon rather than actionable copy. | e.g. `main.rs:947`, `:1036-1040`, `:697-699` | **Fixed**: every emitted kind has a status and hint, and bridge errors lead with per-kind prose before the detail |
| P1-4 | First build may silently clone `r-cash` and need network; no preflight. | `Cargo.toml` `rcash-sdk` git pin; `launch_dsh.py:74-77` | **Fixed**: readiness reports the source-build/network path |
| P1-5 | Local absolute-home defaults leak into docs and Make targets. | `README.md:44`, `Makefile:5` | **Fixed**: remove machine-local defaults; package check rejects them |
| P1-6 | Key-path containment is checked at keygen but not every key load. | `main.rs:782-785` vs `:840-846`, `:1116-1142`, `:1836-1886`, `:2597-2602` | **Fixed for live networks**: every load calls `require_under`; disposable devnet keys may live in the installation |

## P2 — polish

| ID | Finding | Disposition |
|---|---|---|
| P2-1 | Generic scaffolds look finished; `owed` arms are not visible. | **Fixed**: `rhobuilder project status` and the dashboard Project status card render `Scaffold`, `Implemented`, `Tested`, `Proved model`, `Owed`, `Not run` and `Refused` |
| P2-2 | Evidence JSON contains `not_run`/`candidate` machine states that non-technical users misread. | **Fixed**: the dashboard renders measured evidence with plain statements and a raw-JSON disclosure |
| P2-3 | Scaffold `confirm:true` is model-internal, not a user approval. | **Surfaced**: the dashboard shows a read-only plan/file count and the build skill requires explicit confirmation; applying remains a separate chat action |
| P2-4 | Skill jargon has no user-facing glossary. | **Fixed**: landing hero + glossary below |

## Task trace: idea → repo → devnet → testnet

| Step | Current affordance | Reality |
|---|---|---|
| Probe | `rhobuilder_chain` tool | read-only; works |
| Design | composer + validate/save | dashboard composer validates in memory and saves `design.json`; refused drafts write nothing |
| Reuse | design report | refuses duplicate primitives; JSON report |
| Scaffold | plan/apply tool | writes once; UI requires explicit apply |
| Implement | none | agent/editor work |
| Prove | check formal | needs `lake`; generic arms stay `owed` |
| Test | check full | needs a healthy devnet; otherwise `not_run` |
| Deploy plan | plan tool | read-only; refuses missing/duplicate/mainnet |
| Deploy/baseline execute | CLI only | signed, gated, key-bearing |
| Testnet baseline | preflight/plan/verify read-only | `baseline preflight` names every blocker; execution remains CLI-only and gated |

## Plain-language glossary

- **rholo** — the blockchain these dApps run on. Deploys are signed.
- **`rho:id`** — an unforgeable on-chain address for a contract or component.
- **Primitive** — a reusable on-chain capability owned by another app, such as r-cash
  accounts or r-sure escrow.
- **Root** — an app’s directory contract that resolves component keys to `rho:id`s.
- **Scaffold** — a generated project skeleton; it is not a finished app.
- **Linear cell** — one state resource that is taken and reposted exactly once.
- **`owed`** — a proof obligation that has not been written yet.
- **`not_run`** — the check did not execute (for example, the devnet is unhealthy); it is
  not a pass.
- **`proved-model`** — proved about the shallow Lean model, never a refinement of the
  deployed Rholang.
- **Devnet / testnet / mainnet** — local disposable chain / public gated chain / no live
  rhovision mainnet.

## Traceability to the UI work

| Finding | UI/CLI surface |
|---|---|
| P0-1, P0-5 | Dashboard Readiness panel; `launch_dsh --check` setup verdict; `tools/devnet.py status/start/wait/onboard` |
| P0-2 | Complete release tarball and launch wrapper |
| P0-3 | Dashboard composer, `design validate --stdin`, contained `design-save`, design-check status |
| P0-4 | Title, favicon, sidebar wordmark, hero card, Rhobuilder dashboard |
| P1-1 | Curated skill list in `harness.toml` |
| P1-2, P1-3 | Status labels and humanized error copy |
| P1-4 | Readiness “source build” warning |
| P1-5 | Portable launcher/package paths |
| P1-6 | CLI key-load containment |
| P2-1..P2-4 | Status badges, evidence panel, apply confirmation, glossary |

## Boundary

The UI may read, plan, validate, start conversations and save a validated `design.json`.
It must not write code, apply a scaffold, sign, fund, hold keys or execute deploys. Those
operations remain CLI-only with explicit gates.
