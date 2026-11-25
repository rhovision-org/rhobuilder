#!/usr/bin/env python3
"""Check the generated skeleton: design, term budgets, JSON artifacts and docs."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
INSTALL_PLACEHOLDERS = {
    "__DISPUTE_URI__",
    "__PAYMENTS_URI__",
    "__BOOKING_ADAPTER_URI__",
    "__COVER_ADAPTER_URI__",
    "__RCASH_ROOT_URI__",
    "__RSHARE_ROOT_URI__",
    "__RSURE_ROOT_URI__",
}
PLACEHOLDER = re.compile(r"__[A-Z][A-Z0-9_]*__")


def fail(failures: list[str], message: str) -> None:
    failures.append(message)


def check_docs(failures: list[str]) -> int:
    src = ROOT / "docs" / "src"
    summary = src / "SUMMARY.md"
    if not summary.is_file():
        fail(failures, "docs/src/SUMMARY.md is missing")
        return 0
    listed: set[Path] = set()
    for match in LINK.finditer(summary.read_text(encoding="utf-8")):
        target = match.group(1).split("#", 1)[0].strip()
        if target.endswith(".md"):
            listed.add((summary.parent / target).resolve())
    docs = {path.resolve() for path in src.rglob("*.md") if path.name != "SUMMARY.md"}
    for path in sorted(listed):
        if not path.is_file():
            fail(failures, f"SUMMARY links a missing chapter: {path}")
    for path in sorted(docs - listed):
        fail(failures, f"chapter is not listed in SUMMARY: {path.relative_to(ROOT)}")
    for path in sorted(docs):
        for match in LINK.finditer(path.read_text(encoding="utf-8")):
            target = match.group(1).split("#", 1)[0].strip()
            if not target or target.startswith(("http://", "https://", "mailto:")):
                continue
            if target.endswith(".md") and not (path.parent / target).resolve().is_file():
                fail(failures, f"broken link in {path.relative_to(ROOT)}: {target}")
    return len(docs)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--docs", action="store_true", help="also check docs links")
    args = parser.parse_args()
    failures: list[str] = []

    design_path = ROOT / "design.json"
    if not design_path.is_file():
        fail(failures, "design.json is missing")
        design = {}
    else:
        try:
            design = json.loads(design_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            fail(failures, f"design.json is not JSON: {exc}")
            design = {}
    if design.get("schema") != "rhobuilder.design/v1":
        fail(failures, "design.json has the wrong schema")
    components = design.get("components", [])
    if not components:
        fail(failures, "design.json has no components")

    contracts = sorted((ROOT / "contracts" / "src").rglob("*.rho"))
    if not contracts:
        fail(failures, "contracts/src has no Rholang terms")
    for path in contracts:
        text = path.read_text(encoding="utf-8")
        if len(text.encode("utf-8")) > 64 * 1024:
            fail(failures, f"{path.relative_to(ROOT)} exceeds the 64 KiB deploy cap")
        if re.search(r"\bselect\b", text):
            fail(failures, f"{path.relative_to(ROOT)} uses select; committed choice is required")
        if "TODO" in text or "FIXME" in text:
            fail(failures, f"{path.relative_to(ROOT)} contains a TODO/FIXME")
        unknown = sorted(set(PLACEHOLDER.findall(text)) - INSTALL_PLACEHOLDERS)
        if unknown:
            fail(
                failures,
                f"{path.relative_to(ROOT)} contains unresolved placeholders {unknown}",
            )

    for relative in (
        "spec/graph/vertices.yaml",
        "spec/graph/edges.yaml",
        "spec/flows/core.yaml",
        "audit/registers/arms.yaml",
        "audit/registers/findings.yaml",
    ):
        path = ROOT / relative
        if not path.is_file():
            fail(failures, f"{relative} is missing")
            continue
        try:
            json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            fail(failures, f"{relative} is not JSON/YAML: {exc}")

    flows_path = ROOT / "spec" / "flows" / "core.yaml"
    if flows_path.is_file():
        try:
            flows = json.loads(flows_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            flows = {}
        ids = {item.get("id") for item in flows.get("components", []) if isinstance(item, dict)}
        for component in components:
            if component.get("id") not in ids:
                fail(failures, f"component {component.get('id')!r} is missing from spec/flows")

    documents = check_docs(failures)
    if failures:
        print("spec-check: failed")
        for failure in failures:
            print(f"  fail: {failure}")
        return 1
    print(f"spec-check: ok ({len(contracts)} terms, {len(components)} components, {documents} docs)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
