"""Strict fixed PC128 reader. Historical Query/PC32 readers remain unchanged."""
import copy
import hashlib
import json
import sys

import gameplay_cost_query as q
import gameplay_cost_protocol as legacy
from gameplay_cost_query import fields, natural, times, work, delta, case_ordinal, validate_durations

SCHEMA=3
PLAN='CostClosure-PC128-v1'
VALIDATOR='CostClosure-PC128-reader-v1'
SCOPE='Complete'
PC_BATCHES=('PC-PREPARE-128','PC-CANCEL-128','PC-DROP-128')
PC_PHASES=('pc_prepared','pc_cancelled_held','pc_dropped')
BATCHES=q.BATCHES+PC_BATCHES
PHASES=q.PHASES[:14]+('pc_group_ready',)+PC_PHASES*16+('pc_group_released','batch_summary')
PARAMETERS=dict(copy.deepcopy(q.PARAMETERS), batches=list(BATCHES),phases=list(PHASES)+['end'],
                pc_batches=True,pc_owners=128,pc_work_budget=1,pc_time_slots=96,
                closure_time_slots=192,closure_storage_limit=1048576,aggregate_reserved_limit=268435456,failure_supplement_limit=524288)
ROW_FIELDS=q.ROW_FIELDS|{'closure_storage_bytes','pc'}
RESOURCE=('source_reserved','runtime_reserved','source_payload','runtime_payload','source_allocations','runtime_allocations')
OWNER_FIELDS='i frame resources active_work create_work play_work inspection_work calls peaks task_held eligible steps task_work cancel_success key'.split()

def as_query(row):
    value={k:v for k,v in row.items() if k not in ('pc','closure_storage_bytes')}
    value['plan_id']=q.PLAN
    return value

def identity(row,trace,budget,repetition):
    fields(row,ROW_FIELDS,'complete protocol')
    if row['plan_id']!=PLAN:raise ValueError('Wrong complete plan')
    base=q.identity(as_query(row),trace,budget,repetition)
    if not 0<natural(row['query_storage_bytes'],'query storage')<=natural(row['closure_storage_bytes'],'closure storage',1048576):
        raise ValueError('Complete container admission')
    return base

def admission(value,row):
    fields(value,'source_bytes run_bytes task_bytes pool_bytes coordinator_bytes pool_limit aggregate_limit aggregate_required container_bytes'.split(),'PC admission')
    for k,v in value.items():
        if natural(v,k)==0:raise ValueError('Zero admission component')
    b=value['pool_bytes']+value['coordinator_bytes'];r=value['run_bytes'];s=value['source_bytes'];t=value['task_bytes']
    required=128*(s+b+t+2*r)
    if (value['pool_limit']!=8*1024*1024 or value['aggregate_limit']!=256*1024*1024
            or value['container_bytes']!=row['closure_storage_bytes'] or value['aggregate_required']!=required
            or required>value['aggregate_limit'] or max(s,b+t+2*r,r)>value['pool_limit']):
        raise ValueError('PC per-pool/aggregate proof')
    return s,b+r,b+t+2*r

def resources(value):
    fields(value,RESOURCE,'resource snapshot')
    for k,v in value.items():natural(v,k)
    for p in ('source','runtime'):
        if not value[p+'_payload']<=value[p+'_reserved']<=8*1024*1024:raise ValueError('PC pool/payload bound')
    return value

def source(value,expected):
    fields(value,('run','transport','input','revision'),'source')
    if any(type(value[k]) is not int or value[k]!=v for k,v in expected.items()):raise ValueError('PC source recipe')

def owner(value,row,base,kind,r,a):
    fields(value,OWNER_FIELDS,'PC owner')
    i=natural(value['i'],'owner slot',127)
    expected=dict(run=base+100+i,transport=2,input=4 if row['trace']=='a1' else 2,revision=730)
    frame=value['frame']
    fields(frame,'source at publication objects hits misses skipped statistics_at driver status pending_inputs pending_events evidence profile_version'.split(),'PC frame')
    source(frame['source'],expected)
    want=dict(at=5000000000,objects=10000,hits=0,misses=0,skipped=5000,statistics_at=5000000000,
              driver='Automatic' if row['trace']=='a1' else 'Manual',status='Frozen',
              pending_inputs=0,pending_events=0,evidence='SyntheticTest',profile_version=1)
    if any(type(frame[k]) is not type(v) or frame[k]!=v for k,v in want.items()) or not natural(frame['publication'],'publication'):
        raise ValueError('PC frozen baseline')
    u=resources(value['resources'])
    held=kind in ('prepared','cancelled-held')
    if u['source_reserved']!=a[0] or u['runtime_reserved']!=a[2 if held else 1] or u['source_allocations']!=3 or u['runtime_allocations']!=(6 if held else 3):
        raise ValueError('PC source sharing or task lifetime')
    if value['task_held'] is not held or value['eligible'] is not (kind=='prepared'):
        raise ValueError('PC eligibility/ownership confusion')
    for name in ('active_work','create_work','play_work','inspection_work'):legacy.validate_work(value[name])
    if value['create_work']['record_reads']!=10000 or value['create_work']['state_initializations']!=10000 or value['inspection_work']['inspection_objects']!=10000:
        raise ValueError('Missing real PC setup work')
    calls=value['calls'];fields(calls,legacy.METRICS,'PC setup calls')
    required=dict(create_request=1,prepare_step=None,activate=1,task_drop=2,seal=None,advance=None,
                  acknowledge=10000 if row['trace']=='g0' else 20000,freeze=None,
                  inspection_request=1,inspection_step=None,inspection_activate=1)
    if row['trace']=='g1':required['submit']=None
    if row['trace']=='a1':required['automatic_prepare']=None
    for key,count in calls.items():
        natural(count,key,640256*10001);want=required.get(key,0)
        if (want is None and count==0) or (want is not None and count!=want):raise ValueError('PC setup operation '+key)
    peaks=value['peaks'];fields(peaks,('inputs','events','source_reserved','runtime_reserved','source_payload','runtime_payload'),'PC setup peaks')
    for key,n in peaks.items():natural(n,key)
    if peaks['inputs']>64 or peaks['events']>64:raise ValueError('PC setup input/event bound')
    for p in ('source','runtime'):
        if not peaks[p+'_payload']<=peaks[p+'_reserved']<=8*1024*1024:raise ValueError('PC setup pool peak')
    if kind=='ready':
        for key in ('source_reserved','runtime_reserved','source_payload','runtime_payload'):
            if u[key]>peaks[key]:raise ValueError('PC setup peak omitted a live baseline owner')
        if any(value[k] is not None for k in ('task_work','cancel_success','key')) or natural(value['steps'],'steps')!=0:raise ValueError('Unstarted PC baseline')
        return
    legacy.validate_cancel_checkpoint(value['task_work'],10000)
    if not 0<natural(value['steps'],'prepare calls')<=640256:raise ValueError('PC step bound')
    if value['cancel_success'] is not (None if kind=='prepared' else True):raise ValueError('PC actual cancel result')
    key=value['key']
    fields(key,'handle owner base target operation at driver full_eq_checked fresh'.split(),'PC key projection')
    if (not isinstance(key['handle'],list) or len(key['handle'])!=3 or any(type(x) is not int for x in key['handle'])
            or key['handle']!=[row['case_ordinal'],r,i] or natural(key['owner'],'coordinator',2**128-1)!=expected['run']
            or key['operation']!='Inspect' or type(key['at']) is not int or key['at']!=0
            or key['driver']!=frame['driver'] or key['full_eq_checked'] is not True or key['fresh'] is not True):
        raise ValueError('PC handle/key equality receipt')
    source(key['base'],expected)
    source(key['target'],dict(expected,transport=3,input=expected['input']+1))

def validate_row(row,phase,trace,budget,repetition=1):
    return _validate_row(row,phase,trace,budget,repetition,'windows')

def _validate_row(row,phase,trace,budget,repetition,target_os):
    base=identity(row,trace,budget,repetition)
    if phase not in PHASES or row['phase']!=phase:raise ValueError('Complete role/order')
    if phase in q.PHASES:
        if (phase!='batch_summary' and row['pc'] is not None):raise ValueError('PC receipt outside PC role')
        q._validate_row(as_query(row),phase,trace,budget,repetition,target_os)
        if phase=='batch_summary':
            value=row['pc'];fields(value,('kind','scope','product_verdict','batches'),'PC summary')
            if (value['kind'],value['scope'],value['product_verdict'])!=('summary','Complete','NotEvaluated'):raise ValueError('PC summary scope')
            if not isinstance(value['batches'],list) or len(value['batches'])!=3:raise ValueError('PC summary count')
            for i,item in enumerate(value['batches']):
                fields(item,('id','timing','reference'),'PC summary batch')
                if item['id']!=PC_BATCHES[i]:raise ValueError('PC summary order')
                times(item['timing']);times(item['reference'])
        return row
    # PC has a collection of owners, never a synthetic single-owner zero ledger.
    if row['query'] is not None or row['calibration'] is not None or row['managed'] is not None or row['work_counts'] is not None or row['candidate_work'] is not None:
        raise ValueError('PC cannot borrow a single-owner or query receipt')
    if row['target_os']!=target_os or row['target_arch'] not in ('x86_64','x86','aarch64') or row['pointer_bits']!=(32 if row['target_arch']=='x86' else 64):
        raise ValueError('PC target identity')
    # The legacy projection has no collection of owners. Validate the original
    # caller field before replacing it for that single-owner check.
    natural(row['fixture_bytes'],'PC fixture bytes')
    old=dict(row,schema_version=2,phase='released',owner_alive=False,fixture_bytes=0)
    legacy.validate_row(old,'released',trace,budget,operations=False)
    if any(m['calls'] for m in row['metrics']):raise ValueError('PC nested baseline clocks')
    value=row['pc']
    if phase=='pc_group_released':
        fields(value,('kind','owners_dropped','tasks_held','owners','aggregate','reference_scope','reference_ns'),'PC release')
        if (value['kind']!='released' or type(value['owners_dropped']) is not int or value['owners_dropped']!=128
                or type(value['tasks_held']) is not int or value['tasks_held']!=0 or value['owners'] is not None or value['aggregate'] is not None
                or row['owner_alive'] is not False or row['fixture_bytes']!=0 or value['reference_scope']!='outer_loop_only'):
            raise ValueError('PC actual group release')
        if not isinstance(value['reference_ns'],list) or len(value['reference_ns'])!=3:raise ValueError('PC references')
        for v in value['reference_ns']:times(v)
        if sum(v['sum_ns'] for v in value['reference_ns'])>row['elapsed_ns']:raise ValueError('PC references exceed interval')
        return row
    kind={'pc_group_ready':'ready','pc_prepared':'prepared','pc_cancelled_held':'cancelled-held','pc_dropped':'dropped'}[phase]
    fields(value,('kind','admission','aggregate','owners') if kind=='ready' else ('kind','r','action_ns','admission','aggregate','owners'),'PC checkpoint')
    if value['kind']!=kind or row['owner_alive'] is not True or row['fixture_bytes']==0:raise ValueError('PC role/lifetime')
    r=None if kind=='ready' else natural(value['r'],'sample slot',15)
    if r is not None and natural(value['action_ns'],'PC action')>row['elapsed_ns']:raise ValueError('PC action exceeds interval')
    a=admission(value['admission'],row)
    owners=value['owners']
    if not isinstance(owners,list) or len(owners)!=128:raise ValueError('Exactly 128 PC owners')
    for i,v in enumerate(owners):
        owner(v,row,base,kind,r,a)
        if v['i']!=i:raise ValueError('Duplicate/reordered PC owner')
    fields(value['aggregate'],RESOURCE,'aggregate')
    for key,v in value['aggregate'].items():
        if natural(v,key)!=sum(o['resources'][key] for o in owners):raise ValueError('PC aggregate mismatch')
    if value['aggregate']['source_reserved']+value['aggregate']['runtime_reserved']>268435456:raise ValueError('PC aggregate budget')
    return row

def validate_end(row,trace,budget,repetition=1):
    if not isinstance(row,dict) or row.get('plan_id')!=PLAN:raise ValueError('Complete end plan')
    q.validate_end(dict(row,plan_id=q.PLAN),trace,budget,repetition)
    return row

class Stream:
    """Only baseline/current owner receipts and fixed small timing summaries survive a row."""
    def __init__(self, start=0):
        if start not in (0,9):raise ValueError('Stream start')
        self.index=start;self.base=None;self.previous=None;self.admitted=None
        self.query_batches=[];self.pc_times=[[],[],[]];self.refs=None
    def consume(self,row):
        if self.index>=len(PHASES) or row['phase']!=PHASES[self.index]:raise ValueError('Incomplete/reordered complete flow')
        phase=row['phase'];value=row['pc']
        if phase in q.QUERY_PHASES:
            self.query_batches.append({k:row['query'][k] for k in ('id','timing','reference')})
        if phase=='pc_group_ready':
            self.base=value['owners'];self.previous=value['owners'];self.admitted=value['admission']
        if phase in PC_PHASES:
            kind=PC_PHASES.index(phase);r=(self.index-15)//3
            if value['r']!=r or len(self.pc_times[kind])!=r:raise ValueError('PC sample reuse/order')
            if value['admission']!=self.admitted:raise ValueError('PC admission changed')
            for i,o in enumerate(value['owners']):
                base=self.base[i];last=self.previous[i]
                for key in ('frame','create_work','play_work','inspection_work','calls','peaks'):
                    if o[key]!=base[key]:raise ValueError('PC active/setup state changed')
                if delta(last['active_work'],o['active_work'])!=work(summaries=1):raise ValueError('PC active diagnostic work')
                if kind in (1,2):
                    for key in ('key','task_work','steps'):
                        if o[key]!=last[key]:raise ValueError('PC task changed between clocks')
                if kind==1 and o['resources']!=last['resources']:raise ValueError('Cancellation prematurely released storage')
                if kind==2 and o['resources']!=base['resources']:raise ValueError('Drop did not return exact resource baseline')
                if kind==0:
                    for key in ('source_reserved','source_payload','source_allocations'):
                        if o['resources'][key]!=base['resources'][key]:raise ValueError('Inspection duplicated shared source')
                    if o['resources']['runtime_payload']<=base['resources']['runtime_payload']:raise ValueError('No real prepared allocation')
            self.previous=value['owners'];self.pc_times[kind].append(value['action_ns'])
        if phase=='pc_group_released':
            if any(len(x)!=16 for x in self.pc_times):raise ValueError('Incomplete PC batches')
            self.refs=value['reference_ns'];self.base=None;self.previous=None
        if phase=='batch_summary':
            if row['query']['batches']!=self.query_batches:raise ValueError('Query summary mismatch')
            if self.refs is None:raise ValueError('Missing released PC group')
            for i,b in enumerate(value['batches']):
                if b['timing']['raw_ns']!=self.pc_times[i] or b['reference']!=self.refs[i]:raise ValueError('PC raw/summary mismatch')
        self.index+=1
        if value is not None and isinstance(value.get('owners'),list):
            compact=dict(row);receipt={k:v for k,v in value.items() if k!='owners'}
            receipt.update(validated_owners=128,owner_receipts_sha256=hashlib.sha256(json.dumps(value['owners'],sort_keys=True,separators=(',',':')).encode()).hexdigest())
            compact['pc']=receipt
            return compact
        return row
    def finish(self):
        if self.index!=len(PHASES):raise ValueError('Missing complete stream')

def validate_lifecycle(rows):
    if [r['phase'] for r in rows]!=list(PHASES):raise ValueError('Incomplete full lifecycle')
    legacy.validate_lifecycle(rows[:9])
    query_rows=[as_query(r) for r in rows[:14]+rows[-1:]]
    q.validate_lifecycle(query_rows)
    first=rows[0]
    for r in rows:
        if any(r[k]!=first[k] for k in ('plan_id','case_ordinal','target_arch','target_os','pointer_bits','sample_storage_bytes','query_storage_bytes','closure_storage_bytes')):
            raise ValueError('Complete identity/storage drift')
        if r['phase'] in ('pc_group_ready',*PC_PHASES):
            if r['pc'].get('validated_owners')!=128 or 'owners' in r['pc']:raise ValueError('Missing bounded streaming PC validation')
            if r['fixture_bytes']!=rows[2]['fixture_bytes']:raise ValueError('PC caller fixture capacity')
    for a,b in zip(rows,rows[1:]):
        if any(b['peaks'][k]<v for k,v in a['peaks'].items()):raise ValueError('Complete peak regression')

def summarize(row,uncertainty):
    result=q.summarize(row,uncertainty)
    if row['phase']=='batch_summary':
        result['pc']=copy.deepcopy(row['pc'])
        for batch in result['pc']['batches']:
            batch.update(timing_status='Valid' if min(batch['timing']['raw_ns'])>=20*uncertainty else 'Unknown',
                units='one fixed batch; n=16; p95/p99=max; no API-tail inference or subtraction',
                cpu_status='Unknown',stage_peak_status='Unknown')
    return result

def coverage(rows,uncertainty):
    result=q.coverage(rows,uncertainty)
    old_batches=('PC-PREPARE-32','PC-CANCEL-32','PC-DROP-32')
    for item in result['mapping']:
        if item['evidence'] in old_batches:
            item['evidence']=PC_BATCHES[old_batches.index(item['evidence'])]
        if item['calls'] and item['evidence'] not in (*q.FAMILIES,*BATCHES,'diagnostic-reference'):
            item['status']='NotRun'
    for batch in rows[-1]['pc']['batches']:
        result['batches'][batch['id']]='Valid' if min(batch['timing']['raw_ns'])>=20*uncertainty else 'Unknown'
    for item in result['mapping']:
        if item['calls'] and item['evidence'] in PC_BATCHES:item['status']=result['batches'][item['evidence']]
    complete=(all(result['batches'].get(k)=='Valid' for k in BATCHES)
              and all(result['families'].get(k,{}).get('timing_status')=='Valid' for k in q.FAMILIES)
              and all(not m['calls'] or m['status'] in ('Valid','Reference') for m in result['mapping']))
    result.update(status='Satisfied' if complete else 'NeedsEvidence',scope='Complete',missing_batches=[])
    return result

def validate_canary(lines,decoder):
    """Real Rust supplemental rows only; zero clocks are never measurement evidence."""
    target_os={'win32':'windows','linux':'linux','darwin':'macos'}.get(sys.platform)
    if target_os is None:raise ValueError('Canary host')
    cases=[(t,b) for t in ('g0','g1','a1') for b in (1,256)]
    case=0;stream=Stream(9);rows=[];count=0
    for line in lines:
        if not line.startswith('PC_CANARY\t'):continue
        if len(line)>1048576 or case>=6:raise ValueError('Canary capacity')
        row=decoder(line[len('PC_CANARY\t'):]);trace,budget=cases[case]
        _validate_row(row,PHASES[stream.index],trace,budget,1,target_os)
        if any(row[k]!=0 for k in ('elapsed_ns','previous_emit_ns','previous_gate_ns')):raise ValueError('Canary timing')
        rows.append(stream.consume(row));count+=1
        if stream.index==len(PHASES):
            stream.finish()
            q.validate_query_lifecycle([as_query(r) for r in rows[:5]+rows[-1:]])
            if any(v for timeset in stream.pc_times for v in timeset):raise ValueError('PC canary clocks')
            if any(v for ref in stream.refs for v in ref['raw_ns']):raise ValueError('PC reference clocks')
            if any(v for batch in stream.query_batches for key in ('timing','reference') for v in batch[key]['raw_ns']):
                raise ValueError('Query canary clocks')
            case+=1;stream=Stream(9);rows=[]
    if case!=6 or count!=336 or rows:raise ValueError('Six complete supplemental canaries required')
    return case
