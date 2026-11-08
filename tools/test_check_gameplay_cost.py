import copy
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_gameplay_cost as cost
import gameplay_cost_protocol as protocol
from check_core_architecture import validate_metadata as core_graph
from test_check_core_architecture import metadata_fixture

ROOT=Path(__file__).resolve().parents[1]


def row(phase='ready'):
    return dict(schema_version=protocol.SCHEMA,phase=phase,trace='g0',budget=1,objects=10000,api_version=7,profile_version=1,
                pointer_bits=64,target_arch='x86_64',target_os='windows',status='Valid',elapsed_ns=0,
                previous_emit_ns=0,previous_gate_ns=0,sample_storage_bytes=sum(protocol.metric_limits(10000).values())*8,
                fixture_bytes=0,owner_alive=False,events_checked=0,managed=None,work_counts=None,candidate_work=None,
                peaks=dict(inputs=0,events=0,source_reserved=0,runtime_reserved=0,source_payload=0,runtime_payload=0),
                metrics=[dict(name=name,capacity=cap,calls=0,sum_ns=0,p50_ns=None,p95_ns=None,p99_ns=None,max_ns=None,raw_ns=[])
                         for name,cap in protocol.metric_limits(10000).items()])


def graph():
    def package(name,path,deps):
        return dict(name=name,id=name,manifest_path=str(ROOT/path/'Cargo.toml'),source=None,features={},publish=[],
                    edition='2024',rust_version='1.95',targets=[dict(kind=['bin' if name.endswith('cost') else 'lib'],name=name,src_path=str(ROOT/path/'src/main.rs'))],
                    dependencies=deps)
    dependency=dict(name='skin-editor-gameplay',path=str(ROOT/'crates/gameplay'),source=None,kind=None,target=None,optional=False,rename=None)
    return dict(workspace_root=str(ROOT/'tools/gameplay-cost'),workspace_members=['skin-editor-gameplay-cost'],
                packages=[package('skin-editor-gameplay-cost',Path('tools/gameplay-cost'),[dependency]),
                          package('skin-editor-gameplay',Path('crates/gameplay'),[])])


class GameplayCostChecks(unittest.TestCase):
    def test_process_capacity_is_global_and_stops_before_the_next_native_sample(self):
        from test_check_gameplay_query import rows
        class Process:
            pid=123
            def __init__(self):
                self.stdout=io.BytesIO(b''.join((json.dumps(r)+'\n').encode() for r in rows()[:2]));self.stdin=io.BytesIO()
            def poll(self):return 0
            def wait(self,timeout):return 0
        class Monitor:
            calls=0
            def __init__(self,pid):pass
            def sample(self):
                Monitor.calls+=1
                return dict(user_ns=0,kernel_ns=0,working_set_bytes=1,private_commit_bytes=1,os_peak_working_set_bytes=1,os_peak_commit_bytes=1)
            def close(self):pass
        for capacity in (0,3):
            Monitor.calls=0
            with tempfile.TemporaryDirectory() as directory,patch.object(cost.subprocess,'Popen',return_value=Process()),\
                 patch.object(cost,'OwnedProcess',Monitor),patch.object(cost,'SAMPLE_LIMIT',capacity),patch('sys.stdout',io.StringIO()):
                result=cost.run_probe(Path('owned.exe'),'g0',1,1,Path(directory))
            self.assertEqual(result['status'],'Invalid');self.assertLessEqual(Monitor.calls,capacity)
            self.assertEqual(len(result['phases']),0 if capacity==0 else 1)
            self.assertIn('capacity',result['reason'])

    def test_non_object_rows_and_metric_members_are_rejected_as_protocol_errors(self):
        for value in ([],None,True):
            with self.assertRaises(ValueError):protocol.validate_row(value,'ready','g0',1)
        sample=row();sample['metrics'][0]=None
        with self.assertRaises(ValueError):protocol.validate_row(sample,'ready','g0',1)

    def test_end_receipt_requires_complete_typed_fields_and_unique_json_keys(self):
        end=dict(schema_version=protocol.SCHEMA,phase='end',status='Valid',previous_emit_ns=1,previous_gate_ns=2)
        protocol.validate_end(end)
        for field,value in [('schema_version',True),('previous_emit_ns',True),('previous_gate_ns',-1),('status','Invalid')]:
            changed=dict(end);changed[field]=value
            with self.assertRaises(ValueError):protocol.validate_end(changed)
        for value in ([],dict(schema_version=protocol.SCHEMA,phase='end'),dict(end,extra=1)):
            with self.assertRaises(ValueError):protocol.validate_end(value)
        for value in (b'[]\n',b'{"phase":"end","phase":"ready"}\n',b'{"bad":NaN}\n',b'{}'):
            with self.assertRaises(ValueError):cost.decode_line(value)

    def test_release_requires_drop_samples_and_preserves_the_checked_event_total(self):
        sample=row('released')
        with self.assertRaises(ValueError):protocol.validate_row(sample,'released','g0',1)
        sample.update(events_checked=10000,elapsed_ns=2)
        for name in ('owner_drop','fixture_drop'):
            m=next(m for m in sample['metrics'] if m['name']==name)
            m.update(calls=1,raw_ns=[1],sum_ns=1,p50_ns=1,p95_ns=1,p99_ns=1,max_ns=1)
        protocol.validate_row(sample,'released','g0',1)
        for field,value in [('events_checked',0),('fixture_bytes',1)]:
            changed=copy.deepcopy(sample);changed[field]=value
            with self.assertRaises(ValueError):protocol.validate_row(changed,'released','g0',1)
        changed=copy.deepcopy(sample);changed['metrics'][-3].update(calls=0,raw_ns=[],sum_ns=0,p50_ns=None,p95_ns=None,p99_ns=None,max_ns=None)
        with self.assertRaises(ValueError):protocol.validate_row(changed,'released','g0',1)

    def test_cancellation_must_hold_owners_until_drop_and_keep_phase_identity(self):
        phases=[dict(phase=name,target_arch='x86_64',target_os='windows',pointer_bits=64,sample_storage_bytes=1,
                     fixture_bytes=1 if 2<=i<=7 else 0) for i,name in enumerate(protocol.PHASES)]
        base=dict(source_reserved=100,source_payload=80,source_allocations=3,runtime_reserved=200,runtime_payload=100,
                  runtime_allocations=3,run=1,transport=2,input=3,revision=4)
        phases[5]['managed']=dict(base);phases[6]['managed']=dict(base,runtime_reserved=500,runtime_payload=300,runtime_allocations=6)
        phases[7]['managed']=dict(base)
        protocol.validate_lifecycle(phases)
        for index,field,value in [(6,'runtime_reserved',200),(6,'runtime_allocations',3),(7,'runtime_payload',101),(6,'input',2)]:
            changed=copy.deepcopy(phases);changed[index]['managed'][field]=value
            with self.assertRaises(ValueError):protocol.validate_lifecycle(changed)
        for field,value in [('pointer_bits',32),('fixture_bytes',0)]:
            changed=copy.deepcopy(phases);changed[6][field]=value
            with self.assertRaises(ValueError):protocol.validate_lifecycle(changed)

    def test_calibration_must_resolve_the_representative_work_against_empty_call_cost(self):
        sample=row('calibrated');sample['elapsed_ns']=2_100_000_000
        raw=[1000]*20000
        sample['metrics'][0].update(calls=len(raw),raw_ns=raw,sum_ns=sum(raw),p50_ns=1000,p95_ns=1000,p99_ns=1000,max_ns=1000)
        sample['calibration']=dict(valid=True,observed_resolution_ns=1,empty_span_ns=20_000_000,
            small_reference_ns=1000,large_reference_ns=2000,small_work=1,large_work=2,
            cpu_reference_ns=2_000_000_000,cpu_reference_cycles=1,cpu_reference_objects=256)
        with self.assertRaises(ValueError):protocol.validate_calibration(sample)

    def test_native_counters_cannot_be_missing_zero_filled_or_regress_individually(self):
        point=dict(user_ns=10,kernel_ns=20,working_set_bytes=1,private_commit_bytes=1,os_peak_working_set_bytes=1,os_peak_commit_bytes=1)
        protocol.validate_process_point(point,None)
        for value in ({},dict(point,working_set_bytes=0),dict(point,user_ns=True),dict(point,user_ns=11,kernel_ns=19),dict(point,os_peak_commit_bytes=0)):
            with self.assertRaises(ValueError):protocol.validate_process_point(value,point)

    def test_build_snapshot_is_frozen_before_compiling_and_detects_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            binary=Path(directory)/'probe.exe';binary.write_bytes(b'build')
            artifact=json.dumps(dict(reason='compiler-artifact',target=dict(name='skin-editor-gameplay-cost'),executable=str(binary)))
            with patch.object(cost,'tool_environment',return_value=('cargo',{'RUSTC':'rustc'})),patch.object(cost,'linker_arguments',return_value=[]),\
                 patch.object(cost,'tool_graph_errors',return_value=[]),patch.object(cost,'tool_source_errors',return_value=[]),\
                 patch.object(cost.subprocess,'run'),patch.object(cost,'validate_canary',return_value=6),\
                 patch.object(cost.subprocess,'check_output',side_effect=['{}','canary',artifact,'rustc identity']),\
                 patch.object(cost,'identity',side_effect=[{'source':'before'},{'source':'after'}]):
                with self.assertRaises(ValueError):cost.check_tool()

    def test_measurement_uses_the_build_snapshot_instead_of_refreezing_modified_sources(self):
        with tempfile.TemporaryDirectory() as directory:
            binary=Path(directory)/'probe.exe';binary.write_bytes(b'build')
            output=Path(directory)/'new'
            with patch.object(cost,'identity',return_value={'source':'changed'}),\
                 patch.object(cost.subprocess,'check_output',return_value='head'),patch.object(cost,'run_probe') as probe:
                report=cost.measure(binary,'compiler',output,2,{'source':'compiled'},cost.hashlib.sha256(b'build').hexdigest(),[])
            self.assertEqual(report['status'],'Invalid');self.assertEqual(report['product_status'],'NotEvaluated')
            self.assertEqual(report['source_sha256'],{'source':'compiled'});probe.assert_not_called()
            self.assertFalse(output.exists())

    def test_native_peak_scope_does_not_inherit_protocol_valid(self):
        point=dict(user_ns=0,kernel_ns=0,working_set_bytes=1,private_commit_bytes=1,os_peak_working_set_bytes=1,os_peak_commit_bytes=1)
        result=cost.process_summary(point,point,2,2_000_000,1_000_000,None,False)
        self.assertEqual(result['stage_peak_status'],'Unknown')
        self.assertEqual(result['memory_boundary_status'],'Valid')
        self.assertEqual(result['os_peak_scope'],'process_lifetime')
        self.assertFalse(result['sampling_target_met'])
        self.assertEqual(result['cpu_status'],'Unknown')
        self.assertIsNone(result['one_core_percent'])

    def test_wait_timeout_and_handle_close_failure_keep_completed_phases_and_raw_evidence(self):
        class Process:
            pid=123
            def __init__(self):
                from test_check_gameplay_query import rows, end
                self.stdout=io.BytesIO((json.dumps(rows()[0])+'\n'+json.dumps(end())+'\n').encode());self.stdin=io.BytesIO()
            def poll(self):return 0
            def wait(self,timeout):raise cost.subprocess.TimeoutExpired('owned',timeout)
        class Monitor:
            def __init__(self,pid):pass
            def sample(self):return dict(user_ns=0,kernel_ns=0,working_set_bytes=1,private_commit_bytes=1,os_peak_working_set_bytes=1,os_peak_commit_bytes=1)
            def close(self):raise OSError('owned handle close failed')
        with tempfile.TemporaryDirectory() as directory,patch.object(cost.subprocess,'Popen',return_value=Process()),\
             patch.object(cost,'OwnedProcess',Monitor),patch.object(cost,'PHASES',('ready',)):
            result=cost.run_probe(Path('owned.exe'),'g0',1,1,Path(directory))
            self.assertEqual(result['status'],'Invalid');self.assertEqual(len(result['phases']),1)
            self.assertIn('handle close failed',result['reason'])
            self.assertTrue((Path(directory)/result['raw_file']).read_bytes())

    def test_completion_waits_share_the_remaining_whole_child_deadline(self):
        for now,want in ((0,5),(299_000_000_000,1)):
            with patch.object(cost.time,'monotonic_ns',return_value=now):
                self.assertEqual(cost.completion_timeout(0),want)
        with patch.object(cost.time,'monotonic_ns',return_value=300_000_000_000):
            with self.assertRaises(ValueError):cost.completion_timeout(0)

    def test_failed_termination_never_closes_a_pipe_held_by_the_reader(self):
        class HeldPipe:
            closed=False
            def close(self):raise AssertionError('would block on buffered reader lock')
        class Process:
            pid=123
            stdout=HeldPipe()
            stdin=io.BytesIO()
            def poll(self):return None
            def terminate(self):pass
            def kill(self):pass
            def wait(self,timeout):raise cost.subprocess.TimeoutExpired('owned',timeout)
        class Reader:
            ident=123
            def __init__(self,**kwargs):pass
            def start(self):pass
            def join(self,timeout):self.joined=True
            def is_alive(self):return True
        with tempfile.TemporaryDirectory() as directory,patch.object(cost.subprocess,'Popen',return_value=Process()),\
             patch.object(cost.threading,'Thread',Reader),patch.object(cost,'OwnedProcess',side_effect=OSError('observer failed')):
            result=cost.run_probe(Path('owned.exe'),'g0',1,1,Path(directory))
        self.assertEqual(result['status'],'Invalid')
        self.assertFalse(result['completion']['cleanup_complete'])
        self.assertIn('stdout close deferred',result['reason'])

    def test_unrecorded_build_overrides_cannot_change_the_cost_binary_profile(self):
        cost.validate_build_environment({'CARGO_HOME':'owned-cache'})
        for key in ('CARGO_PROFILE_RELEASE_OPT_LEVEL','CARGO_BUILD_TARGET','CARGO_INCREMENTAL','RUSTC_BOOTSTRAP','CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS'):
            with self.assertRaises(ValueError):cost.validate_build_environment({key:'override'})
    def test_cancel_checkpoint_includes_initialization_and_private_inspection_writes(self):
        work=dict(valid=True,**{name:0 for name in protocol.WORK})
        work.update(allocation_attempts=3,state_initializations=15000,inspection_objects=5000)
        protocol.validate_cancel_checkpoint(work,10000)
        for key,value in [('allocation_attempts',2),('state_initializations',10000),('inspection_objects',4999)]:
            changed=dict(work);changed[key]=value
            with self.assertRaises(ValueError):protocol.validate_cancel_checkpoint(changed,10000)
    def test_versions_identity_and_booleans_cannot_impersonate_integer_fields(self):
        protocol.validate_row(row(),'ready','g0',1)
        for field,value in [('schema_version',protocol.SCHEMA+1),('schema_version',True),('budget',True),('phase','played'),('profile_version',0),('objects',9999),('pointer_bits',16)]:
            sample=row();sample[field]=value
            with self.assertRaises(ValueError):protocol.validate_row(sample,'ready','g0',1)

    def test_zero_samples_have_no_percentiles(self):
        self.assertIsNone(protocol.nearest([],95))
        sample=row();sample['metrics'][0]['p50_ns']=0
        with self.assertRaises(ValueError):protocol.validate_row(sample,'ready','g0',1)
        self.assertEqual(protocol.nearest([4,1,3,2],50),2)
        self.assertEqual(protocol.nearest([4,1,3,2],95),4)

    def test_raw_samples_cannot_be_truncated_relabelled_or_change_statistics(self):
        for field,value in [('calls',1),('capacity',1),('sum_ns',1),('p95_ns',1),('calls',True)]:
            sample=row();sample['metrics'][0][field]=value
            with self.assertRaises(ValueError):protocol.validate_row(sample,'ready','g0',1)
        sample=row();sample['metrics']=sample['metrics'][1:]
        with self.assertRaises(ValueError):protocol.validate_row(sample,'ready','g0',1)

    def test_invalid_or_exhausted_receipts_never_pass_on_low_values(self):
        for key,value in [('status','Invalid'),('sample_storage_bytes',0),('work_counts',{})]:
            sample=row();sample[key]=value
            with self.assertRaises(ValueError):protocol.validate_row(sample,'ready','g0',1)
        with self.assertRaises(ValueError):protocol.validate_work(dict(valid=False))

    def test_missing_managed_owners_and_unobserved_release_are_distinct(self):
        sample=row();sample['owner_alive']=True
        with self.assertRaises(ValueError):protocol.validate_row(sample,'ready','g0',1)
        sample=row('released');sample['managed']={}
        with self.assertRaises(ValueError):protocol.validate_row(sample,'released','g0',1)

    def test_peak_fields_are_bounded_and_may_not_disappear(self):
        for key,value in [('inputs',65),('source_reserved',8*1024*1024+1),('runtime_payload',1)]:
            sample=row();sample['peaks'][key]=value
            with self.assertRaises(ValueError):protocol.validate_row(sample,'ready','g0',1)
        sample=row();del sample['peaks']
        with self.assertRaises(ValueError):protocol.validate_row(sample,'ready','g0',1)

    def test_missing_failed_and_underexercised_calibration_is_invalid(self):
        for calibration in [None,dict(valid=False),dict(valid=True)]:
            sample=row('calibrated');sample['calibration']=calibration
            with self.assertRaises(ValueError):protocol.validate_row(sample,'calibrated','g0',1)

    def test_continuous_reference_is_required_before_cpu_interpretation(self):
        self.assertTrue(cost.cpu_reference_valid(1_900_000_000,100_000_000,2_000_000_000,15_625_000))
        for user,kernel,span,quantum in [(0,0,2_000_000_000,1),(2_000_000_000,0,1,1),(1_000_000_000,0,2_000_000_000,None),(1_000_000_000,0,2_000_000_000,1)]:
            self.assertFalse(cost.cpu_reference_valid(user,kernel,span,quantum))

    def test_short_calls_and_batch_granularity_cannot_be_presented_as_resolved_call_percentiles(self):
        sample=row();sample['elapsed_ns']=1000
        m=sample['metrics'][0];m.update(calls=1,sum_ns=5,p50_ns=5,p95_ns=5,p99_ns=5,max_ns=5,raw_ns=[5])
        result=protocol.summarize(sample,100)
        self.assertEqual(result['metrics'][0]['timing_status'],'Unknown')
        self.assertEqual(result['metrics'][0]['p95_ns'],5)
        self.assertEqual(result['stage_timing_status'],'Unknown')
        self.assertIn('4096 pages',result['metrics'][13]['units'])

    def test_diagnostic_has_exactly_one_outgoing_product_dependency(self):
        self.assertEqual(protocol.tool_graph_errors(graph(),ROOT),[])
        for mutate in ('third_party','project','optional','features','build','workspace'):
            data=graph();tool=data['packages'][0]
            if mutate=='third_party':data['packages'].append(copy.deepcopy(tool))
            if mutate=='project':tool['dependencies'][0]['name']='skin-editor-application'
            if mutate=='optional':tool['dependencies'][0]['optional']=True
            if mutate=='features':tool['features']={'trace':[]}
            if mutate=='build':tool['targets'].append(dict(kind=['custom-build']))
            if mutate=='workspace':data['workspace_members'].append('skin-editor-gameplay')
            self.assertTrue(protocol.tool_graph_errors(data,ROOT),mutate)

    def test_tool_cannot_become_a_reverse_core_dependency(self):
        data=metadata_fixture();data['packages'].append(graph()['packages'][0])
        data['packages'][0]['dependencies'].append(dict(name='skin-editor-gameplay-cost'))
        self.assertTrue(core_graph(data,ROOT))

    def test_diagnostic_source_cannot_import_platform_io_project_or_external_includes(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);base=root/'tools/gameplay-cost/src';base.mkdir(parents=True)
            for name in ('main.rs','driver.rs','meter.rs','query.rs'):(base/name).write_text('#![forbid(unsafe_code)]\n',encoding='utf-8')
            self.assertEqual(protocol.tool_source_errors(root),[])
            for content in ('use skin_editor_application::ProjectSession;','use std::fs;','include!("outside.rs");','use wgpu::Device;'):
                (base/'driver.rs').write_text(content,encoding='utf-8');self.assertTrue(protocol.tool_source_errors(root))
            (base/'driver.rs').write_text('',encoding='utf-8')
            (base/'hidden').mkdir();(base/'hidden/mod.rs').write_text('use std::net;',encoding='utf-8')
            self.assertTrue(protocol.tool_source_errors(root))

    def test_output_is_new_and_inside_artifacts(self):
        with tempfile.TemporaryDirectory() as directory,patch.object(cost,'ROOT',Path(directory)):
            base=Path(directory);(base/'artifacts/old').mkdir(parents=True)
            self.assertEqual(cost.safe_output(base/'artifacts/new'),(base/'artifacts/new').resolve())
            for path in (base/'outside',base/'artifacts',base/'artifacts/old'):
                with self.assertRaises(ValueError):cost.safe_output(path)

    def test_current_cargo_binary_is_required(self):
        message=dict(reason='compiler-artifact',target=dict(name='skin-editor-gameplay-cost'),executable='custom/bin.exe')
        self.assertEqual(cost.built_executable(json.dumps(message)),Path('custom/bin.exe'))
        for output in ('',json.dumps(message)+'\n'+json.dumps(message)):
            with self.assertRaises(ValueError):cost.built_executable(output)

    def test_partial_child_exit_bad_protocol_and_progress_timeout_preserve_invalid_evidence(self):
        class Process:
            pid=123
            def __init__(self,content):self.stdout=io.BytesIO(content);self.stdin=io.BytesIO()
            def poll(self):return 0
            def wait(self,timeout):return 0
        class Monitor:
            closed=False
            def __init__(self,pid):assert pid==123
            def sample(self):return dict(user_ns=0,kernel_ns=0,working_set_bytes=1,private_commit_bytes=1,os_peak_working_set_bytes=1,os_peak_commit_bytes=1)
            def close(self):Monitor.closed=True
        for content,timeout in [(b'',60),(b'{"schema_version":9}\n',60),(b'',-1)]:
            with tempfile.TemporaryDirectory() as directory,patch.object(cost.subprocess,'Popen',return_value=Process(content)),patch.object(cost,'OwnedProcess',Monitor),patch.object(cost,'PHASE_TIMEOUT',timeout):
                result=cost.run_probe(Path('owned.exe'),'g0',1,1,Path(directory))
                self.assertEqual(result['status'],'Invalid');self.assertTrue(result['reason']);self.assertTrue(Monitor.closed)
                self.assertEqual(result['pid'],123)
                self.assertTrue((Path(directory)/result['raw_file']).is_file())

    def test_no_unsupported_or_non_monotonic_work_receipt_is_accepted(self):
        work=dict(valid=True,**{name:0 for name in protocol.WORK})
        protocol.validate_work(work)
        for name,value in [('valid',False),('record_reads',-1),('allocation_bytes',True)]:
            changed=dict(work);changed[name]=value
            with self.assertRaises(ValueError):protocol.validate_work(changed)
