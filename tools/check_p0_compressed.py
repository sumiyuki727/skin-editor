"""Optional compressed pull/native evidence. Self-authored fixtures only."""
from array import array
import argparse
import ctypes
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import time
import uuid

from check_p0 import ROOT, windows_linker
from p0_compressed_fixture import digest, make_fixture
from p0_process_metrics import WindowsProcessMetrics, environment_info


def compare_float_files(actual, reference, tolerance=0.00001):
    if actual.stat().st_size % 8 or reference.stat().st_size % 8: raise RuntimeError('Incomplete stereo float frames')
    largest = squares = 0.; count = outside = 0
    with actual.open('rb') as a, reference.open('rb') as b:
        while True:
            x = a.read(65536); y = b.read(65536)
            n = min(len(x), len(y))
            if not n: break
            av = array('f'); bv = array('f'); av.frombytes(x[:n]); bv.frombytes(y[:n])
            if sys.byteorder != 'little': av.byteswap(); bv.byteswap()
            for v, w in zip(av, bv):
                if not math.isfinite(v) or not math.isfinite(w): raise RuntimeError('Non-finite decoded sample')
                error = abs(v - w); largest = max(largest, error); squares += error * error; outside += error > tolerance
            count += len(av)
    if count == 0: raise RuntimeError('No common decoded samples')
    return {'actual_frames': actual.stat().st_size // 8, 'reference_frames': reference.stat().st_size // 8,
            'frames_match': actual.stat().st_size == reference.stat().st_size,
            'compared_samples': count, 'max_error': largest, 'rmse': math.sqrt(squares / count), 'samples_outside_tolerance': outside,
            'tolerance': tolerance, 'uncompared_tail_frames': abs(actual.stat().st_size - reference.stat().st_size) // 8}


def validate_pull(result, directory, reference):
    if result.get('schema_version') != 1 or result.get('rate') != 48000 or result.get('channels') != 2 or result.get('audio_device_opened') is not False:
        raise RuntimeError('Unexpected pull decoder scope')
    pcm = directory / 'decoded.f32'
    if pcm.stat().st_size != result['frames'] * 8: raise RuntimeError('Decoded length differs from metadata')
    for n, row in enumerate(result['seeks']):
        target = [result['frames'] // 2 + 7, 11, result['frames'] - 9, result['frames']][n]
        expected = min(257, result['frames'] - target)
        if row['target'] != target or row['generation'] != n + 1 or row['frames'] != expected or row['eof'] != (target + expected == result['frames']):
            raise RuntimeError('Incorrect seek/EOF identity')
        with pcm.open('rb') as f: f.seek(target * 8); raw = f.read(expected * 8)
        if (directory / ('seek-%d.f32' % n)).read_bytes() != raw: raise RuntimeError('Seek PCM differs from linear decode')
    if len(result['seeks']) != 4: raise RuntimeError('Missing seeks')
    comparison = compare_float_files(pcm, reference)
    if comparison['samples_outside_tolerance']: raise RuntimeError('Decode differs from independent reference')
    return comparison


class NativeReference:
    def __init__(self, path, total_frames=8640000):
        if type(total_frames) is not int or not 0 < total_frames <= 48000 * 600:
            raise RuntimeError('Invalid bounded stereo reference duration')
        if path.stat().st_size != total_frames * 8:
            raise RuntimeError('Incomplete stereo float reference')
        self.path = path
        self.total_frames = total_frames

    def check(self, packet):
        start = packet['source_frame']; count = packet['frames']
        if type(start) is not int or type(count) is not int or not 0 < count <= 192000 or not 0 <= start < start + count <= self.total_frames:
            raise RuntimeError('Unbounded or out-of-source native PCM')
        with self.path.open('rb') as f: f.seek(start * 8); raw = f.read(count * 8)
        if len(raw) != count * 8: raise RuntimeError('Missing independently checked linear PCM')
        values = array('f'); values.frombytes(raw)
        if sys.byteorder != 'little': values.byteswap()
        gain = ctypes.c_float(.002).value
        scaled = array('f', (v * gain for v in values))
        peak = max(map(abs, scaled)); nonzero = sum(v != 0 for v in scaled); probe = list(scaled[::2][:8]); probe += [0] * (8 - len(probe))
        if sys.byteorder != 'little': scaled.byteswap()
        h = 0xcbf29ce484222325
        for byte in scaled.tobytes(): h = ((h ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
        if (packet['hash'] != h or packet['nonzero_samples'] != nonzero or not math.isfinite(packet['peak']) or abs(packet['peak'] - peak) > 1e-7
                or len(packet['probe']) != 8 or any(not math.isfinite(v) or abs(v - w) > 1e-7 for v, w in zip(packet['probe'], probe))):
            raise RuntimeError('Native accepted PCM differs from verified decode/gain/source')


def validate_native(result, reference):
    if (result.get('schema_version') != 1 or result.get('status') != 'passed' or result.get('accepted_inputs') != 1
            or result.get('config') != {'rate': 48000, 'channels': 2, 'frames': 8640000, 'gain': .002}
            or any(result.get(k) is not False for k in ('system_capture_used', 'microphone_or_loopback_used', 'physical_or_acoustic_latency_measured'))):
        raise RuntimeError('Native scope or input isolation differs')
    s = result['final_stats']
    if s['free_blocks'] != 5 or s['queued_blocks'] != 0 or s['in_flight'] or s['active'] or s['underruns'] or s['decode_failures'] or s['trace_overflow']:
        raise RuntimeError('Pool leak, decode failure or underrun')
    generation = 0; cursor = None; active = False; paused = False; submits = 0; origins = []
    for e in result['events']:
        kind = e['kind']
        if kind in ('Activate', 'Pause'):
            if e['generation'] <= generation: raise RuntimeError('Stale PCM generation')
            if kind == 'Pause':
                if not active or cursor is None or abs(cursor - e['first_frame'] - e['native_discarded_frames']) > 1:
                    raise RuntimeError('Unaccounted native pause tail')
            else:
                if active: raise RuntimeError('Replacement stream overlaps old generation')
                origins.append(e['first_frame'])
            generation = e['generation']; cursor = e['first_frame']; active = True; paused = kind == 'Pause'
        elif kind == 'Retire':
            if e['generation'] <= generation: raise RuntimeError('Stale retirement')
            active = False; generation = e['generation']
        elif kind == 'Stale':
            if e['generation'] >= generation: raise RuntimeError('Current block rejected as stale')
        elif kind == 'Submit':
            if not active or paused or e['generation'] != generation or e['first_frame'] != cursor:
                raise RuntimeError('Discontinuous, stale or paused native submission')
            p = e['packet']
            if p['source_frame'] != cursor or p['frames'] != e['frames']: raise RuntimeError('Packet/event source mismatch')
            reference.check(p); cursor += e['frames']; submits += 1
        else: raise RuntimeError('Unexpected PCM event')
    if active or origins != [0, 4320000, 24000] or not 1000 <= submits <= 8192: raise RuntimeError('Missing stream replacements or continuous playback')
    continuous = [r for r in result['checkpoints'] if r['kind'] == 'continuous']
    pauses = [r for r in result['checkpoints'] if r['kind'] == 'pause']
    if len(continuous) != 1 or continuous[0]['position_us'] < 30000000 or continuous[0]['stats']['underruns'] != 0 or len(pauses) != 4:
        raise RuntimeError('Incomplete continuous/pause coverage')
    for p in pauses:
        if p['wall_ms'] < 650 or p['last_sequence'] <= p['first_sequence'] or p['stats']['packets'] != 0: raise RuntimeError('Pause advanced or reused an old observation')
    if [v['content_us'] for v in result['latencies'] if v['kind'] == 'seek_prefill'] != [90000000, 500000]: raise RuntimeError('Missing seek-to-prefill latency')
    return {'submits_checked': submits, 'origins': origins, 'continuous_us': continuous[0]['position_us']}


def identity_paths():
    paths = [ROOT / 'experiments' / c / f for c in ('p0-media', 'p0-egui', 'p0-wasapi', 'p0-preview-contract') for f in ('Cargo.toml', 'Cargo.lock')]
    paths += [p for c in ('p0-media', 'p0-egui', 'p0-wasapi', 'p0-preview-contract') for p in (ROOT / 'experiments' / c / 'src').rglob('*') if p.is_file()]
    paths += [p for p in (ROOT / 'experiments/p0-media/tests/fixtures').glob('*') if p.is_file()]
    paths += list((ROOT / 'tools').rglob('*.py')); return paths


def run(exe, source, mode, reference, codec):
    output = ROOT / 'artifacts/p0' / ('compressed-' + mode + '-' + uuid.uuid4().hex[:8])
    frozen = {p.relative_to(ROOT).as_posix(): p.read_bytes() for p in identity_paths()}
    identities = {rel: hashlib.sha256(data).hexdigest() for rel, data in frozen.items()}; binary = digest(exe)
    source_hash = digest(source); reference_hash = digest(reference)
    samples = []; error = None; metrics = None; released = False
    print(mode + ' ' + source.name + ' -> ' + output.name, flush=True)
    log_path = output.with_suffix('.log')
    with log_path.open('wb') as log:
        p = subprocess.Popen([str(exe), *([str(output), str(source)] if mode == 'native' else [str(source), str(output)])], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        start = time.monotonic()
        try:
            metrics = WindowsProcessMetrics(p)
            while p.poll() is None:
                if time.monotonic() - start > 65: raise RuntimeError('Compressed proof timeout')
                value = metrics.sample()
                if value is not None: samples.append(value)
                if not released and (output / 'ready').exists():
                    if not samples: raise RuntimeError('Missing pre-decode resource baseline')
                    (output / 'go').write_text('1'); released = True
                time.sleep(.025)
        except Exception as exc: error = str(exc)
        finally:
            if p.poll() is None: p.kill(); p.wait(timeout=5)
            if metrics is not None: metrics.close()
    output.mkdir(exist_ok=True); shutil.copyfile(log_path, output / 'run.log')
    for rel, data in frozen.items():
        dst = output / 'source-snapshot' / rel; dst.parent.mkdir(parents=True, exist_ok=True); dst.write_bytes(data)
    e = {'schema_version': 1, 'mode': mode, 'codec': codec, 'source': source.relative_to(ROOT).as_posix(), 'source_file_sha256': source_hash,
         'reference': reference.relative_to(ROOT).as_posix(), 'reference_file_sha256': reference_hash,
         'executable_sha256': binary, 'source_sha256': identities, 'process_exit_code': p.returncode, 'samples': samples,
         'environment': environment_info(), 'error': error}
    try:
        if error or p.returncode: raise RuntimeError(error or 'Native probe failed')
        if digest(exe) != binary or digest(source) != source_hash or digest(reference) != reference_hash or any(digest(ROOT / rel) != h for rel, h in identities.items()): raise RuntimeError('Measured sources, reference or executable changed')
        shutil.copyfile(exe, output / 'source-snapshot' / exe.name)
        result = json.loads((output / 'result.json').read_text())
        if mode == 'pull':
            comparison = validate_pull(result, output, reference)
            e['comparison'] = comparison
            e['status'] = 'validated' if comparison['frames_match'] else 'reference_tail_mismatch_unresolved'
        else: e.update(status='validated', native=validate_native(result, NativeReference(reference)))
        e['peaks'] = {k: max(s[k] for s in samples) for k in ('peak_working_set_bytes', 'peak_commit_bytes', 'working_set_bytes', 'private_bytes')}
        e['cpu_seconds_observed'] = samples[-1]['cpu_seconds'] - samples[0]['cpu_seconds']
    except Exception as exc: e.update(status='rejected', error=str(exc))
    (output / 'evidence.json').write_text(json.dumps(e, indent=2) + '\n', encoding='utf-8')
    print(output.name + ': ' + e['status'] + (' ' + str(e['error']) if e['error'] else ''), flush=True)
    if e['status'] == 'rejected': raise RuntimeError(str(output) + ': ' + str(e['error']))
    return output


def check_compressed():
    e = dict(os.environ); b = ROOT / '.p0-tools/rustup/toolchains/1.95.0-x86_64-pc-windows-msvc/bin'
    e['PATH'] = str(b) + os.pathsep + e.get('PATH', ''); e['CARGO_HOME'] = str(ROOT / '.p0-tools/cargo'); e['RUSTUP_HOME'] = str(ROOT / '.p0-tools/rustup'); link = windows_linker(e)
    metadata = json.loads(subprocess.check_output([str(b / 'cargo.exe'), 'metadata', '--manifest-path', str(ROOT / 'experiments/p0-media/Cargo.toml'), '--no-deps', '--offline', '--format-version', '1'], env=e))
    check_compressed_features(metadata['packages'][0]['features'])
    common = ['--manifest-path', str(ROOT / 'experiments/p0-media/Cargo.toml'), '--offline', '--locked', '-j', '2', '--features', 'compressed']
    for cmd in [['fmt', '--manifest-path', str(ROOT / 'experiments/p0-media/Cargo.toml'), '--', '--check'],
                [*link, 'clippy', *common, '--all-targets', '--', '-D', 'warnings'], [*link, 'test', *common, '--release'], [*link, 'build', *common, '--release']]:
        subprocess.run([str(b / 'cargo.exe'), *cmd], env=e, check=True)
    subprocess.run([sys.executable, str(ROOT / 'tools/check_p0_gui.py'), '--compressed'], check=True)


def check_compressed_features(features):
    if set(features.get('compressed', [])) != {'symphonia/mp3', 'symphonia/ogg', 'symphonia/vorbis'} or features.get('default'):
        raise RuntimeError('Compressed experiment must remain opt-in and limited to MP3/Ogg Vorbis')


def main():
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--run-benchmark', action='store_true'); p.add_argument('--run-native', action='store_true')
    p.add_argument('--skip-checks', action='store_true'); p.add_argument('--fixture', type=Path); p.add_argument('--repetitions', type=int, choices=range(1, 4), default=2); args = p.parse_args()
    if not args.skip_checks: check_compressed()
    if not (args.run_benchmark or args.run_native): return
    if sys.platform != 'win32': raise RuntimeError('Process/native measurements currently verified only on Windows')
    fixture = args.fixture.resolve(strict=True) if args.fixture else ROOT / 'artifacts/p0' / ('compressed-fixture-' + uuid.uuid4().hex[:8])
    if not fixture.exists(): make_fixture(fixture)
    if (ROOT / 'artifacts/p0').resolve() not in fixture.parents: raise RuntimeError('Independent fixture must stay inside artifacts/p0')
    manifest = json.loads((fixture / 'manifest.json').read_text())
    if manifest['generator'] != 'self_authored_stereo_sines_compressed_v1': raise RuntimeError('Wrong fixture identity')
    for name, item in manifest['files'].items():
        path = (fixture / name).resolve(strict=True)
        if fixture not in path.parents or digest(path) != item['sha256']: raise RuntimeError('Fixture hash/path mismatch')
    results = []; linear = {}
    for repeat in range(args.repetitions):
        if args.run_benchmark or not linear:
            for item in manifest['items']:
                out = run(ROOT / 'experiments/p0-media/target/release/compressed-proof.exe', fixture / item['encoded'], 'pull', fixture / item['reference'], item['codec'])
                results.append(out); linear[(item['codec'], item['seconds'])] = out / 'decoded.f32'
            for seconds in (3, 180):
                source = fixture / ('source-%d.wav' % seconds)
                ref = ROOT / 'artifacts/p0' / ('wav-float-reference-' + uuid.uuid4().hex[:8] + '.f32')
                import wave
                with wave.open(str(source), 'rb') as w, ref.open('xb') as f:
                    while True:
                        raw = w.readframes(4096)
                        if not raw: break
                        v = array('h'); v.frombytes(raw)
                        if sys.byteorder != 'little': v.byteswap()
                        floats = array('f', (n / 32768 for n in v))
                        if sys.byteorder != 'little': floats.byteswap()
                        f.write(floats.tobytes())
                out = run(ROOT / 'experiments/p0-media/target/release/compressed-proof.exe', source, 'pull', ref, 'wav')
                results.append(out); linear[('wav', seconds)] = out / 'decoded.f32'
        if args.run_native:
            for codec, source in [('wav', 'source-180.wav'), ('mp3', 'audio-180.mp3'), ('ogg', 'audio-180.ogg')]:
                results.append(run(ROOT / 'experiments/p0-egui/target/release/compressed-output-proof.exe', fixture / source, 'native', linear[(codec, 180)], codec))
    summary = ROOT / 'artifacts/p0' / ('compressed-summary-' + uuid.uuid4().hex[:8] + '.json')
    summary.write_text(json.dumps({'schema_version': 1, 'fixture': fixture.relative_to(ROOT).as_posix(), 'runs': [o.relative_to(ROOT).as_posix() for o in results]}, indent=2) + '\n', encoding='utf-8')
    print('Summary: ' + str(summary), flush=True)


if __name__ == '__main__': main()
