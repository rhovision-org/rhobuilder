# Rhobuilder studio UI

The DSH web launch is branded as **rhobuilder — rholo dApp studio**. Chat remains the
working surface; the UI adds identity, onboarding, readiness, status and safe shortcuts.

## Launch

```sh
python3 tools/launch_dsh.py --with-mcp --port 3081 --no-open web
```

Open the printed token URL. `--no-ui` falls back to the stock shell; `--port` avoids a
collision with another local DSH instance; `--no-open` is useful on a remote/headless
machine.

The UI is a dependency-free DSH package at `agents/deepseek/ui/`. `index.js` is the Node
half; `client.js` is a hand-written React 18 factory-CJS bundle. The launcher mounts it by
absolute path in the `--patch` overlay, disables the shipped `ui-brand-official` row, and
supplies the rhobuilder persona by default.

## What the user sees

- Rhobuilder title, favicon, sidebar wordmark and a welcome hero with four starters.
- A **Rhobuilder** dashboard page:
  - Readiness: required/setup/warn checks with install hints.
  - Networks: devnet local, testnet gated, mainnet refused.
  - Primitive graph summary (16 primitives, 5 edges, owner counts).
  - **Design composer**: build a `rhobuilder.design/v1` draft, validate it in memory,
    then save it as `design.json` for continuity. The save is the only UI write. Blank
    project saves to `designs/<slug>.json`; an existing project (with
    `.rhobuilder.json`) can receive `<project>/design.json` after an explicit overwrite.
  - Examples: bookable-service, payments-ledger, duplicate-escrow. “Plan scaffold
    (no writes)” returns the file count through a read-only CLI plan; applying is a
    separate, explicitly confirmed chat action.
  - Project status: `Scaffold`, `Implemented`, `Tested`, `Proved model`, `Owed`,
    `Not run`, plus `Refused` for a design-gate refusal and `Draft` for a saved workspace
    design. These come from project files and recorded evidence, not from a fresh `make`.
  - Measured evidence: the P5 records with suite statuses and a raw-JSON disclosure.
  - Testnet campaign: read-only blockers and approval checkpoints; no execute action.
  - Truthful status: P5 measured, P6 gated, generic arms `owed`.
  - CLI handoff: copyable devnet lifecycle, keygen/fund/install/deploy/baseline commands.
- Plain labels: `Ready`, `Setup required`, `Not running`, `Draft`, `Scaffold`, `Owed`,
  `Not run`, `Gated`, `Refused`, `Not checked`.
- A persistent boundary: the UI never signs, funds or holds keys. `design.json` is the
  only file it writes.

## API and write boundary

The host half registers `/rhobuilder/api/{summary,readiness,registry,network,examples,
evidence,project-status,preflight}` plus two POST routes:
`/rhobuilder/api/design-validate` (in-memory validation) and
`/rhobuilder/api/design-save` (validated `design.json` only). Routes use the DSH
connection admission fence and return 401 without the browser cookie; they never return
secrets, keystores or environment values. The network route reuses the read-only
`chain probe`; no signing path is exposed.

## Design save containment

`design-save` validates the draft first through `rhobuilder design validate --stdin`; a
refused draft writes nothing. The target is either `designs/<slug>.json` under the
workspace (default: the installation root; override with `RHOBUILDER_WORKSPACE`) or a
relative existing project with a `.rhobuilder.json` marker. Absolute paths,
`..`, symlink escapes and non-project directories are refused. Writes are atomic
(temp file + rename), mode `0644`, and an existing file needs an explicit overwrite
confirmation. No code, scaffold, key or deploy write exists.

## Readiness

`launch_dsh.py --check --json` now separates:

- `ok` — Node, DSH, CLI, plugin, UI, registry, templates.
- `warn` — npx fetch, source build, missing mdBook.
- `setup` — API key, Docker/devnet, Lean/lake, unexported skills.

`ok` is false when any `setup` item is present and the command exits 2. The web UI still
launches so the dashboard can show the setup panel. `setupRequired` lists the blocking
names. The devnet check now shares `tools/devnet.py` and reports container state plus the
next safe command; it does not start or change anything during `--check`.

## Branding

The title/favicon/description are applied through the supported `webServer.tapIndex`
transform, not by editing DSH's minified dist. The mark is an original SVG in
`agents/deepseek/ui/assets`; the CSS uses `--dsw-alias-*` tokens with rhobuilder accents.
Do not fork `@deepseek-ai/dsh-web-frontend`.
