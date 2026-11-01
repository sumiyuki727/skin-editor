import copy
import unittest
from unittest.mock import patch
from pathlib import Path
from measure_audio_load import observe
from measure_staging_recovery import trend, analyze_recovery
from test_check_combined import fixture


def recovery_fixture():
    base,samples=fixture()
    row=[2,2.1,2.11,10,10.1,768,128,1,33,480,100,4,42]
    base.update(gpu_memory_usage_hint=True,reusable_staging=True,staging_peak_bytes=40*1024**2,
                staging_allocations=1,staging_copies=21,staging_remaps=20,
                submission_causal_ms=[[row.copy() for _ in range(64)] for _ in range(2)])
    cycles=[copy.deepcopy(base) for _ in range(20)]
    for r in cycles[1:]:r.update(seconds=3,triggers=24,edits=3,switches=1,source_uploads=2,staging_copies=2,staging_remaps=1)
    samples=[s for s in samples if s['phase']=='playing']
    for i,phase in enumerate(['paused','closed_initial','cycle_01_settling',*['cycle_%02d_closed'%n for n in range(2,20)],'cycle_20_settling','closed_final']):
        times=range(66) if phase in ('paused','closed_initial','closed_final') else [n/10 for n in range(21)]
        for t in times:
            samples.append(dict(samples[0],phase=phase,time=400+i*100+t,cpu_seconds=(400+i*100+t)*.001))
    report=dict(cycles=20,gpu_memory_usage_hint=True,reusable_staging=True,settle_seconds=10,stable_seconds=65,
                stable_ui_frames=[1,1,1],gpu_allocator_snapshots=[dict(stage=stage,supported=False) for stage in
                    ['cycle_01_paused_start','cycle_01_paused_end']+['cycle_%02d_%s'%(n,suffix) for n in range(1,21) for suffix in ('closed_start','closed_end')]])
    return report,cycles,samples


class RecoveryChecks(unittest.TestCase):
    def test_lifecycle_timeout_remains_bounded_before_launch(self):
        with patch('measure_audio_load.subprocess.Popen') as launch:
            with self.assertRaises(ValueError):observe(Path('unused'),Path('unused'),'unused',timeout_seconds=901)
            launch.assert_not_called()
    def test_allocator_checkpoints_cannot_be_missing_or_unbounded(self):
        r,c,s=recovery_fixture();r['gpu_allocator_snapshots'].pop()
        with self.assertRaises(ValueError):analyze_recovery(r,c,s)
        r,c,s=recovery_fixture();r['gpu_allocator_snapshots']*=2
        with self.assertRaises(ValueError):analyze_recovery(r,c,s)
    def test_short_cycles_cannot_replace_300_seconds_or_missing_lifetimes(self):
        r,c,s=recovery_fixture()
        self.assertTrue(all(analyze_recovery(r,c,s)['checks'].values()))
        with self.assertRaises(ValueError):analyze_recovery(r,c[:-1],s)
        c[0]['seconds']=3
        with self.assertRaises(ValueError):analyze_recovery(r,c,s)

    def test_each_cycle_releases_and_obeys_existing_budget(self):
        for key,value in [('staging_final_bytes',4),('pcm_final_bytes',4),('source_cache_final_bytes',4),
                          ('raw_final_bytes',4),('rgba_final_bytes',4),('final_targets',1),
                          ('empty_frames',192),('cpu_frame_ms',[17]*10),('rgba_peak_bytes',81*1024**2)]:
            with self.subTest(key=key):
                r,c,s=recovery_fixture();c[10][key]=value
                self.assertFalse(analyze_recovery(r,c,s)['checks']['all_cycles_cleanup_and_bounds'])

    def test_stable_cpu_frames_and_short_observation_do_not_get_waived(self):
        r,c,s=recovery_fixture();r['stable_ui_frames']=[1,3,1]
        self.assertFalse(analyze_recovery(r,c,s)['checks']['stable_ui_frames_le_2'])
        r,c,s=recovery_fixture()
        for v in s:
            if v['phase']=='paused':v['cpu_seconds']=v['time']*.021
        self.assertFalse(analyze_recovery(r,c,s)['checks']['stable_cpu_one_core_le_2_percent'])
        s=[v for v in s if v['phase']!='paused' or v['time']<450]
        with self.assertRaises(ValueError):analyze_recovery(r,c,s)

    def test_unknown_gpu_data_excess_memory_and_slow_submission_are_failures(self):
        r,c,s=recovery_fixture();s[-1]['gpu_dedicated_bytes']=None
        self.assertFalse(analyze_recovery(r,c,s)['checks']['all_measured_gpu_budget'])
        r,c,s=recovery_fixture();s[-1]['private_bytes']=513*1024**2
        self.assertFalse(analyze_recovery(r,c,s)['checks']['all_process_memory_le_512_mib'])
        r,c,s=recovery_fixture();c[0]['latency_upper_bound_ms']=[[21]*64]*2;c[0]['submission_intervals_ms']=[[[19,21,2,2,0]]*64]*2
        self.assertFalse(analyze_recovery(r,c,s)['checks']['ready_both_lanes_under_graphics_p95_le_20_ms'])

    def test_trend_preserves_retention_without_an_allowed_leak_cutoff(self):
        flat=trend([100]*20);growing=trend(list(range(100,120)))
        self.assertEqual(flat['slope_per_cycle'],0)
        self.assertEqual(growing['slope_per_cycle'],1)
        self.assertEqual(growing['median_change'],15)
        self.assertEqual(growing['consecutive_increases'],19)
        self.assertNotIn('passed',growing)
        with self.assertRaises(ValueError):trend([100]*19)
    def test_cycle_trend_does_not_substitute_a_long_idle_baseline(self):
        r,c,s=recovery_fixture()
        for v in s:
            if v['phase']=='closed_initial':v['private_bytes']=400
            if v['phase']=='closed_final':v['private_bytes']=50
        result=analyze_recovery(r,c,s)
        self.assertEqual(result['closing_trends']['private_bytes']['values'],[100]*20)
        self.assertEqual(result['stable']['closed_initial']['private_bytes_max'],400)


if __name__=='__main__':unittest.main()
