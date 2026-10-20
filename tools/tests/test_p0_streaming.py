import copy
import math
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import wave

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_streaming import compare, digest, fnv, fnv_pcm, make_fixture, reference, summarize_samples, validate


def record(ref, mode='stream'):
    return {'schema_version': 1, 'status': 'passed', 'mode': mode, 'source': 'self_authored_canonical_pcm16_wav',
            'audio_device_opened': False, 'system_capture_used': False, 'idle_ms': 350, 'idle_unchanged': True,
            'rate': 48000, 'channels': 2, 'frames': ref['frames'], 'pcm_hash': ref['pcm_hash'], 'encoded_bytes': ref['encoded_bytes'],
            'decoded_payload_bytes': ref['frames'] * 8, 'retained_pcm_bytes': ref['frames'] * 8 if mode == 'eager' else 0,
            'caller_buffer_bytes': 32768, 'cache_bytes': 32768, 'source_buffer_bytes': 65536, 'sequential_decoded_frames': ref['frames'],
            'open_read_bytes': 1068, 'sequential_read_bytes': ref['encoded_bytes'] + 44,
            'sequential_packets': math.ceil(ref['frames'] / 1152), 'largest_packet_frames': 1152,
            'open_ms': .1, 'first_pull_us': 10, 'read_sum_us': 1000, 'read_p95_us': 10, 'read_max_us': 20, 'work_ms': 5,
            'seeks': [{**s, 'wall_us': 20} for s in ref['seeks']]}


class StreamingTools(unittest.TestCase):
    def test_generator_is_bounded_self_authored_and_preserves_existing_inputs(self):
        with tempfile.TemporaryDirectory(prefix='p0-stream-fixture-') as temp:
            dest = Path(temp) / 'fixture'; m = make_fixture(dest, (1, 2)); before = digest(dest / 'short.wav')
            with wave.open(str(dest / 'long.wav'), 'rb') as f: self.assertEqual(f.getparams()[:4], (2, 2, 48000, 96000))
            self.assertEqual(m['files']['short.wav']['sha256'], before)
            with self.assertRaises(FileExistsError): make_fixture(dest)
            self.assertEqual(digest(dest / 'short.wav'), before)
            for durations in [(0, 2), (1, 601), (1.0, 2), (1,)]:
                with self.assertRaises(ValueError): make_fixture(Path(temp) / 'invalid', durations)

    def test_independent_pcm_reference_preserves_signed_channels_and_float_bits(self):
        self.assertEqual(fnv_pcm(struct.pack('<hhhh', -32768, 32767, 0, 16384)), struct.pack('<ffff', -1., 32767 / 32768, 0., .5))
        self.assertEqual(fnv(b'hello'), 0xa430d84680aabd0b)
        self.assertEqual(fnv(b'cd', fnv(b'ab')), fnv(b'abcd'))

    def test_evidence_rejects_full_read_ahead_stale_pcm_wrong_identity_and_false_device_claims(self):
        with tempfile.TemporaryDirectory(prefix='p0-stream-reference-') as temp:
            dest = Path(temp) / 'fixture'; make_fixture(dest, (1, 2)); ref = reference(dest / 'long.wav'); good = record(ref)
            validate(good, ref); validate(record(ref, 'eager'), ref)
            mutations = [lambda r: r.update(pcm_hash=1), lambda r: r.update(open_read_bytes=ref['encoded_bytes']),
                         lambda r: r.update(cache_bytes=65536), lambda r: r.update(retained_pcm_bytes=1),
                         lambda r: r.update(sequential_decoded_frames=ref['frames'] - 1), lambda r: r.update(idle_unchanged=False),
                         lambda r: r['seeks'][1].update(generation=1), lambda r: r['seeks'][2].update(hash=0),
                         lambda r: r.update(audio_device_opened=True), lambda r: r.update(work_ms=float('nan'))]
            for mutate in mutations:
                bad = copy.deepcopy(good); mutate(bad)
                with self.assertRaises(RuntimeError): validate(bad, ref)

    def test_metrics_require_fresh_baseline_and_separate_sampled_and_process_peaks(self):
        rows = [{'elapsed_seconds': i * .02, 'phase': 'baseline' if i < 10 else 'work', 'private_bytes': 100 + i, 'peak_commit_bytes': 500,
                 'peak_working_set_bytes': 300, 'cpu_seconds': i * .001} for i in range(20)]
        self.assertEqual(summarize_samples(rows)['private_growth_bytes'], 400)
        self.assertEqual(summarize_samples(rows)['peak_commit_bytes'], 500)
        with self.assertRaises(RuntimeError): summarize_samples(rows[:3])
        with self.assertRaises(RuntimeError): summarize_samples([{**r, 'phase': 'work'} for r in rows])

    def test_delayed_metrics_setup_uses_explicit_idle_baseline_not_process_start_time(self):
        rows = [{'elapsed_seconds': 2 + i * .02, 'phase': 'baseline' if i < 10 else 'work',
                 'private_bytes': 100, 'peak_commit_bytes': 500, 'peak_working_set_bytes': 300,
                 'cpu_seconds': .1} for i in range(20)]
        self.assertEqual(summarize_samples(rows)['baseline_private_bytes'], 100)

    def test_comparison_fails_when_growth_is_unbounded_or_baseline_cost_is_missing(self):
        mib = 1024 * 1024
        rows = [{'mode': mode, 'input': name, 'resources': {'peak_commit_bytes': peak * mib},
                 'result': {'decoded_payload_bytes': 64 * mib}} for mode, name, peak in
                [('stream', 'short.wav', 3), ('stream', 'long.wav', 4), ('eager', 'long.wav', 70)]]
        self.assertEqual(compare(rows)['status'], 'passed')
        bad = copy.deepcopy(rows);bad[1]['resources']['peak_commit_bytes'] = 30 * mib
        self.assertEqual(compare(bad)['status'], 'failed')
        bad = copy.deepcopy(rows);bad[2]['resources']['peak_commit_bytes'] = 5 * mib
        self.assertEqual(compare(bad)['status'], 'failed')
        with self.assertRaises(RuntimeError): compare(rows[:2])


if __name__ == '__main__': unittest.main()
