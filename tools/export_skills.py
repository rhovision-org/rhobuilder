#!/usr/bin/env python3
"""Export the rhobuilder skill catalog into DSH and Claude Code layouts.

`skills.toml` is the single metadata source. Canonical skill bodies live in
`skills/<name>/SKILL.md`; the harness-only build procedure lives in
`agents/deepseek/skills/rhobuilder-build.md` and is not a catalog entry.

Targets:
  dsh      <root>/.dsh/skills/<name>/SKILL.md      DeepSeek Harness (directory form)
  claude   <root>/.claude/skills/<name>/SKILL.md   Claude Code (portability)

The catalog is model-visible and expensive. A hard cap of 12 entries (including the
procedure) and a 32 KiB per-skill body budget are enforced.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import tomllib
from collections.abc import Iterable, Sequence
from pathlib import Path

NAME_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "skills.toml"
PROCEDURE = Path("agents/deepseek/skills/rhobuilder-build.md")
PROCEDURE_NAME = "rhobuilder-build"
PROCEDURE_DESCRIPTION = (
    "Guide a new rhovision dApp from idea to a tested, deployable plan: probe, design, "
    "reuse, scaffold, implement, prove, test, plan. Never signs a deploy."
)
MAX_SKILLS = 12
MAX_BODY_BYTES = 32 * 1024

TARGETS: dict[str, str] = {
    "dsh": ".dsh/skills",
    "claude": ".claude/skills",
}


def _quote(value: str) -> str:
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"') + '"'


def strip_frontmatter(text: str) -> str:
    if not text.startswith("---"):
        return text
    lines = text.splitlines(keepends=True)
    if not lines or lines[0].strip() != "---":
        return text
    for index in range(1, len(lines)):
        if lines[index].strip() == "---":
            return "".join(lines[index + 1 :]).lstrip("\n")
    return text


def render(name: str, description: str, body: str) -> str:
    body = strip_frontmatter(body)
    if len(body.encode("utf-8")) > MAX_BODY_BYTES:
        raise ValueError(f"skill {name} body exceeds {MAX_BODY_BYTES} bytes")
    return f"---\nname: {name}\ndescription: {_quote(description)}\n---\n{body}"


def render_extra(
    name: str,
    description: str,
    body: str,
    *,
    user_invocable: bool = True,
    when_to_use: str | None = None,
) -> str:
    body = strip_frontmatter(body)
    if len(body.encode("utf-8")) > MAX_BODY_BYTES:
        raise ValueError(f"skill {name} body exceeds {MAX_BODY_BYTES} bytes")
    lines = ["---", f"name: {name}", f"description: {_quote(description)}"]
    if user_invocable:
        lines.append("user-invocable: true")
    if when_to_use is not None:
        lines.append(f"whenToUse: {_quote(when_to_use)}")
    lines += ["---", body]
    return "\n".join(lines)


def load_catalog(catalog: Path = CATALOG) -> tuple[list[dict[str, object]], list[str]]:
    document = tomllib.loads(catalog.read_text(encoding="utf-8"))
    entries = list(document.get("skill", []))
    curated = list(document.get("curated", {}).get("names", []))
    return entries, curated


def validate_catalog(entries: Sequence[dict[str, object]], root: Path) -> None:
    names = [str(entry.get("name", "")) for entry in entries]
    if len(entries) > MAX_SKILLS:
        raise ValueError(f"catalog has {len(entries)} skills; cap is {MAX_SKILLS}")
    if len(set(names)) != len(names):
        raise ValueError("catalog names must be unique")
    for entry in entries:
        name = str(entry.get("name", ""))
        description = str(entry.get("description", ""))
        path = str(entry.get("path", ""))
        if not NAME_RE.match(name):
            raise ValueError(f"invalid kebab-case skill name: {name!r}")
        if not description:
            raise ValueError(f"skill {name} has no description")
        if not (root / path).is_file():
            raise ValueError(f"skill {name} body is missing: {path}")


def expected(
    root: Path = ROOT,
    targets: Sequence[str] = tuple(TARGETS),
    names: Iterable[str] | None = None,
    *,
    source_root: Path | None = None,
    with_procedure: bool = True,
) -> dict[Path, str]:
    root = Path(root)
    source = Path(source_root) if source_root is not None else root
    entries, _ = load_catalog(source / "skills.toml")
    validate_catalog(entries, source)
    selected = None if names is None else set(names)
    if selected is not None:
        known = {str(entry["name"]) for entry in entries} | {PROCEDURE_NAME}
        unknown = selected - known
        if unknown:
            raise KeyError(f"skills not in the catalog: {sorted(unknown)}")
    files: dict[Path, str] = {}
    for target in targets:
        if target not in TARGETS:
            raise KeyError(f"unknown target: {target}")
        out = root / TARGETS[target]
        for entry in entries:
            name = str(entry["name"])
            if selected is not None and name not in selected:
                continue
            body = (source / str(entry["path"])).read_text(encoding="utf-8")
            files[out / name / "SKILL.md"] = render(name, str(entry["description"]), body)
        if with_procedure and (selected is None or PROCEDURE_NAME in selected):
            procedure = (source / PROCEDURE).read_text(encoding="utf-8")
            files[out / PROCEDURE_NAME / "SKILL.md"] = render_extra(
                PROCEDURE_NAME,
                PROCEDURE_DESCRIPTION,
                procedure,
                user_invocable=True,
                when_to_use=(
                    "The user asks to build, design, scaffold, review or plan a rhovision "
                    "dApp on rholo, or asks which existing primitive to reuse."
                ),
            )
    return files


def write_expected(files: dict[Path, str]) -> int:
    roots: set[Path] = set()
    for path, text in files.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        roots.add(path.parent.parent)
    for root in roots:
        manifest = {
            path.parent.name: hashlib.sha256(text.encode("utf-8")).hexdigest()
            for path, text in files.items()
            if path.parent.parent == root
        }
        (root / ".manifest.json").write_text(
            json.dumps(
                {"schema": "rhobuilder.skills/v1", "skills": manifest},
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
    return len(files)


def check_expected(files: dict[Path, str], roots: Sequence[Path]) -> dict[str, object]:
    stale: list[str] = []
    missing: list[str] = []
    for path, text in files.items():
        if not path.is_file():
            missing.append(str(path))
        elif path.read_text(encoding="utf-8") != text:
            stale.append(str(path))
    expected_paths = {str(path) for path in files}
    orphans: list[str] = []
    for out in roots:
        if not out.is_dir():
            continue
        for path in out.glob("*/SKILL.md"):
            if str(path) not in expected_paths:
                orphans.append(str(path))
    return {
        "schema": "rhobuilder.skill-export-check/v1",
        "ok": not stale and not missing,
        "missing": sorted(missing),
        "stale": sorted(stale),
        "orphans": sorted(orphans),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--target", choices=["all", *TARGETS], default="all")
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--name", action="append", help="export only this catalog name")
    parser.add_argument("--check", action="store_true", help="compare without writing")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--no-procedure", action="store_true")
    args = parser.parse_args()

    targets = list(TARGETS) if args.target == "all" else [args.target]
    try:
        files = expected(
            args.root,
            targets,
            args.name,
            source_root=ROOT,
            with_procedure=not args.no_procedure,
        )
    except (KeyError, ValueError, OSError) as exc:
        if args.json:
            print(json.dumps({"schema": "rhobuilder.skill-export-check/v1", "ok": False, "error": str(exc)}))
        else:
            print(f"export_skills: {exc}", file=sys.stderr)
        return 1

    roots = [args.root / TARGETS[target] for target in targets]
    if args.check:
        report = check_expected(files, roots)
        if args.json:
            print(json.dumps(report, indent=2))
        else:
            print(f"export_skills: {'ok' if report['ok'] else 'stale'} ({len(files)} files)")
            for key in ("missing", "stale", "orphans"):
                for path in report[key]:
                    print(f"  {key}: {path}")
        return 0 if report["ok"] else 1

    written = write_expected(files)
    if args.json:
        print(json.dumps({"schema": "rhobuilder.skill-export/v1", "ok": True, "files": written}))
    else:
        print(f"export_skills: wrote {written} skill file(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
