#!/usr/bin/env python3
"""Check the mdBook source and README: coverage, links, anchors and canonical status."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "rhobuilder.docs-check/v2"
LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
HEADING = re.compile(r"^(#{1,6})\s+(.+?)\s*$")
FENCE = re.compile(r"^\s*(```|~~~)")
STATUS_HEADER = "| Area | Status |"


def _links(path: Path) -> list[tuple[Path, str]]:
    found: list[tuple[Path, str]] = []
    for match in LINK.finditer(path.read_text(encoding="utf-8")):
        raw = match.group(1).strip()
        if not raw or raw.startswith(("http://", "https://", "mailto:")):
            continue
        target, _, anchor = raw.partition("#")
        target = target.strip()
        if not target and not anchor:
            continue
        # Anchors without a path refer to the current document.
        resolved = (path.parent / target).resolve() if target else path.resolve()
        if target and not target.endswith(".md"):
            continue
        if not target:
            resolved = path.resolve()
        found.append((resolved, anchor))
    return found


def _headings(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    headings: list[str] = []
    in_fence = False
    for line in text.splitlines():
        if FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        match = HEADING.match(line)
        if match:
            headings.append(match.group(2).strip())
    return headings


def _slug(text: str) -> str:
    text = re.sub(r"`([^`]*)`", r"\1", text)
    text = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", text)
    text = re.sub(r"[*_~]", "", text)
    text = text.strip().lower()
    text = re.sub(r"[^\w\s-]", "", text, flags=re.UNICODE)
    text = re.sub(r"\s+", "-", text)
    return text.strip("-")


def _anchors(path: Path) -> set[str]:
    found: set[str] = set()
    seen: dict[str, int] = {}
    for heading in _headings(path):
        slug = _slug(heading)
        if not slug:
            continue
        count = seen.get(slug, 0)
        seen[slug] = count + 1
        found.add(slug if count == 0 else f"{slug}-{count}")
    return found


def _relative(root: Path, path: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def check(root: Path = ROOT) -> dict[str, object]:
    docs = root / "docs" / "src"
    summary = docs / "SUMMARY.md"
    readme = root / "README.md"
    if not summary.is_file():
        return {"schema": SCHEMA, "ok": False, "error": f"missing {summary}"}

    listed: set[Path] = set()
    for target, _anchor in _links(summary):
        listed.add(target)
    all_docs = {path.resolve() for path in docs.rglob("*.md") if path.name != "SUMMARY.md"}
    missing_listed = sorted(_relative(root, path) for path in listed if not path.is_file())
    unlisted = sorted(_relative(root, path) for path in all_docs - listed)

    broken: list[str] = []
    anchors_checked = 0
    readme_links = 0
    readme_docs: set[Path] = set()
    sources = sorted(all_docs)
    if readme.is_file():
        sources.append(readme.resolve())
    for source in sources:
        if not source.is_file():
            continue
        if source == readme.resolve():
            readme_links += len(_links(source))
        for target, anchor in _links(source):
            label = f"{_relative(root, source)} -> {_relative(root, target)}"
            if target.suffix != ".md":
                continue
            if not target.is_file():
                broken.append(label)
                continue
            if anchor:
                anchors_checked += 1
                if anchor not in _anchors(target):
                    broken.append(f"{label}#{anchor}")
            if source == readme.resolve() and target.is_relative_to(docs):
                readme_docs.add(target)

    readme_unlisted = sorted(_relative(root, path) for path in readme_docs - listed)
    status_files = sorted(
        _relative(root, path)
        for path in all_docs
        if STATUS_HEADER in path.read_text(encoding="utf-8")
    )
    canonical = (docs / "index.md").resolve()
    canonical_ok = (
        status_files == [_relative(root, canonical)]
        and canonical.is_file()
        and "## Status" in canonical.read_text(encoding="utf-8")
    )

    return {
        "schema": SCHEMA,
        "ok": not missing_listed
        and not unlisted
        and not broken
        and not readme_unlisted
        and canonical_ok,
        "listed": len(listed),
        "documents": len(all_docs),
        "missing_listed": missing_listed,
        "unlisted": unlisted,
        "broken": sorted(broken),
        "readme_links": readme_links,
        "readme_unlisted": readme_unlisted,
        "anchors_checked": anchors_checked,
        "canonical_status": {
            "ok": canonical_ok,
            "files": status_files,
            "required": _relative(root, canonical),
        },
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
            f"({report.get('listed', 0)} listed, {report.get('documents', 0)} documents, "
            f"{report.get('readme_links', 0)} README links)"
        )
        for key in ("missing_listed", "unlisted", "broken", "readme_unlisted"):
            for item in report.get(key, []):
                print(f"  {key}: {item}")
        if not report.get("canonical_status", {}).get("ok", False):
            print(f"  canonical_status: {report.get('canonical_status')}")
        if "error" in report:
            print(f"  error: {report['error']}")
    return 0 if report.get("ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
