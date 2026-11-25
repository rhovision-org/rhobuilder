#!/usr/bin/env python3
"""Safe local-devnet lifecycle and gated onboarding.

This tool only inspects an existing container, starts it, waits for health and (with
``--execute``) runs the existing rhobuilder onboarding commands. It never creates,
removes, resets or restarts a container or volume, and it never prints key material.
"""

from __future__ import annotations

import argparse
import json
import os
import shlex
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Callable

DEFAULT_NODE = "http://localhost:40403"
DEFAULT_CONTAINER = "devnet-bootstrap"
DEFAULT_KEYS = ".ops/devnet/keys.json"
ROOT = Path(__file__).resolve().parent.parent


def probe_node(node: str, timeout: float = 3.0) -> dict[str, Any]:
    """Read the node health. This is a GET; it never submits anything."""
    node = node.rstrip("/")
    try:
        with urllib.request.urlopen(f"{node}/api/v1/status", timeout=timeout) as response:
            status = json.load(response)
    except (urllib.error.URLError, OSError, json.JSONDecodeError) as exc:
        return {
            "reachable": False,
            "healthy": False,
            "node": node,
            "error": str(exc),
        }
    degraded = status.get("degraded", "unknown")
    accepts = bool(status.get("acceptsDeploys", False))
    return {
        "reachable": True,
        "healthy": degraded == "normal" and accepts,
        "node": node,
        "networkId": status.get("networkId"),
        "shardId": status.get("shardId"),
        "latestBlockNumber": status.get("latestBlockNumber"),
        "degraded": degraded,
        "acceptsDeploys": accepts,
    }


def container_status(
    container: str,
    docker: str = "docker",
    run: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
) -> dict[str, Any]:
    """Inspect the named container. Read-only; no create/remove/restart."""
    if shutil.which(docker) is None:
        return {"docker": False, "exists": False, "running": False, "container": container}
    fmt = (
        "{{.State.Status}}|{{.Config.Image}}|"
        "{{if .State.Health}}{{.State.Health.Status}}{{end}}|"
        "{{range $p,$v := .HostConfig.PortBindings}}{{$p}}->{{range $v}}{{.HostPort}}{{end}} {{end}}"
    )
    try:
        result = run(
            [docker, "inspect", container, "--format", fmt],
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return {
            "docker": True,
            "exists": False,
            "running": False,
            "container": container,
            "error": str(exc),
        }
    if result.returncode != 0:
        return {
            "docker": True,
            "exists": False,
            "running": False,
            "container": container,
            "error": (result.stderr or "").strip()[:500],
        }
    parts = (result.stdout or "").strip().split("|")
    state = parts[0] if parts else "unknown"
    return {
        "docker": True,
        "exists": True,
        "running": state == "running",
        "state": state,
        "image": parts[1] if len(parts) > 1 else "",
        "health": parts[2] if len(parts) > 2 else "",
        "ports": (parts[3] if len(parts) > 3 else "").strip(),
        "container": container,
    }


def status(
    container: str = DEFAULT_CONTAINER,
    node: str = DEFAULT_NODE,
    docker: str = "docker",
    probe: Callable[[str], dict[str, Any]] = probe_node,
    run: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
) -> dict[str, Any]:
    node_status = probe(node)
    container_info = container_status(container, docker=docker, run=run)
    healthy = bool(node_status.get("healthy"))
    if not container_info.get("docker"):
        next_command = "install Docker, or set RHOBUILDER_NODE to a remote devnet"
    elif not container_info.get("exists"):
        next_command = (
            "create the devnet container from your RNodeRust checkout; "
            "this tool will not create or reset it"
        )
    elif not container_info.get("running"):
        next_command = f"python3 tools/devnet.py start --container {container}"
    elif not healthy:
        next_command = f"python3 tools/devnet.py wait --node {node}"
    else:
        next_command = "python3 tools/devnet.py onboard --json"
    return {
        "schema": "rhobuilder.devnet-status/v1",
        "ok": healthy,
        "container": container_info,
        "node": node_status,
        "healthy": healthy,
        "canStart": bool(container_info.get("exists") and not container_info.get("running")),
        "nextCommand": next_command,
        "note": "start only starts the existing container; it never creates, resets or removes a volume",
    }


def wait_for_health(
    node: str,
    timeout: float,
    interval: float = 1.0,
    probe: Callable[[str], dict[str, Any]] = probe_node,
) -> dict[str, Any]:
    deadline = time.monotonic() + max(1.0, timeout)
    last = probe(node)
    while not last.get("healthy") and time.monotonic() < deadline:
        time.sleep(min(interval, max(0.0, deadline - time.monotonic())))
        last = probe(node)
    return last


def start(
    container: str = DEFAULT_CONTAINER,
    node: str = DEFAULT_NODE,
    docker: str = "docker",
    dry_run: bool = False,
    timeout: float = 120.0,
    probe: Callable[[str], dict[str, Any]] = probe_node,
    run: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
) -> dict[str, Any]:
    current = status(container, node, docker=docker, probe=probe, run=run)
    if dry_run:
        return {
            "schema": "rhobuilder.devnet-start/v1",
            "ok": True,
            "dryRun": True,
            "command": [docker, "start", container],
            "status": current,
            "note": "dry run; nothing changed",
        }
    if not current["container"].get("docker"):
        return {
            "schema": "rhobuilder.devnet-start/v1",
            "ok": False,
            "error": "docker is not installed",
            "status": current,
        }
    if not current["container"].get("exists"):
        return {
            "schema": "rhobuilder.devnet-start/v1",
            "ok": False,
            "error": "the devnet container does not exist; create it from your RNodeRust checkout",
            "status": current,
        }
    if current["container"].get("running") and current.get("healthy"):
        return {
            "schema": "rhobuilder.devnet-start/v1",
            "ok": True,
            "state": "already_running",
            "status": current,
        }
    if not current["container"].get("running"):
        try:
            result = run([docker, "start", container], capture_output=True, text=True, timeout=30)
        except (OSError, subprocess.TimeoutExpired) as exc:
            return {
                "schema": "rhobuilder.devnet-start/v1",
                "ok": False,
                "error": str(exc),
                "status": current,
            }
        if result.returncode != 0:
            return {
                "schema": "rhobuilder.devnet-start/v1",
                "ok": False,
                "error": (result.stderr or "").strip()[:500],
                "status": current,
            }
    node_status = wait_for_health(node, timeout, probe=probe)
    return {
        "schema": "rhobuilder.devnet-start/v1",
        "ok": bool(node_status.get("healthy")),
        "state": "started" if node_status.get("healthy") else "unhealthy",
        "node": node_status,
    }


def resolve_cli(explicit: str | None = None) -> Path | None:
    candidates = []
    if explicit:
        candidates.append(Path(explicit))
    if os.environ.get("RHOBUILDER_BIN"):
        candidates.append(Path(os.environ["RHOBUILDER_BIN"]))
    candidates.extend(
        [
            ROOT / "bin" / "rhobuilder",
            ROOT / "target" / "release" / "rhobuilder",
            ROOT / "target" / "debug" / "rhobuilder",
        ]
    )
    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return candidate
    return None


def onboarding_commands(cli: Path, keys: Path, force: bool = False) -> list[list[str]]:
    commands = []
    keygen = [str(cli), "--json", "baseline", "keygen", "--network", "devnet", "--out", str(keys)]
    if force:
        keygen.append("--force")
    commands.append(keygen)
    commands.append(
        [str(cli), "--json", "baseline", "fund", "--network", "devnet", "--keys", str(keys), "--execute"]
    )
    commands.append(
        [str(cli), "--json", "baseline", "run", "--network", "devnet", "--keys", str(keys), "--execute"]
    )
    commands.append(
        [str(cli), "--json", "baseline", "verify", "--network", "devnet", "--deep", "--keys", str(keys)]
    )
    return commands


def _brief(document: str) -> str:
    text = document.strip()
    if not text:
        return ""
    try:
        value = json.loads(text)
    except json.JSONDecodeError:
        return text[:500]
    if isinstance(value, dict):
        error = value.get("error")
        if isinstance(error, dict):
            return str(error.get("hint") or error.get("message") or "command failed")[:500]
        if "ok" in value:
            return f"ok={value.get('ok')}"
    return text[:500]


def onboard(
    container: str = DEFAULT_CONTAINER,
    node: str = DEFAULT_NODE,
    keys: Path = Path(DEFAULT_KEYS),
    cli: Path | None = None,
    execute: bool = False,
    force: bool = False,
    start_node: bool = True,
    timeout: float = 120.0,
    docker: str = "docker",
    probe: Callable[[str], dict[str, Any]] = probe_node,
    run: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
) -> dict[str, Any]:
    resolved = cli or resolve_cli()
    if resolved is None:
        return {
            "schema": "rhobuilder.devnet-onboard/v1",
            "ok": False,
            "error": "rhobuilder binary not found; use --rhobuilder, RHOBUILDER_BIN, bin/rhobuilder or target/release/rhobuilder",
        }
    commands = onboarding_commands(resolved, keys, force=force)
    if not execute:
        return {
            "schema": "rhobuilder.devnet-onboard/v1",
            "ok": True,
            "execute": False,
            "keys": str(keys),
            "keygenNeeded": force or not keys.is_file(),
            "commands": [shlex.join(command) for command in commands],
            "note": "dry run; pass --execute to run the devnet onboarding. Testnet is refused.",
        }
    current = status(container, node, docker=docker, probe=probe, run=run)
    if not current["container"].get("exists"):
        return {
            "schema": "rhobuilder.devnet-onboard/v1",
            "ok": False,
            "error": "the local devnet container is missing; create it from your RNodeRust checkout",
            "status": current,
        }
    if start_node and not current.get("healthy"):
        started = start(
            container, node, docker=docker, timeout=timeout, probe=probe, run=run
        )
        if not started.get("ok"):
            return {
                "schema": "rhobuilder.devnet-onboard/v1",
                "ok": False,
                "error": "the local devnet is not healthy",
                "start": started,
            }
        current = status(container, node, docker=docker, probe=probe, run=run)
    if not current.get("healthy"):
        return {
            "schema": "rhobuilder.devnet-onboard/v1",
            "ok": False,
            "error": "the local devnet is not healthy; refusing to onboard",
            "status": current,
        }
    steps = []
    for command in commands:
        name = command[3] if len(command) > 3 else "command"
        if name == "keygen" and keys.is_file() and not force:
            steps.append({"step": "keygen", "status": "skipped", "detail": "key file already exists"})
            continue
        try:
            result = run(command, capture_output=True, text=True, timeout=600)
        except (OSError, subprocess.TimeoutExpired) as exc:
            steps.append({"step": name, "status": "fail", "detail": str(exc)[:500]})
            return {
                "schema": "rhobuilder.devnet-onboard/v1",
                "ok": False,
                "error": f"{name} failed to start",
                "steps": steps,
            }
        detail = _brief(result.stdout or result.stderr or "")
        steps.append(
            {
                "step": name,
                "status": "pass" if result.returncode == 0 else "fail",
                "detail": detail,
            }
        )
        if result.returncode != 0:
            return {
                "schema": "rhobuilder.devnet-onboard/v1",
                "ok": False,
                "error": f"{name} failed",
                "steps": steps,
            }
    return {
        "schema": "rhobuilder.devnet-onboard/v1",
        "ok": True,
        "execute": True,
        "keys": str(keys),
        "steps": steps,
        "note": "devnet only; no live network was touched",
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit one JSON envelope")
    parser.add_argument("--node", default=os.environ.get("RHOBUILDER_NODE", DEFAULT_NODE))
    parser.add_argument(
        "--container", default=os.environ.get("RHOBUILDER_CONTAINER", DEFAULT_CONTAINER)
    )
    parser.add_argument("--docker", default="docker")
    sub = parser.add_subparsers(dest="command", required=True)

    def add_json(parser: argparse.ArgumentParser) -> None:
        parser.add_argument("--json", action="store_true", default=argparse.SUPPRESS)

    status_parser = sub.add_parser("status", help="read-only container and node status")
    add_json(status_parser)
    status_parser.set_defaults(action="status")

    start_parser = sub.add_parser("start", help="start the existing devnet container")
    add_json(start_parser)
    start_parser.add_argument("--dry-run", action="store_true")
    start_parser.add_argument("--timeout", type=float, default=120.0)
    start_parser.set_defaults(action="start")

    wait_parser = sub.add_parser("wait", help="wait for node health")
    add_json(wait_parser)
    wait_parser.add_argument("--timeout", type=float, default=120.0)
    wait_parser.set_defaults(action="wait")

    onboard_parser = sub.add_parser("onboard", help="dry-run or execute devnet onboarding")
    add_json(onboard_parser)
    onboard_parser.add_argument("--execute", action="store_true")
    onboard_parser.add_argument("--keys", type=Path, default=Path(DEFAULT_KEYS))
    onboard_parser.add_argument("--rhobuilder", default=None)
    onboard_parser.add_argument("--force", action="store_true")
    onboard_parser.add_argument("--no-start", action="store_true")
    onboard_parser.add_argument("--timeout", type=float, default=120.0)
    onboard_parser.set_defaults(action="onboard")

    args = parser.parse_args(argv)
    if args.action == "status":
        value = status(args.container, args.node, docker=args.docker)
        code = 0 if value.get("healthy") else 2
    elif args.action == "start":
        value = start(
            args.container,
            args.node,
            docker=args.docker,
            dry_run=args.dry_run,
            timeout=args.timeout,
        )
        code = 0 if value.get("ok") else 2
    elif args.action == "wait":
        value = wait_for_health(args.node, args.timeout)
        code = 0 if value.get("healthy") else 4
    else:
        value = onboard(
            args.container,
            args.node,
            keys=args.keys,
            cli=resolve_cli(args.rhobuilder),
            execute=args.execute,
            force=args.force,
            start_node=not args.no_start,
            timeout=args.timeout,
            docker=args.docker,
        )
        code = 0 if value.get("ok") else 2
    if args.json:
        print(json.dumps(value, indent=2))
    else:
        print(f"devnet {args.action}: {'ok' if code == 0 else 'setup required'}")
        print(json.dumps(value, indent=2))
    return code


if __name__ == "__main__":
    sys.exit(main())
