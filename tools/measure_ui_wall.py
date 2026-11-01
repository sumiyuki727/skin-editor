"""Two ordinary-permission GUI diagnostics. Wall time is NOT CPU acceptance.

No ETW collector, UAC, reference reuse, counter correction or product retry.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
from measure_audio_load import ROOT, observe, summarize, distribution
from measure_cpu_threads import token_info
from measure_independent_cpu import BINARY, fingerprint
from measure_main_work import categories_for_schema, row_limit, MAX_CLOCK_NS, ordinary
from measure_gui_candidate import evaluate_gui
from measure_background_stream import stream_checks, audit_checks

UI_KINDS = (5, 9, 10, 11, 12, 13, 14, 15)


def widget_wall(work, minimum_calls=1):
    if not isinstance(work, dict) or work.get("schema") not in (3,4) or work.get("invalid") != 0:
        raise ValueError("complete schema 3/4 widget boundaries required")
    rows = work.get("rows")
    if not isinstance(rows, list) or not 2 <= len(rows) <= row_limit(work['schema']):
        raise ValueError("widget boundary capacity")
    names = categories_for_schema(work['schema'])
    kinds = UI_KINDS+tuple(range(16,len(names)))
    previous = None; active = None; frames = []; drifts = []; start = 0
    for row in rows:
        if (not isinstance(row, list) or len(row) != 3
                or any(type(v) is not int or v < 0 for v in row) or row[2] >= len(names)):
            raise ValueError("invalid widget boundary")
        epoch, elapsed, kind = row
        if previous and (epoch < previous[0] or elapsed < previous[1]):
            raise ValueError("widget clock reversal")
        drifts.append((epoch - rows[0][0]) - (elapsed - rows[0][1]))
        if active is not None:
            active[previous[2]] += elapsed - previous[1]
            if kind == 8:
                if sum(active.values()) != elapsed - start:
                    raise ValueError("widget wall partition mismatch")
                frames.append(dict(total_ns=elapsed-start, stages_ns=active)); active = None
            elif kind not in kinds:
                raise ValueError("host boundary inside widget call")
        elif kind == 5:
            active = dict.fromkeys(kinds, 0); start = elapsed
        elif kind in kinds:
            raise ValueError("widget phase outside a widget call")
        previous = row
    if active is not None or len(frames) < minimum_calls:
        raise ValueError("incomplete widget calls")
    if max(drifts) - min(drifts) > MAX_CLOCK_NS:
        raise ValueError("widget UTC/monotonic disagreement")
    totals = [f['total_ns'] / 1e6 for f in frames]
    return dict(scope="all complete EditorUi::show calls; monotonic WALL time including preemption; not CPU",
                calls=len(frames), monotonic_disagreement_ns=max(drifts)-min(drifts),
                total_ns=sum(f['total_ns'] for f in frames), total_ms=distribution(totals),
                stages={names[k]: dict(total_ns=sum(f['stages_ns'][k] for f in frames),
                                      per_call_ms=distribution([f['stages_ns'][k]/1e6 for f in frames]))
                        for k in kinds},
                groups={label:dict(total_ns=sum(sum(f['stages_ns'][k] for k in group) for f in frames),
                    per_call_ms=distribution([sum(f['stages_ns'][k] for k in group)/1e6 for f in frames]))
                    for label,group in [('scene_controls',(9,16,17) if work['schema']==4 else (9,)),
                        ('audio_panel',(13,*range(18,25)) if work['schema']==4 else (13,))]})


def diagnostic_checks(checks):
    # Keep all original raw predicates in the report. The known uncalibrated
    # counter cannot authorize CPU acceptance even when its raw value is low.
    return {k: v for k, v in checks.items() if k != 'cpu_average_le_15_one_core'}


def main():
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--output', required=True)
    args = p.parse_args(); output = (ROOT / args.output).resolve()
    if os.name != 'nt': raise ValueError('ordinary Windows diagnostic; no elevation attempted')
    token = token_info()
    if ROOT / 'artifacts' not in output.parents or output.exists() or not ordinary(token):
        raise ValueError('ordinary Windows token and fresh artifacts output required; no elevation attempted')
    output.mkdir(parents=True); initial = fingerprint()
    proof = dict(schema=1, identity=initial, runner_token=token, rounds=[],
                 plan=['observed_gui_1', 'observed_gui_2'], uac_requested=False,
                 cpu_meter_validated=False, cpu_budget_passed=None, p2_a3_accepted=False,
                 diagnostic_complete=False)
    for index in range(1, 3):
        row = dict(round=index)
        try:
            if fingerprint() != initial: raise ValueError('source or binary changed before admission')
            print('Ordinary GUI wall diagnostic %d/2; no CPU acceptance' % index, flush=True)
            report, samples = observe(BINARY, output/('run-%d'%index), '--verify-main-work-gui', timeout_seconds=45)
            metrics = summarize(samples, phases=('idle', 'voices_1', 'voices_32', 'closed'),
                                minimum_spans=(('idle',.7), ('voices_1',1.2), ('voices_32',1.7), ('closed',.7)))
            latency, joined, checks = evaluate_gui(report, metrics)
            checks.update(stream_checks(report, False)); checks.update(audit_checks(report))
            row.update(wall=widget_wall(report.get('main_work'), minimum_calls=128), latency=latency,
                       joined_latency=joined, raw_metrics=metrics, original_checks=checks,
                       diagnostic_checks=diagnostic_checks(checks),
                       memory_peak={k:max(s[k] for s in samples) for k in ('working_set_bytes','private_bytes')},
                       audio_health={k:report[k] for k in ('missing_frames','pcm_peak_bytes','pcm_final_bytes',
                           'source_raw_peak','source_raw_final','callback_resource_releases',
                           'endpoint_returns_in_render','endpoint_returns_outside_render','max_live_targets')})
        except (OSError, RuntimeError, ValueError, KeyError) as error:
            row['error'] = str(error)
        proof['rounds'].append(row)
    proof['unchanged'] = fingerprint() == initial
    proof['diagnostic_complete'] = (proof['unchanged'] and len(proof['rounds']) == 2
        and all('error' not in r and all(r['diagnostic_checks'].values()) for r in proof['rounds']))
    proof['tool_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    (output/'proof.json').write_text(json.dumps(proof, indent=2)+'\n', encoding='utf8')
    if not proof['diagnostic_complete']: raise SystemExit(1)
    print('Wall/lifecycle diagnostic complete; CPU acceptance unavailable', flush=True)


if __name__ == '__main__': main()
