import copy
import unittest
from measure_supply_recovery import PLAN, evaluate
from test_check_voice_phase import fixture
from test_check_scheduling import timing


def inputs():
    report = fixture(160)
    scheduling = timing()
    scheduling.update(slow_count=0, slow=[], event_count=0, events=[])
    report.update(supply_policy='RecoverLateWake', recovery_passes=1, max_recovery_target=1280,
                  scheduling=scheduling)
    samples = [dict(phase=phase, time=t, cpu_seconds=t*.01, working_set_bytes=100,
                    private_bytes=100, lifetime_peak_working_set_bytes=100,
                    lifetime_peak_pagefile_bytes=100)
               for phase, t in [('phase_scan', 0), ('phase_scan', 12), ('closed', 13), ('closed', 15)]]
    return report, samples


class SupplyRecoveryChecks(unittest.TestCase):
    def test_interleaved_plan_and_exact_existing_acceptance(self):
        self.assertEqual(PLAN, ('FixedAhead', 'RecoverLateWake', 'RecoverLateWake', 'FixedAhead'))
        report, samples = inputs()
        self.assertTrue(all(evaluate(report, samples, PLAN[1])['checks'].values()))

    def test_policy_identity_and_bounded_counters_are_required(self):
        report, samples = inputs()
        with self.assertRaises(ValueError):
            evaluate(report, samples, 'FixedAhead')
        for count, target in [(-1, 1280), (1, 2049), (0, 1280), (1, 0)]:
            bad = copy.deepcopy(report)
            bad.update(recovery_passes=count, max_recovery_target=target)
            with self.assertRaises(ValueError):
                evaluate(bad, samples, 'RecoverLateWake')

    def test_empty_frames_and_latency_straddling_are_not_waived(self):
        report, samples = inputs()
        report['exact_voice_intervals_ms'] = [[[19, 21] for _ in range(160)] for _ in range(2)]
        self.assertFalse(evaluate(report, samples, PLAN[1])['checks']['exact_render_p95_le_20_ms'])
        from test_check_voice_phase import with_underrun
        report, samples = inputs()
        self.assertFalse(evaluate(with_underrun(report), samples, PLAN[1])['checks']['no_underflow'])

    def test_cpu_memory_and_incomplete_capture_fail_without_relabelling(self):
        report, samples = inputs()
        samples[1]['cpu_seconds'] = 2
        samples[1]['lifetime_peak_pagefile_bytes'] = 513*1024**2
        report['scheduling']['recent'][3] = None
        result = evaluate(report, samples, PLAN[1])
        self.assertFalse(result['checks']['cpu_one_core_le_15_percent'])
        self.assertFalse(result['checks']['lifetime_peak_le_512_mib'])
        self.assertFalse(result['checks']['scheduling_complete'])
