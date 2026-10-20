import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_resource_cycles import budgets, phase_key, summaries, trends, validate


def fixture():
    cycles = []
    for n in range(2):
        closed = dict(final_stats=dict(active=False, in_flight=False, free_blocks=5, queued_blocks=0,
                       underruns=0, decode_failures=0, trace_overflow=False, max_queued_blocks=4), event_count=10, submit_count=7)
        pause = dict(wall_seconds=1.1, first=dict(phase='Paused', sequence=100, position_us=1100000),
                     last=dict(phase='Paused', sequence=110, position_us=1100000))
        cycles.append(dict(cycle=n, document=n, continuous_device_us=1000000, changes=60,
                           frame_samples=58, command_nodes=2, closed=closed, pause=pause))
    proof = dict(pixels=1024 * 1024, pixels_outside_tolerance=0, display_pixels_outside_tolerance=0,
                 max_channel_error=1, display_max_channel_error=1)
    r = dict(schema_version=1, status='passed', errors=[], smoke=True, target_cycles=2, cycles=cycles,
             accepted_documents=2, uploads=[{}, {}], session_owned_at_finish=False,
             active_document_at_finish=False, background_pending_at_finish=False, system_capture_used=False,
             physical_or_acoustic_latency_measured=False, normal_path_cpu_pixel_readbacks=0,
             pixel_checks=[dict(proof, stage=0), dict(proof, stage=1)],
             eframe_cpu_frame_ms_playing=[1], frame_intervals_ms_playing=[17])
    return r, [dict(events=10, submits=7), dict(events=10, submits=7)]


class ResourceCycleTests(unittest.TestCase):
    def test_complete_independent_cycles_are_accepted(self):
        r, traces = fixture(); validate(r, traces, True)

    def test_missing_or_reused_cycle_cannot_claim_twenty_cycles(self):
        r, traces = fixture()
        with self.assertRaises(RuntimeError): validate(r, traces, False)
        r['cycles'][1]['cycle'] = 0
        with self.assertRaises(RuntimeError): validate(r, traces, True)

    def test_wrong_document_and_trace_counts_are_rejected(self):
        r, traces = fixture(); r['cycles'][1]['document'] = 0
        with self.assertRaises(RuntimeError): validate(r, traces, True)
        r, traces = fixture(); traces[1]['submits'] += 1
        with self.assertRaises(RuntimeError): validate(r, traces, True)

    def test_unretired_blocks_faults_and_pending_graphics_are_rejected(self):
        for key, value in [('free_blocks', 4), ('queued_blocks', 1), ('in_flight', True), ('underruns', 1), ('trace_overflow', True)]:
            r, traces = fixture(); r['cycles'][1]['closed']['final_stats'][key] = value
            with self.assertRaises(RuntimeError): validate(r, traces, True)
        r, traces = fixture(); r['background_pending_at_finish'] = True
        with self.assertRaises(RuntimeError): validate(r, traces, True)

    def test_fresh_paused_device_sequence_and_position_are_both_required(self):
        for key, value in [('sequence', 100), ('position_us', 1100001)]:
            r, traces = fixture(); r['cycles'][1]['pause']['last'][key] = value
            with self.assertRaises(RuntimeError): validate(r, traces, True)

    def test_final_pixel_proof_cannot_be_omitted(self):
        r, traces = fixture(); r['pixel_checks'].pop()
        with self.assertRaises(RuntimeError): validate(r, traces, True)

    def test_repeated_phase_names_keep_cycle_and_transition_identity(self):
        row = dict(cycle=0, phase='Closed', step=8)
        self.assertNotEqual(phase_key(row), phase_key(dict(row, cycle=1)))
        self.assertNotEqual(phase_key(row), phase_key(dict(row, step=9)))

    def test_active_cpu_does_not_include_intervening_closed_time(self):
        samples = []
        for cycle, start, cpu in [(0, 0, 0), (1, 100, 10)]:
            for elapsed in (1, 2):
                samples.append(dict(cycle=cycle, phase='Play', phase_age_seconds=elapsed,
                                    monotonic_seconds=start + elapsed, cpu_seconds=cpu + elapsed,
                                    working_set_bytes=1, private_bytes=2, gpu_dedicated_bytes=3, gpu_shared_bytes=4))
        result = summaries(samples, 2)['play']
        self.assertEqual(result['active_sample_seconds'], 2)
        self.assertEqual(result['cpu_one_logical_core_percent'], 100)

    def test_memory_trends_distinguish_flat_and_accumulating_baselines(self):
        rows = [dict(working_set_bytes_last=(100 + n) * 1048576, private_bytes_last=200 * 1048576,
                     gpu_dedicated_bytes_last=None, gpu_shared_bytes_last=0) for n in range(20)]
        result = trends(rows)
        self.assertEqual(result['working_set_bytes']['linear_slope_mib_per_cycle'], 1)
        self.assertEqual(result['working_set_bytes']['median_delta_mib'], 15)
        self.assertEqual(result['private_bytes']['linear_slope_mib_per_cycle'], 0)

    def test_unavailable_gpu_counter_remains_unknown(self):
        row = dict(working_set_bytes_last=1, private_bytes_last=2, gpu_dedicated_bytes_last=None, gpu_shared_bytes_last=0)
        self.assertIsNone(trends([row, copy.copy(row)])['gpu_dedicated_bytes'])

    def test_short_observation_does_not_pass_resource_gates(self):
        s = dict(closed_cycles=[dict(sample_span_seconds=2)] * 20,
                 final_idle=dict(sample_span_seconds=30, cpu_one_logical_core_percent=0))
        r = dict(final_idle_frames=1, eframe_cpu_frame_ms_playing=[1])
        result = budgets(r, s, dict(working_set_bytes=100 * 1048576), False)
        self.assertFalse(result['all_twenty_cycles_sampled_after_settle'])
        self.assertFalse(result['final_idle_observed_at_least_60_seconds'])
        self.assertIsNone(budgets(r, s, {}, True))


if __name__ == '__main__': unittest.main()
