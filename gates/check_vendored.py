#!/usr/bin/env python3
"""Verify the vendored RhoApp tree hash recorded in spec/lean/VENDORED.json."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEAN = ROOT / "spec" / "lean"
MANIFEST = LEAN / "VENDORED.json"


def files() -> dict[str, bytes]:
    found: dict[str, bytes] = {}
    root = LEAN / "RhoApp.lean"
    if root.is_file():
        found["spec/lean/RhoApp.lean"] = root.read_bytes()
    modules = LEAN / "RhoApp"
    if modules.is_dir():
        for path in sorted(modules.rglob("*")):
            if path.is_file():
                relative = path.relative_to(LEAN).as_posix()
                found[f"spec/lean/{relative}"] = path.read_bytes()
    return found


def tree_hash(found: dict[str, bytes]) -> str:
    hasher = hashlib.sha256()
    for path in sorted(found):
        hasher.update(path.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(found[path])
        hasher.update(b"\0")
    return hasher.hexdigest()


def main() -> int:
    if not MANIFEST.is_file():
        print("vendored-lean: fail (spec/lean/VENDORED.json is missing)")
        return 1
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    expected = manifest.get("treeSha256", "")
    found = files()
    actual = tree_hash(found)
    if not expected or expected != actual:
        print("vendored-lean: fail (tree hash mismatch)")
        print(f"  expected: {expected}")
        print(f"  actual:   {actual}")
        return 1
    print(f"vendored-lean: ok ({len(found)} files)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
