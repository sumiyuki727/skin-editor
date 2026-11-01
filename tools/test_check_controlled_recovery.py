import copy
import unittest
from measure_controlled_recovery import PLAN, evaluate_controlled
from test_check_supply_recovery import inputs


def controlled():
    report, samples = inputs()
    report.update(recovery_probe_mode='AfterRefill', recovery_passes=320,
        cleanup_stopped=True,
        diagnostic_recovery_passes=320, diagnostic_refill_passes=320,
        diagnostic_refill_frames=320*512,
        recovery_rows=[[[2*i+lane, 2*i+lane+1, 2*i+lane, 2*i+lane+1,
                         (2*i+lane)*512, (2*i+lane+1)*512] for i in range(160)] for lane in range(2)])
    return report, samples


class ControlledRecoveryChecks(unittest.TestCase):
    def test_real_refill_rows_and_frozen_plan_are_required(self):
        report, samples = controlled()
        self.assertEqual([row[1] for row in PLAN], ['Natural', 'WithTrigger', 'AfterRefill', 'Natural'])
        result = evaluate_controlled(report, samples, 'RecoverLateWake', 'AfterRefill')
        self.assertTrue(all(result['checks'].values()))
        self.assertEqual(result['actual_diagnostic_refills'], 320)

    def test_counter_discontinuity_missing_mode_and_fake_commit_are_rejected(self):
        report, samples = controlled()
        for change in ['mode', 'missing', 'jump', 'unaligned', 'final']:
            bad = copy.deepcopy(report)
            if change == 'mode': bad['recovery_probe_mode'] = 'Natural'
            if change == 'missing': bad['recovery_rows'][1].pop()
            if change == 'jump': bad['recovery_rows'][0][4][0] += 1
            if change == 'unaligned': bad['recovery_rows'][1][8][5] += 1
            if change == 'final': bad['diagnostic_refill_passes'] -= 1
            with self.subTest(change=change), self.assertRaises(ValueError):
                evaluate_controlled(bad, samples, 'RecoverLateWake', 'AfterRefill')

    def test_latency_failure_is_not_hidden_by_successful_recovery(self):
        report, samples = controlled()
        for lane in range(2):
            report['latency_upper_bound_ms'][lane] = [29]*160
            report['submission_intervals_ms'][lane] = [[27,29,2,2,0] for _ in range(160)]
            report['exact_voice_intervals_ms'][lane] = [[27,29] for _ in range(160)]
        result = evaluate_controlled(report, samples, 'RecoverLateWake', 'AfterRefill')
        self.assertTrue(result['checks']['controlled_refills_committed'])
        self.assertFalse(result['checks']['original_submission_p95_le_20_ms'])
        self.assertFalse(result['checks']['exact_render_p95_le_20_ms'])
