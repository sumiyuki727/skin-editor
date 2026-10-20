import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_gui import evaluate_budgets
from p0_process_metrics import summarize_regions, cycle_statistics


class BenchmarkResultTests(unittest.TestCase):
    def test_cycle_cpu_counts_work_and_wall_time_without_idle_or_round_boundary_loss(self):
        phases = [dict(phase="Cycle",elapsed_seconds=10,cycle=0,ui_frames=100),
                  dict(phase="RoundEnd",elapsed_seconds=12,cycle=10,ui_frames=160),
                  dict(phase="ClosedSettle",elapsed_seconds=15,cycle=20,ui_frames=220)]
        samples = [dict(elapsed_seconds=t,cpu_seconds=c) for t,c in [(9,1),(10.1,2),(12.1,3),(14.9,4),(15.1,100)]]
        result = cycle_statistics(samples,phases)
        self.assertEqual(result["sampled_cpu_seconds"],2)
        self.assertEqual(result["sampled_cpu_ms_per_document"],100)
        self.assertEqual(result["ui_frames"],120)
        self.assertEqual(result["phase_wall_seconds"],5)
        self.assertIsNone(cycle_statistics([],phases))
        self.assertIsNone(cycle_statistics(samples,[]))
    def test_page_summary_excludes_reserved_and_does_not_claim_commit_ownership(self):
        rows = [dict(allocation_base=4096, size=4096, state=0x1000, type=0x20000),
                dict(allocation_base=4096, size=8192, state=0x1000, type=0x20000),
                dict(allocation_base=32768, size=65536, state=0x2000, type=0x20000),
                dict(allocation_base=65536, size=16384, state=0x1000, type=0x40000),
                dict(allocation_base=131072, size=4096, state=0x1000, type=0x1000000)]
        result = summarize_regions(rows)
        self.assertEqual(result["committed_region_bytes"], dict(private=12288, mapped=16384, image=4096, other=0))
        self.assertEqual(result["largest_committed_allocations"][1]["bytes"], 12288)
        self.assertEqual(summarize_regions([])["largest_committed_allocations"], [])
    def inputs(self):
        result = {"eframe_cpu_frame_ms_during_edits": {"p95": 1},
                  "edit_to_gpu_readback_ms": {"p95": 10, "count": 300},
                  "per_edit_gpu_readback": True, "pause_png_refresh_verified": True,
                  "edit_count": 300, "cycles_completed": 20, "idle_ui_frames": 1,
                  "first_document_original_decode_count": 10,
                  "phases": [{"phase": "Closed", "ui_frames": 400},
                             {"phase": "Done", "ui_frames": 401,
                              "original_decoded_images_live": 0, "owned_textures_in_renderer": 0}]}
        summary = [{"phase": "Empty", "working_set_bytes_max": 100 * 1048576},
                   {"phase": "Edit", "working_set_bytes_max": 300 * 1048576},
                   {"phase": "Idle", "working_set_bytes_max": 300 * 1048576,
                    "duration_seconds": 60, "cpu_one_logical_core_percent": 0.1},
                   {"phase": "Closed", "working_set_bytes_max": 100 * 1048576,
                    "duration_seconds": 60, "cpu_one_logical_core_percent": 0.1}]
        return result, summary

    def test_no_readback_never_claims_pixel_or_latency_verification(self):
        result, summary = self.inputs()
        result.update(per_edit_gpu_readback=False, pause_png_refresh_verified=None,
                      edit_to_gpu_readback_ms=None)
        checks, _, _ = evaluate_budgets(result, summary)
        self.assertIsNone(checks["edit_readback_p95_le_50_ms"])
        self.assertIsNone(checks["at_least_300_readback_samples"])
        self.assertIsNone(checks["pause_png_refresh_verified"])
        self.assertTrue(checks["at_least_300_edit_samples"])

    def test_live_refs_and_continuing_idle_frames_fail_separately(self):
        result, summary = self.inputs()
        checks, _, _ = evaluate_budgets(result, summary)
        self.assertTrue(all(checks.values()))
        result["phases"][-1].update(ui_frames=450, original_decoded_images_live=1,
                                    owned_textures_in_renderer=2)
        checks, _, _ = evaluate_budgets(result, summary)
        for key in ("closed_no_continuous_redraw", "original_decoded_images_released",
                    "owned_textures_released_from_renderer"):
            self.assertFalse(checks[key])

    def test_missing_or_over_budget_metrics_do_not_pass(self):
        result, summary = self.inputs()
        missing, _, _ = evaluate_budgets(result, [])
        self.assertFalse(missing["standard_working_set_le_512_mib"])
        self.assertFalse(missing["closed_cpu_le_2_percent_one_core"])
        expensive = copy.deepcopy(summary)
        expensive[1]["working_set_bytes_max"] = 513 * 1048576
        expensive[3]["cpu_one_logical_core_percent"] = 2.1
        checks, _, _ = evaluate_budgets(result, expensive)
        self.assertFalse(checks["standard_working_set_le_512_mib"])
        self.assertFalse(checks["closed_cpu_le_2_percent_one_core"])

    def test_async_limits_and_stale_results_require_evidence(self):
        result, summary = self.inputs()
        result.update(async_png_loading=True)
        checks, _, _ = evaluate_budgets(result, summary)
        for key in ("loading_frame_cpu_p95_le_16_7_ms", "per_frame_upload_bytes_bounded",
                    "per_frame_upload_count_bounded", "obsolete_decode_result_rejected", "worker_idle_at_finish"):
            self.assertFalse(checks[key])
        self.assertIsNone(checks["rapid_switch_winner_verified"])
        result.update(loading_cpu_frame_ms={"p95": 2}, max_upload_bytes_in_one_frame=8*1048576,
                      max_upload_count_in_one_frame=2, rapid_switch_winner_verified=True,
                      loader={"stale_results_rejected": 1, "idle_at_finish": True})
        checks, _, _ = evaluate_budgets(result, summary)
        self.assertTrue(all(checks.values()))
        result["max_upload_bytes_in_one_frame"] += 1
        checks, _, _ = evaluate_budgets(result, summary)
        self.assertFalse(checks["per_frame_upload_bytes_bounded"])


if __name__ == "__main__":
    unittest.main()
