"""Fixed ordinary-window ABBA with explicit official handler observation.

Keeps sound, original gates and every failed round. Collector only asks UAC.
"""
import argparse
import hashlib
import json
import os
from collections import defaultdict
from pathlib import Path
import subprocess
from measure_audio_load import ROOT, distribution
from measure_cpu_threads import token_info, thread_costs
from measure_independent_cpu import fingerprint, build_collector, calibration, scheduler_intervals
from measure_main_work import Collector, run_child, ordinary, reference_workload_valid, main_costs, clock_mapping
from measure_repaint import close_session, require_references, repaint_costs
from measure_gui_base import PLAN, comparison_checks, candidate_passed
from measure_frame_dispatch import validate_spans, span_costs, NAMES, painter_partition, layout_passes
from measure_painter_cpu import exclusive_cpu


def handler_wall(rows):
    """Whole stage-4 wall only; never estimates or substitutes CPU occupancy."""
    callbacks = sorted((r[4], r[5], r[1], r[6], r[7]) for r in rows if 19 <= r[1] <= 24)
    if any(a[1] > b[0] for a, b in zip(callbacks, callbacks[1:])):
        raise ValueError('overlapping handler wall intervals')
    groups = defaultdict(list); bodies = defaultdict(list)
    for a, b, k, start_stage, end_stage in callbacks:
        if start_stage == end_stage == 4:
            bodies[NAMES[k]].append((b-a)/1e6)
    for before, after in zip(callbacks, callbacks[1:]):
        if before[3:] == after[3:] == (4, 4):
            groups[NAMES[before[2]] + '->' + NAMES[after[2]]].append((after[0]-before[1])/1e6)
    if any(len(bodies[NAMES[k]]) < 32 for k in (19,20,21,22)):
        raise ValueError('whole stage handler wall coverage incomplete')
    return dict(callback_wall_ms={k: distribution(v) for k,v in bodies.items()},
                gap_wall_ms={k: distribution(v) for k,v in groups.items()},
                cpu_meter_validated=False, cpu_budget_passed=None,
                scope='whole stage monotonic wall; gaps include blocking; no CPU inference or attribution of old captures')


def handler_gaps(rows, pid, tid, begin, end, frequency, offset, intervals, bound_ns):
    """Only callback-return to next-callback-entry gaps; not pure OS/driver time."""
    handlers = sorted(((r[2] * frequency - offset) // 10**9,
                       (r[3] * frequency - offset) // 10**9, r[1]) for r in rows if 19 <= r[1] <= 24)
    if not handlers or handlers[0][0] > begin or handlers[-1][1] < end:
        raise ValueError('handler coverage missing at measurement edges')
    groups = defaultdict(lambda: dict(count=0, qpc_ticks=0, wall_ticks=0, clipped=0))
    running = [(a, b) for p, t, _, a, b in intervals if p == pid and t == tid]
    for (_, previous_end, before), (following_begin, _, after) in zip(handlers, handlers[1:]):
        if previous_end > following_begin:
            raise ValueError('overlapping/reentrant handler observation')
        a, b = max(previous_end, begin), min(following_begin, end)
        if a >= b:
            continue
        group = groups[NAMES[before] + '->' + NAMES[after]]
        group['count'] += 1; group['wall_ticks'] += b - a
        group['clipped'] += int(a != previous_end or b != following_begin)
        group['qpc_ticks'] += sum(max(0, min(b, y) - max(a, x)) for x, y in running)
    for g in groups.values():
        g['cpu_one_core_percent'] = 100 * g['qpc_ticks'] / (end - begin)
        g['wall_ms'] = 1000 * g['wall_ticks'] / frequency
        g['boundary_uncertainty_percent_upper'] = 2 * g['count'] * bound_ns * frequency / 10**9 * 100 / (end - begin)
    return dict(groups=dict(groups), qpc_ticks=sum(g['qpc_ticks'] for g in groups.values()),
                scope='callback return to next callback entry; includes dispatch, waits, native hooks and observation; not pure driver/OS attribution')


def loop_costs(trace, report, base, raw, intervals):
    if raw.get('schema') != 3:
        raise ValueError('schema 3 official handler capture required')
    rows, spread = validate_spans(raw)
    mains = [t for t in base['threads'] if t['process_entry_start']]
    if len(mains) != 1:
        raise ValueError('unique owned main required')
    offset, clock = clock_mapping(report['clock_pairs'], trace['frequency'])
    w = base['costs']['voices_32']
    args = (rows, report['pid'], mains[0]['tid'], w['qpc_begin'], w['qpc_end'], trace['frequency'], offset, intervals, clock['bound_ns'])
    exclusive = exclusive_cpu(*args); gaps = handler_gaps(*args)
    if (abs(exclusive['main_cpu_one_core_percent'] - mains[0]['cpu_one_core_percent']) > 1e-8
            or gaps['qpc_ticks'] != exclusive['regions']['outside_recorded_scopes']['exclusive_qpc_ticks']):
        raise ValueError('handler gaps/exclusive/main occupancy do not agree')
    return dict(exclusive=exclusive, gaps=gaps, clock=clock, monotonic_disagreement_ns=spread,
                painter_wall=painter_partition(rows))


def delivery_counts(values):
    """Same CPU inner window, actual dispatched events; no FPS normalization."""
    if len(values) != 4 or tuple(v.get('role') for v in values) != PLAN:
        raise ValueError('complete fixed plan required for delivery comparison')
    result = []
    for v in values:
        regions = v.get('framework_spans', {}).get('regions', {})
        names = ('redraw_event', 'handler_user_event', 'handler_new_events', 'handler_about_to_wait')
        if any(type(regions.get(k, {}).get('count')) is not int or regions[k]['count'] < 32 for k in names):
            raise ValueError('actual dispatch counts missing; never report zero work')
        counts = {k: regions[k]['count'] for k in names}
        result.append(dict(role=v['role'], **counts,
                           repaint_events_per_redraw=counts['handler_user_event']/counts['redraw_event']))
    return result


def stage_delivery_counts(raw, ui_calls, span_seconds):
    """Discrete whole-stage evidence ONLY; never qualifies failed UTC alignment.

    Frame/stage identities do not use the UTC/QPC mapping. The original strict
    attribution failure remains, even when these independent counts are usable.
    """
    if (raw.get('schema') != 3 or any(raw.get(k) != 0 for k in ('invalid','active','owners'))
            or type(raw.get('spans')) is not int or not 1 <= raw['spans'] <= 32768
            or not isinstance(raw.get('rows'), list) or len(raw['rows']) != raw['spans']
            or not isinstance(span_seconds, (float,int)) or not 0 < span_seconds < 60):
        raise ValueError('complete bounded stage recording required')
    ids=set(); counts={k:0 for k in (5,19,20,21,22)}
    for r in raw['rows']:
        if (not isinstance(r,list) or len(r)!=8 or any(type(v) is not int or v<0 for v in r)
                or not 2<=r[0]<=32769 or r[0] in ids or not 1<=r[1]<=24
                or r[4]>r[5] or max(r[6:])>17):
            raise ValueError('invalid stage identity, monotonic edge or count')
        ids.add(r[0])
        if r[1] in counts and r[6:]==[4,4]: counts[r[1]]+=1
    if any(n<32 for n in counts.values()):raise ValueError('stage dispatch coverage incomplete')
    layout=layout_passes(raw,ui_calls)
    return dict(counts={NAMES[k]:v for k,v in counts.items()},layout=layout,
                ui_calls_per_second=ui_calls/span_seconds,frames_per_second=layout['whole_stage_frame_ids']/span_seconds,
                cpu_comparison_valid=False,
                scope='discrete whole-stage identities and lifecycle monotonic duration only; not CPU-window counts, not a repair of failed UTC/QPC attribution')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('output', 'baseline', 'candidate'):
        p.add_argument('--' + name, required=True)
    args = p.parse_args(); output = (ROOT / args.output).resolve()
    binaries = {name: (ROOT / getattr(args, name)).resolve() for name in ('baseline', 'candidate')}
    if (os.name != 'nt' or output.exists() or ROOT / 'artifacts' not in output.parents
            or not ordinary(token_info()) or any(ROOT / 'artifacts' not in f.parents or not f.is_file()
            or f.name != 'skin-editor-desktop.exe' for f in binaries.values())):
        raise ValueError('ordinary runner, fresh artifacts and owned frozen binaries required')
    output.mkdir(); initial = fingerprint(); hashes = {k: hashlib.sha256(f.read_bytes()).hexdigest() for k, f in binaries.items()}
    collector_binary = build_collector(output); runner = token_info(); collector = None; refs = []; reports = []
    proof = dict(schema=1, identity=initial, binary_sha256=hashes, plan=PLAN, references=[], products=[],
                 calibrated=False, comparison_valid=False, passed=False, p2_a3_accepted=False)
    try:
        self_test = json.loads(subprocess.check_output([str(collector_binary), '--self-test'], text=True))
        if self_test.get('lifetime_self_test') is not True:
            raise ValueError('collector lifetime test failed')
        proof['collector_self_test'] = self_test
        refpath = output / 'references'; refpath.mkdir()
        print('UAC 1/2: strict references', flush=True); collector = Collector(collector_binary, refpath)
        for i, reverse in enumerate((False, True), 1):
            print('strict reference %d/2' % i, flush=True)
            r = run_child(refpath / ('reference-%d' % i), 'reference', reverse=reverse); refs.append(r)
            if not reference_workload_valid(r):
                raise ValueError('strict workload failed; products not admitted')
        trace, checks = close_session(collector, refpath, runner, refs, 2); collector = None
        proof['reference_permission'] = checks
        proof['references'] = [calibration(dict(trace, root_pid=r['pid']), r) for r in refs]
        require_references(proof['references']); proof['calibrated'] = True
        prodpath = output / 'products'; prodpath.mkdir()
        print('references passed; UAC 2/2: fixed A/B/B/A', flush=True); collector = Collector(collector_binary, prodpath)
        for i, role in enumerate(PLAN, 1):
            if fingerprint() != initial or any(hashlib.sha256(f.read_bytes()).hexdigest() != hashes[k] for k, f in binaries.items()):
                raise ValueError('frozen source/binary changed')
            print('GUI %d/4 %s' % (i, role), flush=True)
            reports.append(run_child(prodpath / ('product-%d' % i), 'product', profile=True,
                                     binary=binaries[role], ui_mode='dispatch'))
        trace, checks = close_session(collector, prodpath, runner, reports, 4); collector = None
        proof['product_permission'] = checks; intervals = scheduler_intervals(trace)
        for i, (role, r) in enumerate(zip(PLAN, reports), 1):
            v = thread_costs(trace, r); v['role'] = role; v['checks']['strict_references_passed'] = True
            try:
                v['main_categories'] = main_costs(trace, r, v); v['repaint_sources'] = repaint_costs(trace, r, v)
                raw = json.loads((prodpath / ('product-%d/gui/framework_trace.json' % i)).read_text(encoding='utf8'))
                v['framework_spans'] = span_costs(trace, r, v, raw, intervals)
                v['event_loop'] = loop_costs(trace, r, v, raw, intervals)
                v['checks']['observations_complete'] = True
            except (ValueError, KeyError) as e:
                v['attribution_error'] = str(e); v['checks']['observations_complete'] = False
            proof['products'].append(v)
        proof['comparison_checks'] = comparison_checks(proof['products'], reports)
        proof['comparison_valid'] = all(proof['comparison_checks'].values())
        try:
            proof['delivery_counts'] = delivery_counts(proof['products'])
        except ValueError as e:
            proof['delivery_error'] = str(e); proof['comparison_valid'] = False
        proof['passed'] = proof['comparison_valid'] and candidate_passed(proof['products'])
    except (OSError, RuntimeError, ValueError, KeyError, subprocess.SubprocessError) as e:
        proof['error'] = str(e); print(str(e), flush=True)
    finally:
        if collector:
            try:
                collector.close()
            except (OSError, RuntimeError) as e:
                proof['cleanup_error'] = str(e); proof['passed'] = False
        proof['unchanged'] = initial == fingerprint() and all(hashlib.sha256(f.read_bytes()).hexdigest() == hashes[k] for k, f in binaries.items())
        proof['collector_sha256'] = hashlib.sha256(collector_binary.read_bytes()).hexdigest()
        (output / 'proof.json').write_text(json.dumps(proof, indent=2) + '\n', encoding='utf8')
    if not proof['passed'] or not proof['unchanged']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
