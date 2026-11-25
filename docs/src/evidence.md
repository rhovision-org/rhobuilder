# Evidence

The machine-readable P5 records are:

- [`evidence/p5-devnet.json`](evidence/p5-devnet.json) — the five pinned roots, the
  hand-authored `bookable-service` root, and nine suite results.
- [`evidence/p5-generic-ledger.json`](evidence/p5-generic-ledger.json) — the generic
  `payments-ledger` root, its four generated contracts, and the static/smoke checks.

They contain no keys. The studio renders both records, including suite statuses, with a
raw-JSON disclosure. `evidence/testnet-campaign.template.json` is a template, not a
record; the UI excludes `.template.json` files from the measured-evidence feed.

## Reading it

- `baseline` names each app, its pinned commit, root URI, status and step count.
- `app` names the generated app root and status.
- `suites.results` is the same `pass`/`fail`/`not_run` record written to
  `deployments/suite-results.json`.
- `notes` records the r-cash core-profile decision and the zero-duplicate adopt run.
- The generic ledger record also lists `.rhobuilder-contracts.json` and notes that its
  design arms remain `owed`; it has no behavioural suite.

Evidence is not authority. The live check is `baseline verify --deep`, which re-resolves
every root at a finalized block.

## What is deliberately absent

- No public-testnet execution record, because no live campaign has been approved.
- No key material.
- No claim of Rholang refinement; per-app Lean obligations remain `owed`.
