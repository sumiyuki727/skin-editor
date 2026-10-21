import tempfile
import json
import unittest
from pathlib import Path
from unittest.mock import patch
import check_core_cost as cost


class CoreCostChecks(unittest.TestCase):
    def phase(self, name="transactions_a", operations=5000):
        return dict(phase=name, operations=operations, wall_ns=1, p50_ns=1, p95_ns=2,
                    p99_ns=3, max_ns=4, nodes=5001, retained_blobs=0, retained_requests=5000)

    def test_phase_order_counts_and_numeric_values_are_required(self):
        self.assertEqual(cost.validate_phase(self.phase(), "transactions_a")["operations"], 5000)
        for field, value in [("operations", 4999), ("wall_ns", -1), ("p95_ns", 100), ("nodes", True), ("phase", "released")]:
            data = self.phase()
            data[field] = value
            with self.assertRaises(ValueError):
                cost.validate_phase(data, "transactions_a")

    def test_branch_and_draft_workloads_cannot_be_shortened(self):
        for name, count in [("cross_branch", 2), ("draft_cancel", 300)]:
            cost.validate_phase(self.phase(name, count), name)
            with self.assertRaises(ValueError):
                cost.validate_phase(self.phase(name, count - 1), name)

    def test_output_rejects_existing_and_external_directories(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(cost, "ROOT", Path(directory)):
            root = Path(directory)
            self.assertEqual(cost.safe_output(root / "artifacts/new"), (root / "artifacts/new").resolve())
            (root / "artifacts/existing").mkdir(parents=True)
            for path in [root / "artifacts", root / "outside", root / "artifacts/existing"]:
                with self.assertRaises(ValueError):
                    cost.safe_output(path)

    def test_metadata_profiles_keep_standard_and_pressure_distinct(self):
        self.assertEqual(cost.PROFILES["standard"], (100, 10, 500))
        self.assertEqual(cost.PROFILES["pressure"], (1000, 100, 5000))
        self.assertEqual(len(cost.PHASES), 8)

    def test_actual_cargo_artifact_is_required_instead_of_a_stale_default_path(self):
        message = dict(reason="compiler-artifact", target=dict(name="skin-editor-core-cost"), executable="custom-target/probe.exe")
        self.assertEqual(cost.built_executable(json.dumps(message)), Path("custom-target/probe.exe"))
        for text in ["", json.dumps(dict(reason="build-finished", success=True)),
                     json.dumps(message) + "\n" + json.dumps(message)]:
            with self.assertRaises(ValueError):
                cost.built_executable(text)
