"""Background WAV -> bounded PCM pool -> actual WASAPI, opt-in only; no capture."""
import argparse
import ctypes
import hashlib
import json
import math
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import time
import uuid
import wave

from check_p0 import ROOT
from check_p0_transport import sources
from p0_process_metrics import WindowsProcessMetrics, environment_info

CHECKS = {'continuous_device_playback', 'pause_flushes_native_and_queued_pcm', 'stale_inflight_on_pause_and_seek_rejected',
          'controlled_underrun_faults_and_freezes_input', 'recovery_requires_explicit_resume', 'pool_recycled_on_shutdown'}


def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def check_decoder_boundary(source):
    code = re.sub(r'//[^\n]*', '', source.split('#[cfg(test)]', 1)[0])
    if re.search(r'\b(?:eframe|egui|windows|wgpu)\s*::|\bstd\s*::\s*(?:fs|net)|\b(?:File|Instant|SystemTime|unsafe)\b|include(?:_str|_bytes)?\s*!|#\[path', code):
        raise RuntimeError('Decoder assembly must not access UI, native APIs, paths, host clocks or external sources')


def make_fixture(output):
    output.mkdir(exist_ok=False); second = bytearray()
    for n in range(48000):
        t = n / 48000; env = min(1, t / .02, (1 - t) / .02)
        second.extend(struct.pack('<hh', round(.35 * env * math.sin(math.tau * 440 * t) * 32767), round(.2 * env * math.sin(math.tau * 660 * t) * 32767)))
    path = output / 'stream.wav'
    with wave.open(str(path), 'wb') as f:
        f.setparams((2, 2, 48000, 48000 * 30, 'NONE', 'not compressed'))
        for _ in range(30): f.writeframesraw(second)
    value = {'schema_version': 1, 'generator': 'self_authored_ramped_stereo_30s_v1', 'files': {'stream.wav': digest(path)}, 'rate': 48000, 'channels': 2, 'frames': 1440000, 'gain': .02}
    (output / 'manifest.json').write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8'); return value


class Reference:
    def __init__(self, fixture):
        with wave.open(str(fixture / 'stream.wav'), 'rb') as f:
            if f.getparams()[:4] != (2, 2, 48000, 1440000): raise RuntimeError('Wrong independent fixture')
            self.data = f.readframes(f.getnframes())
        self.cache = {}

    def packet(self, start, count):
        key = start, count
        if key not in self.cache:
            gain = ctypes.c_float(.02).value; raw = bytearray(); values = []
            for i in range(count):
                for channel in range(2):
                    v = struct.unpack_from('<h', self.data, ((start + i) * 2 + channel) * 2)[0] / 32768 if start + i < 1440000 else 0
                    v = ctypes.c_float(v * gain).value; raw.extend(struct.pack('<f', v)); values.append(v)
            h = 0xcbf29ce484222325
            for byte in raw: h = ((h ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
            probe = values[::2][:8]; probe += [0] * (8 - len(probe))
            self.cache[key] = {'source_frame': start, 'frames': count, 'hash': h, 'peak': max(map(abs, values)), 'nonzero_samples': sum(v != 0 for v in values), 'probe': probe}
        return self.cache[key]

    def check(self, p):
        if type(p.get('source_frame')) is not int or p['source_frame'] < 0 or type(p.get('frames')) is not int or not 0 < p['frames'] <= 131072:
            raise RuntimeError('Invalid native packet range')
        e = self.packet(p['source_frame'], p['frames'])
        if (p['hash'] != e['hash'] or p['nonzero_samples'] != e['nonzero_samples'] or not math.isfinite(p['peak']) or abs(p['peak'] - e['peak']) > 1e-7
                or len(p['probe']) != 8 or any(not math.isfinite(a) or abs(a - b) > 1e-7 for a, b in zip(p['probe'], e['probe']))):
            raise RuntimeError('Accepted native PCM differs from independent source/gain/position')


def validate(r, reference):
    config = {'rate': 48000, 'channels': 2, 'frames': 1440000, 'gain': .02, 'block_frames': 4096, 'queue_blocks': 4, 'pool_blocks': 5, 'trace_limit': 8192}
    if (r.get('schema_version') != 1 or r.get('status') != 'passed' or r.get('source') != 'self_authored_30_second_wav_stream'
            or r.get('config') != config or set(r.get('checks', {})) != CHECKS or any(v is not True for v in r['checks'].values())
            or r.get('accepted_inputs') != 3 or any(r.get(k) is not False for k in ('system_capture_used', 'microphone_or_loopback_used', 'physical_input_or_acoustic_latency_measured'))):
        raise RuntimeError('Missing streaming native checks, configuration or input/measurement identity')
    if any(r['audio_format'].get(k) != v for k, v in {'rate': 48000, 'channels': 2, 'bits': 32}.items()): raise RuntimeError('Wrong output format')
    final = r['final_stats']
    if (final['queued_blocks'] != 0 or final['free_blocks'] != 5 or final['in_flight'] or final['active'] or final['trace_overflow']
            or final['underruns'] != 1 or final['decode_failures'] != 0 or final['stale_blocks'] < 3 or final['max_queued_blocks'] > 4):
        raise RuntimeError('Queue lifetime, underrun or stale-generation counts differ')
    events = r.get('events', [])
    if not 10 <= len(events) <= 8192: raise RuntimeError('Missing/bounded event trace')
    generation = 0; cursor = None; active = False; stale = set(); underruns = 0; submits = 0; pauses = 0
    for e in events:
        kind = e['kind']
        if kind in ('activate', 'pause'):
            if e['generation'] <= generation or not 0 <= e['first_frame'] <= 1440000: raise RuntimeError('Non-monotonic content generation')
            if kind == 'pause':
                pauses += 1
                if cursor is None or abs(cursor - e['first_frame'] - e['native_discarded_frames']) > 1: raise RuntimeError('Native unread tail was not accounted at pause')
            generation = e['generation']; cursor = e['first_frame']; active = True
        elif kind == 'retire':
            if e['generation'] <= generation: raise RuntimeError('Retirement did not invalidate generation')
            generation = e['generation']; active = False
        elif kind == 'submit':
            if not active or e['generation'] != generation or e['first_frame'] != cursor or e['packet']['source_frame'] != cursor or e['frames'] != e['packet']['frames']:
                raise RuntimeError('Stale or discontinuous PCM was submitted to the backend')
            reference.check(e['packet']); cursor += e['frames']; submits += 1
        elif kind == 'stale':
            if e['generation'] >= generation: raise RuntimeError('Current completion incorrectly classified stale')
            stale.add((e['generation'], e['first_frame']))
        elif kind == 'underrun':
            if not active or e['generation'] != generation: raise RuntimeError('Wrong underrun generation')
            underruns += 1
        else: raise RuntimeError('Unknown native trace event')
    if underruns != 1 or pauses < 3 or submits < 100: raise RuntimeError('Missing native continuity, pause or starvation coverage')
    if any(tuple(r[k]) not in stale for k in ('held_pause', 'held_seek', 'held_underrun')): raise RuntimeError('Deliberately held old completions were not rejected')
    checkpoints = r['checkpoints']; continuous = [c for c in checkpoints if c['kind'] == 'continuous']; fault = [c for c in checkpoints if c['kind'] == 'underrun']
    if len(continuous) != 1 or continuous[0]['position_us'] < 10000000 or continuous[0]['stats']['underruns'] != 0 or len(fault) != 1 or fault[0]['wall_ms'] < 300:
        raise RuntimeError('Missing 10-second continuity or frozen fault interval')
    if continuous[0]['stats']['pcm']['submitted_frames'] < continuous[0]['position_us'] * 48000 // 1000000:
        raise RuntimeError('Continuous device position exceeds submitted content')
    first_pause = next(e for e in events if e['kind'] == 'pause')
    if first_pause['frames'] == 0 or first_pause['native_discarded_frames'] == 0:
        raise RuntimeError('Pause did not exercise queued and native unread tail discard')
    seeks = [(c['epoch'], c['position_us']) for c in checkpoints if c['kind'] == 'seek']
    if seeks != [(1, 20000000), (2, 25000000), (3, 500000)]: raise RuntimeError('Wrong forward/backward seek coverage')
    paused = [c for c in checkpoints if c['kind'] == 'pause']
    if [c['name'] for c in paused] != ['discard_inflight_on_pause', 'discard_inflight_on_seek', 'recovery_requires_explicit_play', 'final']: raise RuntimeError('Missing pause checkpoints')
    rows = r['samples']
    if not 100 <= len(rows) <= 4096 or any(b['sequence'] <= a['sequence'] for a, b in zip(rows, rows[1:])): raise RuntimeError('Missing fresh device samples')
    origins = {0: 0, 1: 20000000, 2: 25000000, 3: 500000, 4: fault[0]['position_us']}; submitted = {}; previous_time = {}
    for row in rows:
        q = row['buffered']
        if q is None: continue
        if q['queued_blocks'] > 4 or q['queued_frames'] > 4 * 4096 or q['queued_blocks'] + q['free_blocks'] + int(q['in_flight']) != 5: raise RuntimeError('Unbounded or lost PCM pool')
        p = q['pcm']; g = q['generation']
        if p['submitted_frames'] < submitted.get(g, 0) or p['source_origin_frame'] != p['requested_us'] * 48000 // 1000000: raise RuntimeError('Wrong submission/source position')
        submitted[g] = p['submitted_frames']
        if row['transport_phase'] == row['phase'] and row['phase'] in ('Playing', 'Paused'):
            absolute = origins[row['epoch']] + row['device_us']
            if row['simulation_us'] != absolute or absolute < p['requested_us'] or (absolute - p['requested_us']) * 48000 // 1000000 > p['submitted_frames'] + 1:
                raise RuntimeError('Simulation is not following valid device consumption')
            if absolute < previous_time.get(row['epoch'], absolute): raise RuntimeError('Unrequested device/simulation clock regression')
            previous_time[row['epoch']] = absolute
    for c in paused:
        group = [row for row in rows if row['epoch'] == c['epoch'] and row['phase'] == 'Paused' and row['buffered'] is not None and row['buffered']['generation'] == c['generation']]
        if (c['wall_ms'] < 300 or c['fresh_samples'] < 3 or len(group) < 4 or c['stats']['pcm']['submitted_frames'] != 0
                or any(row['device_us'] != c['device_us'] or row['simulation_us'] != c['position_us'] or row['buffered']['pcm']['submitted_frames'] != 0 for row in group)):
            raise RuntimeError('Pause moved time or submitted PCM')
    if paused[2]['position_us'] != fault[0]['position_us'] or r['final_simulation_us'] < fault[0]['position_us'] + 500000: raise RuntimeError('Recovery skipped or auto-resumed confirmed content')


def run_native(exe, fixture, reference):
    output = ROOT / 'artifacts/p0' / ('stream-output-' + uuid.uuid4().hex[:8]); assert not output.exists()
    paths = sorted(set(sources() + [Path(__file__).resolve(), ROOT / 'tools/check_p0.py', ROOT / 'tools/check_p0_media.py', ROOT / 'tools/tests/test_p0_stream_output.py']))
    frozen = {p.relative_to(ROOT).as_posix(): p.read_bytes() for p in paths}; identity = {name: hashlib.sha256(data).hexdigest() for name, data in frozen.items()}
    inputs = {p.name: digest(p) for p in fixture.iterdir() if p.is_file()}; exe_hash = digest(exe); samples = []
    log_path = output.with_suffix('.log')
    with log_path.open('wb') as log:
        process = subprocess.Popen([str(exe), str(output), str(fixture)], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        metrics = None; start = time.monotonic()
        try:
            metrics = WindowsProcessMetrics(process)
            while process.poll() is None:
                if time.monotonic() - start > 60: raise RuntimeError('Native stream output timed out')
                value = metrics.sample()
                if value is not None: samples.append(value)
                time.sleep(.025)
        finally:
            if process.poll() is None: process.kill(); process.wait(timeout=5)
            if metrics is not None: metrics.close()
    if not output.exists(): output.mkdir()
    for name, data in frozen.items():
        dest = output / 'sources' / name; dest.parent.mkdir(parents=True, exist_ok=True); dest.write_bytes(data)
    shutil.copyfile(log_path, output / 'run.log')
    evidence = {'schema_version': 1, 'executable_sha256': exe_hash, 'source_sha256': identity, 'fixture': fixture.relative_to(ROOT).as_posix(), 'fixture_sha256': inputs,
                'environment': environment_info(), 'samples': samples, 'summary': {'sample_count': len(samples),
                'peak_commit_bytes': max((v['peak_commit_bytes'] for v in samples), default=0), 'peak_working_set_bytes': max((v['peak_working_set_bytes'] for v in samples), default=0),
                'cpu_seconds_total_observed': max((v['cpu_seconds'] for v in samples), default=0)},
                'scope': 'Whole diagnostic process including bounded trace, observations and final JSON allocation; no acoustic or physical latency claim'}
    (output / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    if process.returncode: raise RuntimeError('Native stream output failed: ' + str(output))
    r = json.loads((output / 'result.json').read_text(encoding='utf-8')); validate(r, reference)
    if digest(exe) != exe_hash or any(digest(ROOT / name) != h for name, h in identity.items()) or inputs != {p.name: digest(p) for p in fixture.iterdir() if p.is_file()}:
        raise RuntimeError('Streaming source, executable or inputs changed during run')
    original = {p.name: digest(p) for p in output.iterdir() if p.is_file()}; escaped = ROOT / ('stream-output-escape-' + uuid.uuid4().hex[:8])
    for invalid in [escaped, output]:
        rejected = subprocess.run([str(exe), str(invalid), str(fixture)], cwd=ROOT, capture_output=True, timeout=5)
        if rejected.returncode == 0 or (invalid == escaped and escaped.exists()): raise RuntimeError('Native output boundary/overwrite rejection failed')
    if original != {p.name: digest(p) for p in output.iterdir() if p.is_file()}: raise RuntimeError('Overwrite rejection changed original evidence')
    print('Native streaming output: ' + str(output), flush=True); return output


def main():
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--run-native', action='store_true'); p.add_argument('--repetitions', type=int, choices=range(1, 4), default=3); args = p.parse_args()
    check_decoder_boundary((ROOT / 'experiments/p0-egui/src/streaming_decode.rs').read_text(encoding='utf-8'))
    subprocess.run([sys.executable, str(ROOT / 'tools/check_p0_gui.py'), '--media'], check=True)
    if args.run_native:
        if sys.platform != 'win32': raise RuntimeError('Native stream output is currently Windows-only')
        fixture = ROOT / 'artifacts/p0' / ('stream-output-fixture-' + uuid.uuid4().hex[:8]); make_fixture(fixture); ref = Reference(fixture)
        for _ in range(args.repetitions): run_native(ROOT / 'experiments/p0-egui/target/release/stream-output-proof.exe', fixture, ref)


if __name__ == '__main__': main()
