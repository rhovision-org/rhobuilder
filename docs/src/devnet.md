# Devnet end-to-end

P5 is measured on the local `devnet-bootstrap`, not just scaffolded. The evidence is
`docs/src/evidence/p5-devnet.json`.

## What was run

1. Recovered the node non-destructively: the admin resource-resume was refused, a
   container restart cleared the in-memory latch, and the memory cap was raised to 12 GiB
   against the existing volume. Height and data were preserved.
2. Generated devnet ops keys under `.ops/devnet/` (mode 0600) and funded the deployer
   once from the in-container validator.
3. Installed the vendored, pinned graph in dependency order:
   r-cash core, r-gen, r-share safe, r-sure, r-print.
4. Verified every app root and its required Root keys through a signed read. A second run
   adopted all five with zero duplicate deploys.
5. Generated the `bookable-service` example, deployed its eight terms, and ran nine
   behavioural suites.

## Measured result

| App | Commit | Steps | Root status |
|---|---|---|---|
| r-cash (core) | `49b441d` | 15 | verified |
| r-gen | `bcc840c` | 7 | verified |
| r-share (safe) | `7114acb` | 15 | verified |
| r-sure | `0da5d04` | 17 | verified |
| r-print (safe) | `a2d91b3` | 15 | verified |
| bookable-service | local | 8 | verified |

Suites: `root`, `payments`, `booking`, `cover`, `dispute`, `refusal`, `idempotency`,
`conservation`, `concurrency` — all pass.

The r-cash core profile omits `51_pool` and `50_market`. The pinned node rejects
`51_pool` with `parse depth exceeded`; no app in the graph requires either component.

## Safe lifecycle

```sh
python3 tools/devnet.py status --json
python3 tools/devnet.py start --json
python3 tools/devnet.py wait --json
python3 tools/devnet.py onboard --json            # dry run
python3 tools/devnet.py onboard --json --execute  # devnet only
make devnet-status                               # the same read-only status
```

`status` is read-only. `start` only starts the existing `RHOBUILDER_CONTAINER`
(default `devnet-bootstrap`) and waits for `degraded == "normal"` and
`acceptsDeploys`; it never runs `docker run/create/rm`, `volume rm`, compose, reset or
restart. `onboard` is a dry run unless `--execute` is passed; execution is devnet-only
and runs keygen, funding, the baseline install and deep verify in order, stopping on the
first failure. The tool resolves the CLI from `--rhobuilder`, `RHOBUILDER_BIN`,
`bin/rhobuilder`, then `target/release/rhobuilder`.

## Commands

```sh
# read-only plan and local install
rhobuilder --json baseline plan --network devnet
rhobuilder --json baseline keygen --network devnet --out .ops/devnet/keys.json
rhobuilder --json baseline fund   --network devnet --keys .ops/devnet/keys.json --execute
rhobuilder --json baseline run    --network devnet --keys .ops/devnet/keys.json --execute
rhobuilder --json baseline verify --network devnet --deep --keys .ops/devnet/keys.json

# app
rhobuilder --json deploy plan    --project /tmp/rhobuilder-p5 --network devnet
rhobuilder --json deploy execute --project /tmp/rhobuilder-p5 --network devnet \
  --execute --keys .ops/devnet/keys.json
make -C /tmp/rhobuilder-p5 unit-all
```

The generated app harness reads `RHOBUILDER_DEVNET=1`, `RHOBUILDER_BIN`,
`RHOBUILDER_KEYS`, `RHOBUILDER_NETWORK` and `RHOBUILDER_REGISTRY`.

## Not-run is a result

`run_suites.py` checks node health first. An unreachable or `degraded != "normal"` node
prints `not_run` and exits 2; it never reports a pass. `deployments/suite-results.json`
records `pass`/`fail`/`not_run`.

## Still owed

The full sibling suite set (all 33 r-share suites) is not re-run here. The
`bookable-service` model is a shallow `proved-model` and passes `formal check --strict`;
generic scaffolds keep their arms `owed`, so strict mode fails on them by design.
