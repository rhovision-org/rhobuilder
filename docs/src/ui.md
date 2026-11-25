# Studio tour

The DSH web launch is branded **rhobuilder — rholo dApp studio**. Chat remains the working
surface; the studio adds identity, onboarding, readiness, status and safe shortcuts.

## Launch

```sh
python3 tools/launch_dsh.py --with-mcp --port 3081 --no-open web
```

Open the printed token URL. `--no-ui` falls back to the stock shell; `--port` avoids a
collision; `--no-open` is useful on a remote/headless machine. The launcher mounts
`agents/deepseek/ui/` by absolute path, disables the shipped brand row and supplies the
rhobuilder persona by default. See [DeepSeek Harness](dsh.md) for the overlay mechanics.

## What the user sees

- Rhobuilder title, favicon, sidebar wordmark and a welcome hero with four starters.
- A **Rhobuilder** dashboard:
  - **Readiness** — required/setup/warn checks with install hints.
  - **Networks** — devnet local, testnet gated, mainnet refused.
  - **Primitive graph** — 16 primitives, 5 cross-app edges and owner counts.
  - **Design composer** — build a `rhobuilder.design/v1` draft, validate it in memory, then
    save it as `design.json` for continuity. Blank project: `designs/<slug>.json`; existing
    project: its root, with overwrite confirmation. This is the only UI write.
  - **Examples** — bookable-service, payments-ledger, duplicate-escrow. “Plan scaffold
    (no writes)” returns the file count; applying is a separate, explicitly confirmed chat
    action.
  - **Project status** — `Scaffold`, `Implemented`, `Tested`, `Proved model`, `Owed`,
    `Not run`, `Refused`, plus `Draft` for a saved workspace design. These come from project
    files and recorded evidence, not a fresh `make`.
  - **Measured evidence** — P5 records with suite statuses and a raw-JSON disclosure.
  - **Testnet campaign** — read-only blockers and approval checkpoints; no execute action.
  - **Truthful status** — P5 measured, P6 gated, generic arms `owed`.
  - **CLI handoff** — copyable devnet lifecycle, keygen/fund/install/deploy/baseline
    commands.
- Plain labels: `Ready`, `Setup required`, `Not running`, `Draft`, `Scaffold`, `Owed`,
  `Not run`, `Gated`, `Refused`, `Not checked`.

## Save containment

The composer validates through `rhobuilder design validate --stdin`; a refused draft writes
nothing. The target is either `designs/<slug>.json` under the workspace (default installation
root; override with `RHOBUILDER_WORKSPACE`) or a relative existing project with a
`.rhobuilder.json` marker. Absolute paths, `..`, symlink escapes and non-project directories
are refused. Writes are atomic, mode `0644`, and an existing file needs explicit overwrite
confirmation. No code, scaffold, key or deploy write exists.

## Safety boundary

The UI never signs, funds or holds keys. Signed operations stay in the terminal with
explicit gates. Custom routes use DSH’s connection admission fence and return 401 without
the browser cookie. See [Security model](security.md) and
[Architecture](architecture.md).

## Branding

The title/favicon/description are applied through the supported `webServer.tapIndex`
transform, not by editing DSH’s minified dist. The mark is an original SVG in
`agents/deepseek/ui/assets`; the CSS uses `--dsw-alias-*` tokens with rhobuilder accents.
Do not fork `@deepseek-ai/dsh-web-frontend`.
