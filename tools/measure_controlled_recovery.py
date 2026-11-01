"""Native recovery admission at two trigger boundaries; no OS stalls are injected."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
from measure_audio_load import ROOT, observe
from measure_combined import source_identity
from measure_supply_recovery import evaluate

PLAN = (('FixedAhead', 'Natural', '--verify-voice-phase'),
        ('RecoverLateWake', 'WithTrigger', '--verify-recovery-with-trigger'),
        ('RecoverLateWake', 'AfterRefill', '--verify-recovery-after-refill'),
        ('FixedAhead', 'Natural', '--verify-voice-phase'))


def evaluate_controlled(report, samples, policy, mode):
    result = evaluate(report, samples, policy)
    if report['recovery_probe_mode'] != mode:
        raise ValueError('wrong recovery probe mode')
    rows = report['recovery_rows']
    if len(rows) != 2 or any(len(lane) != 160 for lane in rows):
        raise ValueError('missing per-probe recovery evidence')
    counters = [report[k] for k in ('diagnostic_recovery_passes', 'diagnostic_refill_passes', 'diagnostic_refill_frames')]
    if any(type(v) is not int or v < 0 for v in counters) or counters[0] > report['recovery_passes'] or counters[1] > counters[0]:
        raise ValueError('invalid diagnostic counters')
    previous = [0, 0, 0]
    filled = 0
    for index in range(160):
        for lane in range(2):
            row = rows[lane][index]
            if len(row) != 6 or any(type(v) is not int or v < 0 for v in row):
                raise ValueError('invalid recovery row')
            before, after = row[::2], row[1::2]
            if before != previous or any(a < b for a, b in zip(after, before)):
                raise ValueError('recovery counters discontinuous')
            delta = [a-b for a,b in zip(after,before)]
            if (delta[0] > 1 or delta[1] > delta[0] or delta[2] > 2048 or delta[2] % 128 or
                    bool(delta[1]) != bool(delta[2])):
                raise ValueError('refill attempt/commit mismatch')
            if mode == 'Natural' and any(row):
                raise ValueError('normal control contains injected recovery')
            if mode != 'Natural' and delta[0] != 1:
                raise ValueError('recovery request not executed')
            filled += delta[1]
            previous = after
    if previous != counters:
        raise ValueError('final recovery counters disagree')
    result['checks']['controlled_refills_committed'] = mode == 'Natural' or filled == 320
    result['checks']['old_voices_stopped'] = report['cleanup_stopped'] is True
    result['mode'] = mode
    result['actual_diagnostic_refills'] = filled
    result['diagnostic_refill_frames'] = counters[2]
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    binary = Path(args.binary).resolve()
    out = (ROOT/args.output).resolve()
    if os.name != 'nt' or binary != (ROOT/'desktop/target/release/skin-editor-desktop.exe').resolve() or not binary.is_file():
        raise RuntimeError('explicit Windows release binary required')
    if out.exists() or ROOT/'artifacts' not in out.parents:
        raise RuntimeError('new artifacts directory required')
    out.mkdir(parents=True)
    identity = dict(source_sha256=source_identity(), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), platform=platform.platform())
    (out/'identity.json').write_text(json.dumps(identity, indent=2)+'\n', encoding='utf-8')
    runs = []
    for index, (policy, mode, flag) in enumerate(PLAN):
        print('Starting %d/4 %s %s' % (index+1, policy, mode), flush=True)
        try:
            report, samples = observe(binary, out/('run-%02d' % (index+1)), flag, timeout_seconds=90)
            result = dict(completed=True, **evaluate_controlled(report, samples, policy, mode))
            print(json.dumps(dict(mode=mode, checks=result['checks'], upper_ms=result['old_latency_ms'], causal=result['causal'], actual_refills=result['actual_diagnostic_refills'])), flush=True)
        except (OSError, ValueError, RuntimeError, TypeError, KeyError) as failure:
            result = dict(completed=False, policy=policy, mode=mode, error=str(failure), checks=dict(completed=False))
        runs.append(result)
        unchanged = identity['source_sha256'] == source_identity() and identity['binary_sha256'] == hashlib.sha256(binary.read_bytes()).hexdigest()
        metrics = dict(schema=1, plan=PLAN, sources_unchanged=unchanged, runs=runs,
            passed=len(runs)==4 and unchanged and all(all(r['checks'].values()) for r in runs),
            scope='Synthetic recovery admission on the production owner and native callback, not an OS scheduling stall or long graphics acceptance')
        (out/'metrics.json').write_text(json.dumps(metrics, indent=2)+'\n', encoding='utf-8')
        if not result['completed'] or not unchanged:
            break
    if not metrics['passed']:
        raise RuntimeError('controlled recovery gates failed; evidence retained, no retry')


if __name__ == '__main__':
    main()
