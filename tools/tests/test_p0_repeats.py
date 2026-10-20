from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from p0_gpu_repeats import ORDERS, aggregate, case_metrics, distribution
from p0_modules import graphic_modules


class RepetitionTests(unittest.TestCase):
    def test_orders_balance_each_position_and_condition(self):
        for position in range(3):
            self.assertEqual({order[position] for order in ORDERS}, set(ORDERS[0]))
        for order in ORDERS:
            self.assertEqual(len(order), len(set(order)))

    def test_ranges_expose_missing_values_and_do_not_pool_trials(self):
        self.assertEqual(distribution([1, 8, None]), dict(count=3, valid=2, min=1, median=4.5, max=8))
        self.assertIsNone(distribution([None])["median"])
        cases = [dict(condition="reuse80", summary=dict(latency=n)) for n in (1, 2, 100)]
        result = aggregate(cases)
        self.assertEqual(result["reuse80"]["latency"]["median"], 2)
        self.assertEqual(result["fresh128"]["latency"]["valid"], 0)

    def test_cycle_latency_excludes_initial_completions_and_requires_uninstrumented_runs(self):
        result = dict(status="passed", heap_profile_build=False, gpu_readbacks_enabled=False,
                      document_load_wall_ms=dict(p95=9999), loading_cpu_frame_ms=dict(p95=2))
        metrics = dict(cycle_statistics=dict(documents=200, sampled_cpu_seconds=2, phase_wall_seconds=3),
                       samples=[dict(private_bytes=8, peak_commit_bytes=9, working_set_bytes=4, gpu_dedicated_bytes=None)])
        timings = dict(document_load_wall_ms=[9999] * 3 + list(range(200)))
        summary = case_metrics(result, metrics, timings)
        self.assertEqual(summary["cycle_document_p95_ms"], 189)
        self.assertIsNone(summary["gpu_dedicated_peak_bytes"])
        result["heap_profile_build"] = True
        with self.assertRaises(ValueError):
            case_metrics(result, metrics, timings)

    def test_failed_module_snapshot_never_means_clean_graphics_environment(self):
        missing = graphic_modules(dict(complete=False, modules=[]))
        self.assertTrue(all(value is None for value in missing.values()))
        partial = graphic_modules(dict(complete=False, modules=[dict(name="Graphics-Hook64.dll")]))
        self.assertTrue(partial["graphics-hook64.dll"])
        self.assertIsNone(partial["nvspcap64.dll"])
        clean = graphic_modules(dict(complete=True, modules=[]))
        self.assertTrue(all(value is False for value in clean.values()))


if __name__ == "__main__":
    unittest.main()
