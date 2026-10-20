"""Optional MP3/Vorbis + PNG/GPU soak, with same-source WAV controls."""
from array import array
import argparse
import json
from pathlib import Path
import subprocess
import sys
import uuid
import wave

from check_p0 import ROOT
from check_p0_combined_soak import run, digest
from check_p0_compressed import NativeReference, identity_paths, run as run_pull
from check_p0_edit_composition import fixture_check
from p0_compressed_fixture import make_fixture

CODECS = ('wav', 'mp3', 'ogg')
REQUIRED_BUDGETS = {'playing_working_set_le_512_mib', 'paused_cpu_le_2_percent_one_core',
                    'closed_cpu_le_2_percent_one_core', 'paused_observed_at_least_60_seconds',
                    'closed_observed_at_least_60_seconds', 'paused_no_continuous_redraw',
                    'closed_no_continuous_redraw', 'playing_frame_cpu_p95_le_16_7_ms',
                    'normal_sampled_working_set_le_512_mib'}


def decoder_identities():
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in identity_paths()
            if p.relative_to(ROOT).as_posix().startswith('experiments/p0-media/')}


def artifact_path(path):
    path = path.resolve(strict=True)
    if (ROOT / 'artifacts/p0').resolve() not in path.parents:
        raise RuntimeError('Use an independent fixture/reference under artifacts/p0')
    return path


def prepare_audio(output):
    # Refuse an existing directory; never repair an old fixture in place.
    target = output.resolve()
    if (ROOT / 'artifacts/p0').resolve() not in target.parents:
        raise RuntimeError('Audio preparation outside artifacts/p0')
    if target.exists(): raise FileExistsError(str(target))
    manifest = make_fixture(target, durations=(600,))
    control = target / 'source-600.wav.reference.f32'
    with wave.open(str(target / 'source-600.wav'), 'rb') as source, control.open('xb') as out:
        while True:
            raw = source.readframes(4096)
            if not raw: break
            values = array('h'); values.frombytes(raw)
            if sys.byteorder != 'little': values.byteswap()
            floats = array('f', (v / 32768 for v in values))
            if sys.byteorder != 'little': floats.byteswap()
            out.write(floats.tobytes())
    by_codec = {r['codec']: r for r in manifest['items']}
    entries = []
    for codec in CODECS:
        source = target / ('source-600.wav' if codec == 'wav' else by_codec[codec]['encoded'])
        reference = control if codec == 'wav' else target / by_codec[codec]['reference']
        directory = run_pull(ROOT / 'experiments/p0-media/target/release/compressed-proof.exe', source, 'pull', reference, codec)
        proof = json.loads((directory / 'evidence.json').read_text())
        if proof['status'] != 'validated': raise RuntimeError('Full independent reference did not pass: ' + str(directory))
        entries.append({'codec': codec, 'source': source.name, 'source_sha256': digest(source),
                        'linear_reference': (directory / 'decoded.f32').relative_to(ROOT).as_posix(),
                        'linear_reference_sha256': digest(directory / 'decoded.f32'),
                        'pull_evidence': (directory / 'evidence.json').relative_to(ROOT).as_posix(),
                        'pull_evidence_sha256': digest(directory / 'evidence.json')})
    ready = {'schema_version': 1, 'generator': 'verified_compressed_soak_audio_v1', 'seconds': 600,
             'rate': 48000, 'channels': 2, 'frames': 48000 * 600,
             'source_manifest_sha256': digest(target / 'manifest.json'),
             'decoder_source_sha256': decoder_identities(), 'entries': entries}
    (target / 'soak.json').write_text(json.dumps(ready, indent=2) + '\n', encoding='utf-8')
    return target


def load_audio(path, seconds=600):
    path = artifact_path(path)
    m = json.loads((path / 'soak.json').read_text())
    if (m.get('schema_version') != 1 or m.get('generator') != 'verified_compressed_soak_audio_v1'
            or m.get('seconds') != seconds or m.get('frames') != round(seconds * 48000)
            or (m.get('rate'), m.get('channels')) != (48000, 2)
            or m.get('source_manifest_sha256') != digest(path / 'manifest.json')
            or m.get('decoder_source_sha256') != decoder_identities()):
        raise RuntimeError('Unverified/stale audio fixture or decoder identity')
    entries = m.get('entries', [])
    if len(entries) != 3 or sorted(e.get('codec', '') for e in entries) != sorted(CODECS):
        raise RuntimeError('One verified source per codec required')
    result = {}
    for row in entries:
        source = (path / row['source']).resolve(strict=True)
        reference = artifact_path(ROOT / row['linear_reference'])
        proof_path = artifact_path(ROOT / row['pull_evidence'])
        if (path not in source.parents or digest(source) != row['source_sha256']
                or digest(reference) != row['linear_reference_sha256']
                or digest(proof_path) != row['pull_evidence_sha256']):
            raise RuntimeError('Audio source/reference identity or path differs')
        proof = json.loads(proof_path.read_text())
        comparison = proof.get('comparison', {})
        decoder_sources = {k: v for k, v in proof.get('source_sha256', {}).items()
                           if k.startswith('experiments/p0-media/')}
        if (proof.get('status') != 'validated' or proof.get('mode') != 'pull'
                or proof.get('codec') != row['codec'] or proof.get('source_file_sha256') != row['source_sha256']
                or comparison.get('frames_match') is not True or comparison.get('samples_outside_tolerance') != 0
                or reference != proof_path.parent / 'decoded.f32' or decoder_sources != m['decoder_source_sha256']):
            raise RuntimeError('Linear PCM lacks a full independent reference proof')
        result[row['codec']] = (source, NativeReference(reference, m['frames']))
    return result


def result_status(rows, repetitions, smoke):
    expected = [(repeat, codec) for repeat in range(repetitions) for codec in CODECS]
    if [(r['repeat'], r['codec']) for r in rows] != expected or any(r['status'] != 'validated' for r in rows):
        raise RuntimeError('Incomplete/duplicate combined codec matrix')
    if smoke:
        if any(r['budgets'] is not None for r in rows): raise RuntimeError('Smoke cannot assess long-run budgets')
        return 'passed_smoke_not_budgeted'
    if any(r.get('gpu_samples_available') is not True for r in rows):
        return 'incomplete_gpu_measurement'
    if any(not r['budgets'] or set(r['budgets']) != REQUIRED_BUDGETS
           or any(v is not True for v in r['budgets'].values()) for r in rows):
        return 'failed_budget'
    return 'passed_scoped_combined_budgets'


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--prepare-audio', type=Path); p.add_argument('--audio-fixture', type=Path)
    p.add_argument('--run-native', action='store_true'); p.add_argument('--fixture', type=Path)
    p.add_argument('--skip-checks', action='store_true'); p.add_argument('--smoke', action='store_true')
    p.add_argument('--repetitions', type=int, choices=range(1, 4), default=2)
    args = p.parse_args()
    if not args.skip_checks:
        subprocess.run([sys.executable, str(ROOT / 'tools/check_p0_compressed.py')], check=True)
    if args.prepare_audio: prepare_audio(args.prepare_audio)
    if not args.run_native: return
    if sys.platform != 'win32' or args.fixture is None or args.audio_fixture is None:
        raise RuntimeError('Windows and explicit verified PNG/audio fixtures required')
    png = fixture_check(args.fixture)
    audio = artifact_path(args.audio_fixture)
    references = load_audio(audio)
    frozen = {p.relative_to(ROOT).as_posix(): digest(p) for p in identity_paths()}
    output = ROOT / 'artifacts/p0' / ('compressed-soak-summary-' + uuid.uuid4().hex[:8] + '.json')
    summary = {'schema_version': 1, 'status': 'running', 'smoke': args.smoke,
               'source_sha256': frozen, 'audio_soak_manifest': audio.relative_to(ROOT).as_posix() + '/soak.json',
               'audio_soak_manifest_sha256': digest(audio / 'soak.json'), 'runs': []}
    def save(): output.write_text(json.dumps(summary, indent=2) + '\n', encoding='utf-8')
    save()
    try:
        for repeat in range(args.repetitions):
            for codec in CODECS:
                source, reference = references[codec]
                directory = run(ROOT / 'experiments/p0-egui/target/release/combined-soak.exe', png, audio, args.smoke, reference,
                                source_name=source.name, codec=codec, compressed=True)
                evidence = json.loads((directory / 'evidence.json').read_text())
                if evidence['source_sha256'] != frozen: raise RuntimeError('Source identity changed between runs')
                summary['runs'].append({'repeat': repeat, 'codec': codec, 'directory': directory.relative_to(ROOT).as_posix(),
                                        'status': evidence['status'], 'budgets': evidence['budgets'],
                                        'gpu_samples_available': all(evidence['normal_sampled_peaks'].get(k) is not None
                                                                     for k in ('gpu_dedicated_bytes', 'gpu_shared_bytes'))})
                save()
        if frozen != {p.relative_to(ROOT).as_posix(): digest(p) for p in identity_paths()}:
            raise RuntimeError('Source changed after the matrix')
        if digest(audio / 'soak.json') != summary['audio_soak_manifest_sha256']:
            raise RuntimeError('Verified fixture manifest changed')
        summary['status'] = result_status(summary['runs'], args.repetitions, args.smoke)
        save()
    except Exception as exc:
        summary.update(status='rejected', error=str(exc)); save(); raise
    print('Summary: ' + str(output), flush=True)
    if summary['status'] not in ('passed_smoke_not_budgeted', 'passed_scoped_combined_budgets'):
        raise RuntimeError('See preserved failed/incomplete checks in ' + str(output))


if __name__ == '__main__': main()
