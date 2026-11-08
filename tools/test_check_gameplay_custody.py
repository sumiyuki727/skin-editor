"""Fake files/counters only: these tests never execute a cost probe or sample a PID."""
import copy
import csv
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_gameplay_cost as cost
import gameplay_cost_custody as c
import gameplay_cost_query as q
from test_check_gameplay_query import rows, end

COMPILER='rustc 1.95.0 (test fixture)\nhost: x86_64-pc-windows-msvc\nLLVM version: fixture\n'
BUILD=['cargo','build','--manifest-path','E:/skin-editor/tools/gameplay-cost/Cargo.toml','--offline','--locked','--release','--message-format=json']


def setup(root):
    source=cost.identity()
    binary=root/'built.exe';binary.write_bytes(b'non-executable test fixture')
    output=root/'evidence'
    manifest,target=c.freeze(binary,COMPILER,source,c.hash_file(binary,c.MAX_BINARY)['sha256'],output,BUILD,lambda:source)
    return output,manifest,binary,target


def run_files(root,manifest,name,trace,budget,repetition):
    values=rows(trace,budget,repetition)
    with (root/(name+'.child.jsonl')).open('wb') as stream:
        for value in [*values,end(trace,budget,repetition)]:stream.write(c.canonical(value)+b'\n')
    with (root/(name+'.samples.csv')).open('w',newline='',encoding='utf-8') as stream:
        writer=csv.writer(stream);writer.writerow(('phase','parent_monotonic_ns',*c.COUNTERS))
        clock=1;cpu=0
        for value in values:
            for j in range(2):
                increment=1000011500 if value['phase']=='calibrated' else 1000000
                clock+=increment+100;cpu+=increment
                writer.writerow((value['phase'],clock,cpu,0,1000,2000,1000,2000))
    (root/(name+'.stderr.log')).write_bytes(b'')
    c.write_json(root/(name+'.process.json'),dict(schema_version=3,plan_id=q.PLAN,case_ordinal=q.case_ordinal(trace,budget,repetition),pid=123,
        exit_code=0,eof=True,cleanup_complete=True,observer_cpu_ns_excluded_from_child=1000,elapsed_parent_ns=3000000000,
        source_before=manifest['source_map_sha256'],source_after=manifest['source_map_sha256'],
        binary_before=manifest['binary']['sha256'],binary_after=manifest['binary']['sha256']))


def complete(root):
    output,manifest,binary,target=setup(root)
    for case in c.names():run_files(output,manifest,*case)
    c.publish(output,manifest)
    return output,manifest,binary,target


def replace_json(path,value):
    path.write_bytes(c.canonical(value)+b'\n')


def reindex(output):
    replace_json(output/'evidence-index.json',dict(schema_version=3,plan_id=q.PLAN,validator=q.VALIDATOR,files=c.inventory(output)))


class CustodyChecks(unittest.TestCase):
    def test_unsupported_observer_is_not_run_unknown_without_fake_freeze_or_child(self):
        with tempfile.TemporaryDirectory() as tmp:
            output=Path(tmp)/'new';source=cost.identity()
            with patch.object(cost,'identity',return_value=source),patch.object(cost,'supports_observation',return_value=False),\
                 patch.object(c,'freeze',side_effect=AssertionError('freeze')),patch.object(cost,'run_probe',side_effect=AssertionError('execute')):
                result=cost.measure(Path(tmp)/'unused',COMPILER,output,2,source,'unused',BUILD)
            self.assertEqual(result['status'],'NotRun');self.assertEqual(result['observation_status'],'Unknown')
            self.assertEqual(result['runs'],[]);self.assertTrue((output/'report.partial').exists())
            self.assertFalse((output/'freeze.json').exists());self.assertFalse((output/'evidence-index.json').exists())

    def test_clock_and_build_contracts_cannot_be_replaced_by_unreviewed_metadata(self):
        with tempfile.TemporaryDirectory() as tmp:
            _,manifest,_,_=setup(Path(tmp))
            for change in ('clock','resolution','cpu','diagnostic','build','compiler'):
                value=copy.deepcopy(manifest)
                if change=='clock':value['host_observer']['parent_clock']['monotonic']=1
                if change=='resolution':value['host_observer']['parent_clock']['resolution_seconds']=float('inf')
                if change=='cpu':value['host_observer']['logical_cpus']=True
                if change=='diagnostic':value['diagnostics']['always_on_work_counts']=1
                if change=='build':value['build_arguments'][1:1]=['--config','profile.release.opt-level=0']
                if change=='compiler':value['compiler']=value['compiler'].replace('1.95.0','1.95.01')
                with self.assertRaises(ValueError,msg=change):c.validate_freeze(value)
    def test_measurement_orchestration_uses_only_the_frozen_copy_and_publishes_index_last(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);binary=root/'build.exe';binary.write_bytes(b'fake build')
            source=cost.identity();output=root/'new';seen=[]
            def probe(executable,trace,budget,repetition,directory):
                self.assertEqual(executable,directory/'frozen/probe.exe')
                self.assertFalse((directory/'report.json').exists())
                self.assertFalse((directory/'evidence-index.json').exists())
                manifest=c.read_json(directory/'freeze.json',1048576)
                name=f'{trace}-b{budget}-r{repetition}'
                run_files(directory,manifest,name,trace,budget,repetition)
                result=c.derive_run(directory,manifest,name,trace,budget,repetition)
                (directory/(name+'.process.json')).unlink()
                seen.append(name)
                return result
            with patch.object(cost,'run_probe',side_effect=probe),patch.object(cost,'identity',return_value=source),patch.object(cost,'supports_observation',return_value=True):
                report=cost.measure(binary,COMPILER,output,2,source,c.hash_file(binary,c.MAX_BINARY)['sha256'],BUILD)
            self.assertEqual(report['status'],'Valid',report.get('reason'))
            self.assertEqual(len(seen),12)
            self.assertEqual(c.verify(output)['coverage'],'NeedsEvidence')
            partial=c.read_json(output/'report.partial',16*1024*1024)
            self.assertIs(partial['provisional'],True)
            with self.assertRaises(ValueError):cost.measure(binary,COMPILER,output,2,source,'unused',BUILD)

    def test_publication_or_post_child_hash_failure_keeps_raw_partial_without_valid_index(self):
        with tempfile.TemporaryDirectory() as tmp:
            output,manifest,_,_=setup(Path(tmp))
            for case in c.names():run_files(output,manifest,*case)
            original=c.write_json
            def fail_index(path,value):
                if path.name=='evidence-index.json':raise OSError('index failed')
                return original(path,value)
            with patch.object(c,'write_json',side_effect=fail_index):
                with self.assertRaises(OSError):c.publish(output,manifest)
            self.assertTrue((output/'report.json').exists())
            with self.assertRaises(FileNotFoundError):c.verify(output)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);binary=root/'build.exe';binary.write_bytes(b'fake build');source=cost.identity();output=root/'new'
            def changed(executable,trace,budget,repetition,directory):
                manifest=c.read_json(directory/'freeze.json',1048576);name='g0-b1-r1'
                run_files(directory,manifest,name,trace,budget,repetition)
                result=c.derive_run(directory,manifest,name,trace,budget,repetition)
                (directory/(name+'.process.json')).unlink();executable.write_bytes(b'changed')
                return result
            with patch.object(cost,'run_probe',side_effect=changed),patch.object(cost,'identity',return_value=source),patch.object(cost,'supports_observation',return_value=True):
                report=cost.measure(binary,COMPILER,output,2,source,c.hash_file(binary,c.MAX_BINARY)['sha256'],BUILD)
            self.assertEqual(report['status'],'Invalid');self.assertTrue((output/'report.partial').exists())
            self.assertTrue((output/'g0-b1-r1.child.jsonl').exists());self.assertFalse((output/'evidence-index.json').exists())

    def test_freeze_is_exclusive_and_target_rebuild_does_not_change_custody(self):
        with tempfile.TemporaryDirectory() as tmp:
            output,manifest,binary,target=setup(Path(tmp))
            binary.write_bytes(b'new build')
            self.assertEqual(c.hash_file(target,c.MAX_BINARY)['sha256'],manifest['binary']['sha256'])
            before=(output/'freeze.json').read_bytes()
            with self.assertRaises(FileExistsError):c.write_json(output/'freeze.json',manifest)
            self.assertEqual((output/'freeze.json').read_bytes(),before)
            with self.assertRaises(FileNotFoundError):c.verify(output)

    def test_copy_or_manifest_failure_never_publishes_a_valid_freeze(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);binary=root/'build.exe';binary.write_bytes(b'test')
            source=cost.identity();sha=c.hash_file(binary,c.MAX_BINARY)['sha256']
            with patch.object(c,'write_json',side_effect=OSError('manifest write failure')):
                with self.assertRaises(OSError):c.freeze(binary,COMPILER,source,sha,root/'new',BUILD,lambda:source)
            self.assertFalse((root/'new/freeze.json').exists())
            with patch.object(c.os,'fsync',side_effect=OSError('copy flush failure')):
                with self.assertRaises(OSError):c.freeze(binary,COMPILER,source,sha,root/'failed-copy',BUILD,lambda:source)
            self.assertFalse((root/'failed-copy/freeze.json').exists())
            with self.assertRaises(ValueError):c.freeze(binary,COMPILER,source,sha,root/'changed',BUILD,lambda:{})
            self.assertFalse((root/'changed').exists())

    def test_offline_verification_derives_all_twelve_runs_without_source_build_execute_or_write(self):
        with tempfile.TemporaryDirectory() as tmp:
            output,manifest,binary,target=complete(Path(tmp))
            before=c.inventory(output)
            binary.write_bytes(b'new unrelated build')
            with patch.object(cost,'check_tool',side_effect=AssertionError('build')),patch.object(cost,'identity',side_effect=AssertionError('source read')),\
                 patch.object(cost.subprocess,'Popen',side_effect=AssertionError('execute')),patch.object(cost,'OwnedProcess',side_effect=AssertionError('sample')),\
                 patch.object(c,'write_json',side_effect=AssertionError('write')):
                result=c.verify(output)
            self.assertEqual(result['custody_status'],'Valid');self.assertEqual(result['coverage'],'NeedsEvidence')
            self.assertFalse(result['current_workspace_source_checked']);self.assertEqual(result['verified_runs'],12)
            self.assertEqual(c.inventory(output),before)

    def test_report_hash_alone_cannot_accept_changed_numeric_or_product_conclusions(self):
        with tempfile.TemporaryDirectory() as tmp:
            output,_,_,_=complete(Path(tmp))
            original=c.read_json(output/'report.json',16*1024*1024)
            for field,value in [('coverage','ReadyForReview'),('product_status','Passed')]:
                changed=copy.deepcopy(original);changed[field]=value;replace_json(output/'report.json',changed);reindex(output)
                with self.assertRaises(ValueError):c.verify(output)
            changed=copy.deepcopy(original);changed['runs'][0]['phases'][10]['query']['timing']['p95_ns']=2
            replace_json(output/'report.json',changed);reindex(output)
            with self.assertRaises(ValueError):c.verify(output)

    def test_tampered_copy_raw_missing_end_and_nonzero_exit_remain_invalid_after_reindex(self):
        with tempfile.TemporaryDirectory() as tmp:
            output,_,_,target=complete(Path(tmp))
            data=target.read_bytes();target.write_bytes(data+b'changed');reindex(output)
            with self.assertRaises(ValueError):c.verify(output)
            target.write_bytes(data)
            path=output/'g0-b1-r1.child.jsonl';original=path.read_bytes();path.write_bytes(original.rsplit(b'\n',2)[0]+b'\n');reindex(output)
            with self.assertRaises(ValueError):c.verify(output)
            path.write_bytes(original)
            path=output/'g0-b1-r1.process.json';receipt=c.read_json(path,65536);receipt['exit_code']=1
            replace_json(path,receipt);reindex(output)
            with self.assertRaises(ValueError):c.verify(output)

    def test_partial_missing_index_and_unknown_role_or_path_alias_cannot_verify(self):
        with tempfile.TemporaryDirectory() as tmp:
            output,_,_,_=complete(Path(tmp))
            original=c.read_json(output/'evidence-index.json',65536)
            for name in ('../private','C:/private','//host/private','frozen/../probe.exe','FROZEN/probe.exe','frozen//probe.exe','unknown'):
                value=copy.deepcopy(original);value['files'][0]['path']=name;replace_json(output/'evidence-index.json',value)
                with self.assertRaises(ValueError):c.verify(output)
            replace_json(output/'evidence-index.json',original)
            index=(output/'evidence-index.json').read_bytes();(output/'evidence-index.json').unlink()
            replace_json(output/'report.partial',dict(provisional=True,status='Invalid'))
            with self.assertRaises(FileNotFoundError):c.verify(output)
            (output/'evidence-index.json').write_bytes(index)
            self.assertEqual(c.verify(output)['custody_status'],'Valid')

    def test_strict_json_and_bounds_precede_reads(self):
        for data in (b'{"x":1,"x":2}',b'{"x":NaN}',b'{"x":Infinity}',b'\xff'):
            with self.assertRaises(ValueError):c.decode(data)
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'large';path.write_bytes(b'1234')
            with self.assertRaises(ValueError):c.hash_file(path,3)
            for name in ('../private','a/./b','a\\b','a//b','A/b','a:b','/absolute'):
                with self.assertRaises(ValueError):c.relative(Path(tmp),name)

    def test_reparse_points_are_refused_without_following_target(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'file';path.write_bytes(b'data')
            real=Path.lstat
            class Reparse:
                st_mode=0o100644
                st_file_attributes=0x400
            def info(p):return Reparse() if p==path else real(p)
            with patch.object(Path,'lstat',info):
                with self.assertRaises(ValueError):c.plain_path(path)
            real_stat=Path.stat
            # No symlink privilege or real outside-root file is required by the test.
            resolved=path.resolve()
            def linked(p,*args,**kwargs):
                info=real_stat(p,*args,**kwargs)
                if p==resolved:
                    values=list(info);values[3]=2
                    return type(info)(values)
                return info
            with patch.object(Path,'stat',linked):
                with self.assertRaises(ValueError):c.relative(Path(tmp),'file')

    def test_process_csv_is_streamed_with_global_limit_monotonicity_and_required_boundaries(self):
        with tempfile.TemporaryDirectory() as tmp:
            output,manifest,_,_=setup(Path(tmp));run_files(output,manifest,*next(c.names()))
            path=output/'g0-b1-r1.samples.csv';original=path.read_text()
            phases=c.process_points(path);self.assertEqual(len(phases),15);self.assertEqual(sum(p['samples'] for p in phases),30)
            path.write_text(original.replace('calibrated','ready',1))
            with self.assertRaises(ValueError):c.process_points(path)
            path.write_text(original)
            lines=original.splitlines();lines[2]=lines[1];path.write_text('\n'.join(lines)+'\n')
            with self.assertRaises(ValueError):c.process_points(path)

    def test_verify_cli_returns_before_build_and_rejects_legacy_evidence(self):
        with patch.object(cost.sys,'argv',['check_gameplay_cost.py','--verify','fixture']),\
             patch.object(c,'verify',return_value={'custody_status':'Valid'}) as verify,\
             patch.object(cost,'check_tool',side_effect=AssertionError('build')),patch('sys.stdout',io.StringIO()):
            cost.main();verify.assert_called_once_with(Path('fixture'))
        with tempfile.TemporaryDirectory() as tmp:
            c.write_json(Path(tmp)/'evidence-index.json',dict(schema_version=2,files=[]))
            with self.assertRaises(ValueError):c.verify(Path(tmp))
