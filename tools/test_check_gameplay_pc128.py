"""PC128 protocol/custody regressions use synthetic data, never measurements."""
import copy
import io
import itertools
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_gameplay_cost as cost
import gameplay_cost_custody as c
import gameplay_cost_pc128 as p
import gameplay_cost_closure as old
import gameplay_cost_query as q
from test_check_gameplay_closure import rows, end, validate, setup, run_files


class PC128Checks(unittest.TestCase):
    def test_online_parent_validates_128th_owner_before_ack(self):
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
        for mode in ('complete','tail_frame','missing_end','extra'):
            values=rows(plan=p)+[end(plan=p)]
            if mode=='tail_frame':values[15]['pc']['owners'][127]['frame']['publication']+=1
            if mode=='missing_end':values.pop()
            if mode=='extra':values.append(end(plan=p))
            child=Process(values)
            with tempfile.TemporaryDirectory() as temp,patch.object(cost.subprocess,'Popen',return_value=child),\
                 patch.object(cost,'OwnedProcess',Monitor),patch.object(cost,'cpu_reference_valid',return_value=True),\
                 patch.object(cost.time,'perf_counter_ns',side_effect=itertools.count(1,3000000000)),\
                 patch.object(cost.time,'monotonic_ns',side_effect=itertools.count(1,100000000)),patch('sys.stdout',io.StringIO()):
                result=cost.run_probe(Path('never-executed.exe'),'g0',1,1,Path(temp),p)
            self.assertEqual(result['status'],'Valid' if mode=='complete' else 'Invalid',result.get('reason'))
            self.assertEqual(child.stdin.receipt.count(b'continue\n'),15 if mode=='tail_frame' else 65)
            self.assertTrue(result['completion']['cleanup_complete'])
            self.assertTrue(all('owners' not in r['pc'] for r in result['phases'] if r['phase'] in ('pc_group_ready',*p.PC_PHASES)))

    def test_fixed_plan_complete_matrix_and_compact_retention(self):
        self.assertEqual((p.PARAMETERS['pc_owners'],p.PARAMETERS['aggregate_reserved_limit'],
                          p.PARAMETERS['closure_storage_limit'],p.PARAMETERS['failure_supplement_limit']),
                         (128,256*1024*1024,1024*1024,512*1024))
        identities=set()
        for _,trace,budget,repetition in c.names():
            values=rows(trace,budget,repetition,plan=p)
            compact=validate(values,trace,budget,repetition,plan=p)
            self.assertEqual(len(values),65)
            p.validate_end(end(trace,budget,repetition,plan=p),trace,budget,repetition)
            self.assertEqual(p.coverage(compact,1)['status'],'Satisfied')
            for row in compact[14:63]:
                self.assertNotIn('owners',row['pc'])
                self.assertEqual(row['pc']['validated_owners'],128)
            identities.add(values[14]['pc']['owners'][127]['frame']['source']['run'])
        self.assertEqual(len(identities),12)
        stream=p.Stream()
        for row in rows(plan=p):stream.consume(row)
        self.assertIsNone(stream.base);self.assertIsNone(stream.previous)
        self.assertEqual([len(x) for x in stream.pc_times],[16]*3)

    def test_tail_owner_and_fixed_capacity_fail_closed(self):
        original=rows(plan=p)[15]
        for mode in ('missing','duplicate','slot128','bool','wrong_owner','old_handle','fresh',
                     'eq','target','budget','container','aggregate','shared','work','plan','fixture'):
            value=copy.deepcopy(original);o=value['pc']['owners'][127]
            if mode=='missing':value['pc']['owners'].pop()
            if mode=='duplicate':o['i']=126
            if mode=='slot128':o['i']=128
            if mode=='bool':o['i']=True
            if mode=='wrong_owner':o['key']['owner']-=1
            if mode=='old_handle':o['key']['handle'][1]=15
            if mode=='fresh':o['key']['fresh']=False
            if mode=='eq':o['key']['full_eq_checked']=False
            if mode=='target':o['key']['target']['input']+=1
            if mode=='budget':value['pc']['admission']['aggregate_limit']=64*1024*1024
            if mode=='container':value['closure_storage_bytes']=1048577
            if mode=='aggregate':value['pc']['aggregate']['source_reserved']-=1
            if mode=='shared':o['resources']['source_allocations']+=1
            if mode=='work':o['task_work']['inspection_objects']-=1
            if mode=='plan':value['plan_id']=old.PLAN
            if mode=='fixture':value['fixture_bytes']=float(value['fixture_bytes'])
            with self.subTest(mode=mode),self.assertRaises(ValueError):
                p.validate_row(value,'pc_prepared','g0',1)

    def test_stream_checks_last_owner_and_each_round_actual_lifetime(self):
        values=rows(plan=p)
        for mode,index in (('frame',15),('active_work',15),('cancel_result',16),('early_release',16),
                           ('drop_payload',17),('old_round',18),('old_key',19),('release',63),('summary',64)):
            changed=copy.deepcopy(values[index]);receipt=changed['pc']
            o=receipt.get('owners',[None]*128)[127] if isinstance(receipt.get('owners'),list) else None
            if mode=='frame':o['frame']['publication']+=1
            if mode=='active_work':o['active_work']['record_reads']+=1
            if mode=='cancel_result':o['cancel_success']=False
            if mode=='early_release':o['resources']=copy.deepcopy(values[14]['pc']['owners'][127]['resources'])
            if mode=='drop_payload':
                o['resources']['runtime_payload']+=1;receipt['aggregate']['runtime_payload']+=1
            if mode=='old_round':receipt['r']=0
            if mode=='old_key':o['key']['handle'][1]=0
            if mode=='release':receipt['owners_dropped']=127
            if mode=='summary':receipt['batches'][1]['id']='PC-CANCEL-32'
            trial=values[:index]+[changed]+values[index+1:]
            with self.subTest(mode=mode),self.assertRaises(ValueError):validate(trial,plan=p)
        with self.assertRaises(ValueError):validate(values[:-1],plan=p)

    def test_unknown_and_coverage_mapping_do_not_upgrade_product(self):
        values=validate(rows(plan=p),plan=p)
        self.assertEqual(p.coverage(values,50)['status'],'Satisfied')
        self.assertEqual(p.coverage(values,51)['status'],'NeedsEvidence')
        for batch in p.BATCHES:
            changed=copy.deepcopy(values)
            timing=(next(r['query']['timing'] for r in changed if r['phase'] in q.QUERY_PHASES and r['query']['id']==batch)
                    if batch in q.BATCHES else next(b['timing'] for b in changed[-1]['pc']['batches'] if b['id']==batch))
            timing['raw_ns'][0]=19
            result=p.coverage(changed,1)
            self.assertEqual(result['batches'][batch],'Unknown')
            self.assertEqual(result['status'],'NeedsEvidence')
            self.assertFalse(any(str(m['evidence']).endswith('-32') for m in result['mapping']))
        for family in q.FAMILIES:
            changed=copy.deepcopy(values);next(r for r in changed if r['phase']==family)['elapsed_ns']=99
            self.assertEqual(p.coverage(changed,1)['status'],'NeedsEvidence')
        baseline=q.coverage(values,1)
        missing=next(m for m in baseline['mapping'] if m['calls']);missing['evidence']='unreviewed';missing['status']='Reference'
        with patch.object(q,'coverage',return_value=baseline):
            result=p.coverage(values,1)
        self.assertEqual(missing['status'],'NotRun');self.assertEqual(result['status'],'NeedsEvidence')
        self.assertEqual(values[-1]['pc']['product_verdict'],'NotEvaluated')

    def test_new_custody_matrix_readonly_and_unknown_preservation(self):
        with tempfile.TemporaryDirectory() as temp:
            output,manifest,_=setup(Path(temp),plan=p)
            for case in c.names():run_files(output,manifest,*case,plan=p)
            report=c.derive_report(output,manifest)
            self.assertEqual(report['coverage'],'ReadyForReview')
            (output/'a1-b256-r2.process.json').unlink()
            run_files(output,manifest,'a1-b256-r2','a1',256,2,unknown=True,plan=p)
            report=c.publish(output,manifest)
            self.assertEqual(report['coverage'],'NeedsEvidence');self.assertEqual(report['status'],'Valid')
            self.assertEqual(report['product_status'],'NotEvaluated')
            self.assertEqual(sum(r['coverage']['status']!='Satisfied' for r in report['runs']),1)
            before=c.inventory(output,plan=p)
            with patch.object(cost,'check_tool',side_effect=AssertionError('no build')),\
                 patch.object(cost.subprocess,'Popen',side_effect=AssertionError('no execute')),\
                 patch.object(c,'write_json',side_effect=AssertionError('read only')):
                result=c.verify(output)
            self.assertEqual(result['coverage'],'NeedsEvidence');self.assertEqual(c.inventory(output,plan=p),before)
            (output/'g0-b1-r1.child.jsonl').write_bytes(b'{}\n')
            with self.assertRaises(ValueError):c.verify(output)

    def test_fixed_manifest_sources_and_legacy_readers_remain_separate(self):
        with tempfile.TemporaryDirectory() as temp:
            _,manifest,_=setup(Path(temp),plan=p)
            for name in ('tools/gameplay-cost/src/failure.rs','tools/gameplay_cost_pc128.py','tools/test_check_gameplay_pc128.py'):
                changed=copy.deepcopy(manifest);del changed['source_sha256'][name]
                changed['source_map_sha256']=c.digest(changed['source_sha256'])
                with self.assertRaises(ValueError):c.validate_freeze(changed)
            for key in ('pc_owners','aggregate_reserved_limit','closure_storage_limit','failure_supplement_limit'):
                changed=copy.deepcopy(manifest);changed['parameters'][key]-=1
                with self.assertRaises(ValueError):c.validate_freeze(changed)
            for key in ('pc_result_guards','pc_nested_call_timers','pc_full_key_equality'):
                changed=copy.deepcopy(manifest);del changed['diagnostics'][key]
                with self.assertRaises(ValueError):c.validate_freeze(changed)
            for plan in (q,old):
                changed=copy.deepcopy(manifest)
                changed.update(plan_id=plan.PLAN,validator=plan.VALIDATOR,parameters=copy.deepcopy(plan.PARAMETERS),scope=plan.SCOPE,diagnostics=c.diagnostics(plan))
                for name in ('tools/gameplay-cost/src/failure.rs','tools/gameplay_cost_pc128.py','tools/test_check_gameplay_pc128.py'):
                    del changed['source_sha256'][name]
                changed['source_map_sha256']=c.digest(changed['source_sha256'])
                c.validate_freeze(changed)
            for name in (p.PLAN,q.PLAN,old.PLAN):self.assertEqual(c.plan_for(name).PLAN,name)
            with self.assertRaises(ValueError):c.plan_for('CostClosure-PC129-v1')

    def test_cli_uses_only_new_emission_plan_without_running_it(self):
        result=dict(status='Valid',coverage='NeedsEvidence')
        with patch.object(sys,'argv',['cost','--measure','--output','unused']),\
             patch.object(cost,'safe_output',return_value=Path('unused')),\
             patch.object(cost,'check_tool',return_value=(Path('never.exe'),'compiler',{},'hash',[])),\
             patch.object(cost,'measure',return_value=result) as measure,patch('sys.stdout',io.StringIO()):
            cost.main()
        self.assertIs(measure.call_args.args[-1],p)

    def test_full_canary_runs_exactly_once_and_requires_explicit_ignored_test(self):
        with patch.object(cost,'identity',return_value={'source':'hash'}),\
             patch.object(cost,'tool_environment',return_value=('cargo',{'RUSTC':'rustc'})),\
             patch.object(cost,'linker_arguments',return_value=[]),\
             patch.object(cost,'tool_graph_errors',return_value=[]),patch.object(cost,'tool_source_errors',return_value=[]),\
             patch.object(cost.subprocess,'run') as run,\
             patch.object(cost.subprocess,'check_output',side_effect=['{}','query','build','compiler']),\
             patch.object(cost,'validate_canary',return_value=6),\
             patch.object(cost,'check_full_canary',return_value=6) as canary,\
             patch.object(cost,'built_executable',return_value=Path('never.exe')),\
             patch.object(c,'hash_file',return_value={'sha256':'hash'}),patch('sys.stdout',io.StringIO()):
            cost.check_tool()
        self.assertEqual(canary.call_count,1)
        command=canary.call_args.args[0]
        self.assertEqual(command[-4:],['--','--ignored','--exact','--nocapture'])
        self.assertIn('tests::closure_wire_canary_without_clocks',command)
        self.assertEqual(sum('test' in call.args[0] for call in run.call_args_list),1)

    def test_canary_rejects_filtered_zero_tests_oversize_and_failed_process(self):
        class Process:
            def __init__(self,data,code=0):self.stdout=io.BytesIO(data);self.code=code;self.terminated=False
            def __enter__(self):return self
            def __exit__(self,*args):return False
            def wait(self,timeout):return self.code
            def terminate(self):self.terminated=True
            def kill(self):self.terminated=True
        for data in (b'running 0 tests\ntest result: ok. 0 passed\n',b'x'*(1024*1024+1)):
            child=Process(data)
            with patch.object(cost.subprocess,'Popen',return_value=child),self.assertRaises(ValueError):
                cost.check_full_canary(['not-executed'],{})
            self.assertTrue(child.terminated)
        child=Process(b'',1)
        with patch.object(cost.subprocess,'Popen',return_value=child),\
             patch.object(p,'validate_canary',return_value=6),self.assertRaises(ValueError):
            cost.check_full_canary(['not-executed'],{})
        self.assertTrue(child.terminated)

    def test_canary_references_are_zero_and_case_coverage_is_complete(self):
        def zero(value):
            if isinstance(value,dict):
                for k,v in value.items():
                    if k in ('ns','elapsed_ns','previous_emit_ns','previous_gate_ns','action_ns','sum_ns','p50_ns','p95_ns','p99_ns','max_ns'):
                        if v is not None:value[k]=0
                    elif k=='raw_ns':value[k]=[0]*len(v)
                    else:zero(v)
            elif isinstance(value,list):
                for v in value:zero(v)
        target={'win32':'windows','linux':'linux','darwin':'macos'}[sys.platform]
        for mode in ('complete','pc_reference','query_reference','missing_case'):
            def lines():
                for t in ('g0','g1','a1'):
                    for b in (1,256):
                        if mode=='missing_case' and (t,b)==('a1',256):continue
                        values=rows(t,b,plan=p)[9:]
                        for row in values:zero(row);row['target_os']=target
                        if mode=='pc_reference':
                            for row in (values[-2]['pc']['reference_ns'][0],values[-1]['pc']['batches'][0]['reference']):
                                row['raw_ns'][0]=1;row['sum_ns']=1;row['p95_ns']=1;row['p99_ns']=1;row['max_ns']=1
                            values[-2]['elapsed_ns']=1
                        if mode=='query_reference':
                            values[1]['query']['reference']['raw_ns'][0]=1
                        for row in values:yield 'PC_CANARY\t'+json.dumps(row)+'\n'
            if mode=='complete':self.assertEqual(p.validate_canary(lines(),c.decode),6)
            else:
                with self.subTest(mode=mode),self.assertRaises(ValueError):p.validate_canary(lines(),c.decode)
