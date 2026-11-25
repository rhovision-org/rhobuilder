# Command reference

The `rhobuilder` binary is the stable JSON surface. The DSH plugin, MCP server, studio and
launcher all call it; they do not reimplement its decisions.

## Global flags

| Flag | Meaning |
|---|---|
| `--json` | Emit one JSON envelope on stdout; human output never enters it. |
| `--registry <path>` | Registry root. Default `registry/` or `RHOBUILDER_REGISTRY`. |
| `--templates <path>` | Scaffold template root. Default `templates/dapp/` or `RHOBUILDER_TEMPLATES`. |

Use `rhobuilder <group> --help` for the authoritative flag list.

## Commands

### `version`

```sh
rhobuilder version --json
```

### `chain`

Read-only except the explicitly gated `call`.

| Command | Key flags | Purpose |
|---|---|---|
| `probe` | `--network` | Identity/capabilities probe; no signing or exploratory deploy. |
| `status` | `--network` | Raw node status. |
| `deploy-status` | `--network`, `--deploy-id` | Read one deploy by id. |
| `call` | `--network`, `--keys`, `--file`/`--term`, `--execute`, `--allow-live`, `--confirm` | Signed term for local suites; CLI-only. |

### `registry`

| Command | Key flags | Purpose |
|---|---|---|
| `list` | `--network`, `--owner`, `--kind` | List primitives. |
| `get` | `--id` | Show one primitive. |
| `graph` | `--root`, `--depth` | Show the reuse graph around a vertex. |
| `verify` | `--network` | Probe identity and check catalog claims. |
| `drift` | `--source-root`, `--strict` | Compare catalog pins with sibling checkouts. |

### `design`

`check`, `validate` (alias) and `report` accept either `--path <design.json>` or
`--stdin`. Validation is in memory; no file is written.

```sh
rhobuilder --json design check --path fixtures/designs/booking.json
printf '%s' "$(cat design.json)" | rhobuilder --json design validate --stdin
```

A refusal or invalid design exits 2. The envelope includes `validation`, `reuse`, `owed`,
`writes: false` and `source`.

### `baseline`

| Command | Key flags | Purpose |
|---|---|---|
| `plan` | `--network`, `--apps` | Read-only plan plus rendered vendored steps. |
| `preflight` | `--network`, `--strict` | Testnet pins/roots/funding/blockers; no writes. |
| `keygen` | `--network`, `--out`, `--force` | Generate ops keys; secrets are never printed. |
| `fund` | `--network`, `--keys`, `--plan`, `--execute`, `--allow-live`, `--confirm` | Devnet validator transfer or testnet faucet plan. |
| `run` | `--network`, `--keys`, `--dry-run`, `--execute`, `--allow-live`, `--confirm`, `--apps`, `--op-cache`, `--min-lock-blocks` | Install/adopt the all-five graph. |
| `verify` | `--network`, `--deep`, `--keys`, `--allow-live`, `--confirm` | Identity/catalog claims; `--deep` signs Root reads. |

`preflight` is testnet-only. `--strict` exits 1 while blockers remain.

### `formal`

| Command | Key flags | Purpose |
|---|---|---|
| `check` | `--project`, `--strict` | Vendor hash, flow obligations, forbidden Lean escapes. |
| `sync` | `--project`, `--source` | Refresh the vendored RhoApp from a checkout. |

`--strict` fails on `owed`.

### `deploy`

| Command | Key flags | Purpose |
|---|---|---|
| `plan` | `--project`, `--network` | Read-only app plan; refuses duplicates, missing dependencies, identity mismatch and mainnet. |
| `execute` | `--project`, `--network`, `--plan`, `--dry-run`, `--execute`, `--keys`, `--allow-live`, `--confirm`, `--op-cache` | Signed app deploy; CLI-only. |

### `project status`

```sh
rhobuilder --json project status --project <dir>
```

Read-only report over `design.json`, `.rhobuilder.json`, `.rhobuilder-contracts.json`,
`spec/flows`, `spec/lean` and `deployments/`. It never runs `make`, Docker or the network.
Badges are `Scaffold`, `Implemented`, `Tested`, `Proved model`, `Owed`, `Not run`,
`Refused` and `Draft` (saved workspace design).

### `check`

```sh
rhobuilder --json check --project <dir> --profile static|formal|full
```

Runs the generated project’s Makefile gates. Exit 1 on a failed gate, 2 when nothing ran.

### `new`

```sh
rhobuilder --json new <target> --design <design.json> --template full|headless --plan
rhobuilder --json new <target> --design <design.json> --template full|headless --apply
```

`--plan` is the default and writes nothing. `--apply` writes once. `--force` allows a
non-empty/outside-workspace target.

## Exit codes

The CLI documents these broad meanings:

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | refusal/drift or a failed project gate |
| 2 | usage/config/identity, invalid design, or `not_run` |
| 3 | missing pin or dependency |
| 4 | node unavailable |
| 5 | live gate/plan digest missing, or a failed gate |
| 6 | unsafe key |

The mapping is not uniform: `design check` refuses with 2, `registry drift --strict` with
1, and `check --profile` uses 1 for fail and 2 for not-run. Read the JSON `status` and
`hint` rather than relying on the number alone.

## JSON envelope

Successful commands return a `schema` and `ok`. Errors use:

```json
{
  "schema": "rhobuilder.error/v1",
  "ok": false,
  "kind": "error",
  "error": {
    "type": "IdentityMismatch",
    "message": "…",
    "status": "identity_mismatch",
    "hint": "…",
    "field": "network"
  }
}
```

Design refusals use `rhobuilder.design-check/v1` with `ok:false` and `reuse.conflicts`
instead of the error envelope.

## Tool scripts

| Command | Purpose |
|---|---|
| `python3 tools/launch_dsh.py [--check|--print-config]` | DSH readiness and overlay; `web`/`headless` launch. |
| `python3 tools/devnet.py status|start|wait|onboard` | Safe local devnet lifecycle. |
| `python3 tools/package_release.py --check --build --self-test` | Complete release tarball. |
| `python3 tools/check_registry.py` | Registry shape/identity. |
| `python3 tools/vendor_install.py --check` | Vendored pinned terms. |
| `python3 tools/gate_sync.py --check` | Gates and hashes. |
| `python3 tools/check_docs.py` | Docs/README links, anchors and status. |
| `python3 tools/export_skills.py` | Skill catalog export. |

## Environment

See [Install](install.md#environment-variables) for the full matrix. The most common are
`RHOBUILDER_REGISTRY`, `RHOBUILDER_TEMPLATES`, `RHOBUILDER_BIN`, `RHOBUILDER_WORKSPACE`,
`RHOBUILDER_NODE`, `RHOBUILDER_CONTAINER`, `RHOBUILDER_DEVNET` and `DEEPSEEK_API_KEY`.

## Live gates

`--execute` on a live network needs `--allow-live` and a `--confirm` token binding
`network`, `networkId`, `genesis` and the plan digest. Mainnet is refused before HTTP.
See [Security model](security.md) and [Public testnet](baseline.md).
