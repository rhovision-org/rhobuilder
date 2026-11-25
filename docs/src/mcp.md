# MCP server

`mcp/server.mjs` is a dependency-free, read-only/plan-only MCP server over
`rhobuilder --json`. It is optional; the native DSH plugin is the primary surface.

```sh
node mcp/server.mjs --binary /path/to/rhobuilder
node mcp/server.mjs --binary cargo run -q -p rhobuilder-cli --
```

Mount it with the DSH launcher:

```sh
python3 tools/launch_dsh.py --with-mcp headless
```

The launcher inserts an `@deepseek-ai/dsh-mcp-client` row with `serverName: rhobuilder`;
tools appear as `mcp__rhobuilder__*`.

## Tools

- `rhobuilder_registry_list`, `_get`, `_graph`, `_drift`.
- `rhobuilder_design_check`.
- `rhobuilder_scaffold_plan` (never applies).
- `rhobuilder_chain_probe` (read-only, no exploratory deploy).
- `rhobuilder_deploy_plan` (read-only app plan).
- `rhobuilder_baseline_plan`, `rhobuilder_baseline_verify` (read-only; no deep signed
  verification).
- `rhobuilder_check` (runs the project Makefile gates; no chain write).
- `rhobuilder_formal_check`.

No tool applies a scaffold, signs a deploy, writes a key or mutates the registry. Those
remain CLI-only. See [Security model](security.md) and [Architecture](architecture.md).

## Lanes

Both protocol lanes are implemented and tested:

- `2025-11-25`: `initialize`, `notifications/initialized`, `ping`, `tools/list`,
  `tools/call`.
- `2026-07-28`: handshake-free `server/discover`, per-request
  `_meta.io.modelcontextprotocol/protocolVersion`, and `resultType` on results.

stdio is newline-delimited JSON, one JSON document per line. `make mcp-test` covers both
lanes and a real bridge call.
