#!/usr/bin/env python3
"""Check per-arm flow obligations and forbidden Lean escapes.

`owed` is first-class: the default gate reports it and passes. `--strict` fails on it.
This is a model obligation check, never a Rholang refinement claim.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FORBIDDEN = ("sorry", "admit", "axiom", "native_decide")


def words(text: str) -> set[str]:
    out = set()
    current = []
    for ch in text:
        if ch.isalnum() or ch == "_":
            current.append(ch)
        elif current:
            out.add("".join(current))
            current = []
    if current:
        out.add("".join(current))
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--strict", action="store_true", help="fail on owed")
    args = parser.parse_args()
    violations: list[str] = []
    obligations: list[dict] = []
    flows_path = ROOT / "spec" / "flows" / "core.yaml"
    if not flows_path.is_file():
        print(f"obligations: missing {flows_path}")
        return 1
    try:
        flows = json.loads(flows_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        print(f"obligations: {flows_path} is not JSON: {exc}")
        return 1
    for component in flows.get("components", []):
        arms = component.get("arms", [])
        if not arms:
            continue
        component_id = component.get("id", "unknown")
        status = component.get("lean_status", "missing")
        theorem = component.get("lean", "")
        if status == "missing":
            violations.append(f"{component_id}: mutating arms have no lean_status")
        elif status == "owed":
            if args.strict:
                violations.append(f"{component_id}: strict mode rejects owed")
        elif not theorem:
            violations.append(f"{component_id}: status {status} has no named theorem")
        obligations.append(
            {
                "component": component_id,
                "arms": arms,
                "lean_status": status,
                "lean": theorem,
            }
        )
    lean_files = 0
    lean_root = ROOT / "spec" / "lean"
    if lean_root.is_dir():
        for path in lean_root.rglob("*.lean"):
            lean_files += 1
            found = words(path.read_text(encoding="utf-8"))
            for token in FORBIDDEN:
                if token in found:
                    violations.append(
                        f"{path.relative_to(ROOT)}: forbidden Lean token {token}"
                    )
    print(
        f"obligations: {'ok' if not violations else 'failed'} "
        f"({len(obligations)} component(s), {lean_files} lean file(s), strict={args.strict})"
    )
    for violation in violations:
        print(f"  fail: {violation}")
    return 0 if not violations else 1


if __name__ == "__main__":
    sys.exit(main())
