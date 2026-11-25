---
name: rhobuilder-deploy
description: Plan and execute a deploy: signing, shard and valid-after anchoring, finality ladder, manifests, network identity, faucet, key handling and no-retry rules.
---

# Deploying to rholo

Deployment is a state transition with value at risk. Plan first; sign only in the CLI.

## Signing and submission

- Sign `DeployData` with secp256k1 over blake2b256 of the protobuf bytes:
  `term=2`, `timestamp=3`, `phloPrice=7`, `phloLimit=8`, `validAfterBlockNumber=10`,
  `shardId=11`, `attachments=12`.
- Anchor `validAfterBlockNumber` at the tip. The deploy lifespan is 50 blocks.
- `POST /api/deploy`; the node returns a deploy signature string.
- Poll `GET /api/v1/deploy-status/{sig}`, then `GET /api/is-finalized/{blockHash}`, then
  read finalized state.
- Success requires `ProcessedWithSuccess` **and** finalized **and** a `(true, …)` reply.
  A processed deploy is not success.

## Network identity

Probe `/api/v1/status`, `/api/v1/capabilities`, `/api/v1/shards` and
`/api/last-finalized-block` before any plan. Compare networkId, shard and node build with
the registry's manifest; refuse a mismatch. The full genesis is not exposed by status, so
take it from a verified config/genesis artifact and record the basis.

Testnet measured 2026-11-25: networkId `testnet-133b1d307b175012`, shard `/testnet`, four
validators, `proposeOnDeploy:true`, faucet live. The docs' `c596bef4…` is stale.

## Planning rules

- Reuse registered primitives by `rho:id:`. Deploy only the app's new roots/components.
- Refuse a duplicate commodity primitive, a missing dependency, an identity mismatch, a
  mainnet profile, or an unverified root.
- Manifest: `root_uri`, components, install order, profile, network, block/tx. Generated,
  never hand-edited.
- `deploy plan` is read-only and produces a plan hash. `deploy apply` is CLI-only, requires
  `--allow-live`, the plan hash and an explicit confirmation.
- Write an op cache before submitting; on an ambiguous outcome, reuse the same signed
  deploy/op id. Never create a new op id for a value-moving retry.

## Keys

- Keystore under the working directory, mode 600; never print, log or commit the key.
- Testnet keys are fresh; do not reuse a devnet key.
- No key in a DSH/MCP tool argument, output or error.

## Public testnet limits

`explore-deploy` is rate-limited (HTTP 429). Default probes avoid it; root verification is
opt-in, cached and backoff-aware. A 429 is `not_run`, not a pass.

## Mainnet

There is no live rhovision rholo mainnet. The profile exists, is read-only, and refuses
every deploy before reading a key. Do not claim otherwise.

## Read

- `rholo-fixed/docs/src/developer/building-apps.md`.
- `RWallet/r-wallet/src/api/sign.ts` (signer reference).
- Use the pinned `rcash-chain` crate; do not copy its signing or protobuf code.
