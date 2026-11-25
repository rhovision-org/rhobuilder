#!/usr/bin/env python3
"""Assemble and self-test the complete rhobuilder release tarball.

The old release artifact was a bare binary; the binary's default registry and template
roots were absent. This package includes the registry, templates, plugin/UI, skills,
launcher, gates, MCP and docs, plus a wrapper that resolves the package root.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DIRS = [
    "registry",
    "templates",
    "formal",
    "agents",
    "skills",
    "tools",
    "mcp",
    "gates",
    "fixtures",
    "docs/src",
]
FILES = [
    "README.md",
    "LICENSE",
    "THIRD_PARTY_NOTICES.md",
    "CHANGELOG.md",
    "SECURITY.md",
    "CONTRIBUTING.md",
    "CITATION.cff",
    "book.toml",
    "skills.toml",
    "Cargo.toml",
    "Cargo.lock",
    "Makefile",
]
EXCLUDE_DIRS = {
    ".git",
    ".dsh",
    ".dsh-home",
    ".ops",
    "target",
    "node_modules",
    "__pycache__",
    ".pytest_cache",
    ".lake",
    "book",
}
TEXT_SUFFIXES = {".md", ".py", ".sh", ".toml", ".json", ".yml", ".yaml", ".rs", ".js", ".mjs"}


def version() -> str:
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
    return match.group(1) if match else "0.0.0"


def target_name() -> str:
    machine = platform.machine().lower()
    arch = "aarch64" if machine in {"aarch64", "arm64"} else "x86_64"
    return f"linux-{arch}"


def included_files() -> list[Path]:
    paths: list[Path] = []
    for name in FILES:
        path = ROOT / name
        if path.is_file():
            paths.append(path)
    for name in DIRS:
        base = ROOT / name
        if not base.is_dir():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            relative_parts = path.relative_to(ROOT).parts
            if any(part in EXCLUDE_DIRS for part in relative_parts):
                continue
            if path.suffix in {".pyc", ".tmp"}:
                continue
            paths.append(path)
    return sorted(set(paths))


def check_layout() -> list[str]:
    errors: list[str] = []
    for name in ["registry/primitives.json", "templates/dapp/common/Makefile", "skills.toml"]:
        if not (ROOT / name).is_file():
            errors.append(f"missing {name}")
    for name in ["agents/deepseek/plugin/index.js", "agents/deepseek/ui/index.js", "tools/launch_dsh.py"]:
        if not (ROOT / name).is_file():
            errors.append(f"missing {name}")
    wrapper = ROOT / "tools" / "launch-rhobuilder.sh"
    if not wrapper.is_file():
        errors.append("missing tools/launch-rhobuilder.sh")
    for path in included_files():
        if path.suffix not in TEXT_SUFFIXES:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        machine_path = "/home/" "patrick"
        if machine_path in text:
            errors.append(f"machine-local path in {path.relative_to(ROOT)}")
    return errors


def build(binary: Path, out: Path) -> Path:
    if not binary.is_file():
        raise SystemExit(f"package-release: binary not found: {binary}; run cargo build --release")
    out.mkdir(parents=True, exist_ok=True)
    name = f"rhobuilder-{version()}-{target_name()}"
    tar_path = out / f"{name}.tar.gz"
    staging = Path(tempfile.mkdtemp(prefix="rhobuilder-package-"))
    try:
        package = staging / name
        package.mkdir()
        for path in included_files():
            relative = path.relative_to(ROOT)
            target = package / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
        (package / "bin").mkdir()
        shutil.copy2(binary, package / "bin" / "rhobuilder")
        (package / "bin" / "rhobuilder").chmod(0o755)
        shutil.copy2(ROOT / "tools" / "launch-rhobuilder.sh", package / "launch-rhobuilder.sh")
        (package / "launch-rhobuilder.sh").chmod(0o755)
        with tarfile.open(tar_path, "w:gz") as archive:
            archive.add(package, arcname=name)
        digest = hashlib.sha256(tar_path.read_bytes()).hexdigest()
        tar_path.with_suffix(tar_path.suffix + ".sha256").write_text(
            f"{digest}  {tar_path.name}\n", encoding="utf-8"
        )
        return tar_path
    finally:
        shutil.rmtree(staging, ignore_errors=True)


def run(argv: list[str], cwd: Path, timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, cwd=cwd, text=True, capture_output=True, check=False, timeout=timeout)


def self_test(tar_path: Path) -> list[str]:
    errors: list[str] = []
    with tempfile.TemporaryDirectory(prefix="rhobuilder-selftest-") as temporary:
        temp = Path(temporary)
        with tarfile.open(tar_path, "r:gz") as archive:
            archive.extractall(temp)
        roots = [path for path in temp.iterdir() if path.is_dir()]
        if len(roots) != 1:
            return ["tarball must contain one top-level directory"]
        root = roots[0]
        binary = root / "bin" / "rhobuilder"
        commands = [
            ([str(binary), "--json", "version"], "version"),
            ([str(binary), "--json", "registry", "list"], "registry list"),
            (
                [
                    str(binary),
                    "--json",
                    "design",
                    "check",
                    "--path",
                    "fixtures/designs/booking.json",
                ],
                "design check",
            ),
            (
                [
                    str(binary),
                    "new",
                    str(temp / "scaffold"),
                    "--design",
                    "fixtures/designs/booking.json",
                    "--template",
                    "headless",
                    "--plan",
                    "--force",
                ],
                "new plan",
            ),
        ]
        for argv, label in commands:
            try:
                proc = run(argv, root)
            except (OSError, subprocess.TimeoutExpired) as exc:
                errors.append(f"{label}: {exc}")
                continue
            if proc.returncode != 0:
                errors.append(f"{label}: exit {proc.returncode}: {(proc.stderr or proc.stdout)[-300:]}")
        try:
            proc = run(
                [sys.executable, "tools/launch_dsh.py", "--check", "--json"],
                root,
                timeout=120,
            )
            document = json.loads(proc.stdout) if proc.stdout.strip().startswith("{") else {}
            if document.get("schema") != "rhobuilder.dsh-check/v1":
                errors.append("launcher readiness: no dsh-check JSON")
        except (OSError, subprocess.TimeoutExpired, json.JSONDecodeError) as exc:
            errors.append(f"launcher readiness: {exc}")
        try:
            proc = run([str(root / "launch-rhobuilder.sh"), "--print-config", "web"], root)
            if proc.returncode != 0 or "rhobuilder-ui" not in proc.stdout:
                errors.append("packaged wrapper did not render the rhobuilder web overlay")
        except (OSError, subprocess.TimeoutExpired) as exc:
            errors.append(f"packaged wrapper: {exc}")
        return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="verify the source layout")
    parser.add_argument("--build", action="store_true", help="build the release tarball")
    parser.add_argument("--self-test", action="store_true", help="extract and smoke-test a tarball")
    parser.add_argument("--binary", type=Path, default=ROOT / "target" / "release" / "rhobuilder")
    parser.add_argument("--out", type=Path, default=ROOT / "dist")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    if not (args.check or args.build or args.self_test):
        parser.error("pass --check, --build or --self-test")
    report: dict[str, object] = {"schema": "rhobuilder.package/v1", "ok": True}
    errors: list[str] = []
    tar_path: Path | None = None
    if args.check:
        errors.extend(check_layout())
    if args.build or args.self_test:
        try:
            tar_path = build(args.binary, args.out)
            report["tarball"] = str(tar_path)
            report["sha256"] = hashlib.sha256(tar_path.read_bytes()).hexdigest()
        except SystemExit as exc:
            errors.append(str(exc))
    if args.self_test and tar_path is not None:
        errors.extend(self_test(tar_path))
    report["errors"] = errors
    report["ok"] = not errors
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"package-release: {'ok' if report['ok'] else 'failed'}")
        if tar_path:
            print(f"  tarball: {tar_path}")
        for error in errors:
            print(f"  fail: {error}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
