# Evidence

The machine-readable P5 record is [`evidence/p5-devnet.json`](evidence/p5-devnet.json).

It contains the devnet identity, the five pinned roots, the generated app root, and the
nine suite results. It contains no keys.

## Reading it

- `baseline` names each app, its pinned commit, root URI, status and step count.
- `app` names the generated `bookable-service` root and status.
- `suites.results` is the same `pass`/`fail`/`not_run` record written to
  `deployments/suite-results.json`.
- `notes` records the r-cash core-profile decision and the zero-duplicate adopt run.

Evidence is not authority. The live check is `baseline verify --deep`, which re-resolves
every root at a finalized block.

## What is deliberately absent

- No public-testnet execution record, because no live campaign has been approved.
- No key material.
- No claim of Rholang refinement; per-app Lean obligations remain `owed`.
