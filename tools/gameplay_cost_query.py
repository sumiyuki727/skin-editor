"""Strict schema 3 CostClosure-Query-v1 reader; never upgrades legacy evidence."""
import copy

import gameplay_cost_protocol as legacy
from gameplay_cost_protocol import WORK, METRICS, natural, nearest, validate_work

SCHEMA = 3
PLAN = 'CostClosure-Query-v1'
VALIDATOR = 'CostClosure-Query-reader-v1'
BATCHES = ('Q-SUMMARY-4096', 'Q-LEDGER-N', 'Q-PAGE-4096')
QUERY_PHASES = ('query_summary', 'query_ledger', 'query_page')
PHASES = legacy.PHASES + ('query_group_ready', *QUERY_PHASES, 'query_group_released', 'batch_summary')
PARAMETERS = dict(objects=10000, traces=['g0', 'g1', 'a1'], budgets=[1, 256], repetitions=2,
                  batches=list(BATCHES), batch_samples=16, query_loops=4096, page_capacity=64,
                  input_batch_objects=32, query_time_slots=96, query_storage_limit=1048576,
                  phases=list(PHASES)+['end'], max_line_bytes=33554432, phase_timeout_seconds=60,
                  child_timeout_seconds=300, process_sample_limit=120000, pc_batches=False)
ROW_FIELDS = set(('schema_version phase trace budget objects api_version profile_version status elapsed_ns '
                  'previous_emit_ns previous_gate_ns sample_storage_bytes fixture_bytes owner_alive events_checked '
                  'managed peaks work_counts candidate_work calibration target_arch target_os pointer_bits metrics '
                  'plan_id case_ordinal query_storage_bytes query').split())
END_FIELDS = set('schema_version plan_id case_ordinal trace budget objects phase status previous_emit_ns previous_gate_ns'.split())
FAMILIES = ('prepared', 'played', 'inspected', 'task_dropped', 'released')


def fields(value, expected, name):
    if not isinstance(value, dict) or set(value) != set(expected):
        raise ValueError('Missing/unknown '+name+' fields')


def case_ordinal(trace, budget, repetition):
    if trace not in PARAMETERS['traces'] or type(budget) is not int or budget not in (1, 256) or type(repetition) is not int or repetition not in (1, 2):
        raise ValueError('Unknown fixed case')
    return (PARAMETERS['traces'].index(trace)*2+(budget == 256))*2+repetition-1


def identity(row, trace, budget, repetition):
    ordinal = case_ordinal(trace, budget, repetition)
    for key in ('schema_version', 'case_ordinal', 'budget', 'objects'):
        natural(row.get(key), key)
    if (row.get('schema_version'), row.get('plan_id'), row.get('case_ordinal'), row.get('trace'), row.get('budget'), row.get('objects')) != (SCHEMA, PLAN, ordinal, trace, budget, 10000):
        raise ValueError('Unknown plan/version or case identity')
    return 10000000+ordinal*1000


def times(value):
    fields(value, ('n', 'sum_ns', 'p50_ns', 'p95_ns', 'p99_ns', 'max_ns', 'raw_ns'), 'batch timing')
    raw = value['raw_ns']
    if type(value['n']) is not int or value['n'] != 16 or not isinstance(raw, list) or len(raw) != 16:
        raise ValueError('Exactly sixteen retained batch samples required')
    for sample in raw:
        natural(sample, 'raw batch ns')
    if natural(value['sum_ns'], 'batch sum') != sum(raw):
        raise ValueError('Batch sum mismatch')
    for percent, key in ((50, 'p50_ns'), (95, 'p95_ns'), (99, 'p99_ns'), (100, 'max_ns')):
        if natural(value[key], key) != nearest(raw, percent):
            raise ValueError('Batch statistic mismatch')


def frame(value, row, base):
    fields(value, 'run transport input revision at publication objects hits misses skipped start evidence profile_version'.split(), 'frame')
    expected = dict(run=base+2, transport=1, input=3 if row['trace']=='a1' else 1, revision=730,
                    at=9999300000, objects=10000, hits=0 if row['trace']=='g0' else 10000,
                    misses=10000 if row['trace']=='g0' else 0, skipped=0, start=-200001,
                    evidence='SyntheticTest', profile_version=1)
    for key, want in expected.items():
        if type(value[key]) is not type(want) or value[key] != want:
            raise ValueError('Query golden frame changed: '+key)
    if natural(value['publication'], 'publication') == 0:
        raise ValueError('Missing publication identity')
    for key in ('run', 'transport', 'input', 'revision', 'hits', 'misses', 'skipped'):
        if value[key] != row['managed'][key]:
            raise ValueError('Frame/current source mismatch')


def delta(before, after):
    validate_work(before); validate_work(after)
    result = dict(valid=True)
    for key in WORK:
        result[key] = natural(after[key]-before[key], 'work delta '+key)
    return result


def expected_delta(kind):
    result = dict(valid=True, **dict.fromkeys(WORK, 0))
    result[('summaries', 'ledger_objects', 'frame_objects')[kind]] = (4096, 10000, 4096*64)[kind]
    return result


def validate_row(row, phase, trace, budget, repetition=1):
    fields(row, ROW_FIELDS, 'query protocol')
    base = identity(row, trace, budget, repetition)
    if row['target_arch'] not in ('x86_64','x86','aarch64') or row['target_os']!='windows':
        raise ValueError('Unknown bounded observation target')
    if phase not in PHASES or row['phase'] != phase:
        raise ValueError('Changed phase order')
    if not 0 < natural(row['query_storage_bytes'], 'query storage') <= 1048576:
        raise ValueError('Query container admission failed')
    old = dict(row, schema_version=2)
    if phase in legacy.PHASES:
        if row['query'] is not None or (phase != 'calibrated' and row['calibration'] is not None):
            raise ValueError('Unexpected query/calibration receipt')
        legacy.validate_row(old, phase, trace, budget)
        if phase=='calibrated':
            fields(row['calibration'],('valid','observed_resolution_ns','empty_span_ns','small_reference_ns','large_reference_ns',
                                      'small_work','large_work','cpu_reference_ns','cpu_reference_cycles','cpu_reference_objects'),'calibration')
        if row['managed'] is not None and row['managed']['run'] != base+1:
            raise ValueError('Baseline run identity changed')
        return row
    alive = phase in ('query_group_ready', *QUERY_PHASES)
    alias = 'played' if alive else 'released'
    old['phase'] = alias
    legacy.validate_row(old, alias, trace, budget, operations=False)
    if row['calibration'] is not None or any(m['calls'] for m in row['metrics']):
        raise ValueError('Query group cannot borrow baseline clock/sample slots')
    q = row['query']
    if phase == 'query_group_ready':
        fields(q, ('kind', 'frame', 'prepared_work', 'calls'), 'setup')
        if q['kind'] != 'setup': raise ValueError('Wrong setup role')
        frame(q['frame'], row, base)
        validate_work(q['prepared_work'])
        if q['prepared_work']['record_reads'] != 10000 or q['prepared_work']['state_initializations'] != 10000:
            raise ValueError('Query setup missing prepared work')
        fields(q['calls'], METRICS, 'unmeasured setup calls')
        required = dict(create_request=1, prepare_step=None, activate=1, task_drop=1, seal=None, advance=None,
                        acknowledge=10000 if trace=='g0' else 20000)
        if trace=='g1': required['submit']=None
        if trace=='a1': required.update(automatic_prepare=None, freeze=None)
        for key, count in q['calls'].items():
            natural(count, key, 640256*10001)
            want = required.get(key, 0)
            if (want is None and count == 0) or (want is not None and count != want):
                raise ValueError('Wrong setup operation count: '+key)
    elif phase in QUERY_PHASES:
        kind = QUERY_PHASES.index(phase)
        fields(q, ('kind', 'id', 'frame', 'state_unchanged', 'resource_unchanged', 'reference_scope', 'guard_summaries', 'samples', 'timing', 'reference'), 'query batch')
        if (q['kind'], q['id'], q['reference_scope']) != ('batch', BATCHES[kind], 'outer_loop_only') or q['state_unchanged'] is not True or q['resource_unchanged'] is not True or type(q['guard_summaries']) is not int or q['guard_summaries'] != 34:
            raise ValueError('Query guard/batch changed')
        frame(q['frame'], row, base)
        times(q['timing']); times(q['reference'])
        if q['timing']['sum_ns']+q['reference']['sum_ns'] > row['elapsed_ns']:
            raise ValueError('Batch actions exceed enclosing interval')
        if not isinstance(q['samples'], list) or len(q['samples']) != 16:
            raise ValueError('Missing per-sample work receipts')
        for i, sample in enumerate(q['samples']):
            fields(sample, ('ordinal', 'ns', 'checked', 'before', 'after', 'delta'), 'query sample')
            if natural(sample['ordinal'], 'sample ordinal') != i or sample['checked'] is not True or natural(sample['ns'], 'sample time') != q['timing']['raw_ns'][i]:
                raise ValueError('Reordered/unchecked sample')
            validate_work(sample['delta'])
            if delta(sample['before'], sample['after']) != sample['delta'] or sample['delta'] != expected_delta(kind):
                raise ValueError('Query work difference changed')
            if i:
                gap=delta(q['samples'][i-1]['after'], sample['before'])
                if gap != work(summaries=2):
                    raise ValueError('Unaccounted between-sample work')
    elif phase == 'query_group_released':
        if q != dict(kind='released', owners_dropped=True) or q.get('owners_dropped') is not True:
            raise ValueError('Query owner release missing')
    else:
        fields(q, ('kind', 'scope', 'coverage', 'product_verdict', 'batches'), 'batch summary')
        if (q['kind'], q['scope'], q['coverage'], q['product_verdict']) != ('summary', 'QueryOnly', 'NeedsEvidence', 'NotEvaluated') or row['elapsed_ns'] != 0:
            raise ValueError('Query-only summary cannot upgrade conclusions')
        if not isinstance(q['batches'], list) or len(q['batches']) != 3:
            raise ValueError('Missing query summary')
        for i, batch in enumerate(q['batches']):
            fields(batch, ('id','timing','reference'), 'summary batch')
            if batch['id'] != BATCHES[i]: raise ValueError('Summary batch order')
            times(batch['timing']); times(batch['reference'])
    return row


def validate_end(row, trace, budget, repetition=1):
    fields(row, END_FIELDS, 'end')
    identity(row, trace, budget, repetition)
    if row['phase']!='end' or row['status']!='Valid': raise ValueError('Missing normal end')
    natural(row['previous_emit_ns'],'emit'); natural(row['previous_gate_ns'],'gate')
    return row


def validate_lifecycle(rows):
    if [r['phase'] for r in rows] != list(PHASES): raise ValueError('Incomplete query flow')
    legacy.validate_lifecycle(rows[:9])
    first=rows[0]
    for r in rows:
        if any(r[k]!=first[k] for k in ('target_arch','target_os','pointer_bits','sample_storage_bytes','query_storage_bytes','case_ordinal')):
            raise ValueError('Changing target/storage/case')
    for a,b in zip(rows,rows[1:]):
        if any(b['peaks'][k]<v for k,v in a['peaks'].items()): raise ValueError('Peak regression')
    setup=rows[9]
    if setup['fixture_bytes'] != rows[2]['fixture_bytes']: raise ValueError('Query fixture capacity drift')
    for k in ('source_reserved','runtime_reserved','source_payload','runtime_payload','source_allocations','runtime_allocations'):
        if setup['managed'][k] != rows[3]['managed'][k]: raise ValueError('Query resource baseline drift')
    for i,r in enumerate(rows[10:13]):
        q=r['query']
        if r['managed'] != setup['managed'] or r['fixture_bytes']!=setup['fixture_bytes'] or q['frame'] != setup['query']['frame']:
            raise ValueError('Query frame/resources changed')
        prior=rows[9+i]['work_counts']
        if delta(prior,q['samples'][0]['before']) != work(summaries=1) or delta(q['samples'][-1]['after'],r['work_counts']) != work(summaries=3):
            raise ValueError('Unaccounted query diagnostic work')
        expected={k:q[k] for k in ('id','timing','reference')}
        if rows[14]['query']['batches'][i] != expected: raise ValueError('Summary/raw batch mismatch')


def work(**values):
    return dict(dict.fromkeys(WORK,0), valid=True, **values)


def summarize(row, uncertainty):
    result=legacy.summarize(row,uncertainty)
    q=result['query']
    if row['phase'] in QUERY_PHASES:
        result['query']=copy.deepcopy(q)
        result['query']['timing_status']='Valid' if min(q['timing']['raw_ns'])>=20*uncertainty else 'Unknown'
        result['query']['units']='one fixed batch; n=16; p95/p99=max; no API-tail inference or subtraction'
        result['query']['cpu_status']='Unknown'
        result['query']['stage_peak_status']='Unknown'
    return result


def coverage(rows, uncertainty):
    batches={r['query']['id']: ('Valid' if min(r['query']['timing']['raw_ns'])>=20*uncertainty else 'Unknown') for r in rows if r['phase'] in QUERY_PHASES}
    families={r['phase']:dict(n=1,total_ns=r['elapsed_ns'],timing_status='Valid' if r['elapsed_ns']>=100*uncertainty else 'Unknown') for r in rows if r['phase'] in FAMILIES}
    mapping=[]
    for row in rows[:9]:
        phase=row['phase']
        for m in row['metrics']:
            name=m['name']; evidence=phase if phase in FAMILIES else None
            if phase=='queried': evidence={'summary':BATCHES[0],'ledger':BATCHES[1],'page':BATCHES[2],'query_batch':BATCHES[2]}.get(name)
            if phase=='cancelled': evidence='PC-CANCEL-32' if name=='cancel' else 'PC-PREPARE-32'
            if phase=='task_dropped' and name=='task_drop': evidence='PC-DROP-32'
            if phase=='calibrated': evidence='diagnostic-reference'
            state='NotApplicable' if not m['calls'] else ('NotRun' if evidence and evidence.startswith('PC-') else families.get(evidence,{}).get('timing_status', batches.get(evidence,'Reference')))
            mapping.append(dict(phase=phase,operation=name,calls=m['calls'],evidence=evidence if m['calls'] else None,status=state,
                                role='caller' if name in ('fixture','fixture_drop') else 'diagnostic' if name=='empty' else 'core-or-explicit-family'))
    return dict(status='NeedsEvidence',scope='QueryOnly',missing_batches=['PC-PREPARE-32','PC-CANCEL-32','PC-DROP-32'],batches=batches,families=families,mapping=mapping)
