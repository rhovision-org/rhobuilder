# Changelog

## Unreleased

- Add the vendored all-five install plans and `rcash-sdk` executor with write-ahead op
  cache, finality wait, root capture and signed Root verification.
- Add `baseline keygen|fund|plan|run|verify`, `deploy plan|execute`, `chain call`,
  `chain status` and `chain deploy-status`.
- Recover the devnet non-destructively and record a full-graph P5 pass: five verified
  roots, a verified generated app, and nine passing behavioural suites.
- Add the `bookable-service` worked example with real dispute/panel, r-cash, r-share and
  r-sure adapters.
- Add the formal per-arm obligation gate and `formal check --strict`.
- Add canonical `gates/` checkers with `GATES.json` hashes, `tools/gate_sync.py`, and
  generated-project `check_gates.py`; make `rhobuilder-gates` the real check orchestrator.
- Align the DSH plugin with the real CLI: `check`, `deploy plan`, `chain status`,
  `chain deploy-status`; remove the obsolete `--read-only` and `chain explore` bridge.
- Add the manual release workflow, issue/PR templates, and Cargo package metadata.
- Update docs/status to the measured P5 and gated P6 state.

## 0.1.0

- Initial P0 scaffold: registry, design/reuse gate, deterministic scaffold, vendored
  `RhoApp`, DSH plugin/launcher, MCP server, docs and CI.
