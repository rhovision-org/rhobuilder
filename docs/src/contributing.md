# Contributing

The root [CONTRIBUTING.md](https://github.com/rhovision-org/rhobuilder/blob/main/CONTRIBUTING.md)
is canonical. This page adds the documentation workflow.

## Development checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test agents/deepseek/plugin/test/*.test.mjs
node --test mcp/test/*.test.mjs
python3 -m unittest discover -s tools/tests -v
make check
```

`make docs-check` builds the book and runs the docs/README checker. `make scaffold-e2e`
exercises a generated project; the package self-test is
`python3 tools/package_release.py --check --build --self-test --json`.

## Invariants

- The Rust CLI is the decision boundary. The DSH plugin and MCP server validate and forward;
  they never scrape human output or sign.
- Vendored sibling terms are pinned by commit and hash. Change a plan only with a new pin
  and a regenerated `ORIGIN.json`.
- Do not modify a sibling repository; read pinned blobs with `git show`.
- Keep `owed` first-class. Never describe a shallow Lean model as a Rholang refinement.
- No secret in a fixture, log, JSON envelope, commit or test output.
- Live testnet actions are gated. Tests that would sign against testnet use a mock
  transport or an explicit `not_run`.

## Documentation rules

- Every page under `docs/src/` must be listed in `SUMMARY.md`.
- The full status table lives only in `docs/src/index.md`; README and other pages summarize
  and link.
- `command-reference.md` owns flags and exit codes; tutorials show task sequences and
  expected results.
- Use relative `.md` links. The checker validates paths and heading anchors.
- Keep claims aligned with evidence: P5 measured, P6 live-gated, `owed`/`not_run` real.
- Prefer short pages and cross-links over duplicated prose.

```sh
make docs-check
python3 tools/check_docs.py --json
```

## Release workflow

1. Run `make check`.
2. Build and self-test the tarball:

   ```sh
   cargo build --release -p rhobuilder-cli
   python3 tools/package_release.py --check --build --self-test --json
   ```

3. Update `CHANGELOG.md` and docs if behavior changed.
4. A release tag is separate from ordinary changes; no `v0.1.0` tag exists until an
   approved live baseline records a verified manifest.

## Commit identity and review

Use `patrickmockridge <patrickmockridge@gmail.com>` for commits. Work on `main` unless the
maintainer asks for a branch. Do not push or tag without an explicit request.
