import copy
import unittest
from measure_audio_load import PHASES, checks, distribution, summarize


def samples():
    return [dict(phase=p, time=t, cpu_seconds=100*i+t*.01,
                 working_set_bytes=10, private_bytes=20)
            for i, p in enumerate(PHASES) for t in range(7)]


class AudioLoadChecks(unittest.TestCase):
    def test_phase_cpu_never_includes_other_phase_transitions(self):
        result = summarize(samples())
        for m in result.values():
            self.assertAlmostEqual(m["cpu_one_logical_core_percent"], 1)
            self.assertEqual(m["sample_span_seconds"], 6)

    def test_missing_counters_phases_short_spans_and_negative_cpu_fail(self):
        base = samples()
        variants = [[s for s in base if s["phase"] != "closed"], copy.deepcopy(base), copy.deepcopy(base), copy.deepcopy(base)]
        variants[1][0]["private_bytes"] = 0
        variants[2][6]["cpu_seconds"] = -1
        variants[3] = [s for s in base if s["time"] < 2]
        for data in variants:
            with self.assertRaises(ValueError):
                summarize(data)

    def test_percentile_uses_nearest_rank_and_invalid_samples_fail(self):
        self.assertEqual(distribution(list(range(1, 101)))["p95"], 95)
        for bad in ([], [float("nan")], [-1], [float("inf")]):
            with self.assertRaises(ValueError):
                distribution(bad)

    def test_latency_underflow_and_memory_limits_are_not_silently_waived(self):
        report = dict(latency_upper_bound_ms=[[19]*64, [20]*64], steady_empty_frames=0, startup_empty_frames=0, retimed_idle_frames=1,
                      peak_voices=32, capacity_refusals=4, dual_channel_isolation=True,
                      pcm_final_bytes=0, pcm_peak_bytes=100)
        metrics = summarize(samples())
        self.assertTrue(all(checks(report, metrics).values()))
        report["latency_upper_bound_ms"][0] = [21]*64
        report["steady_empty_frames"] = 1
        report["startup_empty_frames"] = 1
        report["retimed_idle_frames"] = 0
        metrics["voices_32"]["private_bytes_max"] = 513*1024**2
        self.assertFalse(checks(report, metrics)["native_submission_observed_upper_bound_p95_le_20_ms"])
        self.assertFalse(checks(report, metrics)["no_steady_underflow"])
        self.assertFalse(checks(report, metrics)["no_startup_underflow"])
        self.assertFalse(checks(report, metrics)["idle_retiming_exercised"])
        self.assertFalse(checks(report, metrics)["audio_only_process_memory_le_512_mib"])


if __name__ == "__main__":
    unittest.main()
