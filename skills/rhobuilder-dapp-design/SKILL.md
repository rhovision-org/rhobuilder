---
name: rhobuilder-dapp-design
description: Turn an idea into a design.json: unique value, actors, capabilities, resources, lifecycle, conservation, concurrency, remedies and acceptance tests.
---

# Designing the unique part

Rholang is expressive in concurrency and capabilities, not in CRUD. The design should
spend that expressiveness on the part no existing primitive covers.

## Interview, in order

1. **Unique value.** What does this app do that no r-cash/r-sure/r-gen/r-share/r-print/
   r-bridge primitive does? Write one sentence.
2. **Actors and authority.** Who acts? What unforgeable capability does each hold? What
   can each never do?
3. **Resources and cells.** What is the unit of state? One linear cell per entity. Name
   its lifecycle and terminal states.
4. **Interaction protocol.** What messages, in what order, with what reply? Which steps
   must be atomic, which are sequential across blocks, and where can two actors race?
5. **Value.** Which paths move value? State the conservation assertion for each.
6. **Failure and remedy.** What can fail, who can reclaim, what is permissionless, what is
   lazy (block height, no timers)?
7. **Commodity map.** For each component, choose `reuse`, `extend` or `new`. `extend` means
   adapter-only (a local contract calls a registered primitive) or a new-version proposal
   to the sibling repo; deployed Rholang is immutable. `new` needs a reason and a checked
   list of alternatives.
8. **Acceptance.** Given/when/then for each arm, including refusals and replays.

## Artifacts

- `design.json` — machine-readable, schema `rhobuilder.design/v1`.
- `spec/design.md` — the prose version for reviewers.
- `reuse-report.json` — produced by `rhobuilder design check`.
- `spec/graph/{vertices,edges}.yaml` — the integration graph.
- `spec/flows/core.yaml` — arm, guard, theorem and status.

## Rules

- No shared mutable counter, no mutable availability cache, no timers.
- No new payments, escrow, claim, provenance, booking or bridge primitive when a
  registered one exists; use an adapter or a recorded override.
- The core never calls an extension. Edges into an extension are optional and explicit.
- Every mutating arm has a refusal test and a Lean theorem or `owed`.
- Every value-moving path has a conservation assertion.
- Write down what is **not** in scope; an empty domain must be labelled, not hidden.

## Output

`rhobuilder design check --path design.json` must pass before scaffolding. The report is
the design's verdict; do not scaffold on a `refuse`.
