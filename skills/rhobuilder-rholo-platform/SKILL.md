---
name: rhobuilder-rholo-platform
description: Operate rholo: networks, genesis/system contracts, deploy and finality ladder, capabilities, faucet, registry lookup and the no-indexer limit.
---

# The rholo platform

## Networks

| Network | Endpoint | Shard | Identity | Deploy |
|---|---|---|---|---|
| devnet | `http://localhost:40403` | `/root` | local genesis | allowed |
| testnet | `https://testnet.rhobot.net` | `/testnet` | network id `testnet-133b1d307b175012` (measured 2026-11-25) | gated, `--allow-live` |
| mainnet | none for rhovision rholo | — | — | refused |

The testnet is an idle chain: four validators at 250 each, `proposeOnDeploy:true`,
`autopropose:false`, faucet live. It is not production and its chain can be reset. The
documented `c596bef4…` genesis is stale; the live network id encodes the genesis prefix and
matches r-gen's config `133b1d307b…98d44`. Probe before every campaign and refuse a
mismatch. `/api/v1/status` does not expose the full genesis.

## Genesis and system surfaces

- Hardcodable genesis URIs: `rholo-fixed/spec/GENESIS.md` — ListOps, NonNegativeNumber,
  MakeMint, ERTP, and the rgov set. They are stable across chains of this port, not the
  Scala mainnet values in legacy comments.
- Native channels: `rho:rchain:revVault` (`getBalance`, `transfer`, `findOrCreate`),
  `rho:rchain:pos`, `rho:rchain:ertp:ledger`.
- System reply shapes are normative in `rholo-fixed/spec/API-SCHEMA.md`.

## Read and write flow

- Reads: `POST /api/explore-deploy` (or `rnode eval`). It has no block context and no
  deployer; anything needing height or authority is a signed deploy.
- Writes: sign `DeployData` with secp256k1 over blake2b256 of the protobuf bytes
  (`term=2`, `timestamp=3`, `phloPrice=7`, `phloLimit=8`, `validAfterBlockNumber=10`,
  `shardId=11`, `attachments=12`), then `POST /api/deploy`.
- Ladder: submit → `GET /api/v1/deploy-status/{sig}` → `GET /api/is-finalized/{hash}` →
  read finalized state. Success is `ProcessedWithSuccess` **and** finalized **and** a
  `(true, …)` reply.
- Capabilities: `GET /api/v1/status`, `/api/v1/capabilities`, `/api/v1/shards`,
  `/api/last-finalized-block`. Use them to choose faucet/propose behaviour; do not assume
  an admin API on a public network.

## Registry reality

There is no indexer and no enumeration. A primitive is found by a curated manifest plus
`rho:registry:lookup`. A lookup proves the URI resolves; it does not prove the code behind
it. Record `verified: lookup|manifest|missing|mismatch`, never "verified" without a basis.

## Public-testnet read budget

The public testnet rate-limits `explore-deploy` (HTTP 429 after a few calls). Default
identity probes use only `/api/v1/status`, `/api/v1/capabilities`, `/api/v1/shards` and
`/api/last-finalized-block`. Root verification by exploratory deploy is opt-in, cached and
backoff-aware; a 429 is `not_run`, not a pass.

## Deploy prerequisites

- A funded key: the testnet faucet pays 0.3 REV (`30,000,000` drops) per request.
- The shard id and a `validAfterBlockNumber` anchored at the tip.
- Phlo price/limit from the network's capability view; a public node may not expose admin
  propose, so rely on `proposeOnDeploy`.
- A deploy that is accepted can still fail phlo if the key is unfunded. Read the status.

## Read before changing platform assumptions

- `rholo-fixed/docs/src/developer/building-apps.md`.
- `rholo-fixed/docs/src/node/testnet.md` (treat the genesis constant as stale until probed).
- `rholo-fixed/spec/GENESIS.md` and `spec/API-SCHEMA.md`.
