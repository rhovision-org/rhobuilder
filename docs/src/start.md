# Quickstart

This is the shortest path from a clean checkout to a validated design and a scaffold plan.
You do not need to read the registry schema or write Rholang to begin. For prerequisites
and tarball installation, see [Install](install.md).

## 1. Check the machine

```sh
python3 tools/launch_dsh.py --check --json
```

Read `setupRequired` and each `hint`. A `setup` item is not a pass. The common items are
`DEEPSEEK_API_KEY` (chat), devnet (Docker plus a healthy local node), `lake` (formal
checks) and mdBook (docs only). See [Troubleshooting](troubleshooting.md).

## 2. Build and launch

```sh
cargo build -p rhobuilder-cli
python3 tools/launch_dsh.py --with-mcp --port 3081 web
```

Open the printed token URL. The sidebar has **Rhobuilder** for readiness, networks,
primitives, examples, the design composer, project status and testnet preflight. Use
`--no-open` on a remote/headless machine and `--no-ui` to fall back to the stock shell.

## 3. Start a design

Use the dashboard **Design composer** or a starter. The composer builds a
`rhobuilder.design/v1` draft, validates it through the CLI, and saves it for continuity:

- blank project → `designs/<slug>.json`;
- existing project with `.rhobuilder.json` → its root, with overwrite confirmation.

Only a validated design is written; a refused draft writes nothing. You can also ask the
assistant:

> Interview me and create a design.json for a bookable service.

The assistant designs the unique part, reuses r-cash/r-sure/r-gen/r-share/r-print by
`rho:id`, and runs the reuse gate. A `refuse` verdict stops the loop. See
[Design and reuse](design.md) and [Studio tour](ui.md).

## 4. Scaffold, prove, test

Plan a scaffold first; apply is a separate chat/CLI action after you approve the file
count.

```sh
rhobuilder --json new /tmp/app --design designs/<slug>.json --template headless --plan
rhobuilder --json new /tmp/app --design designs/<slug>.json --template headless --apply
rhobuilder --json project status --project /tmp/app
```

`project status` reports `Scaffold`, `Implemented`, `Tested`, `Proved model`, `Owed`,
`Not run` and `Refused` from project files and recorded evidence. A saved design is `Draft`
until scaffolded. `make check` is static; `make formal-check` needs Lean; devnet suites
require a healthy devnet and otherwise report `not_run`. See [Scaffolding](scaffold.md) and
[Testing and gates](testing.md).

## 5. Devnet and deploy

The safe local lifecycle is CLI-only:

```sh
python3 tools/devnet.py status --json
python3 tools/devnet.py start --json
python3 tools/devnet.py onboard --json          # dry run
python3 tools/devnet.py onboard --json --execute  # devnet only
```

`start` only starts the existing `devnet-bootstrap`; it never creates, resets or removes a
container or volume. `onboard` reuses keygen, funding, the baseline install and deep verify
under explicit `--execute`. See [Local devnet](devnet.md).

Signing, funding, deploy execute, testnet baseline run and deep verify are deliberately
CLI-only. Use the copyable commands in the dashboard. Mainnet is refused; testnet needs a
separate approval and funded key. See [Public testnet](baseline.md) and
[Security model](security.md).

## What is real

- P5 devnet: five primitive roots and a generated app are measured and verified; nine suites
  pass.
- P6 testnet: plan/dry-run/executor are implemented; live execution is gated and has not
  run. The r-cash account-factory candidate is the shared blocker.
- Generated scaffolds are not finished apps. `owed` is a real status; `proved-model` is a
  shallow model, never a Rholang refinement.

## Next

- [Studio tour](ui.md) — what the dashboard shows.
- [Core concepts](concepts.md) — primitives, roots, statuses.
- [Command reference](command-reference.md) — flags and exit codes.
