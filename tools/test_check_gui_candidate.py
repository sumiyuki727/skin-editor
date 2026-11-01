import unittest
from measure_gui_candidate import PLAN, evaluate_gui
from test_check_worker_native import report as worker_report, metrics


def report():
    r = worker_report()
    r.update(schema=2, scope="gui-action-dispatch-to-native-copy", physical_input=False, controlled_focus=True, gui_passes=200)
    r["visual"] = [[i, i, 2, 2, 2, 1, 1024, 768, False, False] for i in range(1,18)]
    r["max_live_targets"] = 2
    r["visual"][11][8:10] = [True, True]; r["ui_cost"] = [[1, 100, 100] for _ in range(18)]
    r["lifecycle"] = [[i, float(i), 0, 0, 0, 0, True, False, 0] for i in range(1,18)]
    r["lifecycle"][5][3] = 16; r["lifecycle"][9][6] = False
    for row in r["probes"]: row[4:6] = [12., 24.]
    return r


class GuiCandidateChecks(unittest.TestCase):
    def test_fixed_plan_and_distinct_native_vs_gui_observation(self):
        self.assertEqual([v[0] for v in PLAN], ["worker_before", "gui_1", "gui_2", "worker_after"])
        lat, joined, checks = evaluate_gui(report(), metrics())
        self.assertTrue(all(checks.values())); self.assertEqual(lat[0]["p95"], 12.); self.assertEqual(joined[0]["p95"], 24.)

    def test_wrong_scope_physical_claim_or_missing_gui_rejected(self):
        for key, value in (("scope", "bounded-worker-full-request"), ("physical_input", True), ("controlled_focus", False), ("gui_passes", 0)):
            r = report(); r[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError): evaluate_gui(r, metrics())

    def test_identity_missing_or_invalid_timing_rejected(self):
        for case in ("missing", "request", "voice", "negative", "nan", "reversed", "offset"):
            r = report()
            if case == "missing": r["probes"].pop()
            elif case == "request": r["probes"][1][2] = r["probes"][0][2]
            elif case == "voice": r["probes"][1][3] = r["probes"][0][3]
            elif case == "negative": r["probes"][0][4] = -1
            elif case == "nan": r["probes"][0][4] = float("nan")
            elif case == "reversed": r["probes"][0][5] = 11
            else: r["probes"][0][8] = 480
            with self.subTest(case=case), self.assertRaises(ValueError): evaluate_gui(r, metrics())

    def test_original_latency_underrun_resource_and_cleanup_limits_kept(self):
        for key, value in (("missing_frames", 1), ("pcm_final_bytes", 4), ("endpoint_returns", 0), ("cancel_clean", False), ("history_clean", False), ("pause_isolated", False), ("render_errors", 1)):
            r = report(); r[key] = value
            with self.subTest(key=key): self.assertFalse(all(evaluate_gui(r, metrics())[2].values()))
        r = report()
        for row in r["probes"][:12]: row[4] = 20.001
        self.assertFalse(evaluate_gui(r, metrics())[2]["gui_request_copy_observed_upper_p95_le_20_ms"])
        for key, value in (("cpu_one_logical_core_percent", 15.01), ("private_bytes_max", 512*1024**2+1)):
            m = metrics(); m["test"][key] = value
            self.assertFalse(all(evaluate_gui(report(), m)[2].values()))

    def test_lifecycle_cannot_be_replaced_by_success_flags(self):
        r = report(); r["lifecycle"].pop()
        with self.assertRaises(ValueError): evaluate_gui(r, metrics())
        r = report(); r["lifecycle"][5][3] = 0
        self.assertFalse(evaluate_gui(r, metrics())[2]["observed_gui_lifecycle"])
        r = report(); r["lifecycle"][9][1] = -1
        with self.assertRaises(ValueError): evaluate_gui(r, metrics())
        r = report(); r["visual"][11][8] = False
        self.assertFalse(evaluate_gui(r, metrics())[2]["verified_draft_voice_then_cancel"])
        r = report(); r["visual"][4][2] += 2
        self.assertFalse(evaluate_gui(r, metrics())[2]["bounded_static_publication"])


if __name__ == "__main__": unittest.main()
