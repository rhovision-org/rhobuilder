from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(TOOLS))

import export_skills  # noqa: E402


CATALOG = """\
[[skill]]
name = "alpha-skill"
description = "The first skill."
path = "skills/alpha-skill/SKILL.md"

[[skill]]
name = "beta-skill"
description = "The second skill."
path = "skills/beta-skill/SKILL.md"

[curated]
names = ["alpha-skill"]
"""

BODY = """\
---
name: placeholder
description: placeholder
---
# Body

A rule.
"""

PROCEDURE = """\
---
name: rhobuilder-build
description: placeholder
user-invocable: true
---
# Procedure
"""


class Fixture:
    def __init__(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "skills.toml").write_text(CATALOG, encoding="utf-8")
        for name in ("alpha-skill", "beta-skill"):
            path = self.root / "skills" / name / "SKILL.md"
            path.parent.mkdir(parents=True)
            path.write_text(BODY, encoding="utf-8")
        procedure = self.root / "agents" / "deepseek" / "skills" / "rhobuilder-build.md"
        procedure.parent.mkdir(parents=True)
        procedure.write_text(PROCEDURE, encoding="utf-8")

    def cleanup(self) -> None:
        self.temp.cleanup()


class ExpectedTests(unittest.TestCase):
    def setUp(self) -> None:
        self.fixture = Fixture()

    def tearDown(self) -> None:
        self.fixture.cleanup()

    def test_expected_renders_both_layouts_and_the_procedure(self) -> None:
        files = export_skills.expected(self.fixture.root, ["dsh", "claude"])
        names = {path.parent.name for path in files}
        self.assertEqual(names, {"alpha-skill", "beta-skill", "rhobuilder-build"})
        dsh = self.fixture.root / ".dsh" / "skills" / "alpha-skill" / "SKILL.md"
        self.assertIn(dsh, files)
        self.assertIn("name: alpha-skill", files[dsh])
        self.assertIn("description: \"The first skill.\"", files[dsh])
        self.assertNotIn("name: placeholder", files[dsh])
        self.assertIn("user-invocable: true", files[self.fixture.root / ".dsh" / "skills" / "rhobuilder-build" / "SKILL.md"])

    def test_write_then_check_is_clean(self) -> None:
        files = export_skills.expected(self.fixture.root, ["dsh"])
        export_skills.write_expected(files)
        report = export_skills.check_expected(files, [self.fixture.root / ".dsh" / "skills"])
        self.assertTrue(report["ok"])
        self.assertEqual(report["missing"], [])
        self.assertEqual(report["stale"], [])

    def test_check_reports_stale(self) -> None:
        files = export_skills.expected(self.fixture.root, ["dsh"])
        export_skills.write_expected(files)
        target = self.fixture.root / ".dsh" / "skills" / "alpha-skill" / "SKILL.md"
        target.write_text("tampered", encoding="utf-8")
        report = export_skills.check_expected(files, [self.fixture.root / ".dsh" / "skills"])
        self.assertFalse(report["ok"])
        self.assertEqual(len(report["stale"]), 1)

    def test_curated_selection_can_include_the_procedure(self) -> None:
        files = export_skills.expected(
            self.fixture.root, ["dsh"], ["alpha-skill", "rhobuilder-build"]
        )
        names = {path.parent.name for path in files}
        self.assertEqual(names, {"alpha-skill", "rhobuilder-build"})

    def test_unknown_name_is_refused(self) -> None:
        with self.assertRaises(KeyError):
            export_skills.expected(self.fixture.root, ["dsh"], ["missing-skill"])

    def test_catalog_cap_is_enforced(self) -> None:
        entries = [
            {"name": f"skill-{index}", "description": "x", "path": "skills/alpha-skill/SKILL.md"}
            for index in range(export_skills.MAX_SKILLS + 1)
        ]
        with self.assertRaises(ValueError):
            export_skills.validate_catalog(entries, self.fixture.root)

    def test_invalid_kebab_name_is_refused(self) -> None:
        entries = [{"name": "Not_Kebab", "description": "x", "path": "skills/alpha-skill/SKILL.md"}]
        with self.assertRaises(ValueError):
            export_skills.validate_catalog(entries, self.fixture.root)


if __name__ == "__main__":
    unittest.main()
