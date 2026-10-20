import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from p0_frame_attribution import attribute


class FrameAttributionTests(unittest.TestCase):
    def test_previous_cpu_joins_previous_frame_and_ambiguous_samples_fail(self):
        trace = {"frames": [{"frame": 0, "logic_ms": 53., "ui_ms": 1., "previous_eframe_cpu_ms": None},
                            {"frame": 1, "logic_ms": 2., "ui_ms": 1., "previous_eframe_cpu_ms": 62.},
                            {"frame": 2, "logic_ms": 1., "ui_ms": 1., "previous_eframe_cpu_ms": 4.}],
                 "switch_steps": [{"frame": 0, "preparation": {"pipelines_ms": 52.}}]}
        rows = attribute(trace)
        self.assertEqual((rows[0]["frame"], rows[0]["reporter_frame"], rows[0]["outside_app_spans_ms"]), (0, 1, 8.))
        self.assertEqual(rows[0]["switch"]["preparation"]["pipelines_ms"], 52.)
        for frame in [0, 3]:
            bad = copy.deepcopy(trace); bad["frames"][1]["frame"] = frame
            with self.assertRaises(RuntimeError):
                attribute(bad)
        for value in [None, float("nan"), -1.]:
            bad = copy.deepcopy(trace); bad["frames"][1]["previous_eframe_cpu_ms"] = value
            with self.assertRaises(RuntimeError):
                attribute(bad)
        bad = copy.deepcopy(trace); bad["switch_steps"] *= 2
        with self.assertRaises(RuntimeError):
            attribute(bad)
