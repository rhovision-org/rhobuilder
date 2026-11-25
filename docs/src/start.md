# Start here

This is the non-technical path. You do not need to read the registry schema or write
Rholang to begin. You do need a working machine with Node and, for devnet execution,
Docker/Rust/Lean support.

## 1. Check the machine

```sh
python3 tools/launch_dsh.py --check --json
```

Read `setupRequired` and the `hint` fields. The dashboard shows the same checks. A
`setup` item is not a pass. On this machine the usual items are:

- `DEEPSEEK_API_KEY` — needed for chat; save it in DSH Settings → Models.
- devnet — Docker plus a healthy local node; otherwise the dashboard shows `not_run`.
- `lake` — needed for formal Lean checks.
- `mdbook` — needed only to build docs.

## 2. Launch the studio

```sh
python3 tools/launch_dsh.py --with-mcp --port 3081 web
```

Open the printed token URL. The sidebar has **Rhobuilder** for readiness, networks,
primitives, examples and CLI handoff.

## 3. Start a design

Use a starter, or ask:

> Interview me and create a design.json for a bookable service.

The assistant designs only the unique part, reuses r-cash/r-sure/r-gen/r-share/r-print by
`rho:id`, and runs the reuse gate. A `refuse` verdict stops the loop.

## 4. Scaffold, prove, test

The UI can plan a scaffold and send an explicit apply prompt after you approve the file
count. `make check` is static; `make formal-check` proves the shallow Lean model; devnet
behavioural suites require a healthy devnet and otherwise report `not_run`.

## 5. Sign and deploy

Signing, funding, deploy execute, testnet baseline run and deep verify are deliberately
CLI-only. Use the copyable commands in the dashboard. Mainnet is refused.

## What is real

- P5 devnet: five primitive roots and a generated app are measured and verified.
- P6 testnet: plan/dry-run/executor are implemented; live execution is gated and has not
  run. The r-cash account-factory candidate is the shared blocker.
- Generated scaffolds are not finished apps. `owed` is a real status; `proved-model` is a
  shallow model, never a Rholang refinement.
