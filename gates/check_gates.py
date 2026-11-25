#!/usr/bin/env python3
"""Verify the vendored gate files against GATES.json."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
MANIFEST = ROOT / "GATES.json"


def main() -> int:
    if not MANIFEST.is_file():
        print(f"gates: missing {MANIFEST}")
        return 1
    try:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        print(f"gates: invalid GATES.json: {exc}")
        return 1
    errors = []
    files = manifest.get("files", {})
    for name, expected in sorted(files.items()):
        path = ROOT / name
        if not path.is_file():
            errors.append(f"missing {name}")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            errors.append(f"{name}: hash {actual[:12]} != {expected[:12]}")
    if errors:
        print("gates: failed")
        for error in errors:
            print(f"  fail: {error}")
        return 1
    print(f"gates: ok ({len(files)} file(s), source {manifest.get('sourceCommit', '?')})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
