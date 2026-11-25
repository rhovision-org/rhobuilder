---
name: rhobuilder-rho-calculus
description: Reason from the reflective higher-order rho-calculus: names, processes, reflection, unforgeables, locality, linearity and why capabilities beat shared state.
---

# The reflective higher-order rho-calculus

Rholang executes a reflective higher-order process calculus (Meredith & Radestock 2005).
Design decisions should be justified in the calculus, not only in code style.

## Sorts and reflection

```
Name ::= @Proc | Nil | Ground | Expr | Bundle | Unforgeable | Connective
Proc ::= *Name | Name!(Name,…) | for(Name <- Name,…){Proc} | new … in Proc | match … | Proc|Proc | !Proc
```

- `@` quotes a process into a name; `*` evaluates a name into a process. The two sorts are
  mutually recursive but not identical.
- A **name** is the unit of communication and authority. A fresh `new` name is
  unforgeable; it is the primitive capability.
- A **bundle** (`bundle+{*X}`, `bundle-{*X}`, `bundle0{*X}`) restricts read/write. Publish
  the least authority a caller needs.

## Why the house patterns follow

- **One linear cell per entity** models a single-owner resource. Taking the cell and
  reposting it once is the calculus-level linear handoff; a peek is only an observation.
- **Capabilities passed by name** are unforgeable names. Storing a resolved capability in
  a state map or a bundle widens authority and breaks revocation.
- **No `select`** because the node desugars it into parallel receives; it does not express
  committed choice. Model exclusivity with one consumable datum.
- **Atomic deploys, sequential blocks** mean a multi-step arm is a trace, not one comm
  event. State a trace explicitly; do not claim a single reduction.
- **Locality**: a process should only reach names it was given. A global lookup or a
  mutable shared counter is an escape from locality; prefer a root-resolved capability or
  a per-entity cell.

## Invariants to state in a design

- Single owner: at most one live holder of a cell.
- All-or-none: a commit writes every requested item or none.
- Authority: only a holder of the named capability can invoke the arm.
- Freshness: a new name is never derived from public data.
- Linear use: a received token is consumed or reposted, never duplicated.
- Progress: every accepted request has a reply path, and every refusal reposts state.

## Read

- `rholo-fixed/spec/RHO-CALCULUS.md` (the core grammar, sorts and structural rules).
- `rholo-fixed/spec/TYPE-SYSTEM.md` (the CoC reading; pair with `rhobuilder-coc-types`).
- `rholo-fixed/docs/src/rholang/{processes-names,unforgeable-names,models-of-computation}.md`.
