# Troubleshooting

Start with the read-only checks:

```sh
python3 tools/launch_dsh.py --check --json
python3 tools/devnet.py status --json
./target/debug/rhobuilder --json project status --project <dir>
```

`setup` means a prerequisite is missing, not that the command passed. `not_run` means the
check did not execute.

## Readiness and environment

| Symptom | Cause | Fix |
|---|---|---|
| `setupRequired` contains `api-key` | `DEEPSEEK_API_KEY` is unset. Chat cannot answer. | Export it or save it in DSH Settings → Models. Read-only checks still work. |
| `setupRequired` contains `devnet` | Docker/container/node is missing, stopped or unhealthy. | `python3 tools/devnet.py status --json`; then `start`/`wait` or recover the container manually. |
| `setupRequired` contains `lake` | Lean is not installed. | Install Lean/lake; formal checks report `not_run` without it. |
| `warn` for `mdbook` | Docs cannot build. | Install mdBook; it is not needed for the studio or CLI. |
| `npx` warning | DSH will be fetched on first launch. | Needs network; install DSH locally to avoid the fetch. |
| Node/DSH version mismatch | DSH 0.2.3-alpha.2 needs Node 24.22.3+; 0.2.2-rc.5 uses Node 22. | Use the matching Node, or pin DSH 0.2.2-rc.5. |

## Devnet

| Symptom | Cause | Fix |
|---|---|---|
| Node unreachable | Container stopped or endpoint wrong. | `devnet.py start`, or set `RHOBUILDER_NODE` to the remote node. |
| `degraded: critical` | Resource latch/throttle. | `devnet.py wait`; the tool never restarts automatically. Recover the container deliberately. |
| Container missing | It was never created, or the name differs. | Create it from your RNodeRust checkout; `devnet.py` will not create or reset it. |
| `onboard --execute` refuses | Container missing or node not healthy. | Fix the node; onboarding is devnet-only and stops on the first failure. |
| HTTP 429 from testnet | Public read budget. | Wait/retry; a 429 is `not_run`, never a pass. |

## Design, scaffold and tests

| Symptom | Cause | Fix |
|---|---|---|
| `design check` refuses | Duplicate commodity, missing alternatives, bad acceptance, invalid override. | Read `reuse.conflicts`/`validation.checks`; reuse the primitive or record an approved override. |
| Scaffold refused | The design gate did not pass. | Fix the design first; `--apply` never bypasses the gate. |
| `project status` shows `Owed` | Mutating arms lack a Lean obligation. | Add the model/theorem or keep `owed` honestly. |
| `project status` shows `Not run` | No recorded suite, no lake, or devnet off. | Run the relevant gate/suite; do not treat it as pass. |
| Generated suite says `not_run` | Devnet unreachable or `RHOBUILDER_DEVNET` unset. | Start the devnet and export `RHOBUILDER_DEVNET=1`; check `run_suites.py` first. |
| `formal check` fails on generated scaffold | Generic arms are `owed` and `--strict` rejects that. | Expected by design; the `bookable-service` example is the reference strict pass. |

## Keys and live gates

| Symptom | Cause | Fix |
|---|---|---|
| Exit 6 / `unsafe_key` | Key path outside the allowed location, wrong mode, or a devnet key used live. | Use a mode-0600 key under the project/installation; never paste a key into chat. |
| Exit 5 on `--execute` | Missing `--allow-live`, funded key or confirmation token. | Run the dry-run, get approval, then pass the exact token. |
| Testnet preflight has blockers | r-cash account-factory unverified, missing roots, TBD pins. | Resolve pins/candidates; do not execute a partial campaign. |
| Mainnet refused | No live rhovision mainnet. | Read-only profile only. |

## Docs and UI

| Symptom | Cause | Fix |
|---|---|---|
| `docs-check` reports unlisted page | New `.md` not in `SUMMARY.md`. | Add it under the right part. |
| `docs-check` reports broken link/anchor | Renamed heading or path. | Fix the link; anchors use heading slugs. |
| `docs-check` reports duplicate status | A second `Area | Status` table. | Keep the table only in `docs/src/index.md`. |
| HMR `EMFILE: too many open files` | Host inotify instance limit. | The web server can still serve; increase the host limit or disable/reduce watchers. |
| Studio shows `Not checked` | The CLI/design gate could not run. | Check readiness and the CLI path; this is not a design refusal. |
| Composer save 409 | `design.json` already exists. | Confirm overwrite explicitly. |
| Composer save 400 | Absolute/`..`/symlink path, or missing `.rhobuilder.json`. | Use `designs/<slug>.json` or an existing project under the workspace. |

## Docs and release

| Symptom | Cause | Fix |
|---|---|---|
| Package check rejects machine-local paths | `/home/<user>` leaked into a text file. | Use `$HOME`, `--source-root` or relative paths. |
| Package self-test fails | Missing tool/registry/template, or launcher overlay invalid. | Run the failing command from the extracted tarball; check `tools/package_release.py`. |
| `mdbook build` fails | New page not in SUMMARY or invalid markdown. | Fix `SUMMARY.md`/the page, then rerun `make docs-check`. |

See [Command reference](command-reference.md) for flags and [Security model](security.md)
for the boundaries behind these failures.
