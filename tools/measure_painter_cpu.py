"""Offline exclusive CPU partition of an already captured schema-2 dispatch run.

Never elevates, launches targets, subtracts observer cost or changes acceptance.
"""
import argparse
import hashlib
import json
from pathlib import Path
from measure_audio_load import ROOT
from measure_frame_dispatch import NAMES, validate_spans, painter_partition
from measure_independent_cpu import scheduler_intervals
from measure_main_work import PLAN, clock_mapping
from measure_repaint import require_references


def exclusive_cpu(rows, pid, tid, begin, end, frequency, offset, intervals, bound_ns):
    """Partition actual intervals before intersecting with owned scheduled time.

    Cross-CPU overlap remains counted, matching conservative scheduler totals.
    A parent remainder is an observed gap, never a named driver function.
    """
    if (any(type(v) is not int or v <= 0 for v in (pid, tid, begin, end, frequency))
            or end <= begin or bound_ns < 0 or len(rows) > 32768):
        raise ValueError('invalid exclusive observation window')
    def qpc(epoch):
        return (epoch * frequency - offset) // 1_000_000_000
    mapped = sorted((qpc(r[2]), qpc(r[3]), r[1]) for r in rows)
    mapped.sort(key=lambda r: (r[0], -r[1], r[2]))
    # A synthetic root covers the measurement AND every captured scope, so edge
    # spans are clipped only after structural validation, never silently dropped.
    root = [min(begin, mapped[0][0]) if mapped else begin,
            max(end, max((r[1] for r in mapped), default=end)), 0, []]
    nodes = [root]; stack = [0]; last_by_kind = {}; counts = {}
    painter_children = (1, 2, 3, 4, 16, 17, 18)
    for a, b, kind in mapped:
        if not 1 <= kind < len(NAMES) or b < a:
            raise ValueError('bad scope kind or clock reversal')
        if a < last_by_kind.get(kind, a):
            raise ValueError('same-kind overlapping scopes')
        last_by_kind[kind] = b
        while len(stack) > 1 and nodes[stack[-1]][1] <= a:
            stack.pop()
        parent = nodes[stack[-1]]
        if b > parent[1]:
            raise ValueError('crossing scopes cannot form exclusive regions')
        if kind in painter_children and parent[2] != 15:
            raise ValueError('rendered API/upload/encoding outside Painter')
        if kind == 15 and parent[2] != 13:
            raise ValueError('Painter outside integration')
        index = len(nodes); nodes.append([a, b, kind, []]); parent[3].append(index)
        # Zero-duration markers have no interior, but still require a valid
        # enclosing parent; they must not steal the next scope's ownership.
        if b > a:
            stack.append(index)
        if a < end and b > begin:
            counts[kind] = counts.get(kind, 0) + 1
    running = [(a, b) for p, t, _, a, b in intervals if p == pid and t == tid]
    total = sum(max(0, min(b, end) - max(a, begin)) for a, b in running)
    ticks = {}; edges = {}; wall = {}
    for a, b, kind, children in nodes:
        cursor = a
        for x, y in [(nodes[i][0], nodes[i][1]) for i in children] + [(b, b)]:
            lo, hi = max(cursor, begin), min(x, end)
            if hi > lo:
                ticks[kind] = ticks.get(kind, 0) + sum(max(0, min(v, hi) - max(u, lo)) for u, v in running)
                wall[kind] = wall.get(kind, 0) + hi - lo
                edges[kind] = edges.get(kind, 0) + 2
            cursor = y
    if sum(ticks.values()) != total:
        raise ValueError('exclusive scopes do not partition owned main time')
    names = {0: 'outside_recorded_scopes', 5: 'redraw_other', 13: 'integration_other', 15: 'painter_other'}
    regions = {}
    for kind in range(len(NAMES)):
        regions[names.get(kind, NAMES[kind])] = dict(
            calls=counts.get(kind, 0), exclusive_qpc_ticks=ticks.get(kind, 0),
            cpu_one_core_percent=100 * ticks.get(kind, 0) / (end - begin) if kind == 0 or counts.get(kind) else None,
            exclusive_wall_ms=1000 * wall.get(kind, 0) / frequency,
            boundary_uncertainty_percent_upper=edges.get(kind, 0) * bound_ns * frequency / 1e9 * 100 / (end - begin))
    return dict(regions=regions, main_qpc_ticks=total, main_cpu_one_core_percent=100 * total / (end - begin),
                qpc_begin=begin, qpc_end=end,
                scope='exclusive owned main-thread scheduled occupancy; includes ISR/DPC and observation; no deduction or pure driver attribution')


def analyze(capture):
    proof_path = capture / 'proof.json'
    proof = json.loads(proof_path.read_text(encoding='utf8'))
    if (proof.get('calibrated') is not True or proof.get('unchanged') is not True
            or proof.get('plan') != list(PLAN) or len(proof.get('products', [])) != 4
            or not all(proof.get('product_permission', {}).values()) or not proof.get('product_permission')):
        raise ValueError('complete unchanged ordinary calibrated capture required')
    require_references(proof['references'])
    paths = [proof_path, capture / 'products/trace.json']
    trace = json.loads(paths[-1].read_text(encoding='utf8')); intervals = scheduler_intervals(trace)
    values = []
    for i, base in enumerate(proof['products'], 1):
        if (base['checks'].get('strict_reference_calibration_passed') is not True
                or base['checks'].get('real_framework_spans_complete') is not True):
            raise ValueError('original attribution or strict calibration failed')
        directory = capture / ('products/product-%d' % i)
        paths.extend([directory / 'report.json', directory / 'gui/framework_trace.json'])
        report = json.loads(paths[-2].read_text(encoding='utf8'))
        raw = json.loads(paths[-1].read_text(encoding='utf8')); rows, spread = validate_spans(raw)
        if raw['schema'] not in (2,3):
            raise ValueError('schema 2/3 required; missing Painter scopes are unknown')
        mains = [t for t in base['threads'] if t['process_entry_start']]
        if len(mains) != 1:
            raise ValueError('unique process-entry main required')
        offset, clock = clock_mapping(report['clock_pairs'], trace['frequency'])
        window = base['costs']['voices_32']
        partition = exclusive_cpu(rows, report['pid'], mains[0]['tid'], window['qpc_begin'], window['qpc_end'],
                                  trace['frequency'], offset, intervals, clock['bound_ns'])
        if abs(partition['main_cpu_one_core_percent'] - mains[0]['cpu_one_core_percent']) > 1e-8:
            raise ValueError('main occupancy identity/window mismatch')
        values.append(dict(round=i, clock=clock, monotonic_disagreement_ns=spread, exclusive=partition,
                           painter_wall=painter_partition(rows)))
    return dict(schema=1, capture_identity=proof['identity'], rounds=values, calibrated=True,
                input_sha256={p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
                original_passed=proof['passed'], p2_a3_accepted=False,
                scope='offline analysis of all four fixed rounds; original checks and failures retained')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--capture', required=True); p.add_argument('--output', required=True)
    args = p.parse_args(); capture = (ROOT / args.capture).resolve(); output = (ROOT / args.output).resolve()
    if ROOT / 'artifacts' not in capture.parents or ROOT / 'artifacts' not in output.parents or output.exists():
        raise ValueError('owned existing capture and fresh artifact output required')
    result = analyze(capture)
    with output.open('x', encoding='utf8') as f:
        f.write(json.dumps(result, indent=2) + '\n')


if __name__ == '__main__':
    main()
