from __future__ import annotations

import os
import stat
import sys
import tempfile
import unittest
from unittest import mock
from pathlib import Path
from types import SimpleNamespace

TOOLS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(TOOLS))

import launch_dsh  # noqa: E402


def fake_script(directory: Path, name: str, body: str) -> Path:
    path = directory / name
    path.write_text(f"#!/bin/sh\n{body}\n", encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)
    return path


def args_for(**overrides):
    values = {
        "base_url": None,
        "api": "openai-completions",
        "api_key_env": None,
        "model": "deepseek-v4-flash",
        "reasoning_effort": None,
        "study": "none",
        "mode": "headless",
        "with_mcp": False,
        "no_ui": False,
        "port": None,
        "no_open": False,
        "permission_mode": "workspace-write",
        "dsh_home": "/tmp/rhobuilder-dsh-home",
        "task": [],
        "timeout": 10,
        "dsh": None,
        "rhobuilder": None,
        "node": None,
        "no_npx": True,
        "skills": "curated",
        "root": launch_dsh.ROOT,
    }
    values.update(overrides)
    return SimpleNamespace(**values)


class OverlayTests(unittest.TestCase):
    def setUp(self) -> None:
        self.config = launch_dsh.load_config()
        self.cli = launch_dsh.Command(["/usr/bin/rhobuilder"], "binary")

    def test_plugin_row_mounts_the_absolute_index(self) -> None:
        overlay = launch_dsh.build_overlay(args_for(), self.config, self.cli)
        insert = overlay[0]["insert"]
        assert insert[0]["id"] == "rhobuilder-tools"
        assert insert[0]["name"].endswith("agents/deepseek/plugin/index.js")
        assert insert[0]["config"]["command"] == "/usr/bin/rhobuilder"
        assert insert[0]["config"]["registry"].endswith("/registry")

    def test_default_overlay_names_the_model(self) -> None:
        overlay = launch_dsh.build_overlay(args_for(), self.config, self.cli)
        ids = [row.get("id") for row in overlay]
        self.assertIn("llm-deepseek", ids)
        self.assertIn("agent-default-model", ids)
        default = next(row for row in overlay if row.get("id") == "agent-default-model")
        assert default["config"]["model"] == "deepseek-v4-flash"

    def test_web_mounts_the_branded_ui_and_default_persona(self) -> None:
        overlay = launch_dsh.build_overlay(args_for(mode="web"), self.config, self.cli)
        inserts = [row["insert"] for row in overlay if "insert" in row]
        items = [item for batch in inserts for item in batch]
        ui = next(item for item in items if item.get("id") == "rhobuilder-ui")
        self.assertTrue(ui["name"].endswith("agents/deepseek/ui/index.js"))
        brand = next(row for row in overlay if row.get("id") == "ui-brand-official")
        self.assertTrue(brand["disabled"])
        preset = next(item for item in items if item.get("id") == "preset-rhobuilder")
        self.assertEqual(preset["config"]["id"], "rhobuilder")
        registry = next(row for row in overlay if row.get("id") == "agent-preset-registry")
        self.assertEqual(registry["config"]["default"], "rhobuilder")

    def test_no_ui_omits_the_ui_and_brand_override(self) -> None:
        overlay = launch_dsh.build_overlay(
            args_for(mode="web", no_ui=True), self.config, self.cli
        )
        ids = [row.get("id") for row in overlay]
        self.assertNotIn("ui-brand-official", ids)
        inserts = [row["insert"] for row in overlay if "insert" in row]
        self.assertFalse(any(item.get("id") == "rhobuilder-ui" for batch in inserts for item in batch))

    def test_web_launch_argv_passes_port_and_no_open(self) -> None:
        args = args_for(mode="web", port=3082, no_open=True)
        dsh = launch_dsh.Command(["dsh"], "binary")
        argv = launch_dsh._launch_argv(args, dsh, Path("/tmp/patch.yml"), None)
        self.assertEqual(argv[-3:], ["--port", "3082", "--no-open"])

    def test_devnet_probe_uses_the_shared_status_and_keeps_a_hint(self) -> None:
        healthy = {
            "node": {"reachable": True, "healthy": True, "latestBlockNumber": 9},
            "container": {"exists": True},
            "nextCommand": "python3 tools/devnet.py onboard --json",
        }
        with mock.patch.object(launch_dsh.devnet, "status", return_value=healthy):
            check = launch_dsh._probe_devnet(args_for())
        self.assertEqual(check.status, "ok")
        self.assertIn("block 9", check.detail)

        down = {
            "node": {"reachable": False, "healthy": False, "error": "down"},
            "container": {"exists": True},
            "nextCommand": "python3 tools/devnet.py start --container devnet-bootstrap",
        }
        with mock.patch.object(launch_dsh.devnet, "status", return_value=down):
            check = launch_dsh._probe_devnet(args_for())
        self.assertEqual(check.status, "setup")
        self.assertIn("devnet.py start", check.hint or "")

    def test_api_key_is_setup_required_when_unset(self) -> None:
        with mock.patch.dict(os.environ, {"DEEPSEEK_API_KEY": ""}):
            check = launch_dsh._probe_key(args_for())
        self.assertEqual(check.status, "setup")
        self.assertIn("DEEPSEEK_API_KEY", check.detail)

    def test_study_build_headless_inserts_the_registry(self) -> None:
        overlay = launch_dsh.build_overlay(
            args_for(study="build", mode="headless"), self.config, self.cli
        )
        inserts = [row["insert"] for row in overlay if "insert" in row]
        ids = [item.get("id") for batch in inserts for item in batch]
        self.assertIn("agent-preset-registry", ids)
        self.assertIn("preset-rhobuilder-architect", ids)

    def test_study_build_web_overwrites_the_registry(self) -> None:
        overlay = launch_dsh.build_overlay(
            args_for(study="build", mode="web"), self.config, self.cli
        )
        overwrites = [row for row in overlay if row.get("id") == "agent-preset-registry"]
        self.assertEqual(len(overwrites), 1)
        self.assertEqual(overwrites[0]["config"]["default"], "rhobuilder-architect")

    def test_yaml_renders_a_mapping_and_list(self) -> None:
        text = launch_dsh.render_yaml(
            [{"insert": [{"id": "x", "name": "/tmp/x.js", "config": {"a": [1, 2]}}]}]
        )
        self.assertIn("- insert:", text)
        self.assertIn("    - id: x", text)
        self.assertIn("        a:", text)
        self.assertIn("          - 1", text)

    def test_curated_selection_includes_the_procedure(self) -> None:
        names = launch_dsh._skill_selection(args_for(skills="curated"), self.config)
        self.assertIn("rhobuilder-build", names)
        self.assertIn("rhobuilder-rholang", names)
        self.assertIsNone(launch_dsh._skill_selection(args_for(skills="full"), self.config))
        self.assertEqual(launch_dsh._skill_selection(args_for(skills="none"), self.config), [])


class ResolveTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.directory = Path(self.temp.name)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_resolve_rhobuilder_prefers_an_executable(self) -> None:
        script = fake_script(self.directory, "rhobuilder", "exit 0")
        resolved = launch_dsh.resolve_rhobuilder(args_for(rhobuilder=str(script)))
        assert resolved is not None
        self.assertEqual(resolved.kind, "binary")
        self.assertEqual(resolved.argv[0], str(script.resolve()))

    def test_resolve_dsh_prefers_an_executable(self) -> None:
        script = fake_script(self.directory, "dsh", "echo 0.2.2-rc.5")
        resolved = launch_dsh.resolve_dsh(args_for(dsh=str(script)), launch_dsh.load_config())
        assert resolved is not None
        self.assertEqual(resolved.kind, "binary")

    def test_probe_dsh_reads_a_version(self) -> None:
        script = fake_script(self.directory, "dsh", "echo 0.2.2-rc.5")
        check = launch_dsh._probe_dsh(args_for(dsh=str(script)), launch_dsh.load_config())
        self.assertEqual(check.status, "ok")
        self.assertIn("0.2.2-rc.5", check.detail)

    def test_probe_cli_reads_the_json_version(self) -> None:
        payload = '{"schema":"rhobuilder.version/v1","ok":true,"product":"rhobuilder","version":"0.1.0"}'
        script = fake_script(self.directory, "rhobuilder", f"printf '%s' '{payload}'")
        check = launch_dsh._probe_cli(args_for(rhobuilder=str(script)))
        self.assertEqual(check.status, "ok")
        self.assertIn("0.1.0", check.detail)

    def test_probe_cli_rejects_non_json(self) -> None:
        script = fake_script(self.directory, "rhobuilder", "echo nope")
        check = launch_dsh._probe_cli(args_for(rhobuilder=str(script)))
        self.assertEqual(check.status, "fail")


if __name__ == "__main__":
    unittest.main()
