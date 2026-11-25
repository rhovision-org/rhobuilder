# Glossary

Short definitions. See [Core concepts](concepts.md) for the narrative and
[Testing and gates](testing.md) for operational meaning.

| Term | Meaning |
|---|---|
| **adapter** | A local contract that maps an app’s interface to an existing primitive without editing the sibling repo. |
| **arm** | A mutating operation on a component; each arm needs refusal, idempotency and (where value moves) conservation evidence. |
| **chain call** | A signed term submitted by the CLI for local app suites. Never exposed through DSH or MCP. |
| **devnet** | Disposable local rholo chain used for installs and behavioural suites. |
| **DSH** | DeepSeek Harness, the Node/Cordis agent runtime that hosts the studio, plugin and skills. |
| **gated** | A live action that requires explicit approval, identity/plan confirmation and a funded key. |
| **genesis** | The chain’s first block; its hash is part of network identity. |
| **linear cell** | One state resource taken and reposted exactly once; the basis of the formal model. |
| **mainnet** | No live rhovision rholo mainnet; profile refuses deploys. |
| **`not_run`** | A check that did not execute because a prerequisite was missing or the node was unhealthy. Not a pass. |
| **`owed`** | A proof obligation not written yet; accepted by default, rejected by `formal check --strict`. |
| **phlo** | The compute resource paid for a deploy. |
| **primitive** | A reusable on-chain capability owned by another app, such as r-cash accounts or r-sure escrow. |
| **`proved-model`** | A theorem about the shallow Lean model, never a refinement of deployed Rholang. |
| **`proved-conformance`** | A stronger, still bounded claim requiring a theorem, corpus vector and suite assertion. |
| **registry** | The curated primitive/edge catalog and pins; not an on-chain indexer. |
| **REV** | The native value unit used by r-cash and the faucet. |
| **`rho:id`** | An unforgeable on-chain address for a contract or component. |
| **root** | An app’s directory contract that resolves component keys to `rho:id`s. |
| **rholo** | The blockchain these dApps run on. |
| **scaffold** | A generated project skeleton; not a finished app. |
| **testnet** | The public gated rholo testnet at `https://testnet.rhobot.net`. |
| **verified** | A signed Root resolve or a verified deployment record; not a Lean or static-check result. |
| **witness / evidence** | A machine-readable record of what actually ran. Evidence is not authority. |

## Statuses at a glance

`Draft`, `Scaffold`, `Implemented`, `Tested`, `Proved model`, `Owed`, `Not run`, `Refused`,
`Gated`, `Verified`. Only the first six describe project maturity; the rest describe a
refusal, a gate or a missing run.
