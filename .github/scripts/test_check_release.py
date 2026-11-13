"""Tests of check_release.py, the guard the release workflow runs on a tag before anything else."""

import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_release  # noqa: E402

CARGO = '[package]\nname = "ai-ctl"\nversion = "0.2.0"\nedition = "2024"\n\n[dependencies]\nserde = { version = "1.0.229" }\n'
DATED = "# Changelog — ai-ctl\n\n## 0.2.0 — 2026-11-10\n\n- The rebuild.\n\n## 0.1.0 — 2026-10-01\n\n- The first.\n"
UNRELEASED = DATED.replace("## 0.2.0 — 2026-11-10", "## 0.2.0 — unreleased")


class Problems(unittest.TestCase):
    def test_a_tag_of_the_version_with_a_dated_entry_passes(self):
        self.assertEqual(check_release.problems("v0.2.0", CARGO, DATED), [])

    def test_a_tag_of_another_version_than_cargo_toml_is_refused(self):
        found = check_release.problems("v0.1.0", CARGO, DATED)
        self.assertEqual(len(found), 1)
        self.assertIn("v0.1.0", found[0])
        self.assertIn("0.2.0", found[0])

    def test_an_undated_entry_is_refused(self):
        found = check_release.problems("v0.2.0", CARGO, UNRELEASED)
        self.assertEqual(len(found), 1)
        self.assertIn("not dated", found[0])
        self.assertIn("unreleased", found[0])

    def test_an_entry_dated_otherwise_than_yyyy_mm_dd_is_refused(self):
        for date in ("2026-13-40", "20261110", "2026-11-10 (late)", "10/11/2026"):
            with self.subTest(date=date):
                found = check_release.problems("v0.2.0", CARGO, DATED.replace("2026-11-10", date))
                self.assertEqual(len(found), 1)
                self.assertIn("not dated", found[0])

    def test_a_version_without_an_entry_is_refused(self):
        found = check_release.problems("v0.2.0", CARGO, DATED.replace("## 0.2.0 — 2026-11-10\n", ""))
        self.assertEqual(len(found), 1)
        self.assertIn("no entry", found[0])

    def test_two_entries_for_the_version_are_refused(self):
        twice = DATED + "\n## 0.2.0 — 2026-11-11\n\n- Again.\n"
        found = check_release.problems("v0.2.0", CARGO, twice)
        self.assertEqual(len(found), 1)
        self.assertIn("2 entries", found[0])

    def test_an_entry_of_a_longer_version_is_not_the_versions(self):
        longer = DATED.replace("## 0.2.0 — 2026-11-10", "## 0.2.0 — unreleased\n\n## 0.2.01 — 2026-11-10")
        found = check_release.problems("v0.2.0", CARGO, longer)
        self.assertEqual(len(found), 1)
        self.assertIn("not dated", found[0])

    def test_a_tag_not_of_the_form_v_x_y_z_is_refused(self):
        for tag in ("0.2.0", "v0.2", "v0.2.0-rc1", "release", ""):
            with self.subTest(tag=tag):
                found = check_release.problems(tag, CARGO, DATED)
                self.assertTrue(found)
                self.assertIn("vX.Y.Z", found[0])

    def test_the_version_is_the_packages_not_a_dependencys(self):
        no_package_version = CARGO.replace('version = "0.2.0"\n', "")
        found = check_release.problems("v1.0.229", no_package_version, DATED)
        self.assertTrue(found)
        self.assertIn("no version", found[0])

    def test_a_cargo_toml_that_is_not_toml_is_refused(self):
        found = check_release.problems("v0.2.0", "[package\n", DATED)
        self.assertTrue(found)
        self.assertIn("Cargo.toml", found[0])

    def test_every_problem_is_given_at_once(self):
        found = check_release.problems("v0.1.0", CARGO, UNRELEASED)
        self.assertEqual(len(found), 2)


class Main(unittest.TestCase):
    def run_main(self, tag, cargo, changelog):
        with tempfile.TemporaryDirectory() as root:
            if cargo is not None:
                Path(root, "Cargo.toml").write_text(cargo, encoding="utf-8")
            if changelog is not None:
                Path(root, "CHANGELOG.md").write_text(changelog, encoding="utf-8")
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                code = check_release.main([tag, root])
            return code, out.getvalue()

    def test_a_release_that_passes_exits_0_and_names_its_version_and_date(self):
        code, out = self.run_main("v0.2.0", CARGO, DATED)
        self.assertEqual(code, 0)
        self.assertIn("0.2.0", out)
        self.assertIn("2026-11-10", out)

    def test_a_refused_release_exits_1_with_one_refused_line_per_problem(self):
        code, out = self.run_main("v0.1.0", CARGO, UNRELEASED)
        self.assertEqual(code, 1)
        self.assertEqual([line.startswith("refused: ") for line in out.splitlines()], [True, True])

    def test_a_missing_file_is_refused_not_raised(self):
        code, out = self.run_main("v0.2.0", CARGO, None)
        self.assertEqual(code, 1)
        self.assertIn("refused: ", out)
        self.assertIn("CHANGELOG.md", out)

    def test_a_wrong_number_of_arguments_exits_2(self):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(check_release.main([]), 2)


if __name__ == "__main__":
    unittest.main()
