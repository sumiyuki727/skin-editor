import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_switch_audio import CHECKS, validate


class SharedClockEvidenceTests(unittest.TestCase):
    def test_switch_evidence_requires_one_clock_fresh_input_and_unchanged_old_documents(self):
        specs = [(1, "Playing", 440000, 0)] + [(2, "Paused", 460000, d) for d in [0, 0, 2]]
        specs += [(3, "Playing", 460000, 2), (3, "Playing", 900000, 5)] + [(4, "Paused", 900000, 5)] * 3
        result = {"schema_version": 1, "status": "passed", "checks": dict.fromkeys(CHECKS, True), "errors": [],
                  "accepted_inputs": 2, "edit_count": 30, "nodes_per_document": [1, 1, 2, 1, 1, 1, 1, 1, 1, 1],
                  "accepted_documents": 3, "stale_gpu": 2, "system_capture_used": False, "frozen_us": 460000,
                  "resume_us": 460000, "final_simulation_us": 900000, "second_frozen_us": 900000,
                  "paused_wall_ms": 9000., "paused_samples": 3, "paused_span_us": 0,
                  "fresh_samples": [{"sequence": i+1, "request": request, "phase": phase, "device_us": pos,
                                     "simulation_us": pos, "document": doc} for i, (request, phase, pos, doc) in enumerate(specs)],
                  "proofs": [{"stage": s, "document": d, "pixels": 1024*1024, "max_channel_error": 1,
                              "pixels_outside_tolerance": 0, "display_max_channel_error": 1,
                              "display_pixels_outside_tolerance": 0, "simulation_us": 900000 if s == 4 else 460000,
                              "accepted_inputs": 2 if s == 4 else 1} for s, d in [(0, 0), (5, 0), (1, 2), (2, 2), (3, 2), (4, 5)]],
                  "uploads": [{"uploaded_bytes": 40*1024*1024, "upload_frames": 5,
                               "max_source_bytes_per_frame": 8*1024*1024, "max_sources_per_frame": 2}] * 5}
        validate(result)
        for key, value in [("accepted_inputs", 3), ("stale_gpu", 0), ("resume_us", 900000),
                           ("paused_wall_ms", float("nan")), ("nodes_per_document", [2] * 10)]:
            bad = copy.deepcopy(result); bad[key] = value
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                validate(bad)
        for key, index, field, value in [("fresh_samples", 2, "device_us", 460001),
                                         ("fresh_samples", 2, "sequence", 1), ("fresh_samples", 5, "document", 2),
                                         ("proofs", 1, "simulation_us", 0), ("proofs", 4, "accepted_inputs", 2),
                                         ("uploads", 0, "max_sources_per_frame", 3)]:
            bad = copy.deepcopy(result); bad[key][index][field] = value
            with self.subTest(key=key, field=field), self.assertRaises(RuntimeError):
                validate(bad)
