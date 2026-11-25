"""Helpers for the bookable-service devnet suites."""

from __future__ import annotations

import json
import os
import subprocess
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NETWORK = os.environ.get("RHOBUILDER_NETWORK", "devnet")
BIN = os.environ.get("RHOBUILDER_BIN", "rhobuilder")
KEYS = Path(os.environ.get("RHOBUILDER_KEYS", ".ops/devnet/keys.json"))
REGISTRY = os.environ.get("RHOBUILDER_REGISTRY", "")


class NotRun(RuntimeError):
    pass


def node_status() -> dict:
    node = os.environ.get("RHOBUILDER_NODE", "http://localhost:40403").rstrip("/")
    try:
        with urllib.request.urlopen(f"{node}/api/v1/status", timeout=5) as response:
            return json.load(response)
    except (urllib.error.URLError, OSError, json.JSONDecodeError) as exc:
        raise NotRun(f"{node} is unreachable: {exc}") from exc


def require_devnet() -> None:
    if os.environ.get("RHOBUILDER_DEVNET") != "1":
        raise NotRun("set RHOBUILDER_DEVNET=1 to opt in")
    status = node_status()
    if status.get("degraded", "unknown") != "normal":
        raise NotRun(f"devnet is {status.get('degraded')}")
    if not status.get("acceptsDeploys", False):
        raise NotRun("devnet is not accepting deploys")


def app_slug() -> str:
    value = json.loads((ROOT / ".rhobuilder.json").read_text(encoding="utf-8"))
    return value["app"]["slug"]


def manifest() -> dict:
    path = ROOT / "deployments" / f"{NETWORK}-{app_slug()}.json"
    if not path.is_file():
        raise NotRun(f"missing {path}; run make install first")
    return json.loads(path.read_text(encoding="utf-8"))


def app_root() -> str:
    value = manifest()
    report = value.get("result", {}).get("report", value.get("report", {}))
    root = report.get("rootUri")
    if not root:
        raise NotRun("the app manifest has no rootUri")
    return root


def deployer_public() -> str:
    path = KEYS if KEYS.is_absolute() else ROOT / KEYS
    if not path.is_file():
        raise NotRun(f"missing keys file {path}")
    value = json.loads(path.read_text(encoding="utf-8"))
    try:
        return value["deployer"]["public"]
    except KeyError as exc:
        raise NotRun(f"{path} has no deployer.public") from exc


def call(term: str) -> list:
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
            str(KEYS),
            "--execute",
            "--term",
            term,
        ]
    )
    proc = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        raise AssertionError(f"chain call failed: {proc.stdout}{proc.stderr}")
    try:
        value = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise AssertionError(f"chain call returned non-JSON: {proc.stdout}") from exc
    if not value.get("ok"):
        raise AssertionError(f"chain call refused: {value.get('error')}")
    result = value.get("result", [])
    if len(result) == 1:
        return result[0]
    return result


def resolve_term(root: str, key: str) -> str:
    return f"""
new rl(`rho:registry:lookup`), ch, ret, d(`rho:rchain:deployId`) in {{
  rl!(`{root}`, *ch) |
  for (C <- ch) {{
    C!("resolve", "{key}", 1, *ret) |
    for (@r <- ret) {{ d!(r) }}
  }}
}}
"""


def call_term(uri: str, arm: str, args: str = "") -> str:
    head = f'"{arm}"' + (f", {args}" if args else "")
    return f"""
new rl(`rho:registry:lookup`), ch, ret, d(`rho:rchain:deployId`) in {{
  rl!(`{uri}`, *ch) |
  for (C <- ch) {{
    C!({head}, *ret) |
    for (@r <- ret) {{ d!(r) }}
  }}
}}
"""


def resolve(root: str, key: str) -> list:
    return call(resolve_term(root, key))


def invoke(uri: str, arm: str, args: str = "") -> list:
    return call(call_term(uri, arm, args))


def uri_of(reply: list) -> str:
    if not reply or reply[0] is not True or len(reply) < 2:
        raise AssertionError(f"expected (true, uri): {reply!r}")
    uri = reply[1]
    if not isinstance(uri, str) or not uri.startswith("rho:id:"):
        raise AssertionError(f"not a URI: {uri!r}")
    return uri


def expect(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def rq(value: object) -> str:
    return json.dumps(value)
