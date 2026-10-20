import copy
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_edit_composition import validate


class EditCompositionTests(unittest.TestCase):
    def test_requires_real_edit_coverage_source_reuse_and_both_pixel_proofs(self):
        result = {"status": "passed", "before_equals_cancelled_canvas": True,
                  "draft_changes_canvas": True, "commit_changes_canvas": True,
                  "observations": [{"stage": s, "nodes": count, "committed_node": node,
                                    "visible_position": pos, "simulation_ms": 0, "accepted_game_inputs": 0}
                                   for s, count, node, pos in [(0, 1, 0, [40, 40]), (1, 1, 0, [69.999996, 60]),
                                                              (2, 1, 0, [40, 40]), (3, 2, 1, [100, 70])]],
                  "edit_count": 300, "edit_ui_host_ms": [1.] * 300, "per_edit_readback": False, "edit_readback_ms": [],
                  "composition": {"source_upload_batches": 1, "source_upload_bytes": 40 * 1024 * 1024,
                                  "updates": 303, "operation_upload_bytes": 303 * 4480,
                                  "normal_path_cpu_pixel_readbacks": 0, "normal_path_cpu_texture_uploads": 0},
                  "composition_pixel_checks": [{"stage": s, "pixels": 1024 * 1024, "max_channel_error": 1,
                                                "pixels_outside_tolerance": 0, "display_max_channel_error": 1,
                                                "display_pixels_outside_tolerance": 0} for s in range(4)]}
        validate(result, False)
        for key, value in [("source_upload_batches", 2), ("operation_upload_bytes", 0), ("updates", 302)]:
            bad = copy.deepcopy(result)
            bad["composition"][key] = value
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                validate(bad, False)
        for position in ([float("nan"), 60], [70.01, 60], [70]):
            bad = copy.deepcopy(result)
            bad["observations"][1]["visible_position"] = position
            with self.assertRaises(RuntimeError):
                validate(bad, False)
        bad = copy.deepcopy(result)
        bad["composition_pixel_checks"][1]["display_pixels_outside_tolerance"] = 1
        with self.assertRaises(RuntimeError):
            validate(bad, False)
        with self.assertRaises(RuntimeError):
            validate(result, True)
        bad = copy.deepcopy(result)
        bad["edit_ui_host_ms"][0] = float("nan")
        with self.assertRaises(RuntimeError):
            validate(bad, False)
        result["per_edit_readback"] = True
        result["edit_readback_ms"] = [9.] * 300
        validate(result, True)
