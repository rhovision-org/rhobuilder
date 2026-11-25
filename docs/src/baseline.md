# Public-testnet baseline

The public testnet does not have a complete, committed baseline of the shared primitives.
`rhobuilder` can plan and verify; it deliberately refuses live execution in this build.

```sh
rhobuilder --json baseline plan   --network testnet
rhobuilder --json baseline verify --network testnet
rhobuilder --json baseline run    --network testnet   # exit 2: LiveExecutionGated
```

## Plan

`baseline plan` probes identity first (networkId, shard, genesis, node build, faucet) and
refuses a mismatch. It then reads `registry/networks/testnet.json` and emits one step per
shared app:

| App | Status | Action |
|---|---|---|
| r-cash | candidate | verify-and-adopt the candidate root; the committed `7srr…` manifest is partial |
| r-gen | missing | deploy |
| r-sure | missing | deploy |
| r-print | missing | deploy (optional) |
| r-share | missing | deploy (optional) |

The plan is read-only. It names the prerequisites: an approved campaign, a funded ops key
under the working directory, a faucet allocation, `--allow-live` plus a confirmation token
carrying the observed networkId, backoff-aware root verification (a 429 is `not_run`), and
pinned sibling installers or vendored terms.

## Verify

`baseline verify` runs the same identity check and audits the catalog's network claims:
`candidate` entries need a root; no entry may claim `lookup` without a root and URI. It
does not prove code identity.

## Why run is gated

A live deploy moves value and writes an externally visible manifest. It needs explicit
approval, a funded key and a fresh identity observation. This build returns
`LiveExecutionGated` rather than performing it. No testnet deploy has been executed by
this repository.

## Read budget

Public `explore-deploy` returns HTTP 429 quickly. The default plan uses status,
capabilities, shards and blocks only; root verification is opt-in, cached and
backoff-aware.
