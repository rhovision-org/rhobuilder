# Security

`rhobuilder` treats signing and keys as the sensitive boundary.

## Reporting

Report a vulnerability privately to the maintainer listed in `CITATION.cff` or open a
GitHub security advisory. Do not include key material, funded addresses or live deploy
bodies in a public issue.

## Operational rules

- Keystores stay under the working directory, mode 0600, gitignored and untracked.
- No command prints, logs or serialises a secret. `baseline keygen` prints only public
  addresses.
- `mainnet` is unavailable and every deploy path refuses it.
- Public-testnet execution requires an explicit campaign approval, `--allow-live`, a
  funded key and a confirmation token binding the observed identity and plan digest.
- The executor adopts a resolvable root and never deploys a second copy of a primitive.
  A partial install stops; the write-ahead op cache is preserved.
- `chain call` is a signed local/devnet surface used by suites. It is never exposed to the
  DSH plugin or MCP server.
- Sibling repos are read with `git show` only; rhobuilder never edits them.

## DSH plugin

The DSH plugin's `spawn` is trusted host code. The plugin surface is read-only/plan; it
does not expose signed execution. A model can request a scaffold apply, which writes only
inside the configured target.
