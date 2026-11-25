#!/usr/bin/env python3
"""Keep the canonical gates and the scaffold-vendored copies identical.

`gates/` is the source. `templates/dapp/common/tools/` is the vendored copy. GATES.json
records a SHA-256 per file. `--check` is read-only and is run in CI and by `make check`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "gates"
TARGET = ROOT / "templates" / "dapp" / "common" / "tools"
FILES = [
    "spec_check.py",
    "parse_check.sh",
    "check_vendored.py",
    "check_obligations.py",
    "check_gates.py",
]
SCHEMA = "rhobuilder.gates/v1"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_commit() -> str:
    try:
        proc = subprocess.run(
            ["git", "-C", str(ROOT), "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=False,
        )
        if proc.returncode == 0:
            return proc.stdout.strip()
    except OSError:
        pass
    return "working-tree"


def manifest() -> dict:
    return {
        "schema": SCHEMA,
        "source": "gates",
        "sourceCommit": source_commit(),
        "files": {name: digest(SOURCE / name) for name in FILES},
    }


def write() -> int:
    TARGET.mkdir(parents=True, exist_ok=True)
    for name in FILES:
        shutil.copy2(SOURCE / name, TARGET / name)
    data = manifest()
    text = json.dumps(data, indent=2) + "\n"
    (SOURCE / "GATES.json").write_text(text, encoding="utf-8")
    (TARGET / "GATES.json").write_text(text, encoding="utf-8")
    print(f"gate-sync: wrote {len(FILES)} file(s)")
    return 0


def check() -> int:
    errors: list[str] = []
    source_manifest = SOURCE / "GATES.json"
    target_manifest = TARGET / "GATES.json"
    if not source_manifest.is_file() or not target_manifest.is_file():
        print("gate-sync: missing GATES.json")
        return 1
    try:
        source = json.loads(source_manifest.read_text(encoding="utf-8"))
        target = json.loads(target_manifest.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        print(f"gate-sync: invalid GATES.json: {exc}")
        return 1
    if source.get("schema") != SCHEMA:
        errors.append("gates/GATES.json has the wrong schema")
    if source.get("files") != target.get("files"):
        errors.append("gates and templates GATES.json differ")
    for name in FILES:
        expected = source.get("files", {}).get(name)
        if expected is None:
            errors.append(f"{name}: not in GATES.json")
            continue
        for base in (SOURCE, TARGET):
            path = base / name
            if not path.is_file():
                errors.append(f"{name}: missing in {base.relative_to(ROOT)}")
                continue
            actual = digest(path)
            if actual != expected:
                errors.append(
                    f"{name}: {base.relative_to(ROOT)} hash {actual[:12]} != {expected[:12]}"
                )
    if errors:
        print("gate-sync: failed")
        for error in errors:
            print(f"  fail: {error}")
        return 1
    print(f"gate-sync: ok ({len(FILES)} file(s))")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--write", action="store_true", help="copy gates into templates")
    parser.add_argument("--check", action="store_true", help="verify both trees")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    if not args.write and not args.check:
        parser.error("pass --write or --check")
    code = write() if args.write else check()
    if args.json:
        print(json.dumps({"ok": code == 0, "files": FILES}, indent=2))
    return code


if __name__ == "__main__":
    sys.exit(main())
