# Design

The canonical design is `design.json` (`rhobuilder.design/v1`). The reuse report is
`reuse-report.json`. This chapter is the prose pointer, not a second source of truth.

- Unique value: __unique_value__
- Target networks: see `design.json` and `configs/`.
- Components: `reuse` calls a registered primitive through an adapter; `extend` is
  adapter-only or a sibling proposal; `new` is the unique capability and must name the
  alternatives it checked.
- Graph: `spec/graph/vertices.yaml` and `spec/graph/edges.yaml`.
- Arms and Lean obligations: `spec/flows/core.yaml`; every mutating arm starts `owed`.

A core-to-extension edge must be optional: the core never calls the app. Deployed Rholang
is immutable, so an extension never edits a sibling repository.

## Changing the design

Edit `design.json`, rerun `rhobuilder design check --path design.json`, and update the
prose here only if the unique value or scope changed. Do not add a commodity primitive
without a recorded override.
