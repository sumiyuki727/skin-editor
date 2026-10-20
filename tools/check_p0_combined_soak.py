"""Opt-in same-process WAV/WASAPI + standard PNG/GPU soak; no capture."""
import argparse
import ctypes
import hashlib
import json
import math
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import time
import uuid
import wave

from check_p0 import ROOT
from check_p0_edit_composition import fixture_check
from check_p0_switch import phase_summary
from p0_process_metrics import WindowsProcessMetrics, environment_info
from check_p0_compressed import identity_paths


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''): h.update(block)
    return h.hexdigest()


def make_audio(output):
    output.mkdir(exist_ok=False)
    # Self-authored periodic stereo PCM: independent packet reference caches only
    # (phase, length), rather than retaining the ten-minute file or every packet.
    period = b''.join(struct.pack('<hh', (n - 32) * 700, (31 - n) * 500) for n in range(64))
    second = period * 750
    with wave.open(str(output / 'stream.wav'), 'wb') as f:
        f.setparams((2, 2, 48000, 48000 * 600, 'NONE', 'not compressed'))
        for _ in range(600): f.writeframesraw(second)
    m = {'schema_version': 1, 'generator': 'self_authored_periodic_stereo_600s_v1',
         'rate': 48000, 'channels': 2, 'frames': 48000 * 600, 'gain': .002,
         'files': {'stream.wav': digest(output / 'stream.wav')}}
    (output / 'manifest.json').write_text(json.dumps(m, indent=2) + '\n', encoding='utf-8')
    return output


class Reference:
    def __init__(self, path):
        manifest = json.loads((path / 'manifest.json').read_text())
        if manifest.get('generator') != 'self_authored_periodic_stereo_600s_v1' or digest(path / 'stream.wav') != manifest['files']['stream.wav']:
            raise RuntimeError('Audio fixture identity differs')
        with wave.open(str(path / 'stream.wav'), 'rb') as f:
            if f.getparams()[:4] != (2, 2, 48000, 48000 * 600): raise RuntimeError('Wrong WAV format')
            self.period = f.readframes(64)
            if self.period != b''.join(struct.pack('<hh', (n - 32) * 700, (31 - n) * 500) for n in range(64)):
                raise RuntimeError('Wrong periodic reference')
            for _ in range(600):
                if f.tell() >= f.getnframes(): break
                start = f.tell()
                data = f.readframes(min(48000, f.getnframes() - start))
                expected = (self.period * 751)[(start % 64) * 4:][:len(data)]
                if data != expected: raise RuntimeError('Non-periodic PCM fixture')
        self.cache = {}

    def check(self, p):
        start, count = p['source_frame'], p['frames']
        if type(start) is not int or type(count) is not int or not 0 < count <= 192000 or not 0 <= start < start + count <= 48000 * 600:
            raise RuntimeError('Invalid native packet range')
        key = start % 64, count
        if key not in self.cache:
            if len(self.cache) >= 2048: raise RuntimeError('Packet reference cache limit')
            gain = ctypes.c_float(.002).value
            values = [ctypes.c_float(struct.unpack_from('<h', self.period, (((start + n) % 64) * 2 + c) * 2)[0] / 32768 * gain).value
                      for n in range(count) for c in range(2)]
            raw = struct.pack('<' + 'f' * len(values), *values)
            h = 0xcbf29ce484222325
            for byte in raw: h = ((h ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
            self.cache[key] = {'hash': h, 'nonzero_samples': sum(v != 0 for v in values),
                               'peak': max(map(abs, values)), 'probe': (values[::2][:8] + [0] * 8)[:8]}
        expected = self.cache[key]
        if (p['hash'] != expected['hash'] or p['nonzero_samples'] != expected['nonzero_samples']
                or not math.isfinite(p['peak']) or abs(p['peak'] - expected['peak']) > 1e-7
                or len(p['probe']) != 8 or any(not math.isfinite(a) or abs(a - b) > 1e-7 for a, b in zip(p['probe'], expected['probe']))):
            raise RuntimeError('Accepted PCM differs from independent periodic reference')


def validate_packets(path, reference):
    generation, cursor, events, submits, frames = 0, None, 0, 0, 0
    retired, paused = False, False
    with path.open(encoding='utf-8') as f:
        for line in f:
            e = json.loads(line); events += 1
            if events > 200000: raise RuntimeError('Trace file limit')
            kind = e['kind']
            if kind in ('Activate', 'Pause'):
                if e['generation'] <= generation or retired: raise RuntimeError('Stale activation')
                if kind == 'Pause':
                    if cursor is None or abs(cursor - e['first_frame'] - e['native_discarded_frames']) > 1:
                        raise RuntimeError('Unaccounted native pause tail')
                    paused = True
                elif cursor is not None: raise RuntimeError('Unexpected restart during continuous playback')
                generation, cursor = e['generation'], e['first_frame']
            elif kind == 'Submit':
                p = e['packet']
                if paused or retired or e['generation'] != generation or e['first_frame'] != cursor or p['source_frame'] != cursor or p['frames'] != e['frames']:
                    raise RuntimeError('Discontinuous, paused or stale native packet')
                reference.check(p); cursor += e['frames']; submits += 1; frames += e['frames']
            elif kind == 'Retire':
                if not paused or e['generation'] <= generation: raise RuntimeError('Invalid retirement')
                retired = True; generation = e['generation']
            elif kind == 'Stale':
                if e['generation'] >= generation: raise RuntimeError('Current block mislabelled stale')
            else: raise RuntimeError('Unexpected trace event: ' + kind)
    if not retired or not paused or submits == 0: raise RuntimeError('Incomplete packet lifetime')
    return {'events': events, 'submits': submits, 'submitted_frames': frames}


def timing(values):
    if not values or any(not math.isfinite(v) or v < 0 for v in values): raise RuntimeError('Missing/non-finite timings')
    ordered = sorted(values)
    return {'count': len(values), 'p50': ordered[math.ceil(len(values) * .5) - 1],
            'p95': ordered[math.ceil(len(values) * .95) - 1], 'max': ordered[-1]}


def validate(result, packets, smoke):
    expected_switches = 2 if smoke else 20
    if (result.get('schema_version') != 1 or result.get('status') != 'passed' or result.get('errors')
            or result.get('seconds') != (3 if smoke else 300) or result.get('switches') != expected_switches
            or result.get('accepted_documents') != expected_switches + 1 or len(result.get('switch_ms', [])) != expected_switches
            or result.get('continuous_device_us', 0) < result['seconds'] * 1000000
            or result.get('changes', 0) < (20 if smoke else 300)
            or result.get('event_count') != packets['events'] or result.get('submit_count') != packets['submits']
            or result.get('normal_path_cpu_pixel_readbacks') != 0
            or result.get('system_capture_used') is not False or result.get('physical_or_acoustic_latency_measured') is not False):
        raise RuntimeError('Incomplete combined observation')
    s = result['final_stats']
    if (s['underruns'] != 0 or s['decode_failures'] != 0 or s['trace_overflow'] or s['active'] or s['in_flight']
            or s['free_blocks'] != 5 or s['queued_blocks'] != 0 or s['max_queued_blocks'] > 4):
        raise RuntimeError('Audio fault, trace loss or pool leak')
    checks = result.get('pixel_checks', [])
    if len(checks) != 2 or any(c['pixels_outside_tolerance'] != 0 or c['display_pixels_outside_tolerance'] != 0 or c['pixels'] != 1024 * 1024 for c in checks):
        raise RuntimeError('Missing paused composition proof')
    observations = result['observations']
    if not observations or len(observations) > 700: raise RuntimeError('Missing/bounded audio observations')
    if any(b['simulation_us'] < a['simulation_us'] for a, b in zip(observations, observations[1:])): raise RuntimeError('Content time moved backward')
    paused = [s['simulation_us'] for s in observations if s['phase'] in ('PausedSettle', 'PausedIdle', 'Closing', 'ClosedSettle', 'ClosedIdle')]
    if not paused or len(set(paused)) != 1: raise RuntimeError('Paused clock moved')
    frozen = [s for s in observations if s.get('device', {}).get('phase') == 'Paused' and s['phase'] in ('Diagnostic', 'Closing')]
    if (len(frozen) != 2 or frozen[1]['device']['sequence'] <= frozen[0]['device']['sequence']
            or frozen[1]['device']['position_us'] != frozen[0]['device']['position_us']
            or frozen[1]['elapsed_seconds'] - frozen[0]['elapsed_seconds'] < (2 if smoke else 65)):
        raise RuntimeError('Missing fresh long-paused device observations')
    timing(result['eframe_cpu_frame_ms_playing']); timing(result['frame_intervals_ms_playing'])


def summarize_phases(samples):
    # Exclude one second after each observed transition to avoid sampling across
    # a marker write / CPU-counter boundary. Idle stages last 65 seconds.
    rows = []
    for name in ('Warmup', 'Playing', 'Diagnostic', 'PausedIdle', 'ClosedIdle'):
        selected = [s for s in samples if s.get('phase') == name and
                    (name in ('Warmup', 'Diagnostic') or s.get('phase_age_seconds', 0) >= 1)]
        if not selected: rows.append({'phase': name, 'sample_count': 0}); continue
        labelled = [dict(s, phase='Idle') for s in selected]
        row = next(r for r in phase_summary(labelled) if r['phase'] == 'Idle')
        row['phase'] = name; rows.append(row)
    return rows


def budgets(result, phases, smoke):
    if smoke: return None
    by_name = {r['phase']: r for r in phases}
    play = by_name['Playing']; paused = by_name['PausedIdle']; closed = by_name['ClosedIdle']
    return {'playing_working_set_le_512_mib': 0 < play.get('working_set_bytes_max', 0) <= 512 * 1024 * 1024,
            'paused_cpu_le_2_percent_one_core': paused.get('cpu_one_logical_core_percent', 100) <= 2,
            'closed_cpu_le_2_percent_one_core': closed.get('cpu_one_logical_core_percent', 100) <= 2,
            'paused_observed_at_least_60_seconds': paused.get('sample_span_seconds', 0) >= 60,
            'closed_observed_at_least_60_seconds': closed.get('sample_span_seconds', 0) >= 60,
            'paused_no_continuous_redraw': result['paused_idle_frames'] <= 2,
            'closed_no_continuous_redraw': result['closed_idle_frames'] <= 2,
            'playing_frame_cpu_p95_le_16_7_ms': timing(result['eframe_cpu_frame_ms_playing'])['p95'] <= 16.7}


def validate_decoder_mode(result, compressed):
    if result.get('compressed_feature_enabled') is not compressed:
        raise RuntimeError('Measured decoder feature differs from requested build')


def run(exe, png, audio, smoke, reference, *, source_name='stream.wav', codec='wav', compressed=False):
    audio = audio.resolve(strict=True)
    source = (audio / source_name).resolve(strict=True)
    allowed = (ROOT / 'artifacts/p0').resolve()
    ref_path = getattr(reference, 'path', None)
    if allowed not in audio.parents or audio not in source.parents or codec not in ('wav', 'mp3', 'ogg'):
        raise RuntimeError('Audio input boundary or codec label differs')
    if ref_path is not None and allowed not in ref_path.resolve(strict=True).parents:
        raise RuntimeError('Reference outside independent artifacts')
    source_hash = digest(source)
    reference_hash = digest(ref_path) if ref_path is not None else None
    manifest_hash = digest(audio / 'manifest.json')
    png_hash = digest(png / 'inventory.json')
    output = ROOT / 'artifacts/p0' / ('combined-soak-' + uuid.uuid4().hex[:8])
    paths = identity_paths()
    frozen = {p.relative_to(ROOT).as_posix(): p.read_bytes() for p in paths}
    identity = {rel: hashlib.sha256(data).hexdigest() for rel, data in frozen.items()}
    binary = digest(exe); samples = []; last_phase = None; phase_start = time.monotonic()
    log_path = output.with_suffix('.log'); error = None; gpu_errors = []
    print('Starting ' + str(output) + (' smoke' if smoke else ' 300s playback + pause/close observations'), flush=True)
    with log_path.open('wb') as log:
        process = subprocess.Popen([str(exe), str(output), str(png), str(source), '3' if smoke else '300'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        metrics = None; start = time.monotonic()
        try:
            metrics = WindowsProcessMetrics(process)
            while process.poll() is None:
                if time.monotonic() - start > (110 if smoke else 540): raise RuntimeError('Combined soak timeout')
                try: phase = json.loads((output / 'phase.json').read_text())['phase']
                except (OSError, ValueError): phase = 'Unknown'
                if phase != last_phase:
                    last_phase = phase; phase_start = time.monotonic()
                    print(output.name + ': ' + phase, flush=True)
                value = metrics.sample()
                if value is not None:
                    try: after_phase = json.loads((output / 'phase.json').read_text())['phase']
                    except (OSError, ValueError): after_phase = 'Unknown'
                    if after_phase == phase:
                        value.update(phase=phase, phase_age_seconds=time.monotonic() - phase_start); samples.append(value)
                time.sleep(.2)
        except Exception as exc: error = str(exc)
        finally:
            if process.poll() is None: process.kill(); process.wait(timeout=5)
            if metrics is not None: gpu_errors = metrics.gpu_errors; metrics.close()
    output.mkdir(exist_ok=True)
    shutil.copyfile(log_path, output / 'run.log')
    evidence = {'schema_version': 1, 'smoke': smoke, 'source_sha256': identity, 'executable_sha256': binary,
                'png_fixture': png.relative_to(ROOT).as_posix(), 'png_inventory_sha256': png_hash,
                'audio_fixture': audio.relative_to(ROOT).as_posix(), 'audio_manifest_sha256': manifest_hash,
                'audio_source': source.relative_to(ROOT).as_posix(), 'audio_source_sha256': source_hash,
                'codec': codec, 'compressed_feature_requested': compressed,
                'reference': ref_path.relative_to(ROOT).as_posix() if ref_path is not None else None,
                'reference_sha256': reference_hash,
                'environment': environment_info(), 'samples': samples, 'gpu_errors': gpu_errors,
                'process_exit_code': process.returncode, 'error': error}
    # Write even rejected runs: failure must not lose the resource samples.
    for rel, data in frozen.items():
        path = output / 'source-snapshot' / rel
        path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(data)
    shutil.copyfile(exe, output / 'source-snapshot' / exe.name)
    try:
        if error or process.returncode: raise RuntimeError(error or 'Native process failed')
        if digest(exe) != binary or any(digest(p) != identity[p.relative_to(ROOT).as_posix()] for p in paths): raise RuntimeError('Measured sources changed')
        if (digest(source) != source_hash or digest(audio / 'manifest.json') != manifest_hash
                or digest(png / 'inventory.json') != png_hash
                or (ref_path is not None and digest(ref_path) != reference_hash)):
            raise RuntimeError('Measured fixtures or reference changed')
        fixture_check(png)
        r = json.loads((output / 'result.json').read_text())
        validate_decoder_mode(r, compressed)
        packets = validate_packets(output / 'packets.jsonl', reference); validate(r, packets, smoke)
        if digest(output / 'composition-0-cpu.png') == digest(output / 'composition-1-cpu.png'): raise RuntimeError('Paused edit did not change pixels')
        phases = summarize_phases(samples)
        evidence.update(status='validated', packets=packets, phases=phases, budgets=budgets(r, phases, smoke),
                        timings={k: timing(r[k]) for k in ('eframe_cpu_frame_ms_playing', 'frame_intervals_ms_playing', 'switch_ms')})
        evidence['play_30_second_windows'] = []
        for begin in range(0, r['seconds'], 30):
            window = [dict(s, phase='Idle') for s in samples if s['phase'] == 'Playing'
                      and max(1, begin) <= s['phase_age_seconds'] < begin + 30]
            row = next(p for p in phase_summary(window) if p['phase'] == 'Idle')
            row.update(begin_seconds=begin, end_seconds=min(begin + 30, r['seconds']))
            row.pop('phase'); evidence['play_30_second_windows'].append(row)
        normal = [s for s in samples if s['phase'] not in ('Unknown', 'Diagnostic', 'Done')]
        evidence['normal_sampled_peaks'] = {k: max((s[k] for s in normal if s.get(k) is not None), default=None)
                                             for k in ('working_set_bytes', 'private_bytes', 'gpu_dedicated_bytes', 'gpu_shared_bytes')}
        evidence['historical_peak_counters'] = {k: max((s[k] for s in samples if s.get(k) is not None), default=None)
                                                for k in ('peak_working_set_bytes', 'peak_commit_bytes')}
        if not smoke:
            evidence['budgets']['normal_sampled_working_set_le_512_mib'] = 0 < evidence['normal_sampled_peaks']['working_set_bytes'] <= 512 * 1024 * 1024
    except Exception as exc:
        evidence.update(status='rejected', error=str(exc))
    (output / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    print(output.name + ': ' + evidence['status'] + ' ' + str(evidence.get('budgets', evidence.get('error'))), flush=True)
    if evidence['status'] != 'validated': raise RuntimeError(str(output) + ': ' + str(evidence['error']))
    return output


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--run-native', action='store_true'); p.add_argument('--smoke', action='store_true')
    p.add_argument('--skip-checks', action='store_true', help='Use separately checked existing release executable')
    p.add_argument('--repetitions', type=int, choices=range(1, 4), default=2)
    p.add_argument('--fixture', type=Path); p.add_argument('--audio-fixture', type=Path)
    args = p.parse_args()
    if not args.skip_checks: subprocess.run([sys.executable, str(ROOT / 'tools/check_p0_gui.py'), '--media'], check=True)
    if args.run_native:
        if sys.platform != 'win32' or args.fixture is None: raise RuntimeError('Windows and explicit standard PNG fixture required')
        png = fixture_check(args.fixture)
        audio = args.audio_fixture.resolve(strict=True) if args.audio_fixture else make_audio(ROOT / 'artifacts/p0' / ('combined-audio-fixture-' + uuid.uuid4().hex[:8]))
        if (ROOT / 'artifacts/p0').resolve() not in audio.parents: raise RuntimeError('Use an independent audio fixture inside artifacts/p0')
        reference = Reference(audio)
        exe = ROOT / 'experiments/p0-egui/target/release/combined-soak.exe'
        for _ in range(args.repetitions): run(exe, png, audio, args.smoke, reference)


if __name__ == '__main__': main()
