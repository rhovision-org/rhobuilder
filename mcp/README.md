# rhobuilder MCP server

Read-only/plan-only MCP server over `rhobuilder --json`. Dependency-free stdio.

```sh
node mcp/server.mjs --binary /path/to/rhobuilder
node mcp/server.mjs --binary cargo run -q -p rhobuilder-cli --
```

Mount it through `@deepseek-ai/dsh-mcp-client` with `--with-mcp`, or any MCP client. Tools
appear as `mcp__rhobuilder__*` in DSH.

Lanes supported:

- `2025-11-25`: `initialize`, `notifications/initialized`, `ping`, `tools/list`,
  `tools/call`.
- `2026-07-28`: handshake-free `server/discover`, per-request
  `_meta.io.modelcontextprotocol/protocolVersion`, `resultType` on results.

No tool writes a deploy or touches a key. Signed deploy execution is CLI-only.
