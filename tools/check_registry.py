#!/usr/bin/env python3
"""Validate the committed registry: identity, pins, primitive shape and graph edges.

This is the internal consistency gate. Sibling drift (reading pinned commits with
`git show`) is a later P1 step; this checker never reports drift it did not measure.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "registry"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
OWNERS = {"r-cash", "r-sure", "r-gen", "r-share", "r-print", "r-bridge", "rholo"}
STATUSES = {"scaffold", "implemented", "tested", "deployed", "unverified"}
TESTNET_ID = "testnet-133b1d307b175012"
TESTNET_SHARD = "/testnet"
TESTNET_GENESIS = "133b1d307b1750129cf0cfb69c4c8f950da21c4e8db2a6a076464d0ccfd98d44"


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def check() -> dict[str, object]:
    errors: list[str] = []
    warnings: list[str] = []
    required = [
        "schema.json",
        "pins.json",
        "genesis.json",
        "primitives.json",
        "networks/devnet.json",
        "networks/testnet.json",
        "networks/mainnet.json",
    ]
    for name in required:
        if not (REGISTRY / name).is_file():
            errors.append(f"missing registry/{name}")
    if errors:
        return {"schema": "rhobuilder.registry-check/v1", "ok": False, "errors": errors}

    pins = load(REGISTRY / "pins.json")
    if pins.get("schema") != "rhobuilder.pins/v1":
        errors.append("pins.json: wrong schema")
    if pins.get("rust", {}).get("toolchain") != "1.95.0":
        errors.append("pins.json: rust toolchain must be 1.95.0")
    if not HEX40.match(str(pins.get("crypto", {}).get("rev", ""))):
        errors.append("pins.json: crypto rev must be 40 hex")
    r_cash = pins.get("siblings", {}).get("r-cash", {})
    if r_cash.get("rev") != "49b441dea4f5561992387ed670ab263fcd5dd164":
        errors.append("pins.json: r-cash rev must be 49b441d…")
    r_gen = pins.get("siblings", {}).get("r-gen", {})
    if set(r_gen.get("consumerPins", {})) != {"r-print", "r-sure"}:
        errors.append("pins.json: r-gen must carry r-print and r-sure consumer pins")
    for name in ("r-share", "r-print", "r-bridge"):
        sibling = pins.get("siblings", {}).get(name, {})
        if sibling.get("authoritative") is True and not HEX40.match(str(sibling.get("rev", ""))):
            errors.append(f"pins.json: {name} claims authority without a 40-hex rev")

    testnet = load(REGISTRY / "networks" / "testnet.json")
    if testnet.get("networkId") != TESTNET_ID:
        errors.append(f"networks/testnet.json: networkId must be {TESTNET_ID}")
    if testnet.get("shardId") != TESTNET_SHARD:
        errors.append(f"networks/testnet.json: shardId must be {TESTNET_SHARD}")
    if testnet.get("genesisBlockHash") != TESTNET_GENESIS:
        errors.append("networks/testnet.json: genesisBlockHash must be the measured 133b… value")
    if testnet.get("deploy") != "gated":
        errors.append("networks/testnet.json: deploy must be gated")
    if not testnet.get("faucet"):
        errors.append("networks/testnet.json: faucet must be recorded")

    devnet = load(REGISTRY / "networks" / "devnet.json")
    if devnet.get("deploy") != "allowed":
        errors.append("networks/devnet.json: deploy must be allowed")
    mainnet = load(REGISTRY / "networks" / "mainnet.json")
    if mainnet.get("status") != "unavailable" or mainnet.get("deploy") != "refused":
        errors.append("networks/mainnet.json: must be unavailable/refused")

    registry = load(REGISTRY / "primitives.json")
    if registry.get("schema") != "rhobuilder.registry/v1":
        errors.append("primitives.json: wrong schema")
    primitives = registry.get("primitives", [])
    ids: set[str] = set()
    for primitive in primitives:
        pid = primitive.get("id", "")
        if not pid:
            errors.append("primitives.json: primitive without id")
            continue
        if pid in ids:
            errors.append(f"primitives.json: duplicate id {pid}")
        ids.add(pid)
        if primitive.get("owner") not in OWNERS:
            errors.append(f"primitives.json: {pid} has unknown owner {primitive.get('owner')!r}")
        if primitive.get("status") not in STATUSES:
            errors.append(f"primitives.json: {pid} has unknown status {primitive.get('status')!r}")
        if not primitive.get("source"):
            errors.append(f"primitives.json: {pid} has no source")
        pin = primitive.get("pinnedCommit", "")
        if pin != "TBD-review-required" and not HEX40.match(str(pin)):
            errors.append(f"primitives.json: {pid} pin is neither 40-hex nor TBD-review-required")
        for network, entry in primitive.get("networks", {}).items():
            if entry.get("verified") not in {"lookup", "manifest", "unverified", "candidate", "missing", "mismatch"}:
                errors.append(f"primitives.json: {pid}/{network} has unknown verified value")
    kinds = {primitive.get("id"): primitive.get("kind") for primitive in primitives}
    if kinds.get("rsure.claim") != "insurance-claim":
        errors.append("primitives.json: rsure.claim must be an insurance-claim")
    if kinds.get("rgen.claim") != "knowledge-claim":
        errors.append("primitives.json: rgen.claim must be a knowledge-claim")

    vertices = {vertex.get("id") for vertex in registry.get("vertices", [])}
    edge_ids: set[str] = set()
    for edge in registry.get("edges", []):
        eid = edge.get("id", "")
        if not eid:
            errors.append("primitives.json: edge without id")
        elif eid in edge_ids:
            errors.append(f"primitives.json: duplicate edge {eid}")
        edge_ids.add(eid)
        for endpoint in ("from", "to"):
            if edge.get(endpoint) not in vertices:
                errors.append(f"primitives.json: edge {eid} names unknown vertex {edge.get(endpoint)!r}")

    genesis = load(REGISTRY / "genesis.json")
    if genesis.get("schema") != "rhobuilder.genesis/v1":
        errors.append("genesis.json: wrong schema")
    if len(genesis.get("contracts", [])) < 4:
        errors.append("genesis.json: expected the 4 hardcodable contracts")
    if len(genesis.get("rgov", [])) < 11:
        errors.append("genesis.json: expected the rgov constants")
    if len(genesis.get("native", [])) < 3:
        errors.append("genesis.json: expected the native channels")

    if not primitives:
        warnings.append("primitives.json: catalog is empty")
    if any(primitive.get("status") == "unverified" for primitive in primitives):
        warnings.append("primitives.json: some primitives are unverified")

    return {
        "schema": "rhobuilder.registry-check/v1",
        "ok": not errors,
        "errors": errors,
        "warnings": warnings,
        "primitives": len(primitives),
        "edges": len(edge_ids),
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
            f"registry-check: {'ok' if report['ok'] else 'failed'} "
            f"({report.get('primitives', 0)} primitives, {report.get('edges', 0)} edges)"
        )
        for item in report.get("errors", []):
            print(f"  error: {item}")
        for item in report.get("warnings", []):
            print(f"  warn: {item}")
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
