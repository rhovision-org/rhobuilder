---
name: rhobuilder-build
description: Guide a new rhovision dApp from idea to a tested, deployable plan: probe, design, reuse, scaffold, implement, prove, test, plan. Never signs a deploy.
user-invocable: true
whenToUse: The user asks to build, design, scaffold, review or plan a rhovision dApp on rholo, or asks which existing primitive to reuse.
---

# Build a rhovision dApp

Run this procedure when the user names an app idea or a `design.json`. The deterministic
work is in the `rhobuilder_*` tools; the judgement is in the skills under
`.dsh/skills/rhobuilder-*/SKILL.md`. Read the skill that matches the current step before
acting.

## Before you start

- Read `rhobuilder-dapp-design`, `rhobuilder-primitive-reuse` and
  `rhobuilder-standards`.
- Ask for the target network if it is not stated. Default is `devnet`; `testnet` needs an
  explicit `allowLive`-style confirmation at the deploy step.
- Never ask for a private key. Deploy execution is outside this procedure.

## The loop

1. **Probe.** Call `rhobuilder_chain` with `{action:"probe", network}`. Compare networkId,
   shard and node build with the registry. Refuse a mismatch. Public testnet:
   `explore-deploy` is rate-limited; do not loop it.
2. **Design.** Interview the user for the unique value, actors, capabilities, resources,
   lifecycle, value paths, failure/remedy and acceptance. Write `design.json`
   (`rhobuilder.design/v1`) and `spec/design.md`. Model the unique interaction protocol,
   not a CRUD table.
3. **Reuse.** Call `rhobuilder_registry` (`list`/`get`/`graph`) for every component. Tag it
   `reuse`, `extend` or `new`. `extend` is adapter-only or a sibling proposal; deployed
   Rholang is immutable. Call `rhobuilder_design` with `action:"report"`. A `refuse`
   verdict stops the loop; present the existing primitive and the adapter.
4. **Scaffold.** Call `rhobuilder_scaffold` with the target, design and template. It plans
   by default. Apply only after the user confirms, and only inside the workspace.
5. **Implement.** Write the Rholang contracts and adapters. Read
   `rhobuilder-rholang` first. Every mutating arm gets a refusal and idempotency test.
6. **Prove.** Read `rhobuilder-rho-calculus`, `rhobuilder-coc-types` and
   `rhobuilder-lean4`. Add the model and flow obligations. `owed` is honest; `proved-model`
   is the ceiling, never a refinement.
7. **Test.** Read `rhobuilder-testing`. Run `rhobuilder_check` with `static`, then
   `formal`, then `full` on a devnet. A not-run suite is not a pass.
8. **Plan deploy.** Call `rhobuilder_plan`. It reuses registered primitives and refuses a
   duplicate, missing dependency, identity mismatch or mainnet. Present the plan; do not
   execute it.

## Stop conditions

- The design duplicates a commodity primitive and has no approved override.
- The network identity does not match the registry.
- A required primitive has no verified root on the target network.
- The target is mainnet (no live rhovision mainnet exists).
- A gate fails or a suite is not-run. Report the verdict and the remediation; do not
  paper over it.

## Output

- `design.json`, `spec/design.md`, `reuse-report.json`.
- The scaffolded repo and its proof/test status.
- A deploy plan with a plan hash, never a signed deploy.
