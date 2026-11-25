# DeepSeek Harness package

The DSH side of rhobuilder: a dependency-free plugin, the curated skill catalog, three
role presets and a launcher that renders a mode-specific overlay for the shipped `web` or
`headless` profile.

## Run

```sh
python3 tools/launch_dsh.py --check --json
python3 tools/launch_dsh.py --print-config headless
python3 tools/launch_dsh.py web
python3 tools/launch_dsh.py headless "Design a bookable service"
```

`--check` and `--print-config` are read-only. A launch exports skills to `.dsh/skills/`,
writes `.dsh/patches/rhobuilder-<mode>.yml`, and starts DSH with `--patch`.

## Plugin

`plugin/index.js` imports nothing from `@deepseek-ai/*`; DSH mounts it by absolute path.
It registers six tools over `rhobuilder --json`:

- `rhobuilder_registry` — list/get/graph/verify/drift.
- `rhobuilder_design` — validate/report a design.
- `rhobuilder_scaffold` — plan (default) or apply.
- `rhobuilder_check` — static/formal/full gates.
- `rhobuilder_plan` — read-only deploy plan.
- `rhobuilder_chain` — probe/status/deploy-status.

`plugin/bridge.js` is pure Node: argv, stdin/stdout JSON, timeout, abort and a typed error
envelope. The plugin's `spawn` is trusted host code; the surface stays narrow and never
exposes signed deploys.

## Skills and presets

`skills.toml` is the catalog source; the launcher exports `.dsh/skills/<name>/SKILL.md`.
The catalog is capped at 12 entries including the `/rhobuilder-build` procedure. Critical
rules stay inline in `SKILL.md`; DSH does not auto-read `references/`.

`--study build` enables Agent Teams and the architect (root), reviewer and prover presets.
Web overwrites `agent-preset-registry`; headless inserts it.

## Versions and permissions

Tested on DSH `0.2.2-rc.5` with Node 22. `0.2.3-alpha.2` needs Node 24.22.3+. The launcher
sets `DSH_PERMISSION_MODE=workspace-write`, a repo-local `DSH_HOME`, and never uses
`danger-full-access` or `sdk-minimal`.

## MCP

`--with-mcp` mounts `mcp/server.mjs` through `@deepseek-ai/dsh-mcp-client`. It is
read-only/plan-only and supports both the 2025-11-25 and 2026-07-28 lanes. The native
plugin is the primary surface.

There is no DSH plugin marketplace; install by cloning the repo and launching from it, or
mount the plugin by absolute path in your own overlay.
