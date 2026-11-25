# Harness

Scaffold. The devnet smoke deploys `contracts/src/*.rho` through the devnet's funded
validator key with `rnode deploy --private-key-path` inside the container, then asserts
each deploy is `processedWithSuccess`. The key is never read, printed or copied to the
host.

`make unit-all` returns `not_run` (exit 2) unless a healthy devnet is reachable and
`RHOBUILDER_DEVNET=1` is set:

```sh
RHOBUILDER_DEVNET=1 make unit-all
```

Environment: `RHOBUILDER_NODE` (default `http://localhost:40403`),
`RHOBUILDER_CONTAINER` (default `devnet-bootstrap`), `RHOBUILDER_SHARD` (default `/root`),
`RHOBUILDER_KEY_PATH` (default `/var/lib/rnode/validator.hex`).

This is a deployment smoke, not a behavioural suite. Refusal, idempotency, conservation and
concurrency suites are the app author's next step; `deployments/suite-results.json` records
`pass`/`fail`/`not_run`, and a suite that cannot reach a healthy node is never a pass.
