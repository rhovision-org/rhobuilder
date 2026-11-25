---
name: rhobuilder-rholang
description: Write or review Rholang contracts: names/processes, linear state cells, factories, registry lookup, bundles, the 64/56 KiB budgets and the platform pitfalls.
---

# Rholang for rhovision dApps

Use this when writing, reviewing or debugging a `.rho` term. The platform rules are not
style preferences; each one has a failure mode on rholo.

## The two sorts

- A **name** is usable in channel position: a quoted process `@P`, a ground value, a
  bundle, an unforgeable, or a quoted process.
- A **process** has a top-level send, receive, `new`, `match` or replication.
- `*x` passes a name (a capability, purse, facet, token); `@x` passes data. Passing a
  capability with `@` destroys the authority; passing data with `*` usually makes the
  pattern fail silently.

## The house pattern: one linear cell per entity

A factory creates one contract with one state cell. A mutating arm:

1. takes the cell: `for (@state <- stateCh)`;
2. validates and computes the new state;
3. reposts exactly once: `stateCh!(newState)`;
4. replies `(true, …)` on the caller's channel.

A refusal reposts the unchanged cell and replies `(false, reason)`. Never take without
reposting; never repost twice. A read uses a peek `for (@state <<- stateCh)`, which is a
hint, not a lock. `select` is desugared into parallel receives by the node and is never
committed choice; do not use it for exclusivity.

## Registration and lookup

- Register with `rho:registry:insertArbitrary` and a `bundle+{*Contract}`.
- Resolve with `rho:registry:lookup`; a Root contract exposes
  `Root!("resolve", key, minVersion, ret)` (3-arg) for `component:<name>` keys. r-gen's
  2-arg `GenRoot` is the exception; new apps use the 3-arg shape.
- Resolve sibling capabilities **per arm**, through a `lib/Resolve.rho` body. Never store a
  resolved capability in a bundle or a state map.

## Hard limits and failure modes

| Rule | Failure if broken |
|---|---|
| Deploy term cap 64 KiB inclusive; keep under 56 KiB | The node rejects the term |
| Parser depth limit 128 | Deep nesting fails to parse |
| A deploy is atomic; blocks are sequential | Do not describe multi-step work as one comm event |
| `rho:block:data` timestamp is the proposer's clock | Deadlines must use block heights |
| `explore-deploy` has no block context | A read that needs height must be a signed deploy |
| Deploy lifespan is 50 blocks | Anchor `validAfterBlockNumber` at the tip |
| Unmatched `for` is silent | A wrong arity is a deploy that never replies |
| After value moves, an unexpected branch must `1/0` | A graceful stop would commit a half-state |
| Finality is finalized + processedWithSuccess + `(true, …)` | A processed deploy is not success |

## Review checklist

- Every mutating arm has a refusal test and an idempotency test.
- Every value-moving path has a conservation assertion.
- Every capability is passed by name and resolved at the arm that uses it.
- No `select`, no shared mutable counter, no mutable availability cache, no timers.
- The term is under budget after rendering and inlining.
- The arm is named in `spec/flows/core.yaml` and, if it mutates, in `spec/graph/`.

## Read before changing a contract

- `rholo-fixed/spec/RHO-CALCULUS.md` (grammar and sorts).
- `rholo-fixed/spec/TYPE-SYSTEM.md` (the type discipline; read it with the CoC skill).
- `RShare/AGENTS.md` and `RPrint/AGENTS.md` (platform pitfalls and sibling surfaces).
