# Design and the reuse gate

A design is a `design.json` (`rhobuilder.design/v1`) plus the `reuse-report.json` the gate
emits. The design states the unique value, actors, resources, component tags, graph,
conservation, concurrency and acceptance tests. The gate refuses a `new` commodity
primitive before any code is written.

## The document

```json
{
  "schema": "rhobuilder.design/v1",
  "app": { "name": "…", "slug": "…", "summary": "…", "networks": ["devnet", "testnet"] },
  "uniqueValue": "…",
  "actors": [{ "id": "guest", "capabilities": ["book"] }],
  "resources": [{ "id": "dispute", "owner": "app", "stateCell": "disputeState",
                  "lifecycle": ["open", "panel", "resolved"] }],
  "components": [
    { "id": "payments", "tag": "reuse", "primitive": "rcash.account-factory" },
    { "id": "cover", "tag": "extend", "primitive": "rsure.policy",
      "adapter": "cover-adapter.rho", "mode": "adapter-only", "whyExtend": "…" },
    { "id": "panel", "tag": "new", "kind": "custom", "whyUnique": "…",
      "noRegisteredAlternative": ["rsure.claim"], "arms": ["select", "verdict"] }
  ],
  "graph": { "vertices": [], "edges": [] },
  "conservation": [{ "path": "booking → account", "assertion": "…" }],
  "concurrency": [{ "risk": "…", "mechanism": "…" }],
  "remedies": [],
  "acceptance": [{ "id": "a1", "given": "…", "when": "…", "then": "…" }]
}
```

Tags:

- `reuse` — call a registered primitive through an adapter.
- `extend` — adapter-only, or a `new-version-proposal` to the sibling repo. Deployed
  Rholang is immutable; this never edits a sibling.
- `new` — a genuinely uncovered capability. It needs `whyUnique`, a non-empty
  `noRegisteredAlternative`, arms, and — if it duplicates a protected kind — an approved
  override with `reason`, `approvedBy`, `approvedAt` and `scope`.

Protected kinds: accounts, payments, escrow, insurance-claim, knowledge-claim, provenance,
identity, booking, publication, bridge. The r-sure claim and r-gen claim are different
kinds.

## The gate

```sh
rhobuilder --json design check --path fixtures/designs/booking.json
rhobuilder --json design check --path fixtures/designs/duplicate-escrow.json  # exits 2
```

The report names every conflict (`component`, existing `primitive`, remediation). A
core-to-extension graph edge must be `optional`; the core never calls an app. A reuse target
with no verified root on a target network is a warning, not a pass: the deploy plan will
refuse it later.

The fixture set is the executable statement of the rule:

- `fixtures/designs/booking.json` — passes; reuses payments and booking, extends cover,
  adds one custom panel selector.
- `fixtures/designs/duplicate-escrow.json` — refused; names r-sure (and other escrow
  primitives) as the existing owners.
- `fixtures/designs/duplicate-escrow-override.json` — passes with a recorded override
  warning.

## Exit codes

`0` pass, `2` refusal or invalid design, `3` missing pin/dependency, `4` node unavailable.
The gate uses the committed registry; run `make registry-check` and `make registry-drift`
first when the catalog changes.
