import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_combined_soak import budgets, summarize_phases, timing, validate_packets


class Reference:
    def check(self, packet):
        if packet['hash'] != 123: raise RuntimeError('Incorrect source samples')


class CombinedSoakTests(unittest.TestCase):
    def trace(self, events):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'trace.jsonl'
            path.write_text(''.join(json.dumps(e) + '\n' for e in events), encoding='utf-8')
            return validate_packets(path, Reference())

    def events(self):
        return [
            dict(kind='Activate', generation=1, first_frame=0, frames=0, native_discarded_frames=0),
            dict(kind='Submit', generation=1, first_frame=0, frames=4800, packet=dict(source_frame=0, frames=4800, hash=123)),
            dict(kind='Pause', generation=2, first_frame=4000, frames=0, native_discarded_frames=800),
            dict(kind='Retire', generation=3, first_frame=4000, frames=0, native_discarded_frames=0)]

    def test_complete_native_lifetime_accounts_for_unread_tail(self):
        self.assertEqual(self.trace(self.events()), dict(events=4, submits=1, submitted_frames=4800))

    def test_discontinuity_or_wrong_samples_are_rejected(self):
        for field, value in [('first_frame', 1), ('generation', 2)]:
            events = self.events(); events[1][field] = value
            with self.assertRaises(RuntimeError): self.trace(events)
        events = self.events(); events[1]['packet']['hash'] = 456
        with self.assertRaises(RuntimeError): self.trace(events)

    def test_paused_native_submission_and_unaccounted_tail_are_rejected(self):
        events = self.events(); events.insert(3, events[1].copy())
        with self.assertRaises(RuntimeError): self.trace(events)
        events = self.events(); events[2]['native_discarded_frames'] = 700
        with self.assertRaises(RuntimeError): self.trace(events)

    def test_incomplete_retirement_is_rejected(self):
        with self.assertRaises(RuntimeError): self.trace(self.events()[:-1])

    def test_nonfinite_timing_is_rejected_and_percentile_uses_rank(self):
        with self.assertRaises(RuntimeError): timing([1, float('nan')])
        self.assertEqual(timing(list(range(1, 101)))['p95'], 95)

    def test_phase_transition_guard_excludes_early_samples(self):
        samples = [dict(phase='PausedIdle', phase_age_seconds=t, monotonic_seconds=t,
                        cpu_seconds=(10 if t == 0 else 10.01), working_set_bytes=100,
                        private_bytes=200, gpu_dedicated_bytes=300, gpu_shared_bytes=400)
                   for t in (0, 1, 62)]
        row = next(r for r in summarize_phases(samples) if r['phase'] == 'PausedIdle')
        self.assertEqual(row['sample_count'], 2)
        self.assertEqual(row['sample_span_seconds'], 61)
        self.assertEqual(row['cpu_one_logical_core_percent'], 0)

    def test_resident_budget_is_not_private_commit_budget(self):
        rows = [dict(phase=p, working_set_bytes_max=513 * 1024 * 1024,
                     private_bytes_max=1024, sample_span_seconds=61, cpu_one_logical_core_percent=0)
                for p in ('Playing', 'PausedIdle', 'ClosedIdle')]
        r = dict(paused_idle_frames=1, closed_idle_frames=1, eframe_cpu_frame_ms_playing=[1])
        self.assertFalse(budgets(r, rows, False)['playing_working_set_le_512_mib'])
        rows[0].update(working_set_bytes_max=500 * 1024 * 1024, private_bytes_max=1024 * 1024 * 1024)
        self.assertTrue(budgets(r, rows, False)['playing_working_set_le_512_mib'])
        self.assertIsNone(budgets(r, rows, True))


if __name__ == '__main__': unittest.main()
