#!/usr/bin/env python3
"""Launch DeepSeek Harness with rhobuilder's plugin, skills and presets.

Adapted from the azoth integration's launcher (AGPL-3.0-or-later, commit 8b7810ee):
the generation/`--check` split, shipped-profile overlays and skill export shape are
reused; the azoth-specific prompts, HAZOP presets and calculation bridge are not.

    python3 tools/launch_dsh.py --check --json
    python3 tools/launch_dsh.py --print-config headless
    python3 tools/launch_dsh.py web
    python3 tools/launch_dsh.py headless "Design a bookable service"

`--check` and `--print-config` are read-only. A real launch exports skills to
`.dsh/skills/`, writes `.dsh/patches/rhobuilder-<mode>.yml`, and starts the shipped `web`
or `headless` profile with `--patch`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import devnet

ROOT = Path(__file__).resolve().parent.parent
CONFIG_PATH = ROOT / "agents" / "deepseek" / "harness.toml"
CHECK_SCHEMA = "rhobuilder.dsh-check/v1"
MODES = ("web", "headless")


@dataclass(frozen=True)
class Command:
    argv: list[str]
    kind: str

    def with_args(self, *extra: str) -> "Command":
        return Command([*self.argv, *extra], self.kind)

    def __str__(self) -> str:
        return " ".join(self.argv)


def which(name: str) -> Path | None:
    found = shutil.which(name)
    return Path(found) if found else None


def load_config(path: Path = CONFIG_PATH) -> dict[str, Any]:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def resolve_rhobuilder(args: argparse.Namespace) -> Command | None:
    candidates: list[Path] = []
    if args.rhobuilder:
        candidates.append(Path(args.rhobuilder))
    if os.environ.get("RHOBUILDER_BIN"):
        candidates.append(Path(os.environ["RHOBUILDER_BIN"]))
    candidates.extend(
        [ROOT / "target" / "release" / "rhobuilder", ROOT / "target" / "debug" / "rhobuilder"]
    )
    found = which("rhobuilder")
    if found:
        candidates.append(found)
    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return Command([str(candidate.resolve())], "binary")
    if which("cargo"):
        return Command(
            ["cargo", "run", "-q", "-p", "rhobuilder-cli", "--"], "cargo"
        )
    return None


def resolve_dsh(args: argparse.Namespace, config: dict[str, Any]) -> Command | None:
    candidates: list[str] = []
    if args.dsh:
        candidates.append(args.dsh)
    for env in ("RHOBUILDER_DSH", "DSH_BIN", "AZOTH_DSH"):
        if os.environ.get(env):
            candidates.append(os.environ[env])
    found = which("dsh")
    if found:
        candidates.append(str(found))
    for candidate in candidates:
        path = Path(candidate)
        if path.is_file() and os.access(path, os.X_OK):
            return Command([str(path.resolve())], "binary")
        # Allow a command with arguments, e.g. "npx --yes @deepseek-ai/dsh@x".
        parts = candidate.split()
        if len(parts) > 1 and which(parts[0]):
            return Command(parts, "command")
    if not args.no_npx and which("npx"):
        version = str(config["harness"]["dsh_version"])
        return Command(["npx", "--yes", f"@deepseek-ai/dsh@{version}"], "npx")
    return None


def resolve_node(args: argparse.Namespace) -> Path | None:
    if args.node:
        return Path(args.node)
    return which("node")


def child_env(args: argparse.Namespace, mode: str) -> dict[str, str]:
    env = dict(os.environ)
    env.setdefault("DSH_PERMISSION_MODE", args.permission_mode)
    env.setdefault("DSH_HOME", str(Path(args.dsh_home).resolve()))
    env.setdefault("PYTHONDONTWRITEBYTECODE", "1")
    return env


def _scalar(value: Any) -> str:
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, (int, float)):
        return str(value)
    text = str(value)
    if not text or any(ch in text for ch in ":#[]{}\n\"'&*!|>%@`"):
        return json.dumps(text)
    return text


def _render_mapping(value: dict[str, Any], indent: int) -> list[str]:
    lines: list[str] = []
    pad = " " * indent
    for key, item in value.items():
        if isinstance(item, dict):
            if not item:
                lines.append(f"{pad}{key}: {{}}")
            else:
                lines.append(f"{pad}{key}:")
                lines.extend(_render_mapping(item, indent + 2))
        elif isinstance(item, list):
            if not item:
                lines.append(f"{pad}{key}: []")
            else:
                lines.append(f"{pad}{key}:")
                lines.extend(_render_list(item, indent + 2))
        else:
            lines.append(f"{pad}{key}: {_scalar(item)}")
    return lines


def _render_list(value: list[Any], indent: int) -> list[str]:
    lines: list[str] = []
    pad = " " * indent
    for item in value:
        if isinstance(item, dict):
            rendered = _render_mapping(item, indent + 2)
            if not rendered:
                lines.append(f"{pad}- {{}}")
                continue
            first = rendered[0].lstrip()
            lines.append(f"{pad}- {first}")
            lines.extend(rendered[1:])
        elif isinstance(item, list):
            lines.append(f"{pad}-")
            lines.extend(_render_list(item, indent + 2))
        else:
            lines.append(f"{pad}- {_scalar(item)}")
    return lines


def render_yaml(rows: list[dict[str, Any]]) -> str:
    lines: list[str] = []
    for row in rows:
        rendered = _render_mapping(row, 2)
        if not rendered:
            lines.append("- {}")
            continue
        first = rendered[0].lstrip()
        lines.append(f"- {first}")
        lines.extend(rendered[1:])
    return "\n".join(lines) + "\n"


def _persona(prompt: Path) -> dict[str, Any]:
    return {
        "id": "persona",
        "name": "@deepseek-ai/dsh-persona",
        "config": {"prefix": prompt.read_text(encoding="utf-8").rstrip("\n"), "suffix": ""},
    }


def _role_plugins() -> list[dict[str, Any]]:
    return [
        {
            "id": "agent-instructions",
            "name": "@deepseek-ai/dsh-agent-instructions",
            "config": {"maxBytes": 65536},
        },
        {
            "id": "skill-filesystem",
            "name": "@deepseek-ai/dsh-skill-filesystem",
            "config": {"includeDefaultRoots": True},
        },
        {
            "id": "tool-skill",
            "name": "@deepseek-ai/dsh-tool-skill",
            "config": {"catalogDescriptionMaxLength": 500},
        },
        {"id": "tool-fs", "name": "@deepseek-ai/dsh-tool-fs", "config": {"applyPatch": False}},
        {
            "id": "tool-fs-search",
            "name": "@deepseek-ai/dsh-tool-fs-search",
            "config": {"sampleOverCapGlobResults": False},
        },
        {"id": "tool-bash", "name": "@deepseek-ai/dsh-tool-bash"},
        {
            "id": "tool-todo",
            "name": "@deepseek-ai/dsh-tool-todo",
            "config": {"allowParallelInProgress": True},
        },
    ]


def _base_preset() -> dict[str, Any]:
    """The default rhobuilder persona, available without Agent Teams."""
    prompt = ROOT / "agents" / "deepseek" / "prompts" / "architect.md"
    return {
        "id": "preset-rhobuilder",
        "name": "@deepseek-ai/dsh-agent-preset",
        "config": {
            "id": "rhobuilder",
            "order": 5,
            "plugins": [_persona(prompt), *_role_plugins()],
        },
    }


def _presets(args: argparse.Namespace, config: dict[str, Any]) -> list[dict[str, Any]]:
    roles = config["presets"]
    architect = str(roles["architect"]["id"])
    reviewer = str(roles["reviewer"]["id"])
    prover = str(roles["prover"]["id"])
    rows: list[dict[str, Any]] = []
    for slug in ("architect", "reviewer", "prover"):
        meta = roles[slug]
        prompt = ROOT / "agents" / "deepseek" / str(meta["prompt"])
        plugins = [_persona(prompt), *_role_plugins()]
        if slug == "architect":
            plugins.extend(
                [
                    {
                        "id": "tool-subagent-control",
                        "name": "@deepseek-ai/dsh-tool-subagent-control",
                        "config": {"interrupt": False},
                    },
                    {
                        "id": "tool-subagent",
                        "name": "@deepseek-ai/dsh-tool-subagent",
                        "config": {
                            "provider": "spawn",
                            "toolName": "subagent",
                            "modelSelection": False,
                            "allowedAgentPresets": [reviewer, prover],
                            "maxDepth": 1,
                        },
                    },
                ]
            )
        rows.append(
            {
                "id": f"preset-{meta['id']}",
                "name": "@deepseek-ai/dsh-agent-preset",
                "config": {
                    "id": str(meta["id"]),
                    "order": int(meta["order"]),
                    "plugins": plugins,
                },
            }
        )
    return rows


def _plugin_row(rhobuilder: Command, config: dict[str, Any]) -> dict[str, Any]:
    index = ROOT / "agents" / "deepseek" / "plugin" / "index.js"
    return {
        "id": "rhobuilder-tools",
        "name": str(index.resolve()),
        "config": {
            "command": rhobuilder.argv[0],
            "args": rhobuilder.argv[1:],
            "cwd": str(ROOT),
            "timeoutMs": int(config["plugin"]["timeout_ms"]),
            "maxOutputBytes": int(config["plugin"]["max_output_bytes"]),
            "registry": str((ROOT / "registry").resolve()),
        },
    }


def _ui_row(rhobuilder: Command, config: dict[str, Any]) -> dict[str, Any]:
    ui = ROOT / "agents" / "deepseek" / "ui" / "index.js"
    if not ui.is_file():
        raise FileNotFoundError(f"UI plugin is missing: {ui}")
    return {
        "id": "rhobuilder-ui",
        "name": str(ui.resolve()),
        "config": {
            "command": rhobuilder.argv[0],
            "args": rhobuilder.argv[1:],
            "cwd": str(ROOT),
            "registry": str((ROOT / "registry").resolve()),
        },
    }


def _mcp_row(args: argparse.Namespace, config: dict[str, Any]) -> dict[str, Any] | None:
    if not args.with_mcp:
        return None
    server = ROOT / "agents" / "deepseek" / "mcp" / "dist" / "index.js"
    if not server.is_file():
        server = ROOT / "mcp" / "server.mjs"
    if not server.is_file():
        raise FileNotFoundError(
            "MCP was requested but no server exists (P7); omit --with-mcp"
        )
    node = resolve_node(args)
    if node is None:
        raise FileNotFoundError("MCP was requested but node is not on PATH")
    rhobuilder = resolve_rhobuilder(args)
    if rhobuilder is None:
        raise FileNotFoundError("MCP was requested but no rhobuilder CLI exists")
    return {
        "id": "mcp-rhobuilder",
        "name": "@deepseek-ai/dsh-mcp-client",
        "config": {
            "serverName": "rhobuilder",
            "transport": "stdio",
            "command": str(node),
            "args": [
                str(server.resolve()),
                "--binary",
                *rhobuilder.argv,
            ],
            "cwd": str(ROOT),
            "toolCallTimeoutMs": int(config["mcp"]["tool_call_timeout_ms"]),
            "startupTimeoutMs": int(config["mcp"]["startup_timeout_ms"]),
            "closeTimeoutMs": int(config["mcp"]["close_timeout_ms"]),
            "failOnStartupError": True,
        },
    }


def build_overlay(
    args: argparse.Namespace,
    config: dict[str, Any],
    rhobuilder: Command,
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    inserts: list[dict[str, Any]] = [_plugin_row(rhobuilder, config)]
    ui_enabled = args.mode == "web" and not getattr(args, "no_ui", False)
    if ui_enabled:
        inserts.append(_ui_row(rhobuilder, config))
    mcp = _mcp_row(args, config)
    if mcp is not None:
        inserts.append(mcp)
    rows.append({"insert": inserts})
    if ui_enabled:
        # The UI supplies both sidebar brand slots; the shipped brand would shadow them.
        rows.append({"id": "ui-brand-official", "disabled": True})

    if args.base_url:
        key_env = args.api_key_env or "RHOBUILDER_DSH_API_KEY"
        provider = "rhobuilder-gateway"
        rows.append(
            {
                "id": "llm-pi-ai",
                "config": {
                    "providers": {
                        provider: {
                            "apiKeyEnv": key_env,
                            "api": args.api,
                            "baseURL": args.base_url,
                            "models": [{"id": args.model}],
                        }
                    }
                },
            }
        )
    else:
        provider = str(config["harness"]["default_provider"])
        model: dict[str, Any] = {
            "apiKeyEnv": args.api_key_env or "DEEPSEEK_API_KEY",
            "models": [{"id": args.model}],
        }
        if args.reasoning_effort:
            model["reasoningEffort"] = args.reasoning_effort
        rows.append({"id": "llm-deepseek", "config": model})
    rows.append(
        {"id": "agent-default-model", "config": {"provider": provider, "model": args.model}}
    )

    architect = str(config["presets"]["architect"]["id"])
    default_preset = architect if args.study == "build" else "rhobuilder"
    base = _base_preset()
    if args.mode == "web":
        if args.study != "build":
            rows.append({"id": "agent-preset-registry", "config": {"default": default_preset}})
        rows.append({"insert": [base]})
    else:
        rows.append(
            {
                "insert": [
                    {
                        "id": "agent-preset-registry",
                        "name": "@deepseek-ai/dsh-agent-preset-registry",
                        "config": {"default": default_preset},
                    },
                    base,
                ]
            }
        )

    if args.study == "build":
        rows.append(
            {
                "id": "agent-team",
                "config": {
                    "maxTasks": int(config["harness"]["max_tasks"]),
                    "defaultEnabled": True,
                },
            }
        )
        presets = _presets(args, config)
        if args.mode == "web":
            rows.append({"id": "agent-preset-registry", "config": {"default": architect}})
            rows.append({"insert": presets})
        else:
            rows.append({"insert": presets})
    return rows


def _skill_selection(args: argparse.Namespace, config: dict[str, Any]) -> list[str] | None:
    if args.skills == "none":
        return []
    if args.skills == "curated":
        return [*config["skills"]["curated"], "rhobuilder-build"]
    return None


def _skill_files(args: argparse.Namespace, config: dict[str, Any]) -> dict[Path, str]:
    import export_skills

    names = _skill_selection(args, config)
    if names == []:
        return {}
    return export_skills.expected(ROOT, ["dsh"], names, source_root=ROOT)


def _skill_hashes(files: dict[Path, str]) -> dict[str, str]:
    return {
        path.parent.name: hashlib.sha256(text.encode("utf-8")).hexdigest()
        for path, text in files.items()
    }


def export_skills_for_launch(args: argparse.Namespace, config: dict[str, Any]) -> int:
    files = _skill_files(args, config)
    if not files:
        return 0
    out = ROOT / ".dsh" / "skills"
    out.mkdir(parents=True, exist_ok=True)
    for path, text in files.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    current = _skill_hashes(files)
    expected_names = {path.parent.name for path in files}
    for child in out.iterdir():
        if not child.is_dir() or child.name in expected_names:
            continue
        stale = child / "SKILL.md"
        if stale.is_file():
            stale.unlink()
        try:
            child.rmdir()
        except OSError:
            # Supporting files remain; leave the directory and report it as an orphan.
            pass
    manifest = out / ".manifest.json"
    manifest.write_text(
        json.dumps({"schema": "rhobuilder.skills/v1", "skills": current}, indent=2, sort_keys=True)
        + "\n",
        encoding="utf-8",
    )
    return len(files)


def skills_report(args: argparse.Namespace, config: dict[str, Any]) -> tuple[str, str]:
    import export_skills

    try:
        files = _skill_files(args, config)
    except (KeyError, OSError, ValueError) as exc:
        return "fail", str(exc)
    if not files:
        return "ok", "skill export disabled"
    out = ROOT / ".dsh" / "skills"
    if not out.is_dir():
        return "warn", "skills are not exported yet; a launch writes them"
    report = export_skills.check_expected(files, [out])
    if not report["ok"]:
        return "setup", "; ".join((report["missing"] + report["stale"])[:5])
    if report["orphans"]:
        return "warn", f"orphaned skills: {', '.join(report['orphans'][:5])}"
    return "ok", f"{len(files)} skills"


# ---------------------------------------------------------------------------------------
# checks


@dataclass
class Check:
    name: str
    status: str
    detail: str
    hint: str | None = None


def _run(
    command: list[str] | Command,
    *,
    env: dict[str, str] | None = None,
    timeout: float = 30,
    stdin: str | None = None,
) -> subprocess.CompletedProcess[str]:
    argv = command.argv if isinstance(command, Command) else command
    return subprocess.run(
        argv,
        env=env,
        input=stdin,
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
    )


def _version(text: str) -> str | None:
    for token in text.replace("v", " ").split():
        parts = token.split(".")
        if len(parts) >= 2 and parts[0].isdigit() and parts[1].isdigit():
            return token.strip("(),")
    return None


def _probe_node(args: argparse.Namespace, config: dict[str, Any]) -> Check:
    node = resolve_node(args)
    if node is None:
        return Check("node", "fail", "node not found")
    try:
        proc = _run([str(node), "--version"], timeout=args.timeout)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return Check("node", "fail", str(exc))
    version = proc.stdout.strip().lstrip("v")
    floor = str(config["harness"]["tested_node"])
    try:
        found = tuple(int(part) for part in version.split(".")[:2])
        need = tuple(int(part) for part in floor.split(".")[:2])
    except ValueError:
        return Check("node", "warn", version or "unknown")
    if found < need:
        return Check("node", "fail", f"{version} is below the tested floor {floor}")
    return Check("node", "ok", version)


def _probe_dsh(args: argparse.Namespace, config: dict[str, Any]) -> Check:
    dsh = resolve_dsh(args, config)
    if dsh is None:
        return Check("dsh", "fail", "dsh not found and npx is unavailable")
    if dsh.kind == "npx":
        return Check(
            "dsh",
            "warn",
            f"{dsh} (fetches on first launch)",
            "The first launch needs network access; install DSH locally for an offline setup",
        )
    try:
        proc = _run(dsh.with_args("--version"), timeout=args.timeout)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return Check("dsh", "fail", str(exc))
    if proc.returncode != 0:
        return Check("dsh", "fail", (proc.stderr or proc.stdout).strip()[-200:])
    found = _version(proc.stdout + proc.stderr)
    if found is None:
        return Check("dsh", "warn", f"cannot parse version from {proc.stdout.strip()!r}")
    major_minor = tuple(int(part) for part in found.split("-")[0].split(".")[:2])
    if major_minor != (0, 2):
        return Check("dsh", "warn", f"{found} is outside the tested 0.2 line")
    return Check("dsh", "ok", f"{found} ({dsh})")


def _probe_cli(args: argparse.Namespace) -> Check:
    cli = resolve_rhobuilder(args)
    if cli is None:
        return Check("rhobuilder", "fail", "no rhobuilder CLI and no cargo")
    if cli.kind == "cargo":
        return Check(
            "rhobuilder",
            "warn",
            f"{cli} (builds on launch)",
            "The first launch may clone rcash-sdk and build; install a packaged binary for a faster start",
        )
    try:
        proc = _run(cli.with_args("--json", "version"), timeout=args.timeout)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return Check("rhobuilder", "fail", str(exc))
    if proc.returncode != 0:
        return Check("rhobuilder", "fail", (proc.stderr or proc.stdout).strip()[-200:])
    try:
        document = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return Check("rhobuilder", "fail", "version did not print JSON")
    return Check(
        "rhobuilder",
        "ok",
        f"{document.get('product', '?')} {document.get('version', '?')}",
    )


def _probe_plugin(args: argparse.Namespace) -> Check:
    plugin = ROOT / "agents" / "deepseek" / "plugin"
    index = plugin / "index.js"
    bridge = plugin / "bridge.js"
    if not index.is_file() or not bridge.is_file():
        return Check("plugin", "fail", "plugin/index.js or bridge.js is missing")
    node = resolve_node(args)
    if node is None:
        return Check("plugin", "fail", "node not found; plugin cannot be checked")
    try:
        for source in (index, bridge):
            proc = _run([str(node), "--check", str(source)], timeout=args.timeout)
            if proc.returncode != 0:
                return Check("plugin", "fail", proc.stderr.strip()[-300:])
        tests = sorted((plugin / "test").glob("*.test.mjs"))
        if tests:
            proc = _run(
                [str(node), "--test", *(str(path) for path in tests)],
                timeout=max(args.timeout, 90),
            )
            if proc.returncode != 0:
                return Check("plugin", "fail", proc.stderr.strip()[-300:])
    except (OSError, subprocess.TimeoutExpired) as exc:
        return Check("plugin", "fail", str(exc))
    suffix = f", {len(tests)} node tests" if tests else ""
    return Check("plugin", "ok", f"{index.name}{suffix}")


def _probe_ui(args: argparse.Namespace) -> Check:
    if getattr(args, "no_ui", False):
        return Check("ui", "ok", "disabled by --no-ui")
    ui = ROOT / "agents" / "deepseek" / "ui"
    package = ui / "package.json"
    if not package.is_file():
        return Check("ui", "fail", "agents/deepseek/ui/package.json is missing")
    try:
        manifest = json.loads(package.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return Check("ui", "fail", f"invalid package.json: {exc}")
    if manifest.get("name") != "rhobuilder-dsh-ui":
        return Check("ui", "fail", "UI package name must be rhobuilder-dsh-ui")
    exports = manifest.get("exports", {})
    if not isinstance(exports, dict) or not isinstance(exports.get("./client"), str):
        return Check("ui", "fail", "UI package needs exports['./client']")
    if manifest.get("dsh", {}).get("client", {}).get("platform") != "web":
        return Check("ui", "fail", "UI package needs dsh.client.platform=web")
    node = resolve_node(args)
    if node is None:
        return Check("ui", "fail", "node not found; UI cannot be checked")
    try:
        for source in (ui / "index.js", ui / "client.js"):
            if not source.is_file():
                return Check("ui", "fail", f"{source.name} is missing")
            proc = _run([str(node), "--check", str(source)], timeout=args.timeout)
            if proc.returncode != 0:
                return Check("ui", "fail", proc.stderr.strip()[-300:])
    except (OSError, subprocess.TimeoutExpired) as exc:
        return Check("ui", "fail", str(exc))
    return Check("ui", "ok", f"{manifest['name']} client bundle")


def _probe_templates(args: argparse.Namespace) -> Check:
    templates = ROOT / "templates" / "dapp" / "common"
    if not templates.is_dir():
        return Check("templates", "fail", "templates/dapp/common is missing")
    return Check("templates", "ok", str(templates.relative_to(ROOT)))


def _probe_tool(name: str, missing_status: str, hint: str) -> Check:
    found = shutil.which(name)
    if found is None:
        return Check(name, missing_status, f"{name} not found", hint)
    return Check(name, "ok", found)


def _probe_devnet(args: argparse.Namespace) -> Check:
    node = os.environ.get("RHOBUILDER_NODE", devnet.DEFAULT_NODE).rstrip("/")
    container = os.environ.get("RHOBUILDER_CONTAINER", devnet.DEFAULT_CONTAINER)
    value = devnet.status(container, node)
    node_status = value.get("node", {})
    if not node_status.get("reachable"):
        return Check(
            "devnet",
            "setup",
            f"{node} is unreachable: {node_status.get('error', 'unknown error')}",
            value.get("nextCommand")
            or "Start or recover the local devnet; the dashboard will show not_run until it is healthy",
        )
    if not node_status.get("healthy"):
        return Check(
            "devnet",
            "setup",
            f"{node} is {node_status.get('degraded', 'unknown')}; deploys are not healthy",
            value.get("nextCommand")
            or "Restart the devnet or wait for the resource latch to clear",
        )
    detail = f"{node} normal at block {node_status.get('latestBlockNumber', '?')}"
    if not value.get("container", {}).get("exists"):
        detail += "; the local container is missing (remote node?)"
    return Check("devnet", "ok", detail, value.get("nextCommand"))


def _probe_registry(args: argparse.Namespace) -> Check:
    registry = ROOT / "registry" / "primitives.json"
    if not registry.is_file():
        return Check(
            "registry",
            "fail",
            "registry/primitives.json is missing",
            "Use a complete release tarball or run from the repository root",
        )
    try:
        document = json.loads(registry.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return Check("registry", "fail", f"invalid JSON: {exc}")
    primitives = document.get("primitives", [])
    return Check("registry", "ok", f"{len(primitives)} primitive(s)")


def _probe_mcp(args: argparse.Namespace, config: dict[str, Any]) -> Check | None:
    if not args.with_mcp:
        return None
    try:
        row = _mcp_row(args, config)
    except FileNotFoundError as exc:
        return Check("mcp", "fail", str(exc))
    assert row is not None
    node = resolve_node(args)
    server = row["config"]["args"][0]
    requests = [
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "rhobuilder-check", "version": "0.1.0"},
            },
        },
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
    ]
    payload = "\n".join(json.dumps(request) for request in requests) + "\n"
    try:
        proc = _run(
            [str(node), server, "--binary", *(row["config"]["args"][2:])],
            timeout=max(args.timeout, 20),
            stdin=payload,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return Check("mcp", "fail", f"startup failed: {exc}")
    tools = 0
    for line in proc.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("id") == 2:
            tools = len(message.get("result", {}).get("tools", []))
    if tools == 0:
        return Check("mcp", "fail", "no tools/list answer (the server is not speaking MCP)")
    return Check("mcp", "ok", f"{tools} tools")


def _probe_key(args: argparse.Namespace) -> Check:
    key_env = args.api_key_env or (
        "RHOBUILDER_DSH_API_KEY" if args.base_url else "DEEPSEEK_API_KEY"
    )
    if os.environ.get(key_env):
        return Check("api-key", "ok", f"{key_env} is set")
    if args.base_url:
        return Check(
            "api-key",
            "setup",
            f"{key_env} is not set (the gateway needs it for chat)",
            f"Export {key_env} or save it in DSH Settings → Models",
        )
    return Check(
        "api-key",
        "setup",
        f"{key_env} is not set; chat will not answer",
        f"Export {key_env} or save it in DSH Settings → Models",
    )


def run_checks(args: argparse.Namespace, config: dict[str, Any]) -> list[Check]:
    checks = [
        _probe_node(args, config),
        _probe_dsh(args, config),
        _probe_cli(args),
        _probe_plugin(args),
        _probe_ui(args),
    ]
    status, detail = skills_report(args, config)
    checks.append(Check("skills", status, detail, "A launch exports the curated skills"))
    checks.append(_probe_registry(args))
    checks.append(_probe_templates(args))
    checks.append(
        _probe_tool(
            "cargo",
            "warn",
            "Source builds are unavailable; use the packaged binary",
        )
    )
    checks.append(_probe_devnet(args))
    checks.append(_probe_tool("lake", "setup", "Install Lean/lake for formal checks"))
    checks.append(_probe_tool("mdbook", "warn", "Install mdBook to build the docs"))
    mcp = _probe_mcp(args, config)
    if mcp is not None:
        checks.append(mcp)
    checks.append(_probe_key(args))
    return checks


def _launch_argv(
    args: argparse.Namespace, dsh: Command, patch: Path, task: str | None
) -> list[str]:
    argv = [*dsh.argv, "--profile", args.mode, "--patch", str(patch)]
    if args.mode == "web":
        if getattr(args, "port", None) is not None:
            argv.extend(["--port", str(args.port)])
        if getattr(args, "no_open", False):
            argv.append("--no-open")
    if task:
        argv.append(task)
    return argv


def _launch(args: argparse.Namespace, config: dict[str, Any]) -> int:
    dsh = resolve_dsh(args, config)
    if dsh is None:
        print("launch_dsh: dsh not found and npx is unavailable", file=sys.stderr)
        return 1
    rhobuilder = resolve_rhobuilder(args)
    if rhobuilder is None:
        print("launch_dsh: no rhobuilder CLI and no cargo", file=sys.stderr)
        return 1
    try:
        overlay = build_overlay(args, config, rhobuilder)
    except FileNotFoundError as exc:
        print(f"launch_dsh: {exc}", file=sys.stderr)
        return 1

    written = export_skills_for_launch(args, config)
    patches = ROOT / ".dsh" / "patches"
    patches.mkdir(parents=True, exist_ok=True)
    patch = patches / f"rhobuilder-{args.mode}.yml"
    patch.write_text(render_yaml(overlay), encoding="utf-8")
    env = child_env(args, args.mode)
    argv = _launch_argv(args, dsh, patch, " ".join(args.task) if args.task else None)
    if args.dry_run:
        print(render_yaml(overlay), end="")
        print(f"# argv: {argv}", file=sys.stderr)
        print(f"# skills: {written}", file=sys.stderr)
        return 0
    try:
        proc = subprocess.run(argv, env=env, check=False)
    except OSError as exc:
        print(f"launch_dsh: {exc}", file=sys.stderr)
        return 1
    return proc.returncode


def parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("mode", nargs="?", choices=[*MODES, "check"], default="check")
    p.add_argument("task", nargs=argparse.REMAINDER, help="text passed to dsh")
    p.add_argument("--root", type=Path, default=ROOT, help="checkout root (default: repo)")
    p.add_argument("--rhobuilder", help="rhobuilder binary path")
    p.add_argument("--node", help="node binary path")
    p.add_argument("--dsh", help="dsh binary/command")
    p.add_argument("--no-npx", action="store_true", help="do not fall back to npx")
    p.add_argument("--dsh-home", default=str(ROOT / ".dsh-home"))
    p.add_argument("--model", default=None)
    p.add_argument("--base-url", default=None)
    p.add_argument("--api", choices=["openai-completions", "openai-responses"], default="openai-completions")
    p.add_argument("--api-key-env", default=None)
    p.add_argument("--reasoning-effort", choices=["off", "low", "high", "max"], default=None)
    p.add_argument("--skills", choices=["curated", "full", "none"], default="curated")
    p.add_argument("--study", choices=["none", "build"], default="none")
    p.add_argument(
        "--permission-mode",
        choices=["workspace-write", "read-only"],
        default="workspace-write",
    )
    p.add_argument("--with-mcp", action="store_true", help="mount the optional MCP server")
    p.add_argument("--no-ui", action="store_true", help="do not mount the rhobuilder web UI")
    p.add_argument("--port", type=int, default=None, help="web UI port (default: DSH 3080)")
    p.add_argument("--no-open", action="store_true", help="do not open the web UI in a browser")
    p.add_argument("--check", action="store_true", help="run read-only checks")
    p.add_argument("--print-config", action="store_true", help="print the overlay only")
    p.add_argument("--dry-run", action="store_true", help="print overlay and argv; write nothing")
    p.add_argument("--json", action="store_true")
    p.add_argument("--timeout", type=float, default=60)
    return p


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    if args.mode == "check":
        args.check = True
    config = load_config()
    if args.model is None:
        args.model = str(config["harness"]["default_model"])
    if args.mode not in MODES:
        args.mode = "headless"
    if args.check:
        checks = run_checks(args, config)
        failed = [check for check in checks if check.status == "fail"]
        setup = [check for check in checks if check.status == "setup"]
        ok = not failed and not setup
        if args.json:
            print(
                json.dumps(
                    {
                        "schema": CHECK_SCHEMA,
                        "ok": ok,
                        "status": "ok" if ok else ("failed" if failed else "setup_required"),
                        "setupRequired": [check.name for check in setup],
                        "checks": [
                            {
                                "name": c.name,
                                "status": c.status,
                                "detail": c.detail,
                                **({"hint": c.hint} if c.hint else {}),
                            }
                            for c in checks
                        ],
                    },
                    indent=2,
                )
            )
        else:
            for check in checks:
                suffix = f" ({check.hint})" if check.hint else ""
                print(f"{check.status:5} {check.name:10} {check.detail}{suffix}")
            print(
                "dsh-check: "
                + ("ok" if ok else ("setup required" if setup and not failed else "failed"))
            )
        if failed:
            return 1
        if setup:
            return 2
        return 0
    if args.print_config or args.dry_run:
        rhobuilder = resolve_rhobuilder(args)
        if rhobuilder is None:
            print("launch_dsh: no rhobuilder CLI and no cargo", file=sys.stderr)
            return 1
        try:
            overlay = build_overlay(args, config, rhobuilder)
        except FileNotFoundError as exc:
            print(f"launch_dsh: {exc}", file=sys.stderr)
            return 1
        print(render_yaml(overlay), end="")
        if args.dry_run:
            dsh = resolve_dsh(args, config)
            patch = ROOT / ".dsh" / "patches" / f"rhobuilder-{args.mode}.yml"
            if dsh is not None:
                print(f"# argv: {_launch_argv(args, dsh, patch, ' '.join(args.task) if args.task else None)}", file=sys.stderr)
            print(f"# env: DSH_PERMISSION_MODE={args.permission_mode} DSH_HOME={Path(args.dsh_home).resolve()}", file=sys.stderr)
        return 0
    return _launch(args, config)


if __name__ == "__main__":
    raise SystemExit(main())
