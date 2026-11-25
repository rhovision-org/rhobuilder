# Security model

`rhobuilder` treats signing, funding and key material as the sensitive boundary. Planning,
validation, reading and evidence collection are safe by default; live execution is
explicit and gated.

Report vulnerabilities privately to the maintainer in `CITATION.cff` or through a GitHub
security advisory. Do not include key material, funded addresses or live deploy bodies in a
public issue. The root [SECURITY.md](https://github.com/rhovision-org/rhobuilder/blob/main/SECURITY.md)
is canonical.

## Threat model

The assets are signing keys, funded accounts and the integrity of the primitive graph.
The main risks are:

- a model or UI tricked into signing/funding;
- a duplicate commodity primitive creating a second root;
- a key copied, printed or committed;
- a mainnet/testnet deploy executed without approval;
- a sibling repository or vendored term tampered with;
- a test or evidence record misread as a live verification.

## Boundaries

| Surface | May do | Must not do |
|---|---|---|
| CLI | Validate, plan, keygen, fund/execute under gates, read chain, write manifests | Print secrets; deploy mainnet; skip confirmation on live networks |
| Studio UI | Read local data, validate a draft, write one validated `design.json` | Sign, fund, hold keys, apply a scaffold, execute a deploy |
| DSH plugin | Seven CLI-backed read/plan tools; scaffold apply only with `confirm` | Expose a signed deploy or `chain call` |
| MCP server | Read/plan subset | Apply, sign, write keys or mutate the registry |
| Executor | Adopt or deploy pinned terms in dependency order | Edit a sibling repo; redeploy a duplicate primitive |

## Keys

- Keystores stay under the working directory, mode 0600, gitignored and untracked.
- No command prints, logs or serialises a secret; `baseline keygen` prints only public
  addresses.
- Live key loads are contained under the working directory. Devnet-named keys are refused
  on live networks; the published devnet deployer secret is refused everywhere.
- Never paste a key into chat or a design file.

## Network gates

- **Mainnet** is unavailable and every deploy path refuses it before HTTP.
- **Testnet** needs explicit campaign approval, `--allow-live`, a funded key and a
  `--confirm` token binding `network`, `networkId`, `genesis` and the plan digest.
- **Devnet** onboarding is CLI-only and explicit (`devnet.py onboard --execute`); the tool
  never creates, resets or removes the container or volume.
- The executor adopts a resolvable root and never deploys a second copy of a primitive. A
  partial install stops; the write-ahead op cache is preserved.

## Studio write boundary

The composer validates in memory through `design validate --stdin`. A successful save
writes only `design.json` under `designs/` or an existing project with a
`.rhobuilder.json` marker. Absolute paths, `..`, symlink escapes and non-project
directories are refused; writes are atomic and overwrite needs confirmation. The host
never returns secrets, keystores or environment values. See [Studio tour](ui.md).

## Integrity

- Sibling repos are read with `git show` only; rhobuilder never edits them.
- Vendored terms and `RhoApp` are pinned and hash-checked.
- `GATES.json` is a drift record, not a signature; a deliberate edit is still possible.
- Evidence is a record, not authority; `baseline verify --deep` re-resolves roots.

## Read next

- [Public testnet](baseline.md) — approval checkpoints and blockers.
- [Architecture](architecture.md) — trust boundaries.
- [Troubleshooting](troubleshooting.md) — key and live-gate errors.
