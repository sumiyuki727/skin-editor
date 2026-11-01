import copy
import unittest
from measure_separated_native import evaluate


def report():
    result = dict(schema=1, rate=48000, channels=2,
                  probes=[[i, lane, i*2+lane+1, 12., 123, i+1, 0, 480] for i in range(64) for lane in range(2)],
                  missing_frames=0, device_fault=False, render_errors=0, observation_overflows=0, callbacks=100,
                  nonzero_samples=100, peak_voices=32, refused_33=True, endpoint_returns=1, pcm_final_bytes=0, pcm_peak_bytes=5000000)
    result.update({key: True for key in ("burst_exact", "hot_continuity", "audition_survives_pause", "pause_stable", "resumed", "seek_exact", "seeking_resumed", "source_not_eof")})
    return result


def metrics():
    return {"test": dict(cpu_one_logical_core_percent=10., working_set_bytes_max=10000000, private_bytes_max=10000000)}


class SeparatedNativeChecks(unittest.TestCase):
    def test_complete_exact_plan_passes(self):
        latency, checks = evaluate(report(), metrics())
        self.assertTrue(all(checks.values()))
        self.assertEqual([v["count"] for v in latency], [64, 64])
        with self.assertRaises(ValueError): evaluate(report(), {})

    def test_missing_duplicate_foreign_and_invalid_probes_rejected(self):
        for variant in ("missing", "duplicate", "voice", "nan", "offset", "lane", "schema"):
            r = report()
            if variant == "missing": r["probes"].pop()
            elif variant == "duplicate": r["probes"][1] = copy.copy(r["probes"][0])
            elif variant == "voice": r["probes"][1][2] = r["probes"][0][2]
            elif variant == "nan": r["probes"][0][3] = float("nan")
            elif variant == "offset": r["probes"][0][6] = 480
            elif variant == "lane": r["probes"][0][1] = 2
            else: r["schema"] = 2
            with self.subTest(variant=variant), self.assertRaises(ValueError): evaluate(r, metrics())

    def test_underrun_and_latency_failure_are_retained(self):
        r = report(); r["missing_frames"] = 1
        for row in r["probes"][:12]: row[3] = 20.01
        _, checks = evaluate(r, metrics())
        self.assertFalse(checks["zero_missing_frames"])
        self.assertFalse(checks["native_first_copy_observed_upper_p95_le_20_ms"])

    def test_lifecycle_and_native_failure_are_not_silenced(self):
        for key, value in (("endpoint_returns", 0), ("pcm_final_bytes", 4), ("observation_overflows", 1), ("device_fault", True), ("source_not_eof", False)):
            r = report(); r[key] = value
            with self.subTest(key=key): self.assertFalse(all(evaluate(r, metrics())[1].values()))

    def test_budgets_keep_original_limits(self):
        for key, value in (("cpu_one_logical_core_percent", 15.01), ("working_set_bytes_max", 512*1024**2+1), ("private_bytes_max", 512*1024**2+1)):
            m = metrics(); m["test"][key] = value
            with self.subTest(key=key): self.assertFalse(all(evaluate(report(), m)[1].values()))


if __name__ == "__main__":
    unittest.main()
