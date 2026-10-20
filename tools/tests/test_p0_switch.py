import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_switch import CHECKS, phase_summary, validate


class SwitchTests(unittest.TestCase):
    def test_requires_retention_generation_uploads_and_matching_document_pixels(self):
        result = {"schema_version": 1, "status": "passed", "checks": dict.fromkeys(CHECKS, True), "errors": [],
                  "pixel_diagnostic": True, "long_observation": False, "simulation_ms": 0, "accepted_game_inputs": 0,
                  "accepted_documents": 22, "stale_gpu": 1, "retained_frames": 3, "system_capture_used": False,
                  "switch_ms": [150.] * 20, "request_cpu_ms": [.01] * 25,
                  "uploads": [{"uploaded_bytes": 40 * 1024 * 1024, "upload_frames": 5,
                               "max_source_bytes_per_frame": 8 * 1024 * 1024, "max_sources_per_frame": 2}] * 23,
                  "proofs": [{"document": d, "stage": s, "pixels": 1024 * 1024, "max_channel_error": 1,
                              "pixels_outside_tolerance": 0, "display_max_channel_error": 1,
                              "display_pixels_outside_tolerance": 0} for d, s in [(0, 0), (2, 1), (2, 3), (9, 2)]]}
        validate(result, True, False)
        for key, value in [("stale_gpu", 0), ("simulation_ms", 1), ("switch_ms", [1.] * 19),
                           ("request_cpu_ms", [float("nan")] * 25), ("accepted_documents", 23)]:
            bad = copy.deepcopy(result); bad[key] = value
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                validate(bad, True, False)
        for key, field, value in [("uploads", "max_source_bytes_per_frame", 40 * 1024 * 1024),
                                  ("uploads", "upload_frames", 1), ("proofs", "document", 9),
                                  ("proofs", "display_pixels_outside_tolerance", 1)]:
            bad = copy.deepcopy(result); bad[key][0][field] = value
            with self.subTest(field=field), self.assertRaises(RuntimeError):
                validate(bad, True, False)
        with self.assertRaises(RuntimeError):
            validate(result, False, True)
        result.update(pixel_diagnostic=False, long_observation=True, proofs=[])
        validate(result, False, True)

    def test_phase_cpu_does_not_include_loading_or_close_transition(self):
        def sample(phase, seconds, cpu):
            return {"phase": phase, "monotonic_seconds": seconds, "cpu_seconds": cpu,
                    "working_set_bytes": 100, "private_bytes": 120, "gpu_dedicated_bytes": None, "gpu_shared_bytes": None}
        rows = phase_summary([sample("Cycle", 0, 0), sample("Idle", 5, 50), sample("Idle", 65, 50.1),
                              sample("ClosedSettle", 66, 100), sample("Closed", 70, 110), sample("Closed", 130, 110)])
        idle = next(r for r in rows if r["phase"] == "Idle")
        closed = next(r for r in rows if r["phase"] == "Closed")
        self.assertEqual(idle["sample_span_seconds"], 60)
        self.assertAlmostEqual(idle["cpu_one_logical_core_percent"], 1/6)
        self.assertEqual(closed["cpu_one_logical_core_percent"], 0)
        self.assertIsNone(closed["gpu_dedicated_bytes_max"])
