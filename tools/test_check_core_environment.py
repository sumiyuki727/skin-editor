from pathlib import Path
import unittest
from unittest.mock import patch

from check_core import tool_environment


class CoreEnvironmentChecks(unittest.TestCase):
    def test_missing_registered_toolchain_fails_without_downloading(self):
        with patch.dict("os.environ", {}, clear=True), patch("check_core.shutil.which", return_value="rustup"):
            with patch("check_core.subprocess.check_output", return_value="stable-host (default)\n") as command:
                with self.assertRaisesRegex(RuntimeError, "No download was attempted"):
                    tool_environment()
                command.assert_called_once()
                self.assertEqual(command.call_args[0][0], ["rustup", "toolchain", "list"])

    def test_wrong_compiler_is_rejected_without_changing_settings(self):
        with patch.dict("os.environ", {}, clear=True), patch.object(Path, "is_file", return_value=True):
            with patch("check_core.subprocess.check_output", return_value="rustc 1.80.1 (old)\n"):
                with self.assertRaisesRegex(RuntimeError, "Expected rustc 1.95.0"):
                    tool_environment("existing-toolchain/bin")

    def test_inherited_compiler_overrides_are_rejected_without_echoing_values(self):
        with patch.dict("os.environ", {"RUSTFLAGS": "unreviewed override"}, clear=True):
            with patch.object(Path, "is_file", return_value=True), patch("check_core.subprocess.check_output") as command:
                with self.assertRaisesRegex(RuntimeError, "RUSTFLAGS") as error:
                    tool_environment("existing-toolchain/bin")
                self.assertNotIn("unreviewed override", str(error.exception))
                command.assert_not_called()


if __name__ == "__main__":
    unittest.main()
