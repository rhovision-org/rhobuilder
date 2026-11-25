from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(TOOLS))

import check_docs  # noqa: E402


def make_root(directory: Path) -> Path:
    docs = directory / "docs" / "src"
    docs.mkdir(parents=True)
    (docs / "SUMMARY.md").write_text(
        "# Summary\n\n- [rhobuilder](index.md)\n- [Page](page.md)\n", encoding="utf-8"
    )
    (docs / "index.md").write_text(
        "# rhobuilder\n\n## Status\n\n| Area | Status |\n|---|---|\n| demo | ok |\n",
        encoding="utf-8",
    )
    (docs / "page.md").write_text("# Page\n\n## Details\n", encoding="utf-8")
    (directory / "README.md").write_text(
        "# demo\n\n[start](docs/src/page.md#page)\n", encoding="utf-8"
    )
    return directory


class DocsCheckTests(unittest.TestCase):
    def test_clean_tree_passes_and_checks_readme_anchors(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = make_root(Path(temporary))
            report = check_docs.check(root)
        self.assertTrue(report["ok"], report)
        self.assertEqual(report["listed"], 2)
        self.assertEqual(report["documents"], 2)
        self.assertEqual(report["readme_links"], 1)
        self.assertEqual(report["anchors_checked"], 1)

    def test_unlisted_page_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = make_root(Path(temporary))
            (root / "docs" / "src" / "extra.md").write_text("# Extra\n", encoding="utf-8")
            report = check_docs.check(root)
        self.assertFalse(report["ok"])
        self.assertIn("docs/src/extra.md", report["unlisted"])

    def test_broken_readme_link_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = make_root(Path(temporary))
            (root / "README.md").write_text(
                "# demo\n\n[missing](docs/src/missing.md)\n", encoding="utf-8"
            )
            report = check_docs.check(root)
        self.assertFalse(report["ok"])
        self.assertTrue(any("missing.md" in item for item in report["broken"]))

    def test_bad_anchor_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = make_root(Path(temporary))
            (root / "README.md").write_text(
                "# demo\n\n[bad](docs/src/page.md#nope)\n", encoding="utf-8"
            )
            report = check_docs.check(root)
        self.assertFalse(report["ok"])
        self.assertTrue(any("#nope" in item for item in report["broken"]))

    def test_duplicate_status_table_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = make_root(Path(temporary))
            (root / "docs" / "src" / "page.md").write_text(
                "# Page\n\n| Area | Status |\n|---|---|\n| x | y |\n", encoding="utf-8"
            )
            report = check_docs.check(root)
        self.assertFalse(report["ok"])
        self.assertFalse(report["canonical_status"]["ok"])


if __name__ == "__main__":
    unittest.main()
