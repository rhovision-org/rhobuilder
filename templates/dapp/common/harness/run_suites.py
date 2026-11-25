#!/usr/bin/env python3
"""Run the behavioural suites when a devnet is reachable; otherwise not_run."""

from __future__ import annotations

import os
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NODE = os.environ.get("RHOBUILDER_NODE", "http://localhost:40403")


def not_run(reason: str) -> int:
    print(f"unit-all: not_run ({reason})")
    return 2


def main() -> int:
    if os.environ.get("RHOBUILDER_DEVNET") != "1":
        return not_run("set RHOBUILDER_DEVNET=1 to opt in")
    try:
        with urllib.request.urlopen(f"{NODE}/api/v1/status", timeout=5) as response:
            if response.status != 200:
                return not_run(f"{NODE}/api/v1/status returned {response.status}")
    except (urllib.error.URLError, OSError) as exc:
        return not_run(f"{NODE} is unreachable: {exc}")
    smoke = ROOT / "harness" / "unit_smoke.py"
    if not smoke.is_file():
        return not_run("harness/unit_smoke.py is missing")
    return subprocess.run([sys.executable, str(smoke)], check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
