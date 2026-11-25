# Install

`rhobuilder` runs as a Rust CLI, a DeepSeek Harness (DSH) web/headless profile, and a
read-only MCP server. You can use the complete release tarball or work from a source
checkout.

## Prerequisites

| Need | Version / notes |
|---|---|
| Node | 22.x for DSH 0.2.2-rc.5. DSH 0.2.3-alpha.2 needs Node 24.22.3 or later. |
| Rust | 1.95.0; the repository pins it in `rust-toolchain.toml`. |
| Python | 3.11+ for the launcher (`tomllib`), devnet tool, checkers and release packager. |
| Docker | Only for local devnet execution and onboarding. |
| Lean / `lake` | Only for formal checks; `make formal-check` reports `not_run` without it. |
| mdBook | Only to build this book. |
| git + network | The first source build fetches the pinned `rcash-sdk` git dependency. |

## Check first

```sh
python3 tools/launch_dsh.py --check --json
```

Read `setupRequired` and each `hint`. A `setup` item is not a pass. `--check` is
read-only; it does not start the devnet or change the workspace.

Typical results:

- `ok` — Node, DSH, CLI, plugin, UI, registry and templates.
- `warn` — npx/network fetch, source-build fallback, missing mdBook.
- `setup` — API key, Docker/devnet, Lean/lake, unexported skills.

See [Troubleshooting](troubleshooting.md) for each setup item.

## Source checkout

```sh
git clone https://github.com/rhovision-org/rhobuilder
cd rhobuilder
cargo build -p rhobuilder-cli
./target/debug/rhobuilder version --json
python3 tools/launch_dsh.py --check --json
```

A source build is the fastest way to work on the builder itself. The CLI defaults to
`registry/` and `templates/dapp/` in the current checkout.

## Complete release tarball

Build and self-test the package:

```sh
cargo build --release -p rhobuilder-cli
python3 tools/package_release.py --check --build --self-test --json
```

The artifact is `dist/rhobuilder-<version>-linux-<arch>.tar.gz` plus a `.sha256` file. It
contains the binary, `launch-rhobuilder.sh`, registry, templates, formal library,
DSH plugin/UI, skills, tools, MCP server, gates, fixtures and docs.

```sh
cd dist
sha256sum -c rhobuilder-0.1.0-linux-x86_64.tar.gz.sha256
tar xzf rhobuilder-0.1.0-linux-x86_64.tar.gz
cd rhobuilder-0.1.0-linux-x86_64
./bin/rhobuilder version --json
./launch-rhobuilder.sh --check --json
```

The packaged launcher is the top-level `launch-rhobuilder.sh`, not the source
`tools/launch_dsh.py`. It sets `RHOBUILDER_REGISTRY` and `RHOBUILDER_TEMPLATES` to the
package contents before starting the launcher.

## Environment variables

| Variable | Purpose |
|---|---|
| `RHOBUILDER_REGISTRY` | Registry root; default `registry/` or the package registry. |
| `RHOBUILDER_TEMPLATES` | Scaffold template root; default `templates/dapp/`. |
| `RHOBUILDER_BIN` | CLI path used by `tools/devnet.py`. |
| `RHOBUILDER_WORKSPACE` | Root for composer saves and `designs/`; default installation root. |
| `RHOBUILDER_NODE` | Devnet endpoint; default `http://localhost:40403`. |
| `RHOBUILDER_CONTAINER` | Devnet container; default `devnet-bootstrap`. |
| `RHOBUILDER_DEVNET` | Set to `1` for generated-project devnet suites. |
| `DEEPSEEK_API_KEY` | DSH chat credential; not needed for read-only checks or docs. |

## Upgrade and uninstall

- Tarball: extract the new version into a new directory and point your shell wrapper at it.
  Keep `.ops/` and any project directories; they are not part of the package.
- Source: `git pull`, rebuild, and rerun `python3 tools/package_release.py --check`.
- The launcher and CLI do not write outside the checkout/package except for the workspace
  designs, `.dsh/`, `.ops/` and the directories you explicitly pass to scaffold/deploy.

## Next

- [Quickstart](start.md) — first design, validation and scaffold plan.
- [DeepSeek Harness](dsh.md) — launcher, plugin and presets.
- [Command reference](command-reference.md) — every command and flag.
