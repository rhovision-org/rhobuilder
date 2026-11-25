from __future__ import annotations

import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

TOOLS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(TOOLS))

import devnet  # noqa: E402


def completed(args, code=0, stdout="", stderr=""):
    return subprocess.CompletedProcess(args, code, stdout, stderr)


def fake_script(directory: Path) -> Path:
    path = directory / "rhobuilder"
    path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)
    return path


class DevnetTests(unittest.TestCase):
    def test_probe_node_reports_health_and_failure(self) -> None:
        class Response:
            def __enter__(self):
                return self

            def __exit__(self, *_):
                return False

            def read(self):
                return json.dumps(
                    {"degraded": "normal", "acceptsDeploys": True, "latestBlockNumber": 7}
                ).encode()

        with mock.patch("urllib.request.urlopen", return_value=Response()):
            value = devnet.probe_node("http://localhost:40403")
        self.assertTrue(value["healthy"])
        self.assertEqual(value["latestBlockNumber"], 7)

        with mock.patch("urllib.request.urlopen", side_effect=OSError("down")):
            value = devnet.probe_node("http://localhost:40403")
        self.assertFalse(value["reachable"])

    def test_container_status_is_read_only(self) -> None:
        calls = []

        def run(args, **kwargs):
            calls.append(args)
            return completed(args, 0, "running|rnode:local|healthy|40403->40403 \n")

        with mock.patch("shutil.which", return_value="/usr/bin/docker"):
            value = devnet.container_status(
                "devnet-bootstrap", docker="/usr/bin/docker", run=run
            )
        self.assertTrue(value["exists"])
        self.assertTrue(value["running"])
        self.assertEqual(value["image"], "rnode:local")
        self.assertEqual(calls[0][1], "inspect")
        self.assertNotIn("run", calls[0])
        self.assertNotIn("rm", calls[0])

    def test_status_names_the_next_safe_command(self) -> None:
        def run(args, **kwargs):
            return completed(args, 0, "exited|rnode:local||40403->40403 \n")

        with mock.patch("shutil.which", return_value="/usr/bin/docker"):
            value = devnet.status(
                "devnet-bootstrap",
                probe=lambda node: {"healthy": False, "reachable": True, "degraded": "throttled"},
                run=run,
            )
        self.assertFalse(value["healthy"])
        self.assertTrue(value["canStart"])
        self.assertIn("devnet.py start", value["nextCommand"])

    def test_start_refuses_a_missing_container(self) -> None:
        def run(args, **kwargs):
            return completed(args, 1, stderr="no such container")

        with mock.patch("shutil.which", return_value="/usr/bin/docker"):
            value = devnet.start(
                "devnet-bootstrap",
                probe=lambda node: {"healthy": False, "reachable": False},
                run=run,
            )
        self.assertFalse(value["ok"])
        self.assertIn("does not exist", value["error"])

    def test_start_returns_already_running_and_never_restarts(self) -> None:
        calls = []

        def run(args, **kwargs):
            calls.append(args)
            return completed(args, 0, "running|rnode:local|healthy|40403->40403 \n")

        with mock.patch("shutil.which", return_value="/usr/bin/docker"):
            value = devnet.start(
                "devnet-bootstrap",
                probe=lambda node: {"healthy": True, "reachable": True, "degraded": "normal"},
                run=run,
            )
        self.assertTrue(value["ok"])
        self.assertEqual(value["state"], "already_running")
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][1], "inspect")

    def test_wait_times_out_as_not_run(self) -> None:
        value = devnet.wait_for_health(
            "http://localhost:40403",
            0.01,
            interval=0.001,
            probe=lambda node: {"healthy": False, "reachable": True},
        )
        self.assertFalse(value["healthy"])

    def test_onboard_dry_run_prints_no_secret_and_no_live_flag(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            cli = fake_script(Path(directory))
            keys = Path(directory) / "keys.json"
            value = devnet.onboard(
                keys=keys,
                cli=cli,
                execute=False,
            )
        self.assertTrue(value["execute"] is False)
        self.assertTrue(any("baseline keygen" in command for command in value["commands"]))
        self.assertFalse(any("--allow-live" in command for command in value["commands"]))
        self.assertFalse(any("testnet" in command for command in value["commands"]))

    def test_onboard_executes_only_devnet_steps_and_stops_on_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            cli = fake_script(Path(directory))
            keys = Path(directory) / "keys.json"
            keys.write_text("{}", encoding="utf-8")
            calls = []

            def run(args, **kwargs):
                calls.append(args)
                if args[0] == "docker":
                    return completed(args, 0, "running|rnode:local|healthy|40403->40403 \n")
                return completed(args, 0, json.dumps({"ok": True}))

            with mock.patch("shutil.which", return_value="/usr/bin/docker"):
                value = devnet.onboard(
                    keys=keys,
                    cli=cli,
                    execute=True,
                    probe=lambda node: {"healthy": True, "reachable": True, "degraded": "normal"},
                    run=run,
                )
        self.assertTrue(value["ok"])
        self.assertEqual(value["steps"][0]["status"], "skipped")
        self.assertEqual([step["step"] for step in value["steps"]], ["keygen", "fund", "run", "verify"])
        self.assertFalse(any("--allow-live" in call for call in calls if call[0] != "docker"))
        self.assertTrue(all("devnet" in call for call in calls if call[0] != "docker"))
        self.assertFalse(any("rm" in call or "volume" in call for call in calls if call[0] == "docker"))

    def test_json_after_subcommand_is_accepted(self) -> None:
        with mock.patch.object(
            devnet,
            "status",
            return_value={"schema": "rhobuilder.devnet-status/v1", "ok": True, "healthy": True},
        ):
            self.assertEqual(devnet.main(["status", "--json"]), 0)


if __name__ == "__main__":
    unittest.main()
