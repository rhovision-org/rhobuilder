# Contributing

Work on `main`; do not branch or push without the maintainer asking.

## Before a change

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test agents/deepseek/plugin/test/*.test.mjs
node --test mcp/test/*.test.mjs
python3 -m unittest discover -s tools/tests -v
python3 tools/vendor_install.py --check
make check
```

## Invariants

- The Rust CLI is the decision boundary. The DSH plugin and MCP server only validate and
  forward; they never scrape human output or sign.
- Vendored sibling terms are pinned by commit and hash. Change a plan only with a new pin
  and a regenerated `ORIGIN.json`.
- Do not modify a sibling repository. Read pinned blobs with `git show`.
- Keep `owed` first-class. Never describe a shallow Lean model as a Rholang refinement.
- No secret in a fixture, log, JSON envelope, commit or test output.
- Live testnet actions are gated. A test that would sign against testnet must use a mock
  transport or an explicit `not_run`.

## Commit identity

Use `patrickmockridge <patrickmockridge@gmail.com>`.
