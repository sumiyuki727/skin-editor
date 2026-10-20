import copy
import ctypes
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from p0_audio_device import analyze, Guid, WaveFormat


def samples():
    return ([{"phase": "playing", "wall_seconds": i / 100, "position_ms": i * 10.} for i in range(45)]
            + [{"phase": "paused", "wall_seconds": .45 + i / 100, "position_ms": 450.} for i in range(36)]
            + [{"phase": "resumed", "wall_seconds": .81 + i / 100, "position_ms": 450. + i * 10} for i in range(45)])


class AudioDeviceTests(unittest.TestCase):
    def test_clock_analysis_rejects_pause_drift_and_resume_catchup(self):
        data = samples()
        self.assertEqual(analyze(data)["status"], "passed")
        for scenario in ("drift", "catchup"):
            bad = copy.deepcopy(data)
            if scenario == "drift":
                for i, row in enumerate(bad[45:81]):
                    row["position_ms"] += i * 10
            for row in bad[81:]:
                row["position_ms"] += 350
            self.assertEqual(analyze(bad)["status"], "failed")

    def test_missing_inaccurate_and_nonfinite_samples_cannot_pass(self):
        data = samples()
        with self.assertRaises(RuntimeError):
            analyze(data[:50])
        bad = copy.deepcopy(data)
        bad[20]["position_ms"] = float("nan")
        with self.assertRaises(RuntimeError):
            analyze(bad)
        bad = copy.deepcopy(data)
        bad[20]["position_hresult"] = 1
        self.assertEqual(analyze(bad)["status"], "failed")
        bad = copy.deepcopy(data)
        bad[0]["phase"] = "paused"
        with self.assertRaises(RuntimeError):
            analyze(bad)

    def test_public_abi_layout_uses_fixed_width_fields(self):
        self.assertEqual(ctypes.sizeof(Guid), 16)
        self.assertEqual(ctypes.sizeof(WaveFormat), 18)
        self.assertEqual(WaveFormat.rate.offset, 4)
        self.assertEqual(WaveFormat.extra_bytes.offset, 16)
        self.assertEqual(bytes(Guid("1cb9ad4c-dbfa-4c32-b178-c2f568a703b2").bytes)[:4], b"\x4c\xad\xb9\x1c")
