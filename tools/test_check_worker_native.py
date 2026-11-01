import unittest
from measure_worker_native import evaluate_worker, PLAN


def report():
    r = dict(schema=1, scope="bounded-worker-full-request", rate=48000, channels=2,
             probes=[[i, lane, i*2+lane+1, i*2+lane+1, 2., 12., 123, i+1, 0, 480] for i in range(64) for lane in (0, 1)],
             missing_frames=0, device_fault=False, render_errors=0, observation_overflows=0, callbacks=100, nonzero_samples=1,
             peak_voices=32, refused_33=True, endpoint_returns=1, pcm_final_bytes=0, pcm_peak_bytes=7000000)
    r.update({k: True for k in ("burst_exact", "hot_continuity", "pause_isolated", "stop_quiet", "resumed", "cancel_clean", "history_clean", "source_not_eof")})
    return r


def metrics():
    return {"test": dict(cpu_one_logical_core_percent=12., working_set_bytes_max=20000000, private_bytes_max=10000000)}


class WorkerNativeChecks(unittest.TestCase):
    def test_fixed_plan_and_complete_requests(self):
        self.assertEqual([v[0] for v in PLAN], ["standalone_before", "worker_1", "worker_2", "standalone_after"])
        lat, receipts, checks = evaluate_worker(report(), metrics())
        self.assertTrue(all(checks.values())); self.assertEqual(lat[0]["count"], 64); self.assertEqual(receipts[1]["p95"], 2.)

    def test_scope_duplicates_missing_and_invalid_timing_rejected(self):
        for case in ("scope", "missing", "request", "voice", "negative", "reversed", "nan", "offset"):
            r = report()
            if case == "scope": r["scope"] = "runtime-only"
            elif case == "missing": r["probes"].pop()
            elif case == "request": r["probes"][1][2] = r["probes"][0][2]
            elif case == "voice": r["probes"][1][3] = r["probes"][0][3]
            elif case == "negative": r["probes"][0][4] = -1
            elif case == "reversed": r["probes"][0][4] = 13
            elif case == "nan": r["probes"][0][5] = float("nan")
            else: r["probes"][0][8] = 480
            with self.subTest(case=case), self.assertRaises(ValueError): evaluate_worker(r, metrics())

    def test_underrun_or_latency_cannot_pass(self):
        r = report(); r["missing_frames"] = 1
        for row in r["probes"][:12]: row[5] = 20.001
        checks = evaluate_worker(r, metrics())[2]
        self.assertFalse(checks["zero_missing_frames"]); self.assertFalse(checks["full_request_copy_observed_upper_p95_le_20_ms"])

    def test_cleanup_pause_and_runtime_errors_fail_independently(self):
        for key, value in (("cancel_clean", False), ("history_clean", False), ("stop_quiet", False), ("pause_isolated", False), ("endpoint_returns", 0), ("pcm_final_bytes", 4), ("render_errors", 1), ("observation_overflows", 1)):
            r = report(); r[key] = value
            with self.subTest(key=key): self.assertFalse(all(evaluate_worker(r, metrics())[2].values()))

    def test_original_cpu_memory_and_pcm_limits_kept(self):
        for key, value in (("cpu_one_logical_core_percent", 15.01), ("working_set_bytes_max", 512*1024**2+1), ("private_bytes_max", 512*1024**2+1)):
            m = metrics(); m["test"][key] = value
            with self.subTest(key=key): self.assertFalse(all(evaluate_worker(report(), m)[2].values()))
        r = report(); r["pcm_peak_bytes"] = 32*1024**2+1
        self.assertFalse(all(evaluate_worker(r, metrics())[2].values()))
        with self.assertRaises(ValueError): evaluate_worker(report(), {})


if __name__ == "__main__": unittest.main()
