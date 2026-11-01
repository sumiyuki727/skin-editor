"""Four audio-only phase scans: fixed/recovery/recovery/fixed, no gate changes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
from measure_audio_load import ROOT, observe, summarize
from measure_combined import source_identity
from measure_voice_phase import phase_analysis
from measure_scheduling import analyze_timing

PLAN = ('FixedAhead', 'RecoverLateWake', 'RecoverLateWake', 'FixedAhead')


def evaluate(report, samples, expected):
    if report['supply_policy'] != expected:
        raise ValueError('wrong supply policy')
    count, target = report['recovery_passes'], report['max_recovery_target']
    if (type(count) is not int or count < 0 or type(target) is not int or
            not 0 <= target <= 2048 or bool(count) != bool(target) or
            expected == 'FixedAhead' and count):
        raise ValueError('invalid recovery counters')
    result = phase_analysis(report)
    result['scheduling'] = analyze_timing(report['scheduling'])
    phases = summarize(samples, ('phase_scan', 'closed'), (('phase_scan', 10), ('closed', 1)))
    result['phases'] = phases
    playing = phases['phase_scan']
    result['checks'].update(
        scheduling_complete=result['scheduling']['complete'],
        cpu_one_core_le_15_percent=playing['cpu_one_logical_core_percent'] <= 15,
        memory_le_512_mib=max(s['working_set_bytes'] for s in samples) <= 512*1024**2
        and max(s['private_bytes'] for s in samples) <= 512*1024**2,
        lifetime_peak_le_512_mib=max(s['lifetime_peak_working_set_bytes'] for s in samples) <= 512*1024**2
        and max(s['lifetime_peak_pagefile_bytes'] for s in samples) <= 512*1024**2)
    result['policy'] = expected
    result['recovery_passes'] = count
    result['max_recovery_target'] = target
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    binary = Path(args.binary).resolve()
    out = (ROOT / args.output).resolve()
    if os.name != 'nt' or binary != (ROOT/'desktop/target/release/skin-editor-desktop.exe').resolve() or not binary.is_file():
        raise RuntimeError('explicit Windows release binary required')
    if out.exists() or ROOT/'artifacts' not in out.parents:
        raise RuntimeError('new artifacts directory required')
    out.mkdir(parents=True)
    fingerprint = hashlib.sha256(binary.read_bytes()).hexdigest()
    identity = source_identity()
    (out/'identity.json').write_text(json.dumps(dict(binary_sha256=fingerprint, source_sha256=identity,
        platform=platform.platform()), indent=2)+'\n', encoding='utf-8')
    runs = []
    for index, policy in enumerate(PLAN):
        print('Starting %d/4 %s' % (index+1, policy), flush=True)
        flag = '--verify-voice-phase' if policy == 'FixedAhead' else '--verify-voice-phase-recovery'
        try:
            report, samples = observe(binary, out/('run-%02d' % (index+1)), flag, timeout_seconds=90)
            result = dict(completed=True, **evaluate(report, samples, policy))
            print(json.dumps(dict(policy=policy, checks=result['checks'], old=result['old_latency_ms'],
                recovery_passes=result['recovery_passes'], max_recovery_target=result['max_recovery_target'])), flush=True)
        except (OSError, ValueError, RuntimeError, TypeError, KeyError) as failure:
            result = dict(completed=False, policy=policy, error=str(failure), checks=dict(completed=False))
        runs.append(result)
        unchanged = identity == source_identity() and fingerprint == hashlib.sha256(binary.read_bytes()).hexdigest()
        metrics = dict(schema=1, plan=PLAN, sources_unchanged=unchanged, runs=runs,
            passed=len(runs)==4 and unchanged and all(all(r['checks'].values()) for r in runs),
            scope='Small audio-only native comparison, no injected stalls or GPU; current default device only; no long-combination acceptance')
        (out/'metrics.json').write_text(json.dumps(metrics, indent=2)+'\n', encoding='utf-8')
        if not unchanged or not result['completed']:
            break
    if not metrics['passed']:
        raise RuntimeError('comparison gates failed; all attempts retained, no retry')


if __name__ == '__main__':
    main()
