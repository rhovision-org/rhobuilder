# FAQ

## Is rhobuilder production-ready?

The builder and devnet path are usable and measured. The public-testnet campaign is
code-complete but live-gated, and there is no live rhovision mainnet. Do not treat it as a
production deployment service.

## Has the public testnet campaign run?

No signed public-testnet deploy, faucet spend or consolidation has run. The r-cash
`component:account-factory` candidate is unverified and r-share/r-print pins are
`TBD-review-required`. See [Public testnet](baseline.md).

## What do P5 and P6 mean?

P5 is the measured local-devnet graph: five primitive roots, a generated app root and nine
behavioural suites. P6 is the public-testnet baseline: plan, dry-run, executor, keys and
Root verification implemented, with live execution gated.

## What is the difference between `owed`, `not_run` and `proved-model`?

- `owed` — a proof obligation not written yet.
- `not_run` — the check did not execute; it is not a pass.
- `proved-model` — a theorem about the shallow Lean model, never a refinement of deployed
  Rholang.

See [Glossary](glossary.md) and [Testing and gates](testing.md).

## Can the studio sign, fund or hold keys?

No. The only UI write is a validated `design.json`. Keygen, funding, signing, deploy
execute, baseline execute and deep verification are CLI-only with explicit gates. The
studio shows status and copyable commands.

## Do I need DeepSeek Harness or an API key?

DSH is the chat/agent shell. The CLI, checks, docs and devnet tool work without an API key.
Chat needs `DEEPSEEK_API_KEY`; readiness reports it as `setup` when absent.

## Should I use the tarball or the source checkout?

Use the tarball for a self-contained binary plus registry/templates/UI/tools. Use the source
checkout to work on the builder itself or run the full test suite. See [Install](install.md).

## Why was my design refused?

Most often it introduces a `new` component that duplicates a protected commodity primitive
(accounts, payments, escrow, insurance, booking, provenance, publishing, bridge). Reuse the
existing primitive through an adapter, or record an explicit approved override with a
reason. See [Design and reuse](design.md).

## Can I deploy to mainnet?

No. The mainnet profile exists but refuses deploys before HTTP; there is no live rhovision
rholo mainnet.

## Does the UI need a browser token?

Yes. The web launch prints a token URL. Custom routes use DSH’s connection admission fence
and return 401 without the browser cookie. Loopback access only unless explicitly
configured.

## Where do I start?

- Non-technical author: [Quickstart](start.md) → [Studio tour](ui.md).
- Developer: [Workflow](workflow.md) → [Command reference](command-reference.md).
- Operator: [Install](install.md) → [Local devnet](devnet.md) → [Public testnet](baseline.md).
- Maintainer: [Architecture](architecture.md) → [Contributing](contributing.md).
