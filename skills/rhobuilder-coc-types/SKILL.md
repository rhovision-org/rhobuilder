---
name: rhobuilder-coc-types
description: Use the rho-to-Calculus-of-Constructions reading correctly: sorts, closedness, linear patterns, totality, and what CoC does and does not buy Rholang.
---

# The rho → Calculus of Constructions reading

State the relationship precisely:

- Rholang **executes** the reflective higher-order rho-calculus. It is not a dependently
  typed language.
- The rholo spec embeds that calculus as the base sort of a Calculus of Constructions
  (Lean's CIC) to state and prove properties **about** the calculus and the node. The CoC
  is the meta-logic, not a type system the deployed contract enjoys.
- A `proved-model` result is a theorem about the shallow Lean model. It is not a refinement
  of the deployed Rholang and not a proof that the chain executes it that way.

## What the spec actually gives

- `PSort` has exactly two sorts: `proc` and `name`. `classify : Par → PSort` is structural
  and decidable; `HasVarSort` classifies de Bruijn variables against `Ctx := List PSort`.
- `Closed p` (no free variables), `WellScoped Γ t` (bound levels in scope) and `linear`
  (a pattern binds each free variable at most once) are the well-formedness refinements.
- `TotalOn f := ∀ p, Closed p → Closed (f p)` is the totality shape: a partial operation is
  either total on the refinement or returns `Option`/`Except` at a declared boundary.
- The flat `Par` ADT erases `@`/`*`; the reflective core is recovered in the type layer, not
  by extra constructors.

## How to use it

1. In design, name the sort of each thing: a capability is a name; an arm is a process.
2. In Rholang, enforce closedness and linearity by construction: no free names, one cell,
   one repost.
3. In Lean, prove the shallow model's invariants; state the node's atomicity, single-use
   COMM and unforgeability as explicit assumptions or as the rholo laws you rely on.
4. In docs, say **proved-model**, **proved-tied** or **owed**. Never say "verified" or
   "type-safe" about deployed Rholang unless a named refinement exists.

## Read

- `rholo-fixed/spec/TYPE-SYSTEM.md` — the full type discipline.
- `rholo-fixed/spec/Rchain/Ty.lean` — `PSort`, `classify`, `HasVarSort`, `HasSort`.
- `rholo-fixed/spec/Rchain/Match.lean` — the `linear` predicate.
- `rholo-fixed/docs/src/formal/closedness-coc.md` — the prose companion.
