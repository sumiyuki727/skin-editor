"""Synthetic parser fixtures, not measured evidence or performance results."""
import copy
import json
import sys
import unittest

import gameplay_cost_query as q
import gameplay_cost_protocol as old
from test_check_gameplay_cost import row as old_row


def metric(value, calls, ns=1):
    raw=[ns]*calls
    value.update(calls=calls,raw_ns=raw,sum_ns=sum(raw),p50_ns=ns if calls else None,
                 p95_ns=ns if calls else None,p99_ns=ns if calls else None,max_ns=ns if calls else None)


def times(ns=1000):
    return dict(n=16,sum_ns=16*ns,p50_ns=ns,p95_ns=ns,p99_ns=ns,max_ns=ns,raw_ns=[ns]*16)


def rows(trace='g0',budget=1,repetition=1):
    """Small raw fixture proving reader branches, never a Rust measurement claim."""
    ordinal=q.case_ordinal(trace,budget,repetition); base=10000000+1000*ordinal
    result=[]; n=10000; events=n if trace=='g0' else 2*n
    managed=dict(source_reserved=1000000,runtime_reserved=1000000,source_payload=640000,runtime_payload=490000,
                 source_allocations=3,runtime_allocations=3,hits=0,misses=0,skipped=0,pending_inputs=0,pending_events=0,
                 run=base+1,transport=1,input=1,revision=730,source_capacity=n)
    peaks=dict(inputs=64,events=64,source_reserved=1000000,runtime_reserved=2000000,source_payload=640000,runtime_payload=1000000)
    calls={
        'ready':{},'calibrated':{'empty':20000},
        'prepared':dict(fixture=1,create_request=1,prepare_step=1,activate=1,task_drop=1),
        'played':dict(seal=1,advance=1,acknowledge=events),
        'queried':dict(summary=256,page=157,ledger=n,query_batch=1),
        'inspected':dict(freeze=1,inspection_request=1,inspection_step=1,inspection_activate=1,task_drop=1),
        'cancelled':dict(inspection_request=1,cancel_candidate_step=1,cancel=1),
        'task_dropped':dict(task_drop=1,request_cancel_drop_batch=1),'released':dict(owner_drop=1,fixture_drop=1)}
    if trace=='g1': calls['played']['submit']=313
    if trace=='a1': calls['played'].update(automatic_prepare=1,freeze=4)
    for i,phase in enumerate(q.PHASES):
        row=old_row(phase)
        row.update(schema_version=3,plan_id=q.PLAN,case_ordinal=ordinal,trace=trace,budget=budget,
                   query_storage_bytes=40000,query=None,calibration=None,peaks=dict(peaks),elapsed_ns=1000000)
        if i in range(2,8) or i in range(9,13):
            m=dict(managed); row.update(owner_alive=True,fixture_bytes=320000,managed=m,work_counts=q.work(record_reads=n,state_initializations=n))
            if i in (3,4) or i in range(9,13): m.update(hits=0 if trace=='g0' else n,misses=n if trace=='g0' else 0,input=3 if trace=='a1' else 1)
            if i in (5,6,7): m.update(skipped=n//2,transport=2,input=4 if trace=='a1' else 2)
            if i==6:
                m.update(runtime_reserved=2000000,runtime_payload=1000000,runtime_allocations=6)
                row['candidate_work']=q.work(allocation_attempts=3,state_initializations=15000,inspection_objects=5000)
            if i>=9: m['run']=base+2
        if i>=3: row['events_checked']=events
        for m in row['metrics']: metric(m,calls.get(phase,{}).get(m['name'],0))
        if phase=='calibrated':
            row['calibration']=dict(valid=True,observed_resolution_ns=1,empty_span_ns=20000,
                small_reference_ns=1000,large_reference_ns=2000,small_work=1,large_work=2,
                cpu_reference_ns=2000000000,cpu_reference_cycles=1,cpu_reference_objects=256)
            row['elapsed_ns']=2000023000
        result.append(row)
    frame=dict(run=base+2,transport=1,input=3 if trace=='a1' else 1,revision=730,at=9999300000,publication=100,
               objects=n,hits=0 if trace=='g0' else n,misses=n if trace=='g0' else 0,skipped=0,start=-200001,evidence='SyntheticTest',profile_version=1)
    setup_calls=dict.fromkeys(old.METRICS,0)
    setup_calls.update({k:v for k,v in calls['prepared'].items() if k!='fixture'})
    setup_calls.update(calls['played'])
    result[9]['query']=dict(kind='setup',frame=frame,prepared_work=q.work(record_reads=n,state_initializations=n),calls=setup_calls)
    previous=result[9]['work_counts']
    summaries=[]
    for kind,row in enumerate(result[10:13]):
        samples=[]; delta=q.expected_delta(kind)
        before=dict(previous,summaries=previous['summaries']+1)
        for i in range(16):
            after={k:before[k]+delta[k] for k in old.WORK}; after['valid']=True
            samples.append(dict(ordinal=i,ns=1000,checked=True,before=before,after=after,delta=dict(delta)))
            before=dict(after,summaries=after['summaries']+2)
        row['work_counts']=dict(after,summaries=after['summaries']+3)
        previous=row['work_counts']
        batch=dict(id=q.BATCHES[kind],timing=times(),reference=times(0))
        summaries.append(copy.deepcopy(batch))
        row['query']=dict(batch,kind='batch',frame=frame,state_unchanged=True,resource_unchanged=True,
                          reference_scope='outer_loop_only',guard_summaries=34,samples=samples)
    result[13]['query']=dict(kind='released',owners_dropped=True)
    result[14].update(elapsed_ns=0,query=dict(kind='summary',scope='QueryOnly',coverage='NeedsEvidence',product_verdict='NotEvaluated',batches=summaries))
    return result


def end(trace='g0',budget=1,repetition=1):
    return dict(schema_version=3,plan_id=q.PLAN,case_ordinal=q.case_ordinal(trace,budget,repetition),trace=trace,budget=budget,
                objects=10000,phase='end',status='Valid',previous_emit_ns=0,previous_gate_ns=0)


class QueryProtocolChecks(unittest.TestCase):
    def test_wire_canary_has_all_six_cases_without_becoming_measurement_evidence(self):
        values=[]
        for trace in ('g0','g1','a1'):
            for budget in (1,256):
                for row in rows(trace,budget)[9:]:
                    row.update(target_os={'win32':'windows','darwin':'macos'}.get(sys.platform,'linux'),
                               elapsed_ns=0,previous_emit_ns=0,previous_gate_ns=0)
                    batch=row['query']
                    if batch['kind']=='batch':
                        batch.update(timing=times(0),reference=times(0))
                        for sample in batch['samples']:sample['ns']=0
                    if batch['kind']=='summary':
                        for item in batch['batches']:item.update(timing=times(0),reference=times(0))
                    values.append(row)
        def wire(items):return '\n'.join('QUERY_CANARY\t'+json.dumps(row) for row in items)
        self.assertEqual(q.validate_canary(wire(values),json.loads),6)
        for change in ('missing','extra','duplicate','reorder','clock','work'):
            changed=copy.deepcopy(values)
            if change=='missing':changed.pop()
            if change=='extra':changed.append(changed[0])
            if change=='duplicate':changed[6:12]=changed[:6]
            if change=='reorder':changed[1],changed[2]=changed[2],changed[1]
            if change=='clock':changed[0]['elapsed_ns']=1
            if change=='work':changed[1]['query']['samples'][0]['delta']['summaries']-=1
            with self.assertRaises(ValueError,msg=change):q.validate_canary(wire(changed),json.loads)
        with self.assertRaises(ValueError):q.validate_lifecycle(values[:6])

    def test_baseline_source_recipe_cannot_silently_change_between_roles(self):
        for index in (2,3,4,5,6,7):
            for key in ('transport','input','revision'):
                with self.subTest(index=index,key=key):
                    row=rows()[index];row['managed'][key]+=1
                    with self.assertRaises(ValueError):q.validate_row(row,row['phase'],'g0',1)

    def test_all_fixed_cases_are_distinct_complete_and_query_only(self):
        identities=set()
        for trace in ('g0','g1','a1'):
            for budget in (1,256):
                for repeat in (1,2):
                    values=rows(trace,budget,repeat)
                    for r in values: q.validate_row(r,r['phase'],trace,budget,repeat)
                    q.validate_end(end(trace,budget,repeat),trace,budget,repeat)
                    q.validate_lifecycle(values)
                    identities.add(values[9]['managed']['run'])
                    coverage=q.coverage(values,1)
                    self.assertEqual(coverage['status'],'NeedsEvidence')
                    self.assertEqual(len(coverage['mapping']),9*24)
                    self.assertEqual(coverage['families']['prepared']['n'],1)
        self.assertEqual(len(identities),12)

    def test_unknown_schema_plan_case_role_types_and_new_fields_are_rejected(self):
        for key,value in [('schema_version',2),('schema_version',True),('plan_id','CostClosure-v1'),('case_ordinal',1),('objects',4),('query_storage_bytes',1048577),('target_arch','unreviewed'),('target_os','unknown'),('pointer_bits',32),('future',1)]:
            row=rows()[10];row[key]=value
            with self.assertRaises(ValueError): q.validate_row(row,'query_summary','g0',1)
        for repetition in (0,3,True):
            with self.assertRaises(ValueError): q.case_ordinal('g0',1,repetition)

    def test_each_sample_requires_exact_delta_full_before_after_and_checked_result(self):
        for change in ('wrong_delta','extra_work','invalid','reorder','unchecked','short','gap','time'):
            row=rows()[11]; sample=row['query']['samples'][3]
            if change=='wrong_delta': sample['delta']['ledger_objects']-=1
            if change=='extra_work': sample['after']['publications']+=1
            if change=='invalid': sample['before']['valid']=False
            if change=='reorder': sample['ordinal']=2
            if change=='unchecked': sample['checked']=False
            if change=='short': row['query']['samples'].pop()
            if change=='gap': sample['before']['summaries']+=1;sample['after']['summaries']+=1
            if change=='time': sample['ns']=999
            with self.assertRaises(ValueError,msg=change): q.validate_row(row,'query_ledger','g0',1)

    def test_frame_resources_summary_and_diagnostic_gaps_cross_check(self):
        for change in ('source','frame','summary','work','fixture','release','short'):
            values=rows()
            if change=='source': values[10]['managed']['run']+=1
            if change=='frame': values[10]['query']['frame']=dict(values[10]['query']['frame'],publication=101)
            if change=='summary': values[14]['query']['batches'][0]['timing']=times(999)
            if change=='work': values[10]['work_counts']['summaries']+=1
            if change=='fixture': values[10]['fixture_bytes']+=1
            if change=='release': values[13]['managed']={}
            if change=='short': values.pop()
            with self.assertRaises(ValueError,msg=change):
                for row in values:q.validate_row(row,row['phase'],'g0',1)
                q.validate_lifecycle(values)

    def test_short_minimum_not_median_controls_unknown_without_subtraction(self):
        row=rows()[10];row['query']['timing']['raw_ns'][0]=19
        self.assertEqual(q.summarize(row,1)['query']['timing_status'],'Unknown')
        row['query']['timing']['raw_ns'][0]=20
        result=q.summarize(row,1)
        self.assertEqual(result['query']['timing_status'],'Valid')
        self.assertEqual(result['query']['cpu_status'],'Unknown')
        self.assertEqual(result['query']['stage_peak_status'],'Unknown')
        self.assertNotIn('cpu_percent',result['query'])
        for values in ([],[1]*15,[1]*17):
            value=times();value['raw_ns']=values
            with self.assertRaises(ValueError):q.times(value)

    def test_setup_cannot_reuse_timers_or_skip_real_work_or_accept_pc_receipts(self):
        for change in ('timer','reads','calls','kind','release'):
            row=rows()[9]
            if change=='timer':metric(row['metrics'][2],1)
            if change=='reads':row['query']['prepared_work']['record_reads']=0
            if change=='calls':row['query']['calls']['acknowledge']=0
            if change=='kind':row['query']['kind']='pc_group_ready'
            if change=='release':row['owner_alive']=False
            with self.assertRaises(ValueError):q.validate_row(row,'query_group_ready','g0',1)

    def test_end_is_strict_and_not_a_summary_substitute(self):
        for key,value in [('phase','batch_summary'),('plan_id','CostClosure-v1'),('case_ordinal',True),('extra',1)]:
            row=end();row[key]=value
            with self.assertRaises(ValueError):q.validate_end(row,'g0',1)
