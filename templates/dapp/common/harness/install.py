#!/usr/bin/env python3
"""Deploy the scaffold contracts to a local rholo devnet.

The devnet's own funded validator key is used through `rnode deploy
--private-key-path` inside the container. The key is never read, printed or copied to the
host. This is a devnet smoke, not a behavioural suite.

Environment:
  RHOBUILDER_DEVNET=1            opt in
  RHOBUILDER_NODE                default http://localhost:40403
  RHOBUILDER_CONTAINER           default devnet-bootstrap
  RHOBUILDER_SHARD               default /root
  RHOBUILDER_KEY_PATH            default /var/lib/rnode/validator.hex
  RHOBUILDER_DEPLOY_TIMEOUT_S    default 120
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NODE = os.environ.get("RHOBUILDER_NODE", "http://localhost:40403").rstrip("/")
CONTAINER = os.environ.get("RHOBUILDER_CONTAINER", "devnet-bootstrap")
SHARD = os.environ.get("RHOBUILDER_SHARD", "/root")
KEY_PATH = os.environ.get("RHOBUILDER_KEY_PATH", "/var/lib/rnode/validator.hex")
TIMEOUT = float(os.environ.get("RHOBUILDER_DEPLOY_TIMEOUT_S", "120"))
DEPLOY_ID = re.compile(r"DeployId is:\s*([0-9a-fA-F]+)")


def get_json(path: str) -> dict:
    with urllib.request.urlopen(f"{NODE}{path}", timeout=10) as response:
        return json.load(response)


def not_run(reason: str) -> int:
    print(f"install: not_run ({reason})")
    return 2


def run(argv: list[str], *, timeout: float = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, text=True, capture_output=True, timeout=timeout, check=False)


def deploy(path: Path, height: int) -> dict:
    target = f"/tmp/rhobuilder-{path.name}"
    copied = run(["docker", "cp", str(path), f"{CONTAINER}:{target}"])
    if copied.returncode != 0:
        return {"file": path.name, "status": "copy-failed", "detail": copied.stderr[-300:]}
    argv = [
        "docker",
        "exec",
        "-i",
        CONTAINER,
        "rnode",
        "--grpc-host",
        "localhost",
        "deploy",
        "--phlo-limit",
        "1000000",
        "--phlo-price",
        "1",
        "--private-key-path",
        KEY_PATH,
        "--shard-id",
        SHARD,
        "--valid-after-block-number",
        str(height),
        target,
    ]
    result = run(argv, timeout=120)
    output = result.stdout + result.stderr
    match = DEPLOY_ID.search(output)
    if result.returncode != 0 or not match:
        return {
            "file": path.name,
            "status": "submit-failed",
            "detail": output[-500:],
        }
    deploy_id = match.group(1)
    deadline = time.monotonic() + TIMEOUT
    while time.monotonic() < deadline:
        try:
            status = get_json(f"/api/v1/deploy-status/{deploy_id}")
        except (urllib.error.URLError, OSError) as exc:
            return {"file": path.name, "deployId": deploy_id, "status": "status-failed", "detail": str(exc)}
        if "ProcessedWithSuccess" in status:
            return {"file": path.name, "deployId": deploy_id, "status": "processedWithSuccess"}
        if "ProcessedWithError" in status:
            return {
                "file": path.name,
                "deployId": deploy_id,
                "status": "processedWithError",
                "detail": status["ProcessedWithError"].get("deployError", ""),
            }
        time.sleep(2)
    return {"file": path.name, "deployId": deploy_id, "status": "timeout"}


def main() -> int:
    if os.environ.get("RHOBUILDER_DEVNET") != "1":
        return not_run("set RHOBUILDER_DEVNET=1 to opt in")
    try:
        status = get_json("/api/v1/status")
    except (urllib.error.URLError, OSError) as exc:
        return not_run(f"{NODE} is unreachable: {exc}")
    degraded = status.get("degraded", "unknown")
    if degraded != "normal":
        return not_run(f"devnet is {degraded}; not deploying")
    height = int(status.get("latestBlockNumber", 0))
    contracts = sorted((ROOT / "contracts" / "src").glob("*.rho"))
    if not contracts:
        print("install: no contracts")
        return 1
    results = []
    for contract in contracts:
        result = deploy(contract, height)
        results.append(result)
        print(f"  {result['status']:20} {contract.name}")
    manifest = {
        "schema": "rhobuilder.devnet-install/v1",
        "node": NODE,
        "shard": SHARD,
        "height": height,
        "deploys": results,
    }
    out = ROOT / "deployments" / "devnet.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    failed = [item for item in results if item["status"] != "processedWithSuccess"]
    if failed:
        print(f"install: {len(failed)} of {len(results)} deploy(s) failed")
        return 1
    print(f"install: {len(results)} deploy(s) processedWithSuccess")
    return 0


if __name__ == "__main__":
    sys.exit(main())
