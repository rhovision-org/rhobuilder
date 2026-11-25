# Core concepts

`rhobuilder` assumes most of a dApp is already on-chain. The author should write only the
part that is genuinely new and call the existing primitive graph for the rest.

## Commodity vs unique

- **Commodity** — accounts, payments, escrow, insurance, booking capacity, provenance,
  publishing. These already have owners: r-cash, r-sure, r-share, r-gen and r-print.
- **Unique** — the coordination, policy, selection rule or state machine that no existing
  primitive covers. This is what the design and the `new` component must justify.

The reuse gate refuses a `new` component that duplicates a protected commodity unless an
operator records an approved override. Rebuilding a commodity is the default failure mode.

## The primitive graph

The registry is a curated graph, not an indexer. A primitive is a capability owned by
another app; an edge is a call or reference between components.

```text
r-cash accounts ──payments──► app booking ──cover──► r-sure policy
       ▲                           │
       └──────── capacity ─────── r-share
                                   │
                              provenance ──► r-gen
```

A `rho:id` is an unforgeable on-chain address. A **root** is an app’s directory contract
that resolves component keys to `rho:id`s. A lookup proves resolution, not code identity;
the registry is the identity source of truth.

## Reuse, extend, new

| Tag | Meaning |
|---|---|
| `reuse` | Call a registered primitive through an adapter and resolve its component key. |
| `extend` | Adapter-only, or a new-version proposal to the sibling repository. Deployed Rholang is immutable. |
| `new` | A genuinely uncovered capability, with `whyUnique`, checked alternatives and a recorded override if the kind is protected. |

An **adapter** is a small local contract that translates the app’s needs to an existing
primitive’s interface. It never edits the sibling repository.

## Cells and state

A **linear cell** is one state resource taken and reposted exactly once. Deploy atomicity
and linearity are the reason the formal library exists. A component’s mutating **arms**
are the operations that need refusal, idempotency and conservation tests.

## Status vocabulary

A project moves through separate claims; one does not imply the next:

```text
Draft ──► Scaffold ──► Implemented ──► Tested ──► Proved model
              │              │             │
              └── Owed ──────┴── Not run ──┘
```

- **Draft** — a validated design saved by the composer; no project files implied.
- **Scaffold** — generated skeleton; the unique arms and model are still the author’s work.
- **Implemented** — contracts/adapters exist and a deployment record or passing suite
  supports the claim.
- **Tested** — recorded suites pass; a missing or throttled run is `not_run`.
- **Proved model** — a theorem about the shallow Lean model, never a Rholang refinement.
- **Owed** — a proof obligation not written yet; honest and first-class.
- **Not run** — the check did not execute. It is not a pass.
- **Refused** — the design or reuse gate rejected the input.

`verified` is reserved for a signed Root resolve or a deployment record; it is not a synonym
for a Lean theorem or a static check.

## Networks

- **Devnet** — local disposable chain; deploys allowed, keys may live in the installation.
- **Testnet** — public, live, gated; requires explicit approval, a funded key and an
  identity/plan confirmation token. The r-cash account-factory candidate is the current
  blocker.
- **Mainnet** — no live rhovision rholo mainnet; the profile is read-only and refuses
  deploys before HTTP.

## Read next

- [Workflow](workflow.md) — the end-to-end loop.
- [Glossary](glossary.md) — one-line definitions.
- [Design and reuse](design.md) — the document and gate.
- [Testing and gates](testing.md) — what each status actually means.
