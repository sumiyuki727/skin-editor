import contextlib
import io
import unittest
from unittest.mock import patch

import check_repository as check


class RepositoryChecks(unittest.TestCase):
    def test_private_paths_are_rejected_even_if_already_tracked(self):
        for path in [".env", "src/.env.production", "local-data/skin/cursor.png", "src/client.key",
                     ".p0-tools/runtime.dll", "target/release/app.exe", "notes/credentials.json", "settings.local.json"]:
            with self.subTest(path=path):
                self.assertTrue(check.path_issues(path))

    def test_source_locks_and_self_authored_fixtures_are_allowed(self):
        for path in ["Cargo.lock", "crates/domain/src/lib.rs", ".github/workflows/core.yml",
                     "experiments/p0-media/tests/fixtures/audio-0_37.ogg"]:
            with self.subTest(path=path):
                self.assertEqual(check.path_issues(path), [])

    def test_common_secrets_report_rule_and_line_without_values(self):
        values = [b"-----BEGIN " + b"PRIVATE KEY-----", b"AK" + b"IA" + b"A" * 16,
                  b"gh" + b"p_" + b"x" * 36, b"sk-" + b"proj-" + b"a" * 48,
                  b'password = "' + b"unshared-value-123456" + b'"']
        for value in values:
            with self.subTest(kind=value[:4]):
                result = check.content_issues(b"line one\n" + value)
                self.assertTrue(result)
                self.assertTrue(all(set(f) == {"rule", "line"} and f["line"] == 2 for f in result))

    def test_placeholders_and_evidence_hashes_do_not_trigger(self):
        self.assertEqual(check.content_issues(b'api_key = "<your-placeholder>"\nsha256 = "' + b"a" * 64 + b'"'), [])

    def test_staged_scan_reads_index_bytes_not_clean_worktree(self):
        def fake_git(*args):
            if args[0] == "diff":
                return b"config.json\0"
            if args[0] == "ls-files":
                return b"100644 " + b"0" * 40 + b" 0\tconfig.json\0"
            if args == ("show", ":config.json"):
                return b"-----BEGIN " + b"PRIVATE KEY-----"
            self.fail("unexpected git request")
        with patch.object(check, "git", side_effect=fake_git):
            count, findings = check.inspect(staged=True)
        self.assertEqual(count, 1)
        self.assertEqual(findings, [("config.json", "private-key", 1)])

    def test_staged_deletions_are_not_read(self):
        with patch.object(check, "git", return_value=b""):
            self.assertEqual(check.inspect(staged=True), (0, []))

    def test_invalid_path_is_rejected_without_reading_outside_repository(self):
        with patch.object(check, "git", side_effect=[b"../private.txt\0", b""]):
            self.assertEqual(check.inspect(staged=True), (1, [("../private.txt", "invalid-repository-path", None)]))

    def test_symlink_is_rejected_without_following_target(self):
        with patch.object(check, "git", side_effect=[b"link\0", b"120000 " + b"0" * 40 + b" 0\tlink\0"]):
            self.assertEqual(check.inspect(staged=True), (1, [("link", "symlink-or-submodule-requires-review", None)]))

    def test_failure_output_redacts_secret(self):
        with patch.object(check, "inspect", return_value=(1, [("file.json", "private-key", 2)])):
            output = io.StringIO()
            with patch("sys.argv", ["check_repository.py", "--staged"]), contextlib.redirect_stderr(output):
                self.assertEqual(check.main(), 1)
            self.assertIn("file.json:2", output.getvalue())


if __name__ == "__main__":
    unittest.main()
