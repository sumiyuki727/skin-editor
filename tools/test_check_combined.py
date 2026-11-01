import copy
import io
import json
from pathlib import Path
import queue
import tempfile
import unittest
from unittest.mock import patch
from measure_audio_load import observe
from measure_combined import analyze, pid_total, source_identity, identity_matches, submission_analysis, allocator_analysis, compare_pixel_reports, causal_analysis, preparation_analysis, staging_analysis


def fixture():
    report = dict(original_pngs=100,documents=10,layers=500,seconds=300,triggers=2400,edits=300,switches=20,gpu_memory_usage_hint=False,reusable_staging=False,
                  staging_peak_bytes=0,staging_final_bytes=0,staging_allocations=0,staging_copies=0,staging_remaps=0,
                  latency_upper_bound_ms=[[12]*64,[12]*64],edit_activation_ms=[20]*300,cpu_frame_ms=[2]*1000,
                  submission_intervals_ms=[[[11,12,2,2,0]]*64,[[11,12,2,2,0]]*64],empty_callbacks=0,underrun_observations=[],
                  gpu_allocator_snapshots=[dict(stage='test',supported=False)],
                  native_play_frames=300*48000,rate=48000,empty_frames=0,pcm_peak_bytes=100,raw_peak_bytes=100,
                  rgba_peak_bytes=100,max_targets=2,peak_voices=16,resources_released=True,
                  pcm_final_bytes=0,raw_final_bytes=0,rgba_final_bytes=0,final_targets=0,
                  dual_channel_isolation=True,draft_history_cleanup=True,publications=321,source_uploads=21,source_cache_final_bytes=0,
                  preparation_trace=[dict(started_ms=1,published_ms=11,plan_ms=0,decode_ms=0,pack_ms=0,upload_ms=0,encode_submit_ms=0) for _ in range(321)],
                  slow_ui_ms=[],trace_overflow=False,source_allocations=1)
    samples = [dict(phase=p,time=t,cpu_seconds=t*.1,working_set_bytes=100,private_bytes=100,
                    lifetime_peak_working_set_bytes=100,lifetime_peak_pagefile_bytes=100,
                    gpu_dedicated_bytes=100,gpu_shared_bytes=100)
               for p, span in (("playing",301),("closed",3)) for t in range(span)]
    return report,samples


class CombinedChecks(unittest.TestCase):
    def test_unscoped_staging_modifier_stops_before_opening_a_process(self):
        import measure_combined
        with patch('sys.argv',['measure_combined.py','--binary','unused','--output','unused','--reusable-staging']),patch('sys.stderr',io.StringIO()),patch('measure_combined.subprocess.Popen') as launch:
            with self.assertRaises(SystemExit):measure_combined.main()
            launch.assert_not_called()
    def test_staging_candidate_requires_exact_strategy_and_independent_budget(self):
        r,s=fixture()
        with self.assertRaises(ValueError):analyze(r,s,staging=True)
        r.update(reusable_staging=True,staging_peak_bytes=40*1024**2,staging_allocations=1,staging_copies=21,staging_remaps=20)
        self.assertTrue(all(analyze(r,s,staging=True)['checks'].values()))
        for key,value,gate in (('staging_peak_bytes',81*1024**2,'staging_bounded'),('staging_final_bytes',4,'staging_released'),('staging_allocations',2,'staging_reused'),('staging_remaps',19,'staging_reused')):
            changed=dict(r,**{key:value})
            self.assertFalse(analyze(changed,s,staging=True)['checks'][gate])
        r['latency_upper_bound_ms']=[[21]*64,[21]*64]
        r['submission_intervals_ms']=[[[19,21,2,2,0]]*64]*2
        self.assertFalse(analyze(r,s,staging=True)['checks']['ready_both_lanes_under_graphics_p95_le_20_ms'])

    def test_staging_unknown_counts_and_reference_leases_are_not_zero(self):
        r,s=fixture()
        for key,value in (('staging_final_bytes',None),('staging_copies',1.5),('reusable_staging',0)):
            with self.assertRaises(ValueError):staging_analysis(dict(r,**{key:value}),False)
        r['staging_peak_bytes']=4
        with self.assertRaises(ValueError):staging_analysis(r,False)

    def test_callback_entry_and_commit_do_not_erase_observation_gate_or_overlap(self):
        r,s=fixture()
        row=[2,2.1,2.11,18.1,18.11,768,128,1,33,480,100,4,42]
        r['submission_causal_ms']=[[row.copy() for _ in range(64)] for _ in range(2)]
        r['latency_upper_bound_ms']=[[20.1]*64 for _ in range(2)]
        r['submission_intervals_ms']=[[[18,20.1,2,2,0] for _ in range(64)] for _ in range(2)]
        result=causal_analysis(r)
        self.assertAlmostEqual(result[1]['committed_wait_until_callback_entry_lower_ms']['p95'],15.99)
        self.assertAlmostEqual(result[1]['callback_entry_to_positive_upper_ms']['p95'],2)
        self.assertFalse(analyze(r,s)['checks']['ready_both_lanes_under_graphics_p95_le_20_ms'])
        r['submission_causal_ms'][0][0][3:5]=[1,1.01] # callback started before commit
        self.assertEqual(causal_analysis(r)[0]['callback_entered_before_commit'],1)
        for column,value in ((0,-1),(1,3),(3,21),(8,480),(11,1.5)):
            changed=copy.deepcopy(r);changed['submission_causal_ms'][1][0][column]=value
            with self.assertRaises(ValueError):causal_analysis(changed)

    def test_preparation_lifecycle_separates_worker_and_publish_delay(self):
        row=dict(started_ms=0,worker_started_ms=1,worker_finished_ms=51,received_ms=52,gpu_done_ms=53,published_ms=55,
                 queue_ms=.5,decode_ms=30,pack_ms=8,upload_ms=10,write_ms=8,flush_ms=2,create_ms=0,encode_submit_ms=1,
                 source_lock_ms=0,submission_lock_ms=0,staging_wait_ms=0,staging_alloc_ms=0,staging_copy_ms=0)
        r=dict(preparation_trace=[dict(row,reused=False),dict(row,reused=True)])
        result=preparation_analysis(r)
        self.assertEqual(result['cold']['worker_ms']['p95'],50)
        self.assertEqual(result['cold']['receive_gap_ms']['p95'],1)
        self.assertEqual(result['cold']['gpu_notify_to_publish_ms']['p95'],2)
        for key,value in (('received_ms',50),('gpu_done_ms',None),('queue_ms',-1),('write_ms',float('nan'))):
            changed=copy.deepcopy(r);changed['preparation_trace'][0][key]=value
            with self.assertRaises(ValueError):preparation_analysis(changed)

    def test_memory_policy_cannot_silently_run_the_original_configuration(self):
        r,s=fixture()
        with self.assertRaises(ValueError):analyze(r,s,memory_usage=True)
        r['gpu_memory_usage_hint']=True
        self.assertTrue(all(analyze(r,s,memory_usage=True)['checks'].values()))
        with self.assertRaises(ValueError):analyze(r,s)
        r['gpu_memory_usage_hint']=1
        with self.assertRaises(ValueError):analyze(r,s,memory_usage=True)

    def test_pixel_comparison_rejects_missing_frames_wrong_policy_and_pixel_drift(self):
        r=dict(gpu_memory_usage_hint=False,reusable_staging=False,pixel_trace_sizes=[[256,256]]*6+[[257,193]],adapter='test',max_channel_error=1,unchanged_uploads=0,source_uploads=1,
               paused_draft_refresh=True,cancel_exact_restore=True,branch_restore=True,stale_viewport_rejected=True,shutdown_held_frame_tracked=True,source_buffer_queue_order_verified=True)
        reports=[r,dict(r,gpu_memory_usage_hint=True)];raw=bytes(6*256*256*4+257*193*4)
        self.assertTrue(compare_pixel_reports(reports,[raw,raw])['exact_match'])
        candidates=[dict(r,reusable_staging=True),dict(reports[1],reusable_staging=True)]
        self.assertTrue(compare_pixel_reports(candidates,[raw,raw],staging=True)['exact_match'])
        with self.assertRaises(ValueError):compare_pixel_reports(candidates,[raw,raw])
        with self.assertRaises(ValueError):compare_pixel_reports(reports,[raw,raw],staging=True)
        for traces in ([raw,raw[:-1]],[raw,b'\x01'+raw[1:]]):
            with self.assertRaises(ValueError):compare_pixel_reports(reports,traces)
        for key,value in (('gpu_memory_usage_hint',False),('shutdown_held_frame_tracked',False),('adapter','other'),('pixel_trace_sizes',[[256,256]])):
            with self.assertRaises(ValueError):compare_pixel_reports([r,dict(reports[1],**{key:value})],[raw,raw])

    def test_desktop_rejects_unused_memory_policy_before_toolchain_or_device_access(self):
        import check_desktop
        with patch('sys.argv',['check_desktop.py','--memory-usage']),patch('sys.stderr',io.StringIO()):
            with self.assertRaises(SystemExit) as result:check_desktop.main()
            self.assertEqual(result.exception.code,2)
    def test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples(self):
        class Child:
            returncode=None
            def __init__(self,*args,**kwargs):self.stdout=io.StringIO("")
            def poll(self):return self.returncode
            def kill(self):self.returncode=-9
            def wait(self,timeout):return self.returncode
        class Sampler:
            def __init__(self,child):pass
            def sample(self,phase):return dict(phase=phase,time=0,cpu_seconds=0)
        class EmptyQueue:
            def __init__(self,*args,**kwargs):pass
            def get(self,timeout):raise queue.Empty
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp)/"owned"
            with patch("measure_audio_load.subprocess.Popen",Child), patch("measure_audio_load.queue.Queue",EmptyQueue), patch("measure_audio_load.time.monotonic",side_effect=[0,1]):
                with self.assertRaises(RuntimeError):observe(Path("unused.exe"),directory,"--unused",timeout_seconds=.01,sampler_factory=Sampler)
            recording=json.loads((Path(tmp)/"owned-observation.json").read_text(encoding="utf-8"))
            self.assertEqual(recording["returncode"],-9)
            self.assertEqual(len(recording["samples"]),1)
    def test_recorded_source_identity_rejects_a_changed_or_missing_source(self):
        identity=source_identity()
        self.assertTrue(identity_matches(identity))
        self.assertFalse(identity_matches({"tools/measure_combined.py":"0"*64}))
        self.assertFalse(identity_matches({"tools/no-such-combined-source.rs":"0"*64}))
    def test_complete_standard_run_uses_separate_process_and_gpu_budgets(self):
        result=analyze(*fixture())
        self.assertTrue(all(result["checks"].values()))
        self.assertEqual(result["phases"]["playing"]["cpu_one_logical_core_percent"],10)

    def test_missing_gpu_is_unknown_and_other_pids_are_excluded(self):
        self.assertIsNone(pid_total([("pid_24_a",0,4)],42))
        self.assertEqual(pid_total([("pid_42_a",0,4),("pid_42_b",1,6),("pid_420_a",0,100),("pid_42_c",2,999)],42),10)
        r,s=fixture();s[20]["gpu_dedicated_bytes"]=None
        self.assertFalse(analyze(r,s)["checks"]["gpu_pid_budget"])

    def test_budget_latency_underflow_and_release_failures_are_preserved(self):
        r,s=fixture()
        r.update(empty_frames=192,latency_upper_bound_ms=[[21]*64,[21]*64],resources_released=False,pcm_final_bytes=1,source_allocations=2)
        r['submission_intervals_ms']=[[[20,21,2,2,0]]*64,[[20,21,2,2,0]]*64]
        s[20].update(private_bytes=513*1024**2,gpu_shared_bytes=129*1024**2)
        checks=analyze(r,s)["checks"]
        for key in ("no_underflow_including_startup","ready_both_lanes_under_graphics_p95_le_20_ms","old_resources_released","working_set_private_le_512_mib","gpu_pid_budget","source_allocation_reused"):
            self.assertFalse(checks[key])

    def test_short_schedules_traces_and_clock_spans_are_rejected(self):
        r,s=fixture()
        for key,value in (("triggers",2399),("layers",499),("latency_upper_bound_ms",[[1]*63,[1]*64]),("cpu_frame_ms",[1]*25001)):
            changed=copy.deepcopy(r);changed[key]=value
            with self.assertRaises(ValueError):analyze(changed,s)
        with self.assertRaises(ValueError):analyze(r,[v for v in s if v["time"]<290])

    def test_preparation_overflow_missing_completion_and_long_frame_cannot_be_hidden(self):
        r,s=fixture()
        r["cpu_frame_ms"][0]=50
        result=analyze(r,s)
        self.assertTrue(result["checks"]["cpu_frame_p95_le_16_7_ms"])
        self.assertFalse(result["checks"]["cpu_frame_max_le_16_7_ms"])
        for key,value in (("trace_overflow",True),("preparation_trace",r["preparation_trace"][:-1])):
            changed=copy.deepcopy(r);changed[key]=value
            with self.assertRaises(ValueError):analyze(changed,s)
        r["preparation_trace"][0]["published_ms"]=None
        with self.assertRaises(ValueError):analyze(r,s)

    def test_submission_brackets_distinguish_queue_delay_from_uncertainty_without_passing_borderline(self):
        r,s=fixture()
        r['latency_upper_bound_ms']=[[20.0454]*64,[20.0454]*64]
        r['submission_intervals_ms']=[[[19,20.0454,2,2,0]]*64,[[20.01,20.0454,2,2,0]]*64]
        result=analyze(r,s)
        self.assertFalse(result['checks']['ready_both_lanes_under_graphics_p95_le_20_ms'])
        self.assertEqual([x['p95_classification'] for x in result['submission']],['unresolved','confirmed_over'])
        self.assertEqual(result['submission'][0]['after_receipt_lower_ms']['p95'],17)
        r['submission_intervals_ms'][0][0]=[21,20.0454,2,2,0]
        with self.assertRaises(ValueError):submission_analysis(r)
        r,s=fixture();r['submission_intervals_ms'][0][0]=[0,12,9,10,0]
        self.assertEqual(submission_analysis(r)[0]['p95_classification'],'confirmed_within')
        r['empty_callbacks']=1
        self.assertFalse(analyze(r,s)['checks']['no_underflow_including_startup'])

    def test_allocator_capacity_is_distinct_from_live_allocation_and_unknown_is_explicit(self):
        s=dict(stage='test',supported=True,allocated_bytes=4,reserved_bytes=64,capture_ms=.1,
               allocations=[dict(name_utf8=list(b'source pixels'),bytes=4,offset=0)],blocks=[dict(bytes=64,begin=0,end=1)])
        result=allocator_analysis([s])[0]
        self.assertEqual(result['unused_capacity_bytes'],60)
        self.assertEqual(result['categories'],{'source pixels':4})
        self.assertEqual(allocator_analysis([dict(stage='test',supported=False)])[0],dict(stage='test',supported=False))
        for field,value in (('reserved_bytes',63),('allocated_bytes',5),('blocks',[dict(bytes=64,begin=1,end=1)])):
            changed=copy.deepcopy(s);changed[field]=value
            with self.assertRaises(ValueError):allocator_analysis([changed])


if __name__ == "__main__":unittest.main()
