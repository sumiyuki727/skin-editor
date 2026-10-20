"""Repeat complete PNG + decoder + WASAPI ownership cycles; opt-in, no capture."""
import argparse
import json
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import time
import uuid

from check_p0 import ROOT
from check_p0_combined_soak import Reference, digest, make_audio, timing, validate_packets
from check_p0_edit_composition import fixture_check
from check_p0_switch import phase_summary
from p0_process_metrics import WindowsProcessMetrics, environment_info

MEMORY_KEYS = ('working_set_bytes', 'private_bytes', 'gpu_dedicated_bytes', 'gpu_shared_bytes')


def validate(result, packets, smoke):
    expected = 2 if smoke else 20
    if (result.get('schema_version') != 1 or result.get('status') != 'passed' or result.get('errors')
            or result.get('smoke') is not smoke or result.get('target_cycles') != expected
            or len(result.get('cycles', [])) != expected or len(packets) != expected
            or result.get('accepted_documents') != expected or len(result.get('uploads', [])) != expected
            or any(result.get(k) is not False for k in ('session_owned_at_finish', 'active_document_at_finish',
                        'background_pending_at_finish', 'system_capture_used', 'physical_or_acoustic_latency_measured'))
            or result.get('normal_path_cpu_pixel_readbacks') != 0):
        raise RuntimeError('Incomplete resource lifecycle or incorrect measurement scope')
    for n, (cycle, trace) in enumerate(zip(result['cycles'], packets)):
        closed = cycle['closed']; s = closed['final_stats']; pause = cycle['pause']
        if (cycle['cycle'] != n or cycle['document'] != n % 10
                or cycle['continuous_device_us'] < (1 if smoke else 6) * 1000000
                or cycle['changes'] < (20 if smoke else 300) or cycle['frame_samples'] < (10 if smoke else 290)
                or not 2 <= cycle['command_nodes'] <= 3
                or s['active'] or s['in_flight'] or s['free_blocks'] != 5 or s['queued_blocks'] != 0
                or s['underruns'] != 0 or s['decode_failures'] != 0 or s['trace_overflow'] or s['max_queued_blocks'] > 4
                or closed['event_count'] != trace['events'] or closed['submit_count'] != trace['submits']):
            raise RuntimeError('Cycle coverage, PCM integrity or resource release failed at ' + str(n))
        if (pause['wall_seconds'] < 1 or pause['first']['phase'] != 'Paused' or pause['last']['phase'] != 'Paused'
                or pause['last']['sequence'] <= pause['first']['sequence']
                or pause['first']['position_us'] != pause['last']['position_us']
                or pause['first']['position_us'] < cycle['continuous_device_us']):
            raise RuntimeError('Missing fresh frozen-device observations')
    proofs = result.get('pixel_checks', [])
    if ([p['stage'] for p in proofs] != [0, expected - 1]
            or any(p['pixels'] != 1024 * 1024 or p['pixels_outside_tolerance'] != 0 or p['display_pixels_outside_tolerance'] != 0
                   or p['max_channel_error'] > 2 or p['display_max_channel_error'] > 2 for p in proofs)):
        raise RuntimeError('Missing first/last-cycle composition proofs')
    timing(result['eframe_cpu_frame_ms_playing']); timing(result['frame_intervals_ms_playing'])


def phase_key(row):
    if row is None: return None
    return row['cycle'], row['phase'], row['step']


def summarize(samples):
    labelled = [dict(s, phase='Idle') for s in samples]
    row = next(r for r in phase_summary(labelled) if r['phase'] == 'Idle')
    row.pop('phase'); return row


def summaries(samples, count):
    closed = []
    for cycle in range(count):
        selected = [s for s in samples if s.get('phase') == 'Closed' and s.get('cycle') == cycle and s.get('phase_age_seconds', 0) >= .5]
        closed.append(dict(summarize(selected), cycle=cycle))
    idle = summarize([s for s in samples if s.get('phase') == 'FinalIdle' and s.get('phase_age_seconds', 0) >= 1])
    play = summarize([s for s in samples if s.get('phase') == 'Play' and s.get('phase_age_seconds', 0) >= .5])
    # Play intervals are disjoint; CPU must not span their intervening pauses.
    active_cpu_seconds = active_wall_seconds = 0.
    for cycle in range(count):
        selected = [s for s in samples if s.get('phase') == 'Play' and s.get('cycle') == cycle and s.get('phase_age_seconds', 0) >= .5]
        if len(selected) >= 2:
            active_cpu_seconds += selected[-1]['cpu_seconds'] - selected[0]['cpu_seconds']
            active_wall_seconds += selected[-1]['monotonic_seconds'] - selected[0]['monotonic_seconds']
    play.pop('cpu_one_logical_core_percent', None); play.pop('sample_span_seconds', None)
    if active_wall_seconds > 0:
        play.update(active_sample_seconds=active_wall_seconds, cpu_one_logical_core_percent=active_cpu_seconds / active_wall_seconds * 100)
    return {'closed_cycles': closed, 'final_idle': idle, 'play': play}


def trends(rows):
    result = {}
    for key in MEMORY_KEYS:
        values = [r.get(key + '_last') for r in rows]
        if len(values) < 2 or any(v is None for v in values):
            result[key] = None; continue
        values = [v / 1048576 for v in values]; n = len(values); width = min(5, n // 2)
        xmean = (n - 1) / 2; ymean = sum(values) / n
        slope = sum((i - xmean) * (v - ymean) for i, v in enumerate(values)) / sum((i - xmean) ** 2 for i in range(n))
        result[key] = {'first_mib': values[0], 'last_mib': values[-1], 'delta_mib': values[-1] - values[0],
                       'first_group_median_mib': statistics.median(values[:width]),
                       'last_group_median_mib': statistics.median(values[-width:]),
                       'median_delta_mib': statistics.median(values[-width:]) - statistics.median(values[:width]),
                       'linear_slope_mib_per_cycle': slope}
    return result


def budgets(result, summaries_value, peaks, smoke):
    if smoke: return None
    idle = summaries_value['final_idle']
    return {'all_twenty_cycles_sampled_after_settle': len(summaries_value['closed_cycles']) == 20 and
                all(r.get('sample_span_seconds', 0) >= 2.5 for r in summaries_value['closed_cycles']),
            'normal_sampled_working_set_le_512_mib': 0 < (peaks.get('working_set_bytes') or 0) <= 512 * 1024 * 1024,
            'final_idle_observed_at_least_60_seconds': idle.get('sample_span_seconds', 0) >= 60,
            'final_idle_cpu_le_2_percent_one_core': idle.get('cpu_one_logical_core_percent', 100) <= 2,
            'final_idle_no_continuous_redraw': result['final_idle_frames'] <= 2,
            'playing_cpu_frame_p95_le_16_7_ms': timing(result['eframe_cpu_frame_ms_playing'])['p95'] <= 16.7}


def read_phase(path):
    try: return json.loads(path.read_text())
    except (OSError, ValueError): return None


def run(exe, png, audio, reference, smoke):
    output = ROOT / 'artifacts/p0' / ('resource-cycles-' + uuid.uuid4().hex[:8])
    paths = [ROOT / 'experiments' / c / f for c in ('p0-egui', 'p0-wasapi', 'p0-media', 'p0-preview-contract') for f in ('Cargo.toml', 'Cargo.lock')]
    paths += [p for c in ('p0-egui', 'p0-wasapi', 'p0-media', 'p0-preview-contract') for p in sorted((ROOT / 'experiments' / c / 'src').rglob('*')) if p.is_file()]
    paths += sorted((ROOT / 'tools').rglob('*.py'))
    frozen = {p.relative_to(ROOT).as_posix(): p.read_bytes() for p in paths}
    identity = {p.relative_to(ROOT).as_posix(): digest(p) for p in paths}; binary = digest(exe)
    samples = []; last_key = None; phase_start = time.monotonic(); error = None; gpu_errors = []
    print('Starting ' + str(output) + (' smoke, 2 cycles' if smoke else ' 20 complete resource cycles'), flush=True)
    log_path = output.with_suffix('.log')
    with log_path.open('wb') as log:
        process = subprocess.Popen([str(exe), str(output), str(png), str(audio / 'stream.wav'), 'smoke' if smoke else 'full'], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        metrics = None; start = time.monotonic()
        try:
            metrics = WindowsProcessMetrics(process)
            while process.poll() is None:
                if time.monotonic() - start > (100 if smoke else 630): raise RuntimeError('Resource cycles timed out')
                row = read_phase(output / 'phase.json'); key = phase_key(row)
                if key != last_key:
                    last_key = key; phase_start = time.monotonic()
                    if row and row['phase'] in ('Open', 'FinalIdle', 'Done'):
                        print(output.name + ': cycle ' + str(row['cycle'] + 1) + ' ' + row['phase'], flush=True)
                value = metrics.sample()
                if value is not None and row is not None and key == phase_key(read_phase(output / 'phase.json')):
                    value.update(phase=row['phase'], cycle=row['cycle'], step=row['step'], phase_age_seconds=time.monotonic() - phase_start)
                    samples.append(value)
                time.sleep(.2)
        except Exception as exc: error = str(exc)
        finally:
            if process.poll() is None: process.kill(); process.wait(timeout=5)
            if metrics is not None: gpu_errors = metrics.gpu_errors; metrics.close()
    output.mkdir(exist_ok=True); shutil.copyfile(log_path, output / 'run.log')
    for rel, data in frozen.items():
        dest = output / 'source-snapshot' / rel; dest.parent.mkdir(parents=True, exist_ok=True); dest.write_bytes(data)
    evidence = {'schema_version': 1, 'smoke': smoke, 'source_sha256': identity, 'executable_sha256': binary,
                'png_fixture': png.relative_to(ROOT).as_posix(), 'png_inventory_sha256': digest(png / 'inventory.json'),
                'audio_fixture': audio.relative_to(ROOT).as_posix(), 'audio_manifest_sha256': digest(audio / 'manifest.json'),
                'environment': environment_info(), 'samples': samples, 'gpu_errors': gpu_errors,
                'error': error, 'process_exit_code': process.returncode}
    try:
        if error or process.returncode: raise RuntimeError(error or 'Native cycle process failed')
        if digest(exe) != binary or any(digest(p) != identity[p.relative_to(ROOT).as_posix()] for p in paths): raise RuntimeError('Measured source or binary changed')
        shutil.copyfile(exe, output / 'source-snapshot/resource-cycle-proof.exe')
        result = json.loads((output / 'result.json').read_text())
        packets = [validate_packets(output / ('cycle-%02d-packets.jsonl' % n), reference) for n in range(2 if smoke else 20)]
        validate(result, packets, smoke)
        summary = summaries(samples, len(result['cycles']))
        normal = [s for s in samples if s['phase'] not in ('Diagnostic', 'Done')]
        peaks = {k: max((s[k] for s in normal if s.get(k) is not None), default=None) for k in MEMORY_KEYS}
        evidence.update(status='validated', packets=packets, summary=summary, closed_memory_trends=trends(summary['closed_cycles']),
                        normal_sampled_peaks=peaks, historical_peak_counters={k: max((s[k] for s in samples), default=None) for k in ('peak_working_set_bytes', 'peak_commit_bytes')},
                        budgets=budgets(result, summary, peaks, smoke),
                        timings={k: timing(result[k]) for k in ('eframe_cpu_frame_ms_playing', 'frame_intervals_ms_playing')},
                        open_ms=timing([r['open_ms'] for r in result['cycles']]))
    except Exception as exc: evidence.update(status='rejected', error=str(exc))
    (output / 'evidence.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
    print(output.name + ': ' + evidence['status'] + ' ' + str(evidence.get('budgets', evidence.get('error'))), flush=True)
    if evidence['status'] != 'validated': raise RuntimeError(str(output) + ': ' + str(evidence['error']))
    return output


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--run-native', action='store_true'); p.add_argument('--smoke', action='store_true')
    p.add_argument('--skip-checks', action='store_true', help='Use separately checked release executable')
    p.add_argument('--repetitions', type=int, choices=range(1, 4), default=2)
    p.add_argument('--fixture', type=Path); p.add_argument('--audio-fixture', type=Path); args = p.parse_args()
    if not args.skip_checks: subprocess.run([sys.executable, str(ROOT / 'tools/check_p0_gui.py'), '--media'], check=True)
    if args.run_native:
        if sys.platform != 'win32' or args.fixture is None: raise RuntimeError('Windows and an explicit standard PNG fixture required')
        png = fixture_check(args.fixture)
        audio = args.audio_fixture.resolve(strict=True) if args.audio_fixture else make_audio(ROOT / 'artifacts/p0' / ('cycle-audio-fixture-' + uuid.uuid4().hex[:8]))
        if (ROOT / 'artifacts/p0').resolve() not in audio.parents: raise RuntimeError('Use a self-authored audio fixture inside artifacts/p0')
        ref = Reference(audio)
        for _ in range(args.repetitions): run(ROOT / 'experiments/p0-egui/target/release/resource-cycle-proof.exe', png, audio, ref, args.smoke)


if __name__ == '__main__': main()
