# Public-testnet baseline

The all-five baseline is code-complete and dry-run green. **No signed public-testnet
deploy, faucet spend or consolidation has been executed.** Live execution requires a
separate explicit campaign approval and a funded key.

```sh
rhobuilder --json baseline plan   --network testnet
rhobuilder --json baseline run    --network testnet --dry-run
rhobuilder --json baseline run    --network testnet --execute --keys <path>   # exit 5 without approval
rhobuilder --json baseline verify --network testnet --deep --keys <path> \
  --allow-live --confirm 'network=testnet,networkId=…,genesis=…'
```

## What is implemented

- Vendored, pinned terms for r-cash, r-gen, r-share, r-sure and r-print under
  `registry/install/<app>/<commit>/`; no sibling repo is edited.
- A shared `rcash-sdk` executor with a write-ahead op cache, finality wait, root capture
  and signed Root verification.
- `baseline keygen|fund|plan|run|verify`; `deploy plan|execute`; and a local-only
  `chain call` for the app suites.
- Adopt-before-deploy: a root is skipped only if every required key resolves at a
  finalized block. A partial root stops the campaign; the op cache is preserved.
- Exit codes: 5 for a missing live gate/plan digest, 6 for an unsafe key, 4 for node
  unavailability, 3 for a missing dependency, 2 for identity/config.

## Dry-run gates

`--execute` without `--allow-live`, a funded key and the identity/plan confirmation token
exits 5. The token binds `network`, `networkId`, `genesis` and the plan digest. The
dry-run performs no key load, no signing and no POST.

## Testnet blocker

All five apps require r-cash `component:account-factory`. The committed testnet manifest
leaves `account_factory_uri` null and the candidate roots (`rho:id:7srr…`, `rho:id:dzaz…`,
account factory `rho:id:d9qp…`) are unverified. The executor must adopt only after a
signed Root resolve proves every required key. A partial candidate stops the campaign;
automated partial completion is not implemented, because the root refuses duplicate
registrations and a wrong skip would deploy a duplicate component. Completing a partial
root requires manual reconciliation and an explicit operator override. A full r-cash
deploy is the fallback when no candidate verifies; a second r-cash root is never created.

## Pins and notes

- r-cash `49b441d`, r-gen `bcc840c`, r-sure `0da5d04`.
- r-share `7114acb` and r-print `a2d91b3` are local HEADs recorded as
  `TBD-review-required`; live use needs a maintainer pin or explicit approval.
- r-sure's config names r-gen `c0ae67d`; the baseline uses `bcc840c`. Prove the adapter
  on devnet before a live campaign; never deploy two r-gen roots.
- The faucet is one drip per address (0.3 REV) and rate-limited. `baseline fund --plan`
  must show the arithmetic; a 429 is `not_run`.
- A stored manifest is evidence, not authority. `baseline verify --deep` re-resolves
  every root; on testnet it signs Root reads, so it needs `--allow-live` and the identity
  confirmation token just like execute. A testnet reset invalidates prior roots.

## Why live remains gated

A live deploy auto-proposes a block on this devnet-mode testnet, spends REV/phlo and writes
an externally visible manifest. That is irreversible in practice; Rholang has no undo. The
campaign is a separate approval, not a side effect of `make check`.
