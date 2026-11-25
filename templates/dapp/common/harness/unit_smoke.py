#!/usr/bin/env python3
"""Scaffold smoke: the generated files and design are present and consistent."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REQUIRED = [
    "design.json",
    "reuse-report.json",
    "contracts/src/00_root.rho",
    "spec/graph/vertices.yaml",
    "spec/flows/core.yaml",
    "audit/registers/arms.yaml",
    ".rhobuilder.json",
]


def main() -> int:
    missing = [path for path in REQUIRED if not (ROOT / path).is_file()]
    if missing:
        print("scaffold-smoke: fail")
        for path in missing:
            print(f"  missing: {path}")
        return 1
    design = json.loads((ROOT / "design.json").read_text(encoding="utf-8"))
    print(f"scaffold-smoke: ok ({design['app']['slug']})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
