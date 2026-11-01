import unittest
from measure_current_combined import analyze


def fixture():
    probes = [[i+1, 0 if i < 2400 else 1, i+1, 12., 17., 100+i, i+1, 0, 480, True] for i in range(2464)]
    probes += [[2465+i, 1, 2465+i, 12., 17., 3000+i, 3000+i, 0, 480, False] for i in range(3)]
    r = dict(schema=1, scope="current-separated-streamed-standard-N", completed=True, seconds=300,
             shape=[100,10,500,16], counts=[2400,64,300,20], gpu_memory_usage_hint=False, reusable_staging=False,
             rate=48000, channels=2, controlled_focus=True, normal_auto_controls=True, physical_input_tested=False,
             probes=probes, edit_activation_ms=[20]*300, switch_activation_ms=[25]*20, cpu_frame_ms=[2]*1000,
             hot_continuity=[[i, 1+i, 2+i] for i in range(300)], controls=[True]*6,
             preparation_trace=[dict(started_ms=1, published_ms=2, gpu_done_ms=1.5, decode_ms=0, upload_ms=0)]*322,
             resources_peak=[1,1,1,1,1], resources_final=[0]*5, playing_us=300_000_000, native_play_frames=300*48000,
             native=dict(callbacks=30000, missing_frames=0, render_errors=0, observation_overflows=0,
                         device_fault=False, nonzero_samples=100, endpoint_returns=1,
                         callback_resource_releases=0, endpoint_returns_in_render=0, endpoint_returns_outside_render=1),
             background=dict(start=0, end=300*48000, continuous=True, frames=360*48000, fault=False,
                             raw_peak_bytes=8192, raw_final_bytes=0, read_bytes=300*48000*2*2),
             max_voices=16, staging_final_bytes=0, source_allocations=1, source_uploads=21,
             publications=322, trace_overflow=False, view_valid=True, geometry=[1180,800,1], max_trigger_lateness_us=100)
    s = [dict(phase=phase, time=t, cpu_seconds=t*.2, working_set_bytes=100, private_bytes=100,
              lifetime_peak_working_set_bytes=1000, lifetime_peak_pagefile_bytes=2000,
              gpu_dedicated_bytes=100, gpu_shared_bytes=100) for phase, span in (("playing",301),("closed",3)) for t in range(span)]
    return r,s


class CurrentCombinedChecks(unittest.TestCase):
    def test_cpu_target_is_nonblocking_but_not_inferred_from_raw_counts(self):
        r,s=fixture();a=analyze(r,s)
        self.assertTrue(a['hard_passed']);self.assertFalse(a['cpu_raw_numeric_le_15'])
        self.assertIsNone(a['cpu_goal_met']);self.assertFalse(a['cpu_meter_independently_validated'])
        self.assertFalse(a['p2_accepted'])
        for v in s:v['cpu_seconds']=0
        a=analyze(r,s);self.assertTrue(a['cpu_raw_numeric_le_15']);self.assertIsNone(a['cpu_goal_met'])

    def test_refuses_legacy_route_incomplete_schedule_and_early_completion(self):
        for field,value in [('scope','legacy'),('counts',[2399,64,300,20]),('completed',False),('shape',[100,10,499,16])]:
            r,s=fixture();r[field]=value
            with self.subTest(field=field),self.assertRaises(ValueError):analyze(r,s)

    def test_exact_probes_reject_duplicates_wrong_channels_and_invalid_times(self):
        for mutate in [lambda r:r['probes'].__setitem__(1,r['probes'][0]),
                       lambda r:r['probes'][0].__setitem__(1,1),
                       lambda r:r['probes'][0].__setitem__(3,float('nan')),
                       lambda r:r['probes'][0].__setitem__(4,11)]:
            r,s=fixture();mutate(r)
            with self.assertRaises(ValueError):analyze(r,s)

    def test_zero_underrun_latency_and_callback_ownership_are_hard(self):
        for field,value,gate in [('missing_frames',1,'zero_missing_including_startup'),
                                 ('device_fault',True,'native_healthy'),
                                 ('callback_resource_releases',1,'callback_ownership')]:
            r,s=fixture();r['native'][field]=value;self.assertFalse(analyze(r,s)['hard_checks'][gate])
        r,s=fixture()
        for p in r['probes']:p[3]=21;p[4]=22
        self.assertFalse(analyze(r,s)['hard_checks']['complete_request_copy_p95_le_20_ms'])

    def test_actual_resource_release_and_all_lifecycle_memory_cannot_be_waived(self):
        r,s=fixture();r['resources_final'][0]=1
        self.assertFalse(analyze(r,s)['hard_checks']['actual_managed_release'])
        r,s=fixture();s[0]['private_bytes']=512*1024**2+1
        self.assertFalse(analyze(r,s)['hard_checks']['all_lifecycle_sampled_memory_le_512_mib'])
        r,s=fixture();r['resources_peak'][0]=32*1024**2+1
        self.assertFalse(analyze(r,s)['hard_checks']['bounded_managed_resources'])

    def test_unknown_gpu_and_above_budget_are_not_zero(self):
        for value in (None,256*1024**2+1):
            r,s=fixture();s[0]['gpu_dedicated_bytes']=value
            self.assertFalse(analyze(r,s)['hard_checks']['gpu_pid_budget'])

    def test_clock_continuity_and_publication_completeness(self):
        r,s=fixture();r['hot_continuity'][5][2]=0
        with self.assertRaises(ValueError):analyze(r,s)
        r,s=fixture();r['playing_us']-=1
        self.assertFalse(analyze(r,s)['hard_checks']['continuous_background_and_native_300s'])
        r,s=fixture();r['controls'][4]=False
        self.assertFalse(analyze(r,s)['hard_checks']['current_commit_pause_cancel_history'])
        r,s=fixture();r['edit_activation_ms']=[51]*300
        self.assertFalse(analyze(r,s)['hard_checks']['edit_to_published_gpu_p95_le_50_ms'])
        r,s=fixture();r['cpu_frame_ms']=[17]*1000
        self.assertFalse(analyze(r,s)['hard_checks']['cpu_frame_p95_le_16_7_ms'])


class ProductIdentity(unittest.TestCase):
    def test_product_policy_is_explicit_and_keeps_all_hard_checks(self):
        r,s=fixture()
        with self.assertRaises(ValueError): analyze(r,s,product=True)
        r['production_policy']=True;r['gpu_memory_usage_hint']=True
        self.assertTrue(analyze(r,s,product=True)['hard_passed'])
        with self.assertRaises(ValueError): analyze(r,s)
        r['native']['missing_frames']=1
        self.assertFalse(analyze(r,s,product=True)['hard_passed'])

if __name__ == '__main__':
    unittest.main()
