import copy
import json
from pathlib import Path
import tempfile
import unittest
from measure_resource_cycles import analyze,cycle_checks,label_sample,read_phase,trend

def fixture():
    cycles=[];summaries=[]
    for i in range(1,21):
        probes=[[n+1,0 if n<32 else 1,n+1,12.,16.,100+n,100+n,0,480,True] for n in range(64)]
        probes += [[65+n,n,65+n,12.,16.,200+n,200+n,0,480,False] for n in range(2)]
        r=dict(schema=1,index=i,session=str(i),shape=[100,10,500,16],target=(i-1)%10,counts=[32,32,16],probes=probes,
               cpu_frame_ms=[3]*150,resources_peak=[1,1,1,1,1],resources_final=[0]*5,playing_us=6_000_000,native_play_frames=288000,
               background_start=0,background_end=288000,callbacks=600,missing_frames=0,device_fault=False,render_errors=0,observation_overflows=0,
               post_drop_confirmed=True,staging_final_bytes=0,workers_joined=[True]*3,pending_graphics_at_close=True,voices_at_close=[1,1],
               endpoint_returns=1,callback_resource_releases=0,endpoint_returns_in_render=0,endpoint_returns_outside_render=1,peak_voices=9,
               background_fault=False,background_raw_peak=8192,background_raw_final=0,background_read_bytes=288000*4,images_published=16,
               source_allocations=1,source_uploads=1,view_valid=True,geometry=[1180,800,1],max_trigger_lateness_us=100,ready_ms=30,close_ms=20)
        cycles.append(r);summaries.append(dict(index=i,session=str(i),resources_after_drop=[0]*3,workers_joined=[True]*3))
    report=dict(schema=1,scope='current-separated-full-resource-cycles',smoke=False,completed=True,rounds=20,expected_rounds=20,
                gpu_memory_usage_hint=False,reusable_staging=False,rate=48000,channels=2,controlled_focus=True,physical_input_tested=False,
                cycles=summaries,closed_editor_owners=True,all_post_drop_bytes_zero=True,retained_summary_capacity_bytes=8000)
    samples=[];now=0;epoch=0
    phases=[('initial_closed',4)]
    for i in range(1,21):phases.extend([('cycle_%02d_playing'%i,6),('cycle_%02d_closed'%i,4)])
    phases.append(('final_closed',65))
    for phase,seconds in phases:
        epoch+=1
        for tick in range(seconds*10+1):
            at=now+tick/10;identity=dict(step=epoch,phase=phase)
            samples.append(dict(phase=phase,time=at,cpu_seconds=at*.3,private_bytes=100,working_set_bytes=100,lifetime_peak_working_set_bytes=200,
                                lifetime_peak_pagefile_bytes=200,gpu_dedicated_bytes=100,gpu_shared_bytes=100,phase_epoch_valid=True,phase_before=identity,phase_after=identity))
        now+=seconds+1
    return report,cycles,samples

class ResourceCycles(unittest.TestCase):
    def test_production_policy_requires_its_own_identity_and_unchanged_budgets(self):
        r,c,s=fixture()
        with self.assertRaises(ValueError):analyze(r,c,s,product=True)
        r['production_policy']=True;r['gpu_memory_usage_hint']=True
        self.assertTrue(analyze(r,c,s,product=True)['behavior_and_budget_passed'])
        with self.assertRaises(ValueError):analyze(r,c,s)
        s[0]['private_bytes']=512*1024**2+1
        self.assertFalse(analyze(r,c,s,product=True)['behavior_and_budget_passed'])

    def test_full_plan_keeps_cpu_goal_and_trend_acceptance_separate(self):
        r,c,s=fixture();a=analyze(r,c,s)
        self.assertTrue(a['behavior_and_budget_passed']);self.assertFalse(a['c3_accepted']);self.assertFalse(a['p2_accepted'])
        self.assertGreater(a['raw_playing_one_core_percent'],15);self.assertIsNone(a['cpu_goal_met'])
        self.assertEqual(a['private_trend']['first_last_delta'],0)

    def test_missing_or_reused_project_cycle_is_not_twenty_full_reopens(self):
        r,c,s=fixture();c[2]['session']=c[1]['session']
        with self.assertRaises(ValueError):analyze(r,c,s)
        r,c,s=fixture()
        with self.assertRaises(ValueError):analyze(r,c[:-1],s)
        r['completed']=False
        with self.assertRaises(ValueError):analyze(r,c,s)

    def test_no_live_bytes_pending_work_or_missing_join_may_claim_release(self):
        r,c,s=fixture()
        for name,value,gate in [('resources_final',[0,0,1,0,0],'actual_lease_release'),('post_drop_confirmed',False,'actual_lease_release'),
                                ('workers_joined',[True,False,True],'all_owned_workers_joined'),('pending_graphics_at_close',False,'close_exercised_inflight_work'),
                                ('voices_at_close',[0,1],'close_exercised_inflight_work'),('background_raw_final',1,'background_source_closed')]:
            v=copy.deepcopy(c[0]);v[name]=value
            self.assertFalse(cycle_checks(v,1,False,48000)['checks'][gate])

    def test_zero_missing_and_latency_are_still_hard_per_cycle(self):
        r,c,s=fixture();v=copy.deepcopy(c[0]);v['missing_frames']=1
        self.assertFalse(cycle_checks(v,1,False,48000)['checks']['zero_missing_including_startup'])
        v=copy.deepcopy(c[0])
        for p in v['probes']:p[3]=21;p[4]=22
        self.assertFalse(cycle_checks(v,1,False,48000)['checks']['both_channels_copy_p95_le_20_ms'])

    def test_unknown_gpu_and_lifecycle_outliers_cannot_be_hidden_by_phase_changes(self):
        r,c,s=fixture();s[0]['gpu_dedicated_bytes']=None
        self.assertFalse(analyze(r,c,s)['checks']['complete_gpu_budget'])
        r,c,s=fixture();v=label_sample(dict(s[0],private_bytes=512*1024**2+1),dict(step=1,phase='initial_closed'),dict(step=2,phase='cycle_01_loading'))
        s.insert(1,v);self.assertEqual(v['phase'],'transition');self.assertFalse(analyze(r,c,s)['checks']['lifecycle_memory_budget'])

    def test_phase_epochs_reject_boundary_mislabelling_and_reverse(self):
        r,c,s=fixture();s[0]['phase_epoch_valid']=False
        with self.assertRaises(ValueError):analyze(r,c,s)
        r,c,s=fixture();s[-1]['phase_before']=s[0]['phase_before'];s[-1]['phase_after']=s[0]['phase_after']
        with self.assertRaises(ValueError):analyze(r,c,s)

    def test_fixed_baseline_span_and_unknown_closed_counters_are_rejected(self):
        r,c,s=fixture();s=[v for v in s if v['phase']!='cycle_03_closed' or v['time']%5<.2]
        with self.assertRaises(ValueError):analyze(r,c,s)
        r,c,s=fixture()
        for v in s:
            if v['phase']=='cycle_03_closed':v['gpu_shared_bytes']=None
        with self.assertRaises(ValueError):analyze(r,c,s)

    def test_positive_closed_trend_is_reported_without_a_leak_allowance(self):
        a=trend(list(range(20)));self.assertEqual(a['first_last_delta'],19);self.assertEqual(a['median_delta'],15)
        self.assertEqual(a['last_ten_slope'],1);self.assertEqual(a['last_five_slope'],1)
        self.assertNotIn('passed',a)

    def test_phase_file_is_bounded_and_does_not_accept_partial_or_foreign_data(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'phase.json';self.assertIsNone(read_phase(p))
            p.write_text('{');self.assertIsNone(read_phase(p))
            for v in [[],dict(step=True,phase='initial_closed'),dict(step=300,phase='initial_closed'),dict(step=1,phase='foreign'),dict(step=1,phase='x'*300)]:
                p.write_text(json.dumps(v));self.assertIsNone(read_phase(p))
            good=dict(step=5,phase='cycle_01_closed');p.write_text(json.dumps(good));self.assertEqual(read_phase(p),good)

if __name__=='__main__':unittest.main()
