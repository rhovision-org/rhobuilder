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
  - Examples: bookable-service, payments-ledger, duplicate-escrow.
  - Truthful status: P5 measured, P6 gated, generic arms `owed`.
  - CLI handoff: copyable keygen/fund/install/deploy/baseline commands.
- Plain labels: `Ready`, `Setup required`, `Not running`, `Scaffold`, `Owed`, `Not run`,
  `Gated`, `Refused`.
- A persistent boundary: the UI never signs, funds or holds keys.

## Read-only API

The host half registers `/rhobuilder/api/{summary,readiness,registry,network,examples,
evidence}`. Routes use the DSH connection admission fence and return 401 without the
browser cookie; they never return secrets, keystores or environment values. The network
route reuses the read-only `chain probe`; no signing path is exposed.

## Readiness

`launch_dsh.py --check --json` now separates:

- `ok` — Node, DSH, CLI, plugin, UI, registry, templates.
- `warn` — npx fetch, source build, missing mdBook.
- `setup` — API key, Docker/devnet, Lean/lake, unexported skills.

`ok` is false when any `setup` item is present and the command exits 2. The web UI still
launches so the dashboard can show the setup panel. `setupRequired` lists the blocking
names.

## Branding

The title/favicon/description are applied through the supported `webServer.tapIndex`
transform, not by editing DSH's minified dist. The mark is an original SVG in
`agents/deepseek/ui/assets`; the CSS uses `--dsw-alias-*` tokens with rhobuilder accents.
Do not fork `@deepseek-ai/dsh-web-frontend`.
