# Design and reuse

A design is a `design.json` (`rhobuilder.design/v1`) plus the `reuse-report.json` the gate
emits. The design states the unique value, actors, resources, component tags, graph,
conservation, concurrency and acceptance tests. The gate refuses a `new` commodity
primitive before any code is written.

The studio composer can build and validate the same document in memory and save it for
continuity; see [Studio tour](ui.md). The CLI remains the authority.

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
  "acceptance": [{ "id": "a1", "given": "…", "when": "…", "then": "…" }]
}
```

Tags:

- `reuse` — call a registered primitive through an adapter.
- `extend` — adapter-only, or a new-version proposal to the sibling repo. Deployed Rholang
  is immutable; this never edits a sibling.
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
printf '%s' "$(cat design.json)" | rhobuilder --json design validate --stdin
```

The report names every conflict (`component`, existing `primitive`, remediation) and lists
`owed` components. A core-to-extension graph edge must be `optional`; the core never calls
an app. A reuse target with no verified root on a target network is a warning, not a pass:
the deploy plan refuses it later.

The fixture set is the executable statement of the rule:

- `fixtures/designs/booking.json` — passes; reuses payments and booking, extends cover,
  adds one custom panel selector.
- `fixtures/designs/duplicate-escrow.json` — refused; names r-sure and other escrow owners.
- `fixtures/designs/duplicate-escrow-override.json` — passes with an override warning.

## Flags and exit codes

`design check` refuses with exit 2; a `--stdin` read is bounded to 1 MiB. See
[Command reference](command-reference.md) for the full surface and JSON envelope.

## Read next

- [Registry and reuse](registry.md) — the catalog and pins.
- [Scaffolding](scaffold.md) — what happens after the gate passes.
- [Core concepts](concepts.md) — reuse/extend/new without the schema.
