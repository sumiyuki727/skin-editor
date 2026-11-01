import copy
import math
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from measure_budget_review import review, release_receipt, PHASES, LEGACY_CPU, REQUIRED_HARD


class BudgetReviewTests(unittest.TestCase):
    def fixture(self):
        original = {k: True for k in REQUIRED_HARD | {LEGACY_CPU}}
        costs = {p: dict(cpu_one_core_percent=19.) for p in PHASES}
        return original, costs, {"working_set_bytes": 160*1024**2, "private_bytes": 250*1024**2}

    def test_cpu_over_goal_is_visible_without_rewriting_legacy_or_granting_p2(self):
        original, costs, peaks = self.fixture()
        original[LEGACY_CPU] = False
        before = copy.deepcopy((original, costs, peaks))
        result = review(original, costs, True, peaks)
        self.assertTrue(result["bounded_hard_checks_passed"])
        self.assertFalse(result["legacy_checks"][LEGACY_CPU])
        self.assertFalse(any(result["cpu_target_met"].values()))
        self.assertFalse(result["p2_accepted"])
        self.assertFalse(result["standard_workload_accepted"])
        self.assertEqual((original, costs, peaks), before)

    def test_each_existing_hard_failure_still_blocks(self):
        for key in REQUIRED_HARD:
            original, costs, peaks = self.fixture()
            original[key] = False
            with self.subTest(key=key):
                result = review(original, costs, True, peaks)
                self.assertFalse(result["bounded_hard_checks_passed"])
                self.assertFalse(result["hard_checks"][key])

    def test_unknown_cpu_named_predicate_is_not_silently_waived(self):
        original, costs, peaks = self.fixture()
        original["cpu_idle_busy_loop"] = False
        self.assertFalse(review(original, costs, True, peaks)["bounded_hard_checks_passed"])

    def test_missing_checks_phases_or_invalid_values_are_rejected(self):
        original, costs, peaks = self.fixture()
        for key in REQUIRED_HARD | {LEGACY_CPU}:
            bad = dict(original)
            del bad[key]
            with self.assertRaises(ValueError):
                review(bad, costs, True, peaks)
        for value in (-1, math.nan, math.inf, True):
            bad = copy.deepcopy(costs)
            bad["voices_32"]["cpu_one_core_percent"] = value
            with self.assertRaises(ValueError):
                review(original, bad, True, peaks)
        with self.assertRaises(ValueError):
            review(original, {"voices_32": costs["voices_32"]}, True, peaks)

    def test_invalid_meter_never_turns_low_counters_into_cpu_success(self):
        original, costs, peaks = self.fixture()
        costs = {p: dict(cpu_one_core_percent=1.) for p in PHASES}
        result = review(original, costs, False, peaks)
        self.assertFalse(result["cpu_meter_valid"])
        self.assertTrue(all(v is None for v in result["cpu_target_met"].values()))

    def test_lifecycle_peak_is_hard_and_short_idle_is_not_B05(self):
        original, costs, peaks = self.fixture()
        peaks["private_bytes"] = 512*1024**2+1
        result = review(original, costs, True, peaks)
        self.assertFalse(result["bounded_hard_checks_passed"])
        self.assertEqual(result["phases"]["idle"], "short-idle-not-B05")
        self.assertEqual(result["phases"]["voices_1"], "light-proxy-not-standard-N")

    def test_feature_or_mismatched_release_receipt_is_rejected(self):
        import hashlib
        with tempfile.TemporaryDirectory() as folder:
            binary = Path(folder) / "owned.exe"
            binary.write_bytes(b"self-authored-receipt-fixture")
            digest = hashlib.sha256(binary.read_bytes()).hexdigest()
            receipt = dict(schema=1, profile="release", requested_features=[], host_resolved_features=[],
                           repeat_binary_sha256=[digest, digest], source={"owned.rs": "fixed"})
            with patch("measure_budget_review.fingerprint", return_value={"source": receipt["source"]}):
                self.assertEqual(release_receipt(receipt, binary), digest)
                for change in (dict(requested_features=["framework-trace"]), dict(host_resolved_features=["framework-trace"]),
                               dict(repeat_binary_sha256=[digest, "changed"]), dict(source={"owned.rs": "changed"})):
                    with self.subTest(change=change), self.assertRaises(ValueError):
                        release_receipt(dict(receipt, **change), binary)


if __name__ == "__main__":
    unittest.main()
