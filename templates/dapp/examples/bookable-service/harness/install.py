#!/usr/bin/env python3
"""Install the generated app through the shared rhobuilder deploy path."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NETWORK = os.environ.get("RHOBUILDER_NETWORK", "devnet")
BIN = os.environ.get("RHOBUILDER_BIN", "rhobuilder")
KEYS = os.environ.get("RHOBUILDER_KEYS", ".ops/devnet/keys.json")
REGISTRY = os.environ.get("RHOBUILDER_REGISTRY", "")


def not_run(reason: str) -> int:
    print(f"install: not_run ({reason})")
    return 2


def main() -> int:
    if os.environ.get("RHOBUILDER_DEVNET") != "1":
        return not_run("set RHOBUILDER_DEVNET=1 to opt in")
    if not Path(KEYS).is_file() and not (ROOT / KEYS).is_file():
        return not_run(f"missing keys file {KEYS}")
    command = [BIN, "--json"]
    if REGISTRY:
        command.extend(["--registry", REGISTRY])
    command.extend(
        [
            "deploy",
            "execute",
            "--project",
            str(ROOT),
            "--network",
            NETWORK,
            "--execute",
            "--keys",
            KEYS,
        ]
    )
    proc = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    if proc.returncode == 2:
        return not_run((proc.stdout or proc.stderr).strip()[:500])
    if proc.returncode != 0:
        print(f"install: fail ({proc.returncode})")
        print((proc.stdout or "")[-2000:])
        print((proc.stderr or "")[-1000:])
        return 1
    try:
        value = json.loads(proc.stdout)
    except json.JSONDecodeError:
        print("install: fail (non-JSON deploy result)")
        return 1
    if not value.get("ok"):
        print(f"install: fail ({value.get('error')})")
        return 1
    report = value.get("result", {}).get("report", {})
    print(
        f"install: {report.get('status')} root={report.get('rootUri')} "
        f"verified={report.get('verified')}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
