"""Versioned SyntheticTest cost protocol, bounded validation and architecture rules."""
from pathlib import Path
import re

SCHEMA = 2
PHASES = ('ready', 'calibrated', 'prepared', 'played', 'queried', 'inspected', 'cancelled', 'task_dropped', 'released')
METRICS = ('empty', 'fixture', 'create_request', 'prepare_step', 'activate', 'automatic_prepare', 'submit', 'seal',
           'advance', 'acknowledge', 'summary', 'page', 'ledger', 'query_batch', 'freeze', 'inspection_request',
           'inspection_step', 'inspection_activate', 'cancel_candidate_step', 'cancel', 'task_drop', 'owner_drop',
           'fixture_drop', 'request_cancel_drop_batch')
WORK = ('record_reads', 'record_copies', 'index_initializations', 'id_comparisons', 'index_moves', 'state_initializations',
        'search_objects', 'inspection_objects', 'automatic_objects', 'publications', 'summaries', 'frame_objects',
        'ledger_objects', 'management_steps', 'allocation_attempts', 'allocation_bytes')


def natural(value, name, maximum=2**64-1):
    if type(value) is not int or not 0 <= value <= maximum:
        raise ValueError('Invalid integer: ' + name)
    return value


def nearest(raw, percent):
    if not raw:
        return None
    ordered = sorted(raw)
    return ordered[(len(raw)*percent+99)//100-1]


def metric_limits(n):
    levels = (max(n, 2)-1).bit_length()
    return dict(zip(METRICS, (20000, 4, 4, n*(2*levels+16)+128, 4, n*4+1024, (n+31)//32+8,
                            (n+31)//32+32, n*32+256, 2*n+16, 1024, 2*((n+63)//64)+16, 2*n+16, 4,
                            n*32+256, 4, 2*n+128, 4, 2*n+128, 4, 8, 4, 4, 4)))


def validate_work(work):
    if not isinstance(work, dict) or work.get('valid') is not True or set(work) != set(WORK) | {'valid'}:
        raise ValueError('Invalid or missing raw work receipt')
    for name in WORK:
        natural(work[name], name)


def validate_cancel_checkpoint(candidate,n):
    validate_work(candidate)
    # Run initialization writes N states; the private inspection then writes
    # another state for each checked object. This is the existing K1-6B count.
    if (candidate['allocation_attempts'],candidate['state_initializations'],candidate['inspection_objects'])!=(3,n+n//2,n//2):
        raise ValueError('Cancellation checkpoint changed')


def validate_row(row, phase, trace, budget, n=10000):
    if not isinstance(row,dict):
        raise ValueError('Protocol row must be an object')
    for field in ('schema_version', 'budget', 'objects', 'api_version', 'profile_version', 'pointer_bits'):
        natural(row.get(field), field)
    if row['pointer_bits'] not in (32,64) or not isinstance(row.get('target_arch'),str) or not row['target_arch'] or not isinstance(row.get('target_os'),str) or not row['target_os']:
        raise ValueError('Missing child target identity')
    if (row.get('schema_version'), row.get('phase'), row.get('trace'), row.get('budget'), row.get('objects'),
        row.get('api_version'), row.get('profile_version')) != (SCHEMA, phase, trace, budget, n, 7, 1):
        raise ValueError('Protocol identity/version/order changed')
    if row.get('status') != 'Valid':
        raise ValueError('Child declared invalid measurement')
    for name in ('elapsed_ns', 'previous_emit_ns', 'previous_gate_ns', 'sample_storage_bytes', 'fixture_bytes', 'events_checked'):
        natural(row.get(name), name)
    metrics = row.get('metrics')
    if not isinstance(metrics, list) or not all(isinstance(m,dict) for m in metrics) or [m.get('name') for m in metrics] != list(METRICS):
        raise ValueError('Missing/duplicated/reordered metrics')
    limits = metric_limits(n)
    for metric in metrics:
        if set(metric)!={'name','capacity','calls','sum_ns','p50_ns','p95_ns','p99_ns','max_ns','raw_ns'}:
            raise ValueError('Changed raw metric fields')
        name = metric['name']
        natural(metric.get('capacity'),'capacity')
        natural(metric.get('calls'),'calls')
        if metric.get('capacity') != limits[name]:
            raise ValueError('Sample capacity changed')
        raw = metric.get('raw_ns')
        if not isinstance(raw, list) or len(raw) > limits[name] or metric.get('calls') != len(raw):
            raise ValueError('Missing/truncated/overfull samples')
        for value in raw:
            natural(value, 'raw_ns')
        if natural(metric.get('sum_ns'), 'sum_ns') != sum(raw):
            raise ValueError('Raw timing sum mismatch')
        for percent, field in ((50, 'p50_ns'), (95, 'p95_ns'), (99, 'p99_ns'), (100, 'max_ns')):
            if metric.get(field) is not None:
                natural(metric[field],field)
            if field not in metric or metric[field] != nearest(raw, percent):
                raise ValueError('Nearest-rank sample statistic mismatch')
    if sum(m['sum_ns'] for m in metrics) > row['elapsed_ns']:
        raise ValueError('Call times exceed their enclosing stage')
    if row['sample_storage_bytes'] != sum(limits.values())*8:
        # Global/Vec capacity may change with a toolchain; reject drift, do not
        # silently reuse this observer memory baseline.
        raise ValueError('Diagnostic sample storage changed')
    peaks=row.get('peaks')
    peak_keys={'inputs','events','source_reserved','runtime_reserved','source_payload','runtime_payload'}
    if not isinstance(peaks,dict) or set(peaks)!=peak_keys:
        raise ValueError('Missing/changed peak observations')
    for key in peak_keys:
        natural(peaks[key],key)
    if peaks['inputs']>64 or peaks['events']>64:
        raise ValueError('Queue capacity exceeded')
    for prefix in ('source','runtime'):
        if not peaks[prefix+'_payload']<=peaks[prefix+'_reserved']<=8*1024*1024:
            raise ValueError('Peak managed budget exceeded')
    alive = phase not in ('ready', 'calibrated', 'released')
    if row.get('owner_alive') is not alive:
        raise ValueError('Owner lifetime changed')
    managed = row.get('managed')
    if not alive:
        if managed is not None or row.get('work_counts') is not None:
            raise ValueError('Cannot observe a dropped coordinator as a live zero ledger')
    else:
        managed_keys={'source_reserved','runtime_reserved','source_payload','runtime_payload','source_allocations','runtime_allocations',
                      'hits','misses','skipped','pending_inputs','pending_events','run','transport','input','revision','source_capacity'}
        if not isinstance(managed, dict) or set(managed)!=managed_keys:
            raise ValueError('Missing managed resource snapshot')
        for key, value in managed.items():
            natural(value, key, 2**128-1)
        if any(managed[key]==0 for key in ('run','transport','input')):
            raise ValueError('Invalid active source identity')
        for prefix in ('source', 'runtime'):
            if not managed[prefix+'_payload'] <= managed[prefix+'_reserved'] <= 8*1024*1024:
                raise ValueError('Managed budget exceeded')
            if managed[prefix+'_reserved']>peaks[prefix+'_reserved'] or managed[prefix+'_payload']>peaks[prefix+'_payload']:
                raise ValueError('Peak snapshot missed a current managed owner')
        if managed['source_capacity'] != max(n, 4) or managed['source_allocations'] != 3:
            raise ValueError('Source capacity/ownership changed')
        if managed['pending_inputs'] != 0 or managed['pending_events'] != 0:
            raise ValueError('Stage left unconsumed input/events')
        expected = (0, 0, 0)
        if phase in ('played', 'queried'):
            expected = (0, n, 0) if trace == 'g0' else (n, 0, 0)
        if phase in ('inspected', 'cancelled', 'task_dropped'):
            expected = (0, 0, n//2)
        if (managed['hits'], managed['misses'], managed['skipped']) != expected:
            raise ValueError('Golden ledger changed')
        validate_work(row.get('work_counts'))
    if phase in ('played', 'queried', 'inspected', 'cancelled', 'task_dropped','released'):
        if row['events_checked'] != n*(1 if trace == 'g0' else 2):
            raise ValueError('Incomplete golden event stream')
    if phase == 'prepared' and row['work_counts']['state_initializations'] != n:
        raise ValueError('Incomplete stepped initialization')
    if phase == 'cancelled':
        candidate = row.get('candidate_work')
        validate_cancel_checkpoint(candidate,n)
    elif row.get('candidate_work') is not None:
        raise ValueError('Unexpected candidate ownership')
    if phase == 'queried':
        by_name = {m['name']: m for m in metrics}
        if (by_name['summary']['calls'], by_name['page']['calls'], by_name['ledger']['calls'], by_name['query_batch']['calls']) != (256, (n+63)//64, n, 1):
            raise ValueError('Query workload changed')
    if phase == 'calibrated':
        validate_calibration(row)
    validate_operations(row,phase,trace)
    if (phase in ('ready','calibrated','released')) != (row['fixture_bytes']==0):
        raise ValueError('Caller fixture lifetime changed')
    return row


def validate_operations(row,phase,trace):
    calls={m['name']:m['calls'] for m in row['metrics']}
    required={
        'ready':{},'calibrated':{'empty':20000},
        'prepared':{'fixture':1,'create_request':1,'prepare_step':None,'activate':1,'task_drop':1},
        'played':{'seal':None,'advance':None,'acknowledge':row['objects']*(1 if trace=='g0' else 2)},
        'queried':{'summary':256,'page':(row['objects']+63)//64,'ledger':row['objects'],'query_batch':1},
        'inspected':{'freeze':None,'inspection_request':1,'inspection_step':None,'inspection_activate':1,'task_drop':1},
        'cancelled':{'inspection_request':1,'cancel_candidate_step':None,'cancel':1},
        'task_dropped':{'task_drop':1,'request_cancel_drop_batch':1},
        'released':{'owner_drop':1,'fixture_drop':1},
    }[phase].copy()
    if phase=='played' and trace=='g1':required['submit']=None
    if phase=='played' and trace=='a1':required.update(automatic_prepare=None,freeze=None)
    for name,count in calls.items():
        expected=required.get(name,0)
        if (expected is None and count==0) or (expected is not None and count!=expected):
            raise ValueError('Missing/changed stage operation: '+name)


def validate_end(row):
    if not isinstance(row,dict) or set(row)!={'schema_version','phase','status','previous_emit_ns','previous_gate_ns'}:
        raise ValueError('Missing/changed end receipt fields')
    for name in ('schema_version','previous_emit_ns','previous_gate_ns'):natural(row[name],name)
    if row['schema_version']!=SCHEMA or row['phase']!='end' or row['status']!='Valid':
        raise ValueError('Invalid versioned end receipt')
    return row


def validate_process_point(point,previous):
    keys={'user_ns','kernel_ns','working_set_bytes','private_commit_bytes','os_peak_working_set_bytes','os_peak_commit_bytes'}
    if not isinstance(point,dict) or set(point)!=keys:
        raise ValueError('Missing/changed process counter fields')
    for name,value in point.items():natural(value,name)
    for current,peak in (('working_set_bytes','os_peak_working_set_bytes'),('private_commit_bytes','os_peak_commit_bytes')):
        if not 0<point[current]<=point[peak]:
            raise ValueError('Missing/inconsistent process memory counters')
    for name in ('user_ns','kernel_ns','os_peak_working_set_bytes','os_peak_commit_bytes'):
        if previous is not None and point[name]<previous[name]:
            raise ValueError('Process counter regressed: '+name)
    return point


def validate_lifecycle(phases):
    if [p['phase'] for p in phases]!=list(PHASES):
        raise ValueError('Incomplete phase set')
    first=phases[0]
    for phase in phases:
        if any(phase[key]!=first[key] for key in ('target_arch','target_os','pointer_bits','sample_storage_bytes')):
            raise ValueError('Child target/sample identity changed between phases')
    for before,after in zip(phases,phases[1:]):
        if 'peaks' in before and 'peaks' in after and any(after['peaks'][key]<value for key,value in before['peaks'].items()):
            raise ValueError('Cumulative managed/queue observations regressed')
    inspected,cancelled,dropped=(phases[i]['managed'] for i in (5,6,7))
    resource_keys=('source_reserved','runtime_reserved','source_payload','runtime_payload','source_allocations','runtime_allocations')
    if any(inspected[key]!=dropped[key] for key in resource_keys):
        raise ValueError('Cancelled task did not return to the active inspection baseline')
    for key in ('source_reserved','source_payload','source_allocations','run','transport','input','revision'):
        if cancelled[key]!=inspected[key] or dropped[key]!=inspected[key]:
            raise ValueError('Cancellation changed the active source')
    if (cancelled['runtime_reserved']<=inspected['runtime_reserved'] or cancelled['runtime_payload']<=inspected['runtime_payload']
            or cancelled['runtime_allocations']!=inspected['runtime_allocations']+3):
        raise ValueError('Cancelled task lost its held allocations before Drop')
    if any(p['fixture_bytes']!=phases[2]['fixture_bytes'] for p in phases[2:8]):
        raise ValueError('Caller fixture changed before explicit Drop')


def validate_calibration(row):
    c = row.get('calibration')
    if not isinstance(c, dict) or c.get('valid') is not True:
        raise ValueError('Clock calibration failed or missing')
    for key in ('observed_resolution_ns', 'empty_span_ns', 'small_reference_ns', 'large_reference_ns',
                'small_work', 'large_work', 'cpu_reference_ns', 'cpu_reference_cycles', 'cpu_reference_objects'):
        natural(c.get(key), key)
    empty = row['metrics'][0]
    proxy=max(c['observed_resolution_ns'],empty['p95_ns'] or 0,1)
    if not (c['observed_resolution_ns'] > 0 and c['small_reference_ns'] > 100*proxy
            and c['large_reference_ns'] > 100*proxy and c['large_work'] > c['small_work'] > 0
            and c['cpu_reference_ns'] >= 2_000_000_000 and 0 < c['cpu_reference_cycles'] <= 100000
            and c['cpu_reference_objects'] == 256):
        raise ValueError('Representative calibration/reference not exercised')
    if empty['calls'] != 20000 or c['empty_span_ns'] < empty['sum_ns']:
        raise ValueError('Missing bounded empty-call reference')
    if sum(c[k] for k in ('empty_span_ns','small_reference_ns','large_reference_ns','cpu_reference_ns'))>row['elapsed_ns']:
        raise ValueError('Reference times exceed the enclosing calibration stage')


def summarize(row, uncertainty):
    """Keep raw quantiles, explicitly distinguish unresolved calls and batches."""
    result = {k: v for k, v in row.items() if k != 'metrics'}
    result['stage_timing_status'] = 'Valid' if row['elapsed_ns'] >= 100*uncertainty else 'Unknown'
    result['timing_status_scope']='empirical duration resolution only; not statistical or product acceptance'
    result['metrics'] = []
    for metric in row['metrics']:
        copy = {k: v for k, v in metric.items() if k != 'raw_ns'}
        copy['timing_status'] = ('NotRun' if not metric['calls'] else
                                 'Valid' if metric['p50_ns'] >= 20*uncertainty else 'Unknown')
        copy['units'] = 'one batch (4096 pages)' if metric['name'] == 'query_batch' else (
            'one batch (256 request/cancel/drop cycles)' if metric['name'] == 'request_cancel_drop_batch' else 'one API call or explicitly named operation')
        result['metrics'].append(copy)
    return result


def tool_graph_errors(metadata, root):
    root = Path(root).resolve()
    paths = {'skin-editor-gameplay-cost': root/'tools/gameplay-cost', 'skin-editor-gameplay': root/'crates/gameplay'}
    packages = metadata.get('packages', [])
    if len(packages) != 2 or {p['name'] for p in packages} != set(paths):
        return ['diagnostic graph must be tool -> gameplay only']
    errors = []
    if Path(metadata['workspace_root']).resolve() != paths['skin-editor-gameplay-cost']:
        errors.append('diagnostic workspace changed')
    for package in packages:
        name = package['name']
        if (Path(package['manifest_path']).resolve() != paths[name]/'Cargo.toml' or package.get('source') is not None
                or package.get('features') or package.get('publish') != [] or package.get('edition') != '2024'
                or package.get('rust_version') != '1.95'):
            errors.append('unreviewed diagnostic package configuration')
        allowed = [] if name == 'skin-editor-gameplay' else ['skin-editor-gameplay']
        dependencies = package.get('dependencies', [])
        if [d['name'] for d in dependencies] != allowed:
            errors.append('unreviewed diagnostic dependency')
        for d in dependencies:
            if (d['name'] not in paths or Path(d.get('path', '')).resolve() != paths[d['name']]
                    or d.get('source') is not None or d.get('kind') is not None or d.get('target') is not None
                    or d.get('optional') or d.get('rename') or d.get('features')):
                errors.append('diagnostic dependencies must be direct, unconditional local paths')
        if any('custom-build' in t['kind'] or 'proc-macro' in t['kind'] for t in package.get('targets', [])):
            errors.append('diagnostic cannot introduce build scripts or macros')
    tool = next(p for p in packages if p['name'] == 'skin-editor-gameplay-cost')
    targets=tool.get('targets',[])
    if (len(targets)!=1 or targets[0].get('kind')!=['bin']
            or targets[0].get('name')!='skin-editor-gameplay-cost'
            or Path(targets[0].get('src_path','')).resolve()!=paths['skin-editor-gameplay-cost']/'src/main.rs'):
        errors.append('diagnostic must expose only its reviewed binary target')
    if metadata.get('workspace_members') != [tool['id']]:
        errors.append('diagnostic must be its own single-member workspace')
    return errors


def tool_source_errors(root):
    base = Path(root)/'tools/gameplay-cost/src'
    expected = {'main.rs', 'driver.rs', 'meter.rs'}
    if {p.relative_to(base).as_posix() for p in base.rglob('*.rs')} != expected:
        return ['unreviewed diagnostic source/module set']
    if '#![forbid(unsafe_code)]' not in (base/'main.rs').read_text(encoding='utf-8'):
        return ['diagnostic must forbid unsafe code']
    forbidden = r'\b(?:skin_editor_(?:domain|application|preview|rendering|audio|infrastructure|ui|desktop)|egui|wgpu|cpal)\b|\bstd\s*::\s*(?:fs|net|thread|os|ffi)\b|\b(?:include|include_str|include_bytes)\s*!|#\s*\[\s*path\s*='
    return [str(p.relative_to(root))+': forbidden diagnostic dependency/source access'
            for p in base.rglob('*.rs') if re.search(forbidden, p.read_text(encoding='utf-8'))]
