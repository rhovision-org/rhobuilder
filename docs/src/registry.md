# Registry and reuse

The registry is the builder's source of truth for what already exists. It is curated, not
discovered: rholo has no indexer, and a `rho:id` lookup proves resolution, not code
identity.

## Protected domains

| Domain | Owner |
|---|---|
| accounts, transfers, tokens, FX, stablecoins | r-cash |
| escrow, insurance, syndicates, policies, payouts | r-sure |
| knowledge claims, KAssets, provenance, identity, credentials | r-gen |
| booking, rental capacity, slot allocation | r-share |
| publishing, editions, commissions | r-print |
| cross-chain bridge/relayer | r-bridge |
| governance, external identity/DID | rgov / BYOID / FreeTrust |

`r-sure.claim` (insurance) and `r-gen.claim` (knowledge) are different. `r-share.cover` is
booking-internal, not general insurance. r-gen identity/credential is the on-chain
primitive; BYOID/FreeTrust are external references.

## Modes

- **reuse** — call the registered primitive through an adapter and resolve its component
  key per arm.
- **extend** — adapter-only, or a new-version proposal to the sibling repo. Never edit
  deployed Rholang.
- **new** — a genuinely uncovered capability, with `whyUnique`, a checked
  `noRegisteredAlternative`, and a recorded user override.

## Pins

- r-cash `49b441d…`, rholo `841cb84…`, node `e2d073d…`, toolchain 1.95.0.
- r-gen is per-consumer: `bcc840c…` in r-print, `c0ae67d…` in r-sure.
- r-share/r-print/r-bridge local HEADs are not configured pins; record
  `TBD-review-required` until a maintainer pins them.
- Drift reads `git show <rev>:<path>`, never a dirty worktree, and fails "pin unavailable"
  rather than reporting false drift.

## Networks

- Devnet: shard `/root`.
- Testnet (measured 2026-11-25): `https://testnet.rhobot.net`, network id
  `testnet-133b1d307b175012`, shard `/testnet`, genesis
  `133b1d307b1750129cf0cfb69c4c8f950da21c4e8db2a6a076464d0ccfd98d44` from
  `/api/blocks/0/0`, four validators. The docs' `c596bef4…` is stale.
- Mainnet: no live rhovision rholo mainnet; the profile is read-only and refuses deploys.

## Testnet baseline

The committed r-cash `testnet-v4.json` (`7srr…`) is partial. A newer uncommitted
`.testnet/testnet-v4.json` (`dzaz…`, account factory `d9qp…`) is a candidate, not a
verified manifest. r-gen has no committed testnet install manifest; r-print/r-sure have
none. Treat all testnet roots as `candidate/unverified` until a live resolve succeeds.

`rhobuilder baseline verify` re-resolves every recorded root; a stored manifest is
evidence, not authority. A testnet reset invalidates it.

## Read budget

Public `explore-deploy` returns HTTP 429 quickly. Identity probes use
status/capabilities/shards/blocks. Root verification is opt-in, cached and backoff-aware;
a 429 is `not_run`, not a pass.
