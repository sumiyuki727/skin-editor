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
    return dict(schema_version=1,phase=phase,trace='g0',budget=1,objects=10000,api_version=7,profile_version=1,
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
        for field,value in [('schema_version',2),('schema_version',True),('budget',True),('phase','played'),('profile_version',0),('objects',9999),('pointer_bits',16)]:
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
            for name in ('main.rs','driver.rs','meter.rs'):(base/name).write_text('#![forbid(unsafe_code)]\n',encoding='utf-8')
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
