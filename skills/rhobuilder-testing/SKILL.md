---
name: rhobuilder-testing
description: Design and run the behavioural harness: suites, refusal and idempotency tests, conservation, concurrency, pass/fail/not-run, devnet and testnet preflight.
---

# Behavioural testing

The harness is Python under `harness/`. It installs contracts on a devnet with signed
deploys and drives arms through their real reply shapes. A static gate is not a test.

## Suite shape

```text
harness/
  deploy.py       render and submit terms
  install.py      signed installer; writes deployments/devnet.json
  actors.py       keys, funding, component resolve
  assertions.py   expect_ok / expect_refusal
  preflight.py    node identity, capabilities, manifests
  run_suites.py   runs suites; exit 0 all pass, 1 any fail, 2 any not-run
  unit_*.py       one behaviour per file
  requirements.txt
```

Every suite writes its result to `deployments/suite-results.json`. A suite that cannot
reach a healthy node is `not_run` and exits 2; it never reports pass.

## What every mutating arm needs

1. A positive path.
2. A refusal path for each guard, with the exact reason.
3. An idempotency or replay check (`opId`/nonce first).
4. A conservation assertion for every value-moving path.
5. A concurrency check where two requests can race; one consumable datum, one winner.
6. A pause/authority check where the arm is gated.

## Harness rules

- Read the node through the public HTTP API; sign through the pinned client. Do not
  reimplement signing in Python.
- Use `explore-deploy` only for context-free reads. A read that needs block height or the
  deployer is a signed deploy.
- Anchor `validAfterBlockNumber` at the tip; the deploy lifespan is 50 blocks.
- Finality: finalized + processedWithSuccess + a `(true, …)` reply.
- Never retry a value-moving deploy under a new op id unless you have proved the first had
  no effect.
- One key file per campaign; `--allow-live` for testnet; fresh testnet keys, never devnet
  reuse.
- Public testnet `explore-deploy` is rate-limited (HTTP 429). Cache reads, back off, and
  do not use exploratory deploys in a loop.

## Profiles

- `static`: build, parse, docs, spec, graph, flows. No node.
- `formal`: static + Lean build/audit/vectors.
- `full`: formal + devnet install + all suites.
- `testnet`: preflight and read-only checks by default; deploy is a separate, gated
  operator command.

## Commands

```sh
make venv
make devnet-up
make keys
make install
make unit-all
make spec-check
make formal-check
make testnet-preflight   # read-only
```

## Read

- `RShare/harness/run_suites.py`, `RShare/harness/actors.py`, `RShare/harness/assertions.py`.
- `RPrint/harness/sibling_*_driver.py` for cross-app fixtures.
- The app's `AGENTS.md` for the suite and arm rules.
