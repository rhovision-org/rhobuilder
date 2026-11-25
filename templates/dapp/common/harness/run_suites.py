#!/usr/bin/env python3
"""Run the scaffold devnet smoke when a healthy devnet is reachable; otherwise not_run."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NODE = os.environ.get("RHOBUILDER_NODE", "http://localhost:40403").rstrip("/")


def not_run(reason: str) -> int:
    print(f"unit-all: not_run ({reason})")
    return 2


def main() -> int:
    if os.environ.get("RHOBUILDER_DEVNET") != "1":
        return not_run("set RHOBUILDER_DEVNET=1 to opt in")
    try:
        with urllib.request.urlopen(f"{NODE}/api/v1/status", timeout=5) as response:
            status = json.load(response)
    except (urllib.error.URLError, OSError, json.JSONDecodeError) as exc:
        return not_run(f"{NODE} is unreachable: {exc}")
    degraded = status.get("degraded", "unknown")
    if degraded != "normal":
        return not_run(f"devnet is {degraded}")

    install = subprocess.run([sys.executable, str(ROOT / "harness" / "install.py")], check=False)
    if install.returncode == 2:
        return not_run("installer reported not_run")
    if install.returncode != 0:
        print("unit-all: fail (installer failed)")
        return 1
    smoke = subprocess.run([sys.executable, str(ROOT / "harness" / "unit_smoke.py")], check=False)
    result = {
        "schema": "rhobuilder.suite-results/v1",
        "suite": "scaffold_devnet_smoke",
        "status": "pass" if smoke.returncode == 0 else "fail",
        "checks": ["install", "unit_smoke"],
    }
    out = ROOT / "deployments" / "suite-results.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return smoke.returncode


if __name__ == "__main__":
    sys.exit(main())
