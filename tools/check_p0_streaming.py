"""Check a pull-only WAV decoder; opt-in same-process-scope memory comparison, no audio."""
import argparse
from array import array
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
import wave

from check_p0 import ROOT, windows_linker
from check_p0_media import MANIFEST, check_core
from check_p0_transport import sources
from p0_process_metrics import WindowsProcessMetrics, environment_info


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''): h.update(chunk)
    return h.hexdigest()


def make_fixture(output, durations=(3, 180)):
    if len(durations) != 2 or any(type(s) is not int or not 1 <= s <= 600 for s in durations):
        raise ValueError('Two bounded integer durations required')
    output.mkdir(exist_ok=False)
    # One self-authored second, streamed repeatedly to disk, no client/music files.
    second = b''.join(struct.pack('<hh', (n * 29) % 65536 - 32768, (n * 53 + 17) % 65536 - 32768) for n in range(48000))
    files = {}
    for name, seconds in zip(('short.wav', 'long.wav'), durations):
        p = output / name
        with wave.open(str(p), 'wb') as f:
            f.setparams((2, 2, 48000, seconds * 48000, 'NONE', 'not compressed'))
            for _ in range(seconds): f.writeframesraw(second)
        files[name] = {'seconds': seconds, 'sha256': digest(p), 'bytes': p.stat().st_size}
    manifest = {'schema_version': 1, 'generator': 'self_authored_repeating_pcm16_v1', 'files': files}
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    return manifest


def fnv_pcm(pcm):
    values = array('h'); values.frombytes(pcm)
    if sys.byteorder != 'little': values.byteswap()
    raw = struct.pack('<' + 'f' * len(values), *(v / 32768 for v in values))
    return raw


def fnv(data, state=0xcbf29ce484222325):
    for byte in data: state = ((state ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return state


def reference(path):
    with wave.open(str(path), 'rb') as f:
        if f.getparams()[:3] != (2, 2, 48000): raise RuntimeError('Unexpected reference fixture')
        frames = f.getnframes(); state = 0xcbf29ce484222325
        while True:
            pcm = f.readframes(8192)
            if not pcm: break
            state = fnv(fnv_pcm(pcm), state)
        seeks = []
        for target in (frames // 2 + 7, 11, frames - 9, frames):
            f.setpos(target); pcm = f.readframes(257)
            seeks.append({'target': target, 'generation': len(seeks) + 1, 'frames': len(pcm) // 4,
                          'eof': target + len(pcm) // 4 == frames, 'hash': fnv(fnv_pcm(pcm))})
    return {'frames': frames, 'pcm_hash': state, 'seeks': seeks, 'encoded_bytes': path.stat().st_size}


def validate(result, expected):
    if (result.get('schema_version') != 1 or result.get('status') != 'passed'
            or result.get('source') != 'self_authored_canonical_pcm16_wav' or result.get('mode') not in ('stream', 'eager')
            or result.get('audio_device_opened') is not False or result.get('system_capture_used') is not False
            or result.get('idle_unchanged') is not True or result.get('idle_ms', 0) < 300):
        raise RuntimeError('Missing streaming identity, idle or no-device evidence')
    exact = {'rate': 48000, 'channels': 2, 'frames': expected['frames'], 'pcm_hash': expected['pcm_hash'],
             'encoded_bytes': expected['encoded_bytes'], 'decoded_payload_bytes': expected['frames'] * 8,
             'retained_pcm_bytes': expected['frames'] * 8 if result['mode'] == 'eager' else 0,
             'caller_buffer_bytes': 32768, 'cache_bytes': 32768, 'source_buffer_bytes': 65536,
             'sequential_decoded_frames': expected['frames']}
    if any(type(result.get(k)) is not int or result[k] != v for k, v in exact.items()):
        raise RuntimeError('Decoded output, bounded buffers or retention baseline differs')
    if (not 44 <= result.get('open_read_bytes', 0) <= 128 * 1024
            or not expected['encoded_bytes'] <= result.get('sequential_read_bytes', 0) <= expected['encoded_bytes'] + 128 * 1024
            or not 0 < result.get('largest_packet_frames', 0) <= 4096 or result.get('sequential_packets', 0) <= 0):
        raise RuntimeError('Unexpected eager read-ahead, source traffic or packet size')
    for k in ('open_ms', 'first_pull_us', 'read_sum_us', 'read_p95_us', 'read_max_us', 'work_ms'):
        if type(result.get(k)) not in (int, float) or not math.isfinite(result[k]) or result[k] < 0:
            raise RuntimeError('Invalid timing')
    if result['read_p95_us'] > result['read_max_us'] or result['read_sum_us'] > result['work_ms'] * 1000 + 1:
        raise RuntimeError('Inconsistent read timing')
    seeks = result.get('seeks', [])
    if len(seeks) != len(expected['seeks']): raise RuntimeError('Missing seek coverage')
    for actual, e in zip(seeks, expected['seeks']):
        if (any(actual.get(k) != v for k, v in e.items()) or not math.isfinite(actual.get('wall_us', -1)) or actual['wall_us'] < 0):
            raise RuntimeError('Seek returned stale/wrong samples, identity, EOF or timing')


def summarize_samples(samples):
    if len(samples) < 10: raise RuntimeError('Insufficient process samples')
    baseline = [s for s in samples if s.get('phase') == 'baseline']
    if len(baseline) < 3: raise RuntimeError('Insufficient startup baseline samples')
    base = min(s['private_bytes'] for s in baseline)
    peak = max(s['peak_commit_bytes'] for s in samples)
    return {'sample_count': len(samples), 'baseline_private_bytes': base, 'peak_commit_bytes': peak,
            'private_growth_bytes': max(0, peak - base), 'peak_working_set_bytes': max(s['peak_working_set_bytes'] for s in samples),
            'cpu_seconds': max(s['cpu_seconds'] for s in samples) - min(s['cpu_seconds'] for s in samples)}


def run_probe(exe, fixture, name, mode, expected):
    output = ROOT / 'artifacts/p0' / ('stream-' + uuid.uuid4().hex[:8]); output.mkdir(exist_ok=False)
    paths = sources() + [Path(__file__).resolve(), ROOT / 'tools/check_p0_media.py', ROOT / 'tools/check_p0.py', ROOT / 'tools/tests/test_p0_streaming.py']
    identity = {p.relative_to(ROOT).as_posix(): digest(p) for p in paths}
    exe_hash = digest(exe); input_hash = digest(fixture / name)
    started = time.monotonic(); samples = []
    process = subprocess.Popen([str(exe), mode, str(fixture / name)], cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    metrics = None
    try:
        metrics = WindowsProcessMetrics(process)
        if json.loads(process.stdout.readline()) != {'phase': 'ready'}:
            raise RuntimeError('Probe did not enter baseline handshake')
        for _ in range(10):
            sample = metrics.sample()
            if sample is not None:
                sample.update(elapsed_seconds=time.monotonic() - started, phase='baseline'); samples.append(sample)
            time.sleep(.02)
        if len(samples) < 3: raise RuntimeError('Insufficient explicit baseline')
        process.stdin.write(b'run\n'); process.stdin.flush()
        while process.poll() is None:
            if time.monotonic() - started > 30: raise RuntimeError('Streaming probe timeout')
            sample = metrics.sample()
            if sample is not None:
                sample.update(elapsed_seconds=time.monotonic() - started, phase='work'); samples.append(sample)
            time.sleep(.02)
        stdout, stderr = process.communicate(timeout=2)
        (output / 'run.log').write_bytes(stderr)
        (output / 'result.json').write_bytes(stdout)
        if process.returncode: raise RuntimeError('Streaming probe failed: ' + str(output))
    finally:
        if process.poll() is None: process.kill(); process.wait(timeout=5)
        if metrics is not None: metrics.close()
    result = json.loads(stdout); validate(result, expected)
    if (digest(exe) != exe_hash or digest(fixture / name) != input_hash
            or any(digest(p) != identity[p.relative_to(ROOT).as_posix()] for p in paths)):
        raise RuntimeError('Input/source changed during measurement')
    for p in paths:
        dest = output / 'sources' / p.relative_to(ROOT); dest.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(p, dest)
    evidence = {'schema_version': 1, 'executable_sha256': exe_hash, 'source_sha256': identity,
                'fixture': (fixture / name).relative_to(ROOT).as_posix(), 'fixture_sha256': input_hash,
                'environment': environment_info(), 'samples': samples, 'summary': summarize_samples(samples),
                'scope': 'No device, GPU, playback or real-time scheduling; process CPU includes hashing, copying and sampling holds'}
    (output / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    print('Streaming resource evidence: ' + str(output), flush=True)
    return {'directory': output.relative_to(ROOT).as_posix(), 'mode': mode, 'input': name,
            'result_sha256': digest(output / 'result.json'), 'evidence_sha256': digest(output / 'evidence.json'),
            'result': result, 'resources': evidence['summary']}


def compare(rows):
    groups = {(mode, name): [r for r in rows if r['mode'] == mode and r['input'] == name]
              for mode, name in [('stream', 'short.wav'), ('stream', 'long.wav'), ('eager', 'long.wav')]}
    if any(not g for g in groups.values()): raise RuntimeError('Missing resource comparison case')
    if len({len(g) for g in groups.values()}) != 1: raise RuntimeError('Unbalanced repetitions')
    peaks = {k: max(r['resources']['peak_commit_bytes'] for r in g) for k, g in groups.items()}
    # Diagnostic growth gates, not the editor's total product resource budget.
    bounded_growth = peaks['stream', 'long.wav'] <= peaks['stream', 'short.wav'] + 16 * 1024 * 1024
    retention_visible = min(r['resources']['peak_commit_bytes'] for r in groups['eager', 'long.wav']) - peaks['stream', 'long.wav'] >= groups['stream', 'long.wav'][0]['result']['decoded_payload_bytes'] // 2
    return {'schema_version': 1, 'status': 'passed' if bounded_growth and retention_visible else 'failed',
            'stream_growth_within_16_mib': bounded_growth, 'long_retention_cost_visible': retention_visible,
            'product_total_budget_passed': False, 'audio_output_streaming_integrated': False, 'runs': rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-benchmark', action='store_true')
    parser.add_argument('--repetitions', type=int, choices=range(1, 4), default=3)
    args = parser.parse_args()
    environment = dict(os.environ); private = ROOT / '.p0-tools/rustup/toolchains/1.95.0-x86_64-pc-windows-msvc/bin'
    if os.name == 'nt' and (private / 'cargo.exe').is_file():
        environment['PATH'] = str(private) + os.pathsep + environment.get('PATH', '')
        environment['CARGO_HOME'] = str(ROOT / '.p0-tools/cargo'); environment['RUSTUP_HOME'] = str(ROOT / '.p0-tools/rustup')
        cargo = str(private / 'cargo.exe')
    else: cargo = shutil.which('cargo')
    if cargo is None: raise RuntimeError('Rust 1.95 and native linker required')
    linker = windows_linker(environment); check_core(environment, cargo, linker)
    subprocess.run([cargo, *linker, 'build', '--manifest-path', str(MANIFEST), '--offline', '--locked', '--release', '--bin', 'stream-proof', '-j', '2'], env=environment, check=True)
    if args.run_benchmark:
        if os.name != 'nt': raise RuntimeError('Process memory comparison is Windows-only')
        fixture = ROOT / 'artifacts/p0' / ('stream-fixture-' + uuid.uuid4().hex[:8]); make_fixture(fixture)
        print('Calculating independent WAV-to-float hashes (no audio)...', flush=True)
        refs = {name: reference(fixture / name) for name in ('short.wav', 'long.wav')}
        exe = ROOT / 'experiments/p0-media/target/release/stream-proof.exe'; rows = []
        for _ in range(args.repetitions):
            for mode, name in [('stream', 'short.wav'), ('stream', 'long.wav'), ('eager', 'long.wav')]:
                rows.append(run_probe(exe, fixture, name, mode, refs[name]))
        result = compare(rows)
        dest = ROOT / 'artifacts/p0' / ('stream-comparison-' + uuid.uuid4().hex[:8] + '.json')
        dest.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8'); print('Comparison: ' + str(dest), flush=True)
        if result['status'] != 'passed': raise RuntimeError('Resource comparison gates failed')


if __name__ == '__main__': main()
