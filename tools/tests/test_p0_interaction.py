import copy
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_interaction import validate
from check_p0_composition import validate_scale


class InteractionEvidenceTests(unittest.TestCase):
    def test_gesture_proof_rejects_partial_cancel_extra_commits_and_clock_or_input_leaks(self):
        result = {"status": "passed", "before_equals_cancelled_canvas": True,
                  "draft_changes_canvas": True, "commit_changes_canvas": True,
                  "observations": [{"stage": s, "nodes": count, "committed_node": node,
                                    "visible_position": pos, "simulation_ms": 0, "accepted_game_inputs": 0}
                                   for s, count, node, pos in [(0, 1, 0, [40, 40]), (1, 1, 0, [70, 60]),
                                                              (2, 1, 0, [40, 40]), (3, 2, 1, [100, 70])]]}
        validate(result)
        for field, value in [("nodes", 2), ("visible_position", [70, 60]), ("simulation_ms", 10), ("accepted_game_inputs", 1)]:
            bad = copy.deepcopy(result)
            bad["observations"][2][field] = value
            with self.subTest(field=field), self.assertRaises(RuntimeError):
                validate(bad)
        result["observations"].pop()
        with self.assertRaises(RuntimeError):
            validate(result)

    def test_scale_proof_rejects_missing_documents_insufficient_samples_and_pixel_errors(self):
        result = {"status": "passed", "warm_dispatches": 300,
                  "cases": [{"document": d, "layers": 50, "groups": 10, "ops": 70,
                             "pixels": 1024 * 1024, "pixels_outside_tolerance": 0, "max_channel_error": 1,
                             "source_ids": list(range(d * 10, d * 10 + 10)), "warm_gpu_ms": [None] * 30,
                             "warm_submit_wait_readback_ms": [7.] * 30} for d in range(10)]}
        validate_scale(result)  # Unsupported timestamps stay unknown, never zero.
        for field, value in [("layers", 49), ("ops", 50), ("pixels_outside_tolerance", 1),
                             ("max_channel_error", 3), ("warm_gpu_ms", [1.] * 29)]:
            bad = copy.deepcopy(result)
            bad["cases"][4][field] = value
            with self.subTest(field=field), self.assertRaises(RuntimeError):
                validate_scale(bad)
        result["cases"].pop()
        with self.assertRaises(RuntimeError):
            validate_scale(result)
