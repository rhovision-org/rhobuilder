#!/usr/bin/env bash
# Launch the packaged rhobuilder DSH studio from any working directory.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$root"
export RHOBUILDER_REGISTRY="${RHOBUILDER_REGISTRY:-$root/registry}"
export RHOBUILDER_TEMPLATES="${RHOBUILDER_TEMPLATES:-$root/templates/dapp}"
exec python3 "$root/tools/launch_dsh.py" --root "$root" --rhobuilder "$root/bin/rhobuilder" "$@"
