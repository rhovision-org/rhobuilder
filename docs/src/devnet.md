# Devnet smoke

A generated app ships a deployment smoke, not a behavioural suite. It deploys
`contracts/src/*.rho` through the devnet's own funded validator key with `rnode deploy
--private-key-path` inside the container, then asserts each deploy is
`processedWithSuccess`.

The key is never read, printed or copied to the host. This is a dev-only path: the devnet
root owns the key.

```sh
RHOBUILDER_DEVNET=1 make unit-all
```

Environment:

| Variable | Default |
|---|---|
| `RHOBUILDER_NODE` | `http://localhost:40403` |
| `RHOBUILDER_CONTAINER` | `devnet-bootstrap` |
| `RHOBUILDER_SHARD` | `/root` |
| `RHOBUILDER_KEY_PATH` | `/var/lib/rnode/validator.hex` |
| `RHOBUILDER_DEPLOY_TIMEOUT_S` | `120` |

## Not-run is a result

`run_suites.py` checks `/api/v1/status` first. If the node is unreachable, or
`degraded != "normal"`, it prints `not_run` and exits 2; it never reports a pass. The
installer does the same. `deployments/suite-results.json` records `pass`/`fail`/`not_run`.

## Current status

On 2026-11-25 the local `devnet-bootstrap` was running but critically throttled
(`degraded: critical`, proposals withheld under checkpoint resource pressure). The harness
correctly reported `not_run`; the devnet was not reset, because that would discard its
data. A fresh healthy devnet is required to execute the smoke.

## What is still owed

Refusal, idempotency, conservation and concurrency suites, and a real per-app model. The
scaffold smoke only proves that the generated terms deploy.
