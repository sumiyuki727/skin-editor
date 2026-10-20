import copy
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_gui import check_audio_port_boundary
from check_p0_recovery import CHECKS, validate


class RecoveryTests(unittest.TestCase):
    def test_audio_port_rejects_native_ui_io_and_wall_clock_imports(self):
        check_audio_port_boundary(["use crate::audio_transport::Intent; pub trait Output {}"])
        for source in ["use p0_wasapi::Device;", "use windows::core;", "use egui::Context;",
                       "use std::time::Instant;", "use std::{fs, sync::Arc};", 'include!("outside.rs");']:
            with self.subTest(source=source), self.assertRaises(RuntimeError):
                check_audio_port_boundary([source])

    def test_recovery_evidence_requires_anchored_epochs_frozen_edits_and_fresh_inputs(self):
        rows = [{"epoch": 0, "sequence": 1, "request": 1, "phase": "Playing", "device_us": 400000,
                 "origin_us": 0, "base_us": 0, "simulation_us": 400000}]
        for epoch, base in [(1, 400000), (2, 800000)]:
            rows += [{"epoch": epoch, "sequence": n, "request": 0, "phase": "Paused", "device_us": 0,
                      "origin_us": 0, "base_us": base, "simulation_us": base} for n in range(1, 4)]
            rows += [{"epoch": epoch, "sequence": 4, "request": 1, "phase": "Playing", "device_us": 400000,
                      "origin_us": 0, "base_us": base, "simulation_us": base+400000}]
        rows += [{"epoch": 2, "sequence": n, "request": 2, "phase": "Paused", "device_us": 400000,
                  "origin_us": 0, "base_us": 800000, "simulation_us": 1200000} for n in range(5, 8)]
        good = {"schema_version": 1, "status": "passed", "checks": dict.fromkeys(CHECKS, True), "errors": [],
                "accepted_inputs": 2, "nodes": 2, "edit_count": 30, "stale_rejected": 2, "system_capture_used": False,
                "frozen_us": [400000, 800000], "recovered_us": [400000, 800000], "final_simulation_us": 1200000,
                "fresh_samples": rows,
                "checkpoints": [{"stage": n, "epoch": e, "phase": p, "nodes": nodes, "inputs": inputs, "simulation_us": us}
                                for n, e, p, nodes, inputs, us in [(0, 0, "Faulted", 1, 1, 400000), (1, 0, "Faulted", 2, 1, 400000),
                                                                 (2, 1, "Paused", 2, 1, 400000), (3, 2, "Paused", 2, 2, 800000)]],
                "proofs": [{"stage": n, "pixels": 1024*1024, "max_channel_error": 1, "pixels_outside_tolerance": 0,
                            "display_max_channel_error": 1, "display_pixels_outside_tolerance": 0} for n in range(4)]}
        validate(good)
        for field, value in [("recovered_us", [0, 0]), ("accepted_inputs", 3), ("stale_rejected", 1), ("nodes", 3)]:
            bad = copy.deepcopy(good); bad[field] = value
            with self.subTest(field=field), self.assertRaises(RuntimeError):
                validate(bad)
        for field, index, nested, value in [("fresh_samples", 1, "base_us", 0), ("fresh_samples", 2, "sequence", 1),
                                            ("fresh_samples", 1, "phase", "Playing"), ("fresh_samples", 6, "epoch", 0),
                                            ("fresh_samples", -1, "request", 1),
                                            ("checkpoints", 1, "simulation_us", 500000), ("checkpoints", 1, "nodes", 1),
                                            ("proofs", 1, "display_pixels_outside_tolerance", 1)]:
            bad = copy.deepcopy(good); bad[field][index][nested] = value
            with self.subTest(field=field, nested=nested), self.assertRaises(RuntimeError):
                validate(bad)


if __name__ == "__main__":
    unittest.main()
