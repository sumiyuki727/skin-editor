import copy
import unittest
from measure_audio_edit import PHASES, analyze


def fixture():
    report = dict(hot_ms=[[10, 1, 1, 1, 1, 1]]*300, warm_ms=[[20, 1, 1, 1, 1, 1]]*64,
                  hot_native_upper_ms=[30]*300, warm_native_upper_ms=[40]*64, warm_file_stage_ms=[2]*64,
                  queued_hot=300, queued_warm=64, new_origin_triggers=364, draft_single_commit=True,
                  cancel_history_cleanup=True, settled_old_pcm_released=True, steady_empty_frames=0, startup_empty_frames=0, retimed_idle_frames=1,
                  pcm_final_bytes=0, raw_final_bytes=0, pcm_peak_bytes=100)
    samples = [dict(phase=p, time=t, cpu_seconds=t*.01+i*100, working_set_bytes=10, private_bytes=20)
               for i, p in enumerate(PHASES) for t in range(4)]
    return report, samples


class AudioEditChecks(unittest.TestCase):
    def test_warm_read_is_separate_and_added_for_budget(self):
        result = analyze(*fixture())
        self.assertEqual(result["latency"]["warm_file_plus_activation_ms"]["p95"], 22)
        self.assertTrue(all(result["checks"].values()))

    def test_missing_samples_nonfinite_and_reverse_clocks_are_rejected(self):
        for field, value in (("hot_ms", []), ("warm_ms", [[1]]*64), ("warm_file_stage_ms", [float("nan")]*64),
                             ("hot_native_upper_ms", [0]*300)):
            report, samples = fixture(); report[field] = value
            with self.assertRaises(ValueError):
                analyze(report, samples)

    def test_latency_cache_cleanup_and_memory_fail_closed(self):
        report, samples = fixture()
        report.update(hot_ms=[[51, 1, 1, 1, 1, 1]]*300, hot_native_upper_ms=[60]*300,
                      warm_file_stage_ms=[240]*64, queued_warm=63, pcm_final_bytes=1,
                      settled_old_pcm_released=False, steady_empty_frames=1, startup_empty_frames=1, retimed_idle_frames=0)
        samples = copy.deepcopy(samples); samples[5]["private_bytes"] = 513*1024**2
        gates = analyze(report, samples)["checks"]
        for key in ("hot_300_p95_le_50_ms", "warm_file_plus_activation_p95_le_250_ms", "all_preparations_real_cache_misses",
                    "managed_resources_released", "settled_old_pcm_released", "no_startup_underflow", "idle_retiming_exercised", "no_steady_underflow", "audio_only_process_memory_le_512_mib"):
            self.assertFalse(gates[key])


if __name__ == "__main__":
    unittest.main()
