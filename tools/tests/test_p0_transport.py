import copy
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_gui import check_audio_adapter_boundary
from check_p0_transport import CHECKS, validate


class TransportTests(unittest.TestCase):
    def test_audio_boundary_rejects_ui_files_native_leaks_and_extra_apis(self):
        dependency = {"name": "windows", "req": "=0.62.2", "target": "cfg(windows)",
                      "features": ["Win32_Foundation", "Win32_Media_Audio", "Win32_System_Com",
                                   "Win32_System_Com_StructuredStorage", "Win32_System_Variant"]}
        sources = {"lib.rs": "use std::sync::Arc; // unsafe belongs in platform.rs",
                   "platform.rs": "unsafe { native_call(); }"}
        check_audio_adapter_boundary([dependency], sources)
        for source in ("use egui::Context;", "use std::fs;", "use std::{sync::Arc, net::TcpStream};",
                       'include_str!("external.rs");', '#[path="external.rs"] mod old;',
                       "unsafe { native_call(); }"):
            with self.subTest(source=source), self.assertRaises(RuntimeError):
                check_audio_adapter_boundary([dependency], {"lib.rs": source})
        for key, value in [("req", "*"), ("target", None), ("path", "../old-source"),
                           ("features", dependency["features"] + ["Win32_Networking_WinSock"])]:
            bad = dict(dependency, **{key: value})
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                check_audio_adapter_boundary([bad], sources)
        with self.assertRaises(RuntimeError):
            check_audio_adapter_boundary([dependency, {"name": "egui"}], sources)

    def test_transport_proof_requires_fresh_device_clock_input_history_and_pixels(self):
        result = {"schema_version": 1, "status": "passed", "checks": dict.fromkeys(CHECKS, True), "errors": [],
                  "edit_count": 300, "accepted_inputs": 2, "frozen_us": 460000, "resume_us": 460000,
                  "final_simulation_us": 900000, "paused_wall_ms": 8500., "paused_device_samples": 3,
                  "paused_device_span_us": 0,
                  "fresh_samples": [{"sequence": i + 1, "request": 2, "phase": "Paused", "device_us": 460000,
                                     "simulation_us": 460000} for i in range(3)],
                  "captures": [{"stage": i, "nodes": n, "accepted_inputs": 1, "simulation_us": 460000}
                               for i, n in enumerate([1, 1, 1, 2])],
                  "composition_pixel_checks": [{"pixels": 1024 * 1024, "max_channel_error": 1,
                                                "pixels_outside_tolerance": 0, "display_max_channel_error": 1,
                                                "display_pixels_outside_tolerance": 0} for _ in range(4)],
                  "composition": {"source_upload_batches": 1, "source_upload_bytes": 40 * 1024 * 1024,
                                  "updates": 303, "operation_upload_bytes": 303 * 4480,
                                  "normal_path_cpu_pixel_readbacks": 0, "normal_path_cpu_texture_uploads": 0}}
        validate(result)
        changes = [("schema_version", 2), ("accepted_inputs", 3), ("paused_device_samples", 4),
                   ("paused_device_span_us", 1), ("resume_us", 1000000), ("paused_wall_ms", float("nan"))]
        for key, value in changes:
            bad = copy.deepcopy(result)
            bad[key] = value
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                validate(bad)
        for key, nested, field, value in [
                ("fresh_samples", 1, "device_us", 460001), ("fresh_samples", 1, "sequence", 1),
                ("captures", 1, "nodes", 2), ("captures", 2, "accepted_inputs", 2),
                ("composition_pixel_checks", 0, "display_pixels_outside_tolerance", 1)]:
            bad = copy.deepcopy(result)
            bad[key][nested][field] = value
            with self.subTest(key=key, field=field), self.assertRaises(RuntimeError):
                validate(bad)
        bad = copy.deepcopy(result)
        bad["composition"]["normal_path_cpu_pixel_readbacks"] = 1
        with self.assertRaises(RuntimeError):
            validate(bad)
        bad = copy.deepcopy(result)
        bad["checks"].pop("single_commit")
        with self.assertRaises(RuntimeError):
            validate(bad)
