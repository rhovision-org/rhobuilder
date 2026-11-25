# Architecture

`rhobuilder` is a Rust decision core with several thin front ends. The front ends validate
and forward; they do not reimplement the gates or sign.

```text
┌───────────────────────────── DSH / editor / agent ─────────────────────────────┐
│  studio UI (client.js + host index.js)   native plugin (bridge.js)   MCP server │
└───────────────┬──────────────────────────────────┬──────────────────┬──────────┘
                │ /rhobuilder/api/*                │ tools            │ MCP tools
                └──────────────────┬───────────────┴──────────────────┘
                                   ▼
                     rhobuilder CLI (`--json` envelopes)
        ┌──────────────┬──────────────┬──────────────┬──────────────┐
        │ core/design  │ registry     │ formal/Lean  │ chain/executor│
        │ scaffold     │ reuse gate   │ gates        │ node client   │
        └──────────────┴──────────────┴──────────────┴──────────────┘
                                   │
             registry/ templates/ gates/ formal/ fixtures/ evidence/
                                   │
                         rholo devnet / gated testnet
```

## Components

| Path | Responsibility |
|---|---|
| `crates/rhobuilder-cli` | Stable JSON command surface, exit codes and live gates. |
| `crates/rhobuilder-core` | Design model/validation, scaffold, formal checks, project status and contracts. |
| `crates/rhobuilder-registry` | Curated primitive graph, pins, drift and reuse gate. |
| `crates/rhobuilder-chain` | Identity probes, executor, funding, Root verification and keystore. |
| `crates/rhobuilder-formal` / `gates` | Lean vendoring/obligations and canonical gate orchestration. |
| `agents/deepseek/` | DSH plugin, studio UI, harness config, presets and the build procedure. |
| `mcp/` | Read-only/plan MCP server over the CLI. |
| `tools/` | Launcher, devnet lifecycle, packaging, skill export and checkers. |
| `registry/`, `templates/`, `formal/`, `gates/`, `fixtures/` | Data and generated-project sources. |
| `docs/` | This book and the measured P5 evidence. |

## Data flow

```text
idea → design.json → design/reuse gate → scaffold
     → project gates + Lean + devnet suites
     → deploy plan → CLI-only signed execute → Root/manifest evidence
```

The studio’s composer calls `design validate --stdin`; a validated save writes only
`design.json`. The dashboard reads registry/evidence files and fixed read-only CLI
commands. The plugin exposes seven CLI-backed tools. MCP exposes a read-only/plan subset.

## Trust boundaries

- The CLI owns every decision: validation, identity, plan digests, key checks and signing.
- The DSH plugin and MCP server can plan/read; they cannot apply a scaffold, sign, fund or
  write a key.
- The studio host can serve local files behind DSH’s browser-token fence and write one
  contained `design.json`; it never returns secrets.
- The executor never edits a sibling repository; it consumes vendored, pinned terms.
- A stored manifest is evidence, not authority; `baseline verify --deep` re-resolves roots.

See [Security model](security.md), [Testing and gates](testing.md) and
[Command reference](command-reference.md).
