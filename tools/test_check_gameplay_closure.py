"""Synthetic full-plan fixtures; never execution or measurement evidence."""
import copy
import csv
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import check_gameplay_cost as cost
import gameplay_cost_closure as p
import gameplay_cost_custody as c
import gameplay_cost_query as q
import gameplay_cost_protocol as old
from test_check_gameplay_query import rows as query_rows, end as query_end, times
from test_check_gameplay_custody import COMPILER, BUILD

def rows(trace='g0',budget=1,repetition=1):
    query=query_rows(trace,budget,repetition)
    for row in query:row.update(plan_id=p.PLAN,closure_storage_bytes=200000,pc=None)
    admission=dict(source_bytes=1000000,run_bytes=400000,task_bytes=1000,pool_bytes=100,coordinator_bytes=100,
        pool_limit=8388608,aggregate_limit=67108864,aggregate_required=32*1801200,container_bytes=200000)
    baseline=dict(source_reserved=1000000,runtime_reserved=400200,source_payload=640000,runtime_payload=390000,source_allocations=3,runtime_allocations=3)
    held=dict(baseline,runtime_reserved=801200,runtime_payload=780000,runtime_allocations=6)
    ordinal=q.case_ordinal(trace,budget,repetition);owners=[]
    for i in range(32):
        source=dict(run=10000000+1000*ordinal+100+i,transport=2,input=4 if trace=='a1' else 2,revision=730)
        calls=dict.fromkeys(old.METRICS,0)
        calls.update(create_request=1,prepare_step=1,activate=1,task_drop=2,seal=1,advance=1,
            acknowledge=10000 if trace=='g0' else 20000,freeze=1,inspection_request=1,inspection_step=1,inspection_activate=1)
        if trace=='g1':calls['submit']=1
        if trace=='a1':calls['automatic_prepare']=1
        owners.append(dict(i=i,frame=dict(source=source,at=5000000000,publication=100,objects=10000,hits=0,misses=0,skipped=5000,
            statistics_at=5000000000,driver='Automatic' if trace=='a1' else 'Manual',status='Frozen',pending_inputs=0,pending_events=0,evidence='SyntheticTest',profile_version=1),
            resources=dict(baseline),active_work=q.work(summaries=1),create_work=q.work(record_reads=10000,state_initializations=10000),
            play_work=q.work(record_reads=10000,state_initializations=10000),inspection_work=q.work(inspection_objects=10000),
            calls=calls,peaks=dict(query[3]['peaks']),task_held=False,eligible=False,steps=0,task_work=None,cancel_success=None,key=None))
    def checkpoint(phase,kind,r=None):
        value=copy.deepcopy(query[13]);value.update(phase=phase,query=None,owner_alive=True,fixture_bytes=320000,elapsed_ns=1000000)
        value['pc']=dict(kind=kind,admission=dict(admission),owners=copy.deepcopy(owners),
            aggregate={k:sum(o['resources'][k] for o in owners) for k in p.RESOURCE})
        if r is not None:value['pc'].update(r=r,action_ns=1000)
        return value
    result=query[:14]+[checkpoint('pc_group_ready','ready')]
    for r in range(16):
        for kind,phase in enumerate(p.PC_PHASES):
            for i,o in enumerate(owners):
                o['active_work']['summaries']+=1
                o.update(resources=dict(baseline if kind==2 else held),task_held=kind<2,eligible=kind==0,steps=15005,
                    task_work=q.work(allocation_attempts=3,state_initializations=15000,inspection_objects=5000),
                    cancel_success=None if kind==0 else True,
                    key=dict(handle=[ordinal,r,i],owner=o['frame']['source']['run'],base=o['frame']['source'],
                        target=dict(o['frame']['source'],transport=3,input=o['frame']['source']['input']+1),
                        operation='Inspect',at=0,driver=o['frame']['driver'],full_eq_checked=True,fresh=True))
            result.append(checkpoint(phase,('prepared','cancelled-held','dropped')[kind],r))
    released=copy.deepcopy(query[13]);released.update(phase='pc_group_released',query=None,
        pc=dict(kind='released',owners_dropped=32,tasks_held=0,owners=None,aggregate=None,reference_scope='outer_loop_only',reference_ns=[times(0) for _ in range(3)]))
    summary=query[-1];summary['pc']=dict(kind='summary',scope='Complete',product_verdict='NotEvaluated',
        batches=[dict(id=k,timing=times(),reference=times(0)) for k in p.PC_BATCHES])
    return result+[released,summary]

def end(trace='g0',budget=1,repetition=1):
    return dict(query_end(trace,budget,repetition),plan_id=p.PLAN)

def validate(values,trace='g0',budget=1,repetition=1):
    stream=p.Stream();compact=[]
    for row in values:
        p.validate_row(row,row['phase'],trace,budget,repetition)
        compact.append(stream.consume(row))
    stream.finish();p.validate_lifecycle(compact)
    return compact

def setup(root):
    source=cost.identity();binary=root/'built.exe';binary.write_bytes(b'not executable: full fixture')
    output=root/'evidence'
    manifest,target=c.freeze(binary,COMPILER,source,c.hash_file(binary,c.MAX_BINARY)['sha256'],output,BUILD,lambda:source,p)
    return output,manifest,target

def run_files(root,manifest,name,trace,budget,repetition,*,unknown=False):
    values=rows(trace,budget,repetition)
    if unknown:
        values[16]['pc']['action_ns']=19
        raw=values[-1]['pc']['batches'][1]['timing']['raw_ns'];raw[0]=19
        values[-1]['pc']['batches'][1]['timing']['sum_ns']=sum(raw)
    with (root/(name+'.child.jsonl')).open('wb') as stream:
        for value in [*values,end(trace,budget,repetition)]:stream.write(c.canonical(value)+b'\n')
    with (root/(name+'.samples.csv')).open('w',newline='',encoding='utf-8') as stream:
        writer=csv.writer(stream);writer.writerow(('phase','parent_monotonic_ns',*c.COUNTERS));clock=1;cpu=0
        for value in values:
            for _ in range(2):
                increment=1000011500 if value['phase']=='calibrated' else 1000000
                clock+=increment+100;cpu+=increment
                writer.writerow((value['phase'],clock,cpu,0,1000,2000,1000,2000))
    (root/(name+'.stderr.log')).write_bytes(b'')
    c.write_json(root/(name+'.process.json'),dict(schema_version=3,plan_id=p.PLAN,case_ordinal=q.case_ordinal(trace,budget,repetition),pid=123,
        exit_code=0,eof=True,cleanup_complete=True,observer_cpu_ns_excluded_from_child=1000,elapsed_parent_ns=3000000000,
        source_before=manifest['source_map_sha256'],source_after=manifest['source_map_sha256'],
        binary_before=manifest['binary']['sha256'],binary_after=manifest['binary']['sha256']))

class CompleteProtocolChecks(unittest.TestCase):

    def test_pc_fixture_types_are_checked_before_legacy_projection(self):
        values=rows()
        for phase in ('pc_group_ready',*p.PC_PHASES,'pc_group_released'):
            original=next(r for r in values if r['phase']==phase)
            for bad in (False,True,float(original['fixture_bytes']),None,'0',[]):
                changed=copy.deepcopy(original);changed['fixture_bytes']=bad
                with self.subTest(phase=phase,value=bad),self.assertRaises(ValueError):
                    p.validate_row(changed,phase,'g0',1)

    def test_setup_resource_peak_cannot_omit_a_live_baseline_owner(self):
        original=rows()[14]
        for key in ('source_reserved','runtime_reserved','source_payload','runtime_payload'):
            changed=copy.deepcopy(original)
            owner=changed['pc']['owners'][0]
            owner['peaks'].update({k:owner['resources'][k] for k in ('source_reserved','runtime_reserved','source_payload','runtime_payload')})
            owner['peaks'][key]-=1
            with self.subTest(key=key),self.assertRaises(ValueError):
                p.validate_row(changed,'pc_group_ready','g0',1)

    def test_each_required_batch_and_family_keeps_its_own_unknown_boundary(self):
        values=validate(rows())
        for phase in q.FAMILIES:
            changed=copy.deepcopy(values)
            next(r for r in changed if r['phase']==phase)['elapsed_ns']=99
            result=p.coverage(changed,1)
            self.assertEqual(result['status'],'NeedsEvidence',phase)
            self.assertEqual(result['families'][phase]['timing_status'],'Unknown')
        for batch in p.BATCHES:
            changed=copy.deepcopy(values)
            if batch in q.BATCHES:
                timing=next(r['query']['timing'] for r in changed if r['phase'] in q.QUERY_PHASES and r['query']['id']==batch)
            else:timing=next(b['timing'] for b in changed[-1]['pc']['batches'] if b['id']==batch)
            timing['raw_ns'][0]=19
            result=p.coverage(changed,1)
            self.assertEqual(result['status'],'NeedsEvidence',batch)
            self.assertEqual(result['batches'][batch],'Unknown')
        self.assertTrue(all(m['status']=='NotApplicable' for m in p.coverage(values,1)['mapping'] if not m['calls']))


    def test_parent_consumes_all_65_ack_boundaries_then_end_eof_and_cleanup(self):
        import io
        import itertools
        import json
        class Input(io.BytesIO):
            receipt=b''
            def close(self):self.receipt=self.getvalue();super().close()
        class Process:
            pid=123
            def __init__(self,values):
                self.stdout=io.BytesIO(b''.join(c.canonical(v)+b'\n' for v in values));self.stdin=Input()
            def poll(self):return 0
            def wait(self,timeout):return 0
        class Monitor:
            def __init__(self,pid):self.n=0
            def sample(self):
                self.n+=1000000
                return dict(user_ns=self.n,kernel_ns=0,working_set_bytes=1,private_commit_bytes=1,os_peak_working_set_bytes=1,os_peak_commit_bytes=1)
            def close(self):pass
        for mode in ('complete','missing_end','extra','wrong_round','typed_fixture'):
            values=rows()+[end()]
            if mode=='missing_end':values.pop()
            if mode=='extra':values.append(end())
            if mode=='wrong_round':values[18]['pc']['r']=0
            if mode=='typed_fixture':values[15]['fixture_bytes']=float(values[15]['fixture_bytes'])
            child=Process(values)
            with tempfile.TemporaryDirectory() as temp,patch.object(cost.subprocess,'Popen',return_value=child),patch.object(cost,'OwnedProcess',Monitor),\
                 patch.object(cost,'cpu_reference_valid',return_value=True),patch.object(cost.time,'perf_counter_ns',side_effect=itertools.count(1,3000000000)),\
                 patch.object(cost.time,'monotonic_ns',side_effect=itertools.count(1,100000000)),patch('sys.stdout',io.StringIO()):
                result=cost.run_probe(Path('never-executed.exe'),'g0',1,1,Path(temp),p)
                self.assertEqual(result['status'],'Valid' if mode=='complete' else 'Invalid',result.get('reason'))
                self.assertEqual(child.stdin.receipt.count(b'continue\n'),18 if mode=='wrong_round' else 15 if mode=='typed_fixture' else 65)
                self.assertTrue(result['completion']['cleanup_complete'])
                self.assertTrue((Path(temp)/result['raw_file']).is_file())
                self.assertTrue(all('owners' not in r['pc'] for r in result['phases'] if r['phase'] in ('pc_group_ready',*p.PC_PHASES)))

    def test_complete_matrix_is_streamed_with_only_baseline_and_current_owners(self):
        identities=set()
        for _,trace,budget,repetition in c.names():
            values=rows(trace,budget,repetition);compact=validate(values,trace,budget,repetition)
            self.assertEqual(len(values),65)
            p.validate_end(end(trace,budget,repetition),trace,budget,repetition)
            self.assertEqual(p.coverage(compact,1)['status'],'Satisfied')
            self.assertTrue(all('owners' not in r['pc'] for r in compact[14:63]))
            identities.add(values[14]['pc']['owners'][0]['frame']['source']['run'])
        self.assertEqual(len(identities),12)
        stream=p.Stream()
        for value in rows():stream.consume(value)
        self.assertIsNone(stream.base);self.assertIsNone(stream.previous)
        self.assertEqual([len(v) for v in stream.pc_times],[16]*3)

    def test_caps_roles_keys_work_and_actual_resource_release_fail_closed(self):
        original=rows()
        for mode in ('missing','reorder','repeat','wrong_owner','old_handle','fresh','eq','target','budget','container','sum',
                     'source_shared','cancel_early','drop_payload','drop_count','active_frame','active_work','task_work','steps',
                     'partial_cancel','ready','fixture','summary','release','extra','plan','bool'):
            values=copy.deepcopy(original);row=values[15];o=row['pc']['owners'][0]
            if mode=='missing':values.pop(20)
            if mode=='reorder':values[16],values[17]=values[17],values[16]
            if mode=='repeat':values[18]=copy.deepcopy(values[15])
            if mode=='wrong_owner':o['key']['owner']+=1
            if mode=='old_handle':o['key']['handle'][1]=15
            if mode=='fresh':o['key']['fresh']=False
            if mode=='eq':o['key']['full_eq_checked']=False
            if mode=='target':o['key']['target']['transport']+=1
            if mode=='budget':row['pc']['admission']['aggregate_required']=67108865
            if mode=='container':row['closure_storage_bytes']=1048577
            if mode=='sum':row['pc']['aggregate']['source_reserved']-=1
            if mode=='source_shared':o['resources']['source_allocations']+=1
            if mode=='cancel_early':values[16]['pc']['owners'][0]['resources']=copy.deepcopy(values[14]['pc']['owners'][0]['resources'])
            if mode=='drop_payload':
                values[17]['pc']['owners'][0]['resources']['runtime_payload']+=1;values[17]['pc']['aggregate']['runtime_payload']+=1
            if mode=='drop_count':values[17]['pc']['owners'][0]['resources']['runtime_allocations']=6
            if mode=='active_frame':o['frame']['publication']+=1
            if mode=='active_work':o['active_work']['record_reads']+=1
            if mode=='task_work':o['task_work']['inspection_objects']-=1
            if mode=='steps':values[16]['pc']['owners'][0]['steps']+=1
            if mode=='partial_cancel':values[16]['pc']['owners'][0]['cancel_success']=False
            if mode=='ready':values[14]['pc']['owners'][0]['task_work']=q.work()
            if mode=='fixture':row['fixture_bytes']+=1
            if mode=='summary':values[-1]['pc']['batches'][0]['timing']=times(999)
            if mode=='release':values[-2]['pc']['aggregate']={}
            if mode=='extra':o['key']['generation']=3
            if mode=='plan':row['plan_id']=q.PLAN
            if mode=='bool':o['key']['handle'][0]=False
            with self.assertRaises(ValueError,msg=mode):validate(values)

    def test_unknown_is_not_protocol_failure_or_product_acceptance(self):
        values=validate(rows())
        self.assertEqual(p.coverage(values,50)['status'],'Satisfied')
        self.assertEqual(p.coverage(values,51)['status'],'NeedsEvidence')
        values[-1]['pc']['batches'][1]['timing']['raw_ns'][0]=19
        result=p.coverage(values,1)
        self.assertEqual(result['batches']['PC-CANCEL-32'],'Unknown')
        self.assertEqual(result['status'],'NeedsEvidence')
        self.assertEqual(next(x['status'] for x in result['mapping'] if x['operation']=='cancel' and x['calls']),'Unknown')
        self.assertEqual(values[-1]['pc']['product_verdict'],'NotEvaluated')

    def test_full_custody_readonly_round_trip_and_old_plan_separation(self):
        for unknown in (False,True):
            with self.subTest(unknown=unknown),tempfile.TemporaryDirectory() as temp:
                output,manifest,_=setup(Path(temp))
                for case in c.names():run_files(output,manifest,*case,unknown=unknown and case[0]=='a1-b256-r2')
                report=c.publish(output,manifest)
                self.assertEqual(report['coverage'],'NeedsEvidence' if unknown else 'ReadyForReview')
                self.assertEqual(sum(r['coverage']['status']!='Satisfied' for r in report['runs']),int(unknown))
                self.assertEqual(report['status'],'Valid');self.assertEqual(report['product_status'],'NotEvaluated')
                before=c.inventory(output,plan=p)
                with patch.object(cost,'check_tool',side_effect=AssertionError('no build')),patch.object(cost.subprocess,'Popen',side_effect=AssertionError('no execute')),patch.object(c,'write_json',side_effect=AssertionError('read only')):
                    result=c.verify(output)
                self.assertEqual(result['coverage'],report['coverage'])
                self.assertEqual(c.inventory(output,plan=p),before)
                changed=copy.deepcopy(manifest);changed['plan_id']=q.PLAN
                with self.assertRaises(ValueError):c.validate_freeze(changed)
                (output/'g0-b1-r1.child.jsonl').write_bytes(b'{}\n')
                with self.assertRaises(ValueError):c.verify(output)

    def test_full_freeze_requires_pc_sources_and_fixed_parameters(self):
        with tempfile.TemporaryDirectory() as temp:
            _,manifest,_=setup(Path(temp))
            for name in ('tools/gameplay-cost/src/pc.rs','tools/gameplay_cost_closure.py','tools/test_check_gameplay_closure.py'):
                changed=copy.deepcopy(manifest);del changed['source_sha256'][name]
                changed['source_map_sha256']=c.digest(changed['source_sha256'])
                with self.assertRaises(ValueError):c.validate_freeze(changed)
            changed=copy.deepcopy(manifest);changed['validator']='CostClosure-reader-v1'
            with self.assertRaises(ValueError):c.validate_freeze(changed)
            changed=copy.deepcopy(manifest);changed['parameters']['pc_owners']=31
            with self.assertRaises(ValueError):c.validate_freeze(changed)
