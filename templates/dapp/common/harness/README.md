# Harness

Scaffold. The behavioural suites are not wired yet; `make unit-all` returns `not_run`
(exit 2) unless a devnet is reachable and `RHOBUILDER_DEVNET=1` is set.

The eventual shape is the sibling harness: a signed installer, fixtures, refusal and
idempotency suites, conservation and concurrency checks, and `deployments/suite-results.json`
with `pass`/`fail`/`not_run`. A suite that cannot reach a healthy node is never a pass.

```sh
RHOBUILDER_DEVNET=1 make unit-all
```
