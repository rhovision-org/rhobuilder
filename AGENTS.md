# rhobuilder operating notes

Authority order: `crates/` + `tools/` + their tests and gates > `registry/` > `docs/` >
this file. A claim in a skill or doc that contradicts a gate or a measured chain fact is
wrong.

## What this repo is

A DSH app builder for rhovision dApps on rholo. It must not reimplement the sibling
primitives. It scaffolds the standard skeleton, points the app at existing `rho:id:`
primitives, and refuses duplicate commodity primitives.

## Hard rules

- **Do not modify sibling repos.** r-cash, r-sure, r-gen, r-share, r-print and r-bridge
  are read-only references, pinned by commit. Registry drift fails loudly if a pin moves.
- **Do not redeploy an existing primitive.** The registry is the source of truth; on
  testnet/mainnet a deploy plan must reuse what is already on chain. A duplicate is
  refused unless a recorded user override names the reason.
- **Mainnet is gated.** No rhovision mainnet exists for rholo. The profile is read-only
  and refuses deploys.
- **No secrets in the tree.** Keys stay in a keystore under the working directory, mode
  600. `.devnet/`, `.testnet/`, `.dsh-home/` and generated manifests are ignored.
- **DSH is a preview.** Pin the tested version in `agents/deepseek/harness.toml`. Keep all
  DSH configuration in `tools/launch_dsh.py`; verify with `--dump-config` when a runtime is
  available. The plugin imports nothing from `@deepseek-ai/*`; its `spawn` is trusted host
  code, so keep tools narrow and never build a shell string from model input.
- **Skills are curated.** The DSH skill catalog is model-visible. Critical rules stay
  inline in `SKILL.md`; `references/` are not auto-read, so name the files to read.
- **Honest labels.** `scaffold` < `implemented` < `tested`; `proved-model` is not a
  refinement proof. Never claim a suite ran when it did not; a harness that cannot reach a
  healthy node exits `not_run` (2), never 0.

## Rholang/rholo pitfalls to keep in the builder

One linear state cell per entity, consumed and reposted exactly once. Peeks are hints.
`select` is forbidden. Node deploy cap 64 KiB; client budget 56 KiB. Deploys are atomic;
blocks are sequential. No timers; block heights only. `*x` sends a name, `@x` sends data.
After value moves, `1/0` rolls back. `explore-deploy` has no block context. Deploy lifespan
is 50 blocks. Finality = finalized + processedWithSuccess + `(true, …)`.

## Commands

```sh
make check          # rust + plugin + launcher + docs
make dsh-check      # read-only DSH preflight
make registry-check # P1
make scaffold-e2e   # P2
make formal-check   # P4
```

## Repo hygiene

Work on `main`. Do not create branches, push, or create the GitHub repo without an
explicit request. Commit as `patrickmockridge <patrickmockridge@gmail.com>`. `rhobuilder`
is its own git repository nested inside the home checkout; do not operate on the parent.
