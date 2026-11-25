#!/usr/bin/env python3
"""Scaffold smoke: the design, generated files and app install are consistent."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NETWORK = os.environ.get("RHOBUILDER_NETWORK", "devnet")
BIN = os.environ.get("RHOBUILDER_BIN", "rhobuilder")
KEYS = os.environ.get("RHOBUILDER_KEYS", ".ops/devnet/keys.json")
REGISTRY = os.environ.get("RHOBUILDER_REGISTRY", "")
REQUIRED = [
    "design.json",
    "reuse-report.json",
    "contracts/src/00_root.rho",
    "spec/graph/vertices.yaml",
    "spec/flows/core.yaml",
    "audit/registers/arms.yaml",
    ".rhobuilder.json",
]


def resolve_roots(root: str, keys: list[str]) -> bool:
    for key in keys:
        term = f"""
new rl(`rho:registry:lookup`), ch, ret, d(`rho:rchain:deployId`) in {{
  rl!(`{root}`, *ch) |
  for (C <- ch) {{
    C!("resolve", "{key}", 1, *ret) |
    for (@r <- ret) {{ d!(r) }}
  }}
}}
"""
        command = [BIN, "--json"]
        if REGISTRY:
            command.extend(["--registry", REGISTRY])
        command.extend(
            [
                "chain",
                "call",
                "--network",
                NETWORK,
                "--keys",
                KEYS,
                "--execute",
                "--term",
                term,
            ]
        )
        proc = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
        if proc.returncode != 0:
            print(f"  fail: chain call for {key}: {proc.stdout}{proc.stderr}")
            return False
        try:
            value = json.loads(proc.stdout)
        except json.JSONDecodeError:
            print(f"  fail: chain call for {key} returned non-JSON")
            return False
        result = value.get("result", [])
        reply = result[0] if len(result) == 1 else result
        if not (
            isinstance(reply, list)
            and reply
            and reply[0] is True
            and isinstance(reply[1], str)
            and reply[1].startswith("rho:id:")
        ):
            print(f"  fail: {key} did not resolve: {reply!r}")
            return False
    return True


def report_of(value: dict) -> dict:
    if isinstance(value.get("report"), dict):
        return value["report"]
    result = value.get("result")
    if isinstance(result, dict) and isinstance(result.get("report"), dict):
        return result["report"]
    return value


def main() -> int:
    missing = [path for path in REQUIRED if not (ROOT / path).is_file()]
    if missing:
        print("scaffold-smoke: fail")
        for path in missing:
            print(f"  missing: {path}")
        return 1
    manifest = json.loads((ROOT / ".rhobuilder.json").read_text(encoding="utf-8"))
    slug = manifest["app"]["slug"]
    path = ROOT / "deployments" / f"{NETWORK}-{slug}.json"
    if path.is_file():
        value = json.loads(path.read_text(encoding="utf-8"))
        report = report_of(value)
        if not report.get("verified", False):
            print(f"scaffold-smoke: fail (app root not verified: {report.get('status')})")
            return 1
        root = report.get("rootUri") or report.get("root_uri")
        if not root:
            print("scaffold-smoke: fail (app report has no rootUri)")
            return 1
        contracts_path = ROOT / ".rhobuilder-contracts.json"
        if contracts_path.is_file():
            contracts = json.loads(contracts_path.read_text(encoding="utf-8"))
            keys = contracts.get("requiredKeys", [])
            if keys and not resolve_roots(root, keys):
                return 1
            print(f"  root-resolve: ok ({len(keys)} key(s))")
    else:
        legacy = ROOT / "deployments" / "devnet.json"
        if legacy.is_file():
            install = json.loads(legacy.read_text(encoding="utf-8"))
            failed = [
                item
                for item in install.get("deploys", [])
                if item.get("status") != "processedWithSuccess"
            ]
            if failed:
                print(f"scaffold-smoke: fail ({len(failed)} deploy(s) not processed)")
                return 1
    design = json.loads((ROOT / "design.json").read_text(encoding="utf-8"))
    print(f"scaffold-smoke: ok ({design['app']['slug']})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
