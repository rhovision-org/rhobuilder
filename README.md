# rhobuilder

[![CI](https://github.com/rhovision-org/rhobuilder/actions/workflows/ci.yml/badge.svg)](https://github.com/rhovision-org/rhobuilder/actions/workflows/ci.yml)
[![Licence: AGPL-3.0-only](https://img.shields.io/badge/licence-AGPL--3.0--only-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](CHANGELOG.md)

Design the unique part of a rholo dApp, reuse the rest by `rho:id`, and deploy only what is
new.

`rhobuilder` is a DeepSeek Harness (DSH) app builder for rhovision dApps on the rholo
blockchain. It makes the existing primitive graph the default: r-cash for accounts and
payments, r-sure for escrow and insurance, r-gen for claims and provenance, r-share for
booking capacity, r-print for publishing, and r-bridge for cross-chain work.

## What it is

Most dApps should not rebuild accounts, payments or escrow. `rhobuilder` starts from the
opposite assumption: those capabilities already exist on-chain. The author writes only the
coordination, policy or state machine that is genuinely new, and the builder refuses a
`new` component that duplicates a protected commodity primitive.

The result is a design, a generated project skeleton, static/formal/devnet checks and a
deploy plan that touches only the new contracts. Signing remains a separate, explicit CLI
action.

```text
idea
  └─► design.json ──► reuse gate (registry + graph)
                       └─► scaffold ──► static / Lean / devnet suites
                                          └─► deploy plan ──► gated signed execute
```

The stack is deliberately thin:

```text
DSH studio / native plugin / MCP server
                 │
                 ▼
        rhobuilder CLI (JSON envelopes, decisions, gates)
                 │
    registry · templates · gates · formal · fixtures
                 │
                 ▼
        rholo devnet / gated public testnet
```

Read [Core concepts](docs/src/concepts.md) for the mental model and
[Architecture](docs/src/architecture.md) for the component map.

## What is real today

- **P5 is measured on devnet.** Five primitive roots and a generated `bookable-service`
  app root were installed and verified; nine behavioural suites pass. See
  [Evidence](docs/src/evidence.md).
- **P6 is code-complete but live-gated.** The all-five public-testnet plan, dry-run,
  executor, keys, funding and Root verification exist. No signed testnet deploy, faucet
  spend or consolidation has run.
- **The shared testnet blocker is real.** The r-cash `component:account-factory` candidate
  is unverified, four app roots are missing, and r-share/r-print pins are
  `TBD-review-required`. See [Public testnet](docs/src/baseline.md).
- **`owed` and `not_run` are outcomes, not failures to hide.** A shallow Lean theorem is
  `proved-model`, never a Rholang refinement. A check that did not run is not a pass.
- **Mainnet is refused.** There is no live rhovision mainnet profile that deploys.

The canonical status table lives in [docs/src/index.md](docs/src/index.md#status). This
README intentionally does not duplicate it.

## Choose your path

| You are… | Start with |
|---|---|
| A non-technical author | [Quickstart](docs/src/start.md) → [Studio tour](docs/src/ui.md) → [Core concepts](docs/src/concepts.md) |
| A developer | [Workflow](docs/src/workflow.md) → [Design and reuse](docs/src/design.md) → [Command reference](docs/src/command-reference.md) |
| An operator | [Install](docs/src/install.md) → [Local devnet](docs/src/devnet.md) → [Public testnet](docs/src/baseline.md) |
| An evaluator or maintainer | [Architecture](docs/src/architecture.md) → [Security model](docs/src/security.md) → [Testing and gates](docs/src/testing.md) |

## Prerequisites

| Need | Version / notes |
|---|---|
| Node | 22.x for DSH 0.2.2-rc.5. DSH 0.2.3-alpha.2 needs Node 24.22.3 or later. |
| Rust | 1.95.0; pinned in `rust-toolchain.toml`. |
| Python | 3.11+ for the launcher (`tomllib`), devnet tool, checkers and packager. |
| Docker | Only for local devnet execution and onboarding. |
| Lean / `lake` | Only for formal checks; otherwise `not_run`. |
| mdBook | Only to build the manual. |
| git + network | The first source build fetches the pinned `rcash-sdk` dependency. |

## Quickstart

The first command is a readiness check, not a build. It is read-only and tells you what is
missing before you spend time on the rest.

```sh
git clone https://github.com/rhovision-org/rhobuilder
cd rhobuilder

python3 tools/launch_dsh.py --check --json
```

`setupRequired` is honest. A typical fresh machine shows `api-key` and `devnet`; install
hints are in the response and in [Troubleshooting](docs/src/troubleshooting.md). A `setup`
item is not a pass.

```sh
cargo build -p rhobuilder-cli
./target/debug/rhobuilder version --json
```

```json
{
  "ok": true,
  "product": "rhobuilder",
  "schema": "rhobuilder.version/v1",
  "version": "0.1.0"
}
```

Launch the studio (chat needs `DEEPSEEK_API_KEY`; the CLI and docs do not):

```sh
python3 tools/launch_dsh.py --with-mcp --port 3081 --no-open web
```

Open the printed token URL. The sidebar has **Rhobuilder** for readiness, networks,
primitives, examples and the design composer.

Validate the worked example and plan a scaffold without writing:

```sh
./target/debug/rhobuilder --json design check --path fixtures/designs/booking.json
./target/debug/rhobuilder --json new /tmp/rb-app \
  --design fixtures/designs/booking.json --template headless --plan
```

After an explicit apply, read-only project status separates the claims:

```sh
./target/debug/rhobuilder new /tmp/rb-app \
  --design fixtures/designs/booking.json --template headless --apply --force
./target/debug/rhobuilder --json project status --project /tmp/rb-app
```

The report uses `Scaffold`, `Implemented`, `Tested`, `Proved model`, `Owed`, `Not run` and
`Refused`; it never turns a missing run into a pass.

Check the local devnet and the gated testnet without changing either:

```sh
python3 tools/devnet.py status --json
./target/debug/rhobuilder --json baseline preflight --network testnet
```

An unhealthy devnet returns `healthy:false` with a `nextCommand`; it is not treated as a
pass. The testnet preflight reports the current blockers, including the unverified
account-factory and the TBD pins. That is the real state.

The full author path is in [Quickstart](docs/src/start.md); the full command surface is in
[Command reference](docs/src/command-reference.md).

## Install from a release tarball

If you do not want a source build:

```sh
cargo build --release -p rhobuilder-cli
python3 tools/package_release.py --check --build --self-test --json
```

The artifact is `dist/rhobuilder-<version>-linux-<arch>.tar.gz` plus `.sha256`. It contains
the binary, `launch-rhobuilder.sh`, registry, templates, formal library, DSH plugin/UI,
skills, tools, MCP server, gates, fixtures and docs.

```sh
cd dist
sha256sum -c rhobuilder-0.1.0-linux-x86_64.tar.gz.sha256
tar xzf rhobuilder-0.1.0-linux-x86_64.tar.gz
cd rhobuilder-0.1.0-linux-x86_64
./bin/rhobuilder version --json
./launch-rhobuilder.sh --check --json
```

The packaged launcher is the top-level `launch-rhobuilder.sh`, not the source
`tools/launch_dsh.py`. See [Install](docs/src/install.md) for environment variables,
upgrade and uninstall.

## Command overview

The CLI is the decision surface; the plugin, MCP server and studio call it.

| Group | What it covers |
|---|---|
| `chain` | `probe`, `status`, `deploy-status`; `call` is signed and CLI-only. |
| `registry` | `list`, `get`, `graph`, `verify`, `drift`. |
| `design` | `check`, `validate`, `report`; `--path` or `--stdin`. |
| `baseline` | `plan`, `preflight`, `keygen`, `fund`, `run`, `verify`. |
| `formal` | `check`, `sync`. |
| `deploy` | `plan`; `execute` is gated and CLI-only. |
| `project` | `status` — read-only scaffold/test/model/deployment badges. |
| `check` | Run a generated project’s `static`, `formal` or `full` Makefile gates. |
| `new` | Plan or apply a `full`/`headless` skeleton. |

Tooling: `tools/launch_dsh.py`, `tools/devnet.py`, `tools/package_release.py`,
`tools/check_registry.py`, `tools/gate_sync.py`, `tools/check_docs.py`.

Every command supports `--json`; flags and exit codes are in
[Command reference](docs/src/command-reference.md).

## Security model

- The studio never signs, funds or holds keys. Its only write is a validated `design.json`
  under a contained path; keys, signing and deploy execution are CLI-only.
- Testnet execution needs an explicit campaign approval, `--allow-live`, a funded key and a
  `--confirm` token binding network, genesis and the plan digest.
- Mainnet is refused before HTTP.
- Keys stay mode 0600, under the working directory, and are never printed. Devnet-named keys
  are refused on live networks.
- The executor adopts a resolvable root and never deploys a second copy of a primitive; a
  partial install stops.
- Sibling repositories are read with `git show` only; rhobuilder never edits them.

See [Security model](docs/src/security.md) and [SECURITY.md](SECURITY.md).

## Layout

- `crates/` — Rust core, registry, chain, formal, gates and CLI.
- `agents/deepseek/` — DSH plugin, studio UI, `harness.toml`, presets and procedure.
- `skills/` — canonical skill sources (`skills.toml` is the metadata source).
- `tools/` — launcher, devnet lifecycle, release packager, skill export and checkers.
- `docs/` — the mdBook manual and measured P5 evidence.
- `gates/` — canonical checkers and their hash manifest; vendored into the scaffold.
- `registry/`, `templates/`, `formal/`, `fixtures/` — catalog, scaffold, Lean and examples.

## Docs map

| Part | Entry |
|---|---|
| Front | [Install](docs/src/install.md), [Quickstart](docs/src/start.md), [Studio tour](docs/src/ui.md), [Core concepts](docs/src/concepts.md), [Glossary](docs/src/glossary.md) |
| Build loop | [Workflow](docs/src/workflow.md), [Design](docs/src/design.md), [Scaffold](docs/src/scaffold.md), [Formal](docs/src/formal.md), [Registry](docs/src/registry.md), [Testing](docs/src/testing.md) |
| Operate | [DSH](docs/src/dsh.md), [Devnet](docs/src/devnet.md), [Testnet](docs/src/baseline.md), [MCP](docs/src/mcp.md), [Command reference](docs/src/command-reference.md), [Troubleshooting](docs/src/troubleshooting.md), [FAQ](docs/src/faq.md) |
| Understand | [Architecture](docs/src/architecture.md), [Security](docs/src/security.md), [Gates](docs/src/gates.md), [Evidence](docs/src/evidence.md) |
| Appendix | [Contributing](docs/src/contributing.md), [Fitness review](docs/src/review/non-technical-fitness.md) |

Build the book with `make docs-check` or `mdbook build`.

## Contributing and release

See [CONTRIBUTING.md](CONTRIBUTING.md) for invariants, checks and commit identity. Release
packaging and the no-tag-until-live-baseline policy are in
[Contributing](docs/src/contributing.md). Release notes are in [CHANGELOG.md](CHANGELOG.md);
citation metadata is in [CITATION.cff](CITATION.cff); third-party notices are in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Maintainer and licence

Originated and maintained by
[@PatrickMockridge](https://github.com/PatrickMockridge).

AGPL-3.0-only. DeepSeek Harness is MIT and is not vendored.
