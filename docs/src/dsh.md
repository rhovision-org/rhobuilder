# DeepSeek Harness

DSH is MIT, a 0.2 developer preview, Node/Cordis based. `rhobuilder` targets the 0.2 line
and keeps every DSH-specific value in `agents/deepseek/harness.toml` and
`tools/launch_dsh.py`.

## What the launcher does

1. exports the curated skill catalog to `.dsh/skills/<name>/SKILL.md`;
2. writes a mode-specific overlay to `.dsh/patches/rhobuilder-<mode>.yml`;
3. starts the shipped `web` or `headless` profile with `--patch`.

```sh
python3 tools/launch_dsh.py --check --json
python3 tools/launch_dsh.py --print-config headless
python3 tools/launch_dsh.py web
python3 tools/launch_dsh.py headless "Design a bookable service"
```

`--check` and `--print-config` are read-only. A launch writes the skill export and the
overlay, then runs DSH.

## The studio UI

The web launch mounts `agents/deepseek/ui` by absolute path through the launcher patch.
It replaces the stock brand, sets the page title/favicon/description with the supported
`webServer.tapIndex` transform, adds a rhobuilder welcome hero and a **Rhobuilder**
dashboard page. The dashboard reads only local registry/evidence files and fixed read-only
CLI commands; every route uses the DSH browser-token fence. See [Rhobuilder studio
UI](ui.md).

The launcher applies a rhobuilder persona by default (`rhobuilder` preset). Agent Teams and
the architect/reviewer/prover subagents remain opt-in with `--study build`.

## The plugin

`agents/deepseek/plugin/index.js` registers six tools over `rhobuilder --json`:

- `rhobuilder_registry` — list/get/graph/verify/drift primitives.
- `rhobuilder_design` — validate or report a design.
- `rhobuilder_scaffold` — plan (default) or apply a skeleton.
- `rhobuilder_check` — static/formal/full project gates via its Makefile.
- `rhobuilder_plan` — read-only deploy plan.
- `rhobuilder_chain` — probe/status/deploy-status, read-only.

The bridge maps every tool to a real CLI subcommand. `registry verify` is read-only by
construction; the obsolete `--read-only` flag is gone. `chain explore` is not exposed:
the node's exploratory deploy cannot see the native registry, so the CLI uses a signed
`chain call` only for local suites, never for DSH.

The plugin imports nothing from `@deepseek-ai/*`; it mounts by absolute path with no
`node_modules`. Its `spawn` is trusted host code, so the surface is narrow and never
exposes a signed deploy.

## Skills and presets

The catalog is capped at 12 entries including the `/rhobuilder-build` procedure. The
procedure orchestrates probe, design, reuse, scaffold, implement, prove, test and plan.
Critical platform rules stay inline in each `SKILL.md`; DSH does not auto-read
`references/`.

`--study build` enables Agent Teams and mounts the architect (root), reviewer and prover
presets. Web and headless compose presets differently: web overwrites
`agent-preset-registry`; headless inserts it plus the presets.

## Versions and permissions

Tested on DSH `0.2.2-rc.5` with Node 22.22.2. `0.2.3-alpha.2` needs Node 24.22.3+.
A live model session has not been run because `DEEPSEEK_API_KEY` is unset; `--check`
reports that honestly. The overlay and plugin contract are validated.
The launcher sets `DSH_PERMISSION_MODE=workspace-write`, a repo-local `DSH_HOME`, and never
uses `danger-full-access` or `sdk-minimal`.

## Distribution

There is no DSH plugin marketplace. Install by cloning the repo and launching from it, or
mount the dependency-free plugin by absolute path in your own overlay. A Claude Code
`SKILL.md` export is secondary portability only.
