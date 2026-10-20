import copy
from pathlib import Path
import sys
import tempfile
import unittest
import wave

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_stream_output import CHECKS, Reference, check_decoder_boundary, digest, make_fixture, validate


def stats(generation, origin=0, submitted=0):
    return {'generation': generation, 'active': True, 'queued_blocks': 0, 'queued_frames': 0, 'free_blocks': 5, 'in_flight': False,
            'max_queued_blocks': 4, 'stale_blocks': 3, 'discarded_blocks': 4, 'discarded_frames': 8192, 'native_discarded_frames': 4800,
            'underruns': 0, 'decode_failures': 0, 'trace_overflow': False,
            'pcm': {'requested_us': origin, 'source_origin_frame': origin * 48000 // 1000000, 'submitted_frames': submitted, 'packets': submitted // 4800}}


def record(ref):
    events = []; rows = []; checkpoints = []
    def event(kind, gen, frame, count=0, drop=0, packet=None):
        events.append({'kind': kind, 'generation': gen, 'first_frame': frame, 'frames': count, 'native_discarded_frames': drop, 'packet': packet})
    def submit(gen, frame, count): event('submit', gen, frame, count, packet=ref.packet(frame, count))
    def row(epoch, device, simulation, phase, q):
        rows.append({'epoch': epoch, 'sequence': len(rows) + 1, 'phase': phase, 'transport_phase': phase,
                     'device_us': device, 'simulation_us': simulation, 'buffered': q})
    def pause(name, gen, epoch, position, device, origin):
        q = stats(gen, origin)
        for _ in range(4): row(epoch, device, position, 'Paused', copy.deepcopy(q))
        checkpoints.append({'kind': 'pause', 'name': name, 'generation': gen, 'epoch': epoch, 'position_us': position,
                            'device_us': device, 'fresh_samples': 3, 'wall_ms': 350, 'stats': q})
    event('activate', 1, 0)
    for i in range(101):
        submit(1, i * 4800, 4800); row(0, i * 100000, i * 100000, 'Playing', stats(1, 0, (i + 1) * 4800))
    checkpoints.append({'kind': 'continuous', 'position_us': 10000000, 'stats': stats(1, 0, 484800)})
    event('pause', 2, 480000, 8192, 4800);event('stale', 1, 496000, 4096)
    pause('discard_inflight_on_pause', 2, 0, 10000000, 10000000, 10000000)
    submit(2, 480000, 4800);event('pause', 3, 484800);event('retire', 4, 484800);event('activate', 5, 960000)
    for i in range(5): submit(5, 960000 + i * 4800, 4800)
    event('pause', 6, 984000);event('retire', 7, 984000);event('activate', 8, 1200000);event('retire', 9, 1200000)
    event('activate', 10, 24000);event('stale', 8, 1200000, 4096)
    pause('discard_inflight_on_seek', 10, 3, 500000, 0, 500000)
    for i in range(5): submit(10, 24000 + i * 4800, 4800)
    row(3, 500000, 1000000, 'Playing', stats(10, 500000, 24000))
    event('underrun', 10, 48000);event('retire', 11, 48000);event('stale', 10, 50000, 4096)
    checkpoints.extend([{'kind': 'seek', 'epoch': e, 'position_us': p} for e, p in [(1, 20000000), (2, 25000000), (3, 500000)]])
    checkpoints.append({'kind': 'underrun', 'position_us': 1000000, 'wall_ms': 350})
    event('activate', 12, 48000);pause('recovery_requires_explicit_play', 12, 4, 1000000, 0, 1000000)
    for i in range(5): submit(12, 48000 + i * 4800, 4800)
    event('pause', 13, 72000);pause('final', 13, 4, 1500000, 500000, 1500000);event('retire', 14, 72000)
    final = stats(14, 1500000);final.update(active=False, underruns=1)
    return {'schema_version': 1, 'status': 'passed', 'source': 'self_authored_30_second_wav_stream',
            'config': {'rate': 48000, 'channels': 2, 'frames': 1440000, 'gain': .02, 'block_frames': 4096, 'queue_blocks': 4, 'pool_blocks': 5, 'trace_limit': 8192},
            'audio_format': {'rate': 48000, 'channels': 2, 'bits': 32, 'buffer_frames': 4800}, 'checks': {k: True for k in CHECKS},
            'accepted_inputs': 3, 'system_capture_used': False, 'microphone_or_loopback_used': False, 'physical_input_or_acoustic_latency_measured': False,
            'events': events, 'samples': rows, 'checkpoints': checkpoints, 'final_stats': final, 'final_simulation_us': 1500000,
            'held_pause': [1, 496000], 'held_seek': [8, 1200000], 'held_underrun': [10, 50000]}


class StreamOutputTools(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix='p0-stream-output-');cls.fixture = Path(cls.temp.name) / 'fixture'
        make_fixture(cls.fixture);cls.ref = Reference(cls.fixture);cls.good = record(cls.ref)
    @classmethod
    def tearDownClass(cls): cls.temp.cleanup()

    def test_fixture_is_self_authored_exact_length_and_never_overwrites(self):
        p = self.fixture / 'stream.wav';before = digest(p)
        with wave.open(str(p), 'rb') as f: self.assertEqual(f.getparams()[:4], (2, 2, 48000, 1440000))
        self.assertEqual(p.stat().st_size, 5760044)
        with self.assertRaises(FileExistsError): make_fixture(self.fixture)
        self.assertEqual(digest(p), before)

    def test_decoder_boundary_keeps_native_ui_files_and_wall_clock_out(self):
        check_decoder_boundary('use p0_media::stream::Stream; use p0_wasapi::buffered::Input;')
        for source in ['use std::fs::File;', 'use egui::Context;', 'unsafe {}', 'include_str!("outside.rs");', 'use std::time::Instant;']:
            with self.assertRaises(RuntimeError): check_decoder_boundary(source)

    def test_independent_reference_checks_all_channel_values_and_gain(self):
        p = self.ref.packet(600, 8);self.ref.check(p);self.assertGreater(p['nonzero_samples'], 0)
        bad = copy.deepcopy(p);bad['hash'] ^= 1
        with self.assertRaises(RuntimeError): self.ref.check(bad)

    def test_native_trace_rejects_old_generation_discontinuity_wrong_content_and_unrejected_holds(self):
        validate(self.good, self.ref)
        index = next(i for i, e in enumerate(self.good['events']) if e['kind'] == 'submit')
        mutations = [lambda r: r['events'][index].update(generation=0), lambda r: r['events'][index].update(first_frame=1),
                     lambda r: r['events'][index]['packet'].update(hash=0), lambda r: r.update(held_seek=[1, 1])]
        for mutate in mutations:
            r = copy.deepcopy(self.good);mutate(r)
            with self.assertRaises(RuntimeError): validate(r, self.ref)

    def test_pause_reset_clock_pool_and_recovery_evidence_are_independently_required(self):
        mutations = [lambda r: r['samples'][102].update(device_us=10000001),
                     lambda r: r['samples'][0]['buffered'].update(free_blocks=6),
                     lambda r: r['checkpoints'][1].update(fresh_samples=1),
                     lambda r: r['final_stats'].update(underruns=0), lambda r: r['final_stats'].update(stale_blocks=0),
                     lambda r: r.update(physical_input_or_acoustic_latency_measured=True)]
        for mutate in mutations:
            r = copy.deepcopy(self.good);mutate(r)
            with self.assertRaises(RuntimeError): validate(r, self.ref)


if __name__ == '__main__': unittest.main()
