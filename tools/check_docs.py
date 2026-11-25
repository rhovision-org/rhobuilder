#!/usr/bin/env python3
"""Check the mdBook source: SUMMARY coverage, relative links and orphans."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
SRC = DOCS / "src"
SUMMARY = SRC / "SUMMARY.md"
LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def _targets(path: Path) -> list[Path]:
    found: list[Path] = []
    for match in LINK.finditer(path.read_text(encoding="utf-8")):
        raw = match.group(1).split("#", 1)[0].strip()
        if not raw or raw.startswith(("http://", "https://", "mailto:")):
            continue
        if not raw.endswith(".md"):
            continue
        found.append((path.parent / raw).resolve())
    return found


def check() -> dict[str, object]:
    if not SUMMARY.is_file():
        return {"ok": False, "error": f"missing {SUMMARY}"}
    listed: set[Path] = set()
    for target in _targets(SUMMARY):
        listed.add(target)
    all_docs = {path.resolve() for path in SRC.rglob("*.md") if path.name != "SUMMARY.md"}
    missing_listed = sorted(str(path.relative_to(ROOT)) for path in listed if not path.is_file())
    unlisted = sorted(str(path.relative_to(ROOT)) for path in all_docs - listed)
    broken: list[str] = []
    for path in sorted(all_docs):
        for target in _targets(path):
            if not target.is_file():
                broken.append(f"{path.relative_to(ROOT)} -> {target.relative_to(ROOT) if target.is_relative_to(ROOT) else target}")
    return {
        "schema": "rhobuilder.docs-check/v1",
        "ok": not missing_listed and not unlisted and not broken,
        "listed": len(listed),
        "documents": len(all_docs),
        "missing_listed": missing_listed,
        "unlisted": unlisted,
        "broken": sorted(broken),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    report = check()
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(
            f"docs-check: {'ok' if report.get('ok') else 'failed'} "
            f"({report.get('listed', 0)} listed, {report.get('documents', 0)} documents)"
        )
        for key in ("missing_listed", "unlisted", "broken"):
            for item in report.get(key, []):
                print(f"  {key}: {item}")
        if "error" in report:
            print(f"  error: {report['error']}")
    return 0 if report.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
