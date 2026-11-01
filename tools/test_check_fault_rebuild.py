import copy
import unittest
from measure_fault_rebuild import analyze


def samples():
    return [dict(phase=p,time=i,working_set_bytes=100,private_bytes=200,lifetime_peak_working_set_bytes=120,
                 lifetime_peak_pagefile_bytes=240,gpu_dedicated_bytes=300,gpu_shared_bytes=400)
            for i,p in enumerate(('graphics_active','graphics_active','graphics_rebuilt','graphics_rebuilt'))]


def graphics():
    return dict(schema=4,adapter_utf8=list(b'fixture'),full_editor=False,scope='offscreen-own-device-destroy-rebuild',physical_dpi_minimize=False,
                frame_keys=[1,3],device_lost_reason='Destroyed',inflight_prepare=True,held_targets_after_loss=1,
                old_cpu_output_discarded=True,device_owner_released_before_open=True,parent_context_released_before_open=True,late_draw_request_rejected=True,old_signal_rejected=True,pixels_exact=True,viewport_requests=32,
                final_targets=0,raw_final=0,rgba_final=0,worker_closed=True,shutdown_ok=True)


def audio():
    rows=[[i+1,i%2,i+1,12.,15.,100+i,i+1,0,480] for i in range(64)]
    stats=dict(device_fault=True,callbacks=10,render_errors=0,observation_overflows=0,missing_frames=0,endpoint_returns=1,
               endpoint_returns_outside_render=1,endpoint_returns_in_render=0,callback_resource_releases=0)
    return dict(schema=1,full_editor=False,scope='controlled-native-device-error-reopen',physical_unplug=False,rounds=2,
                rate=48000,channels=2,rows=[rows,copy.deepcopy(rows)],stats=[stats,copy.deepcopy(stats)],pcm_final=0,
                raw_final=0,old_reader_raw_final=0,pcm_peak=3000,raw_peak=2000,old_signal_rejected=True,
                frozen_rebuild=True,copy_anchors=[10,20])


class FaultRebuild(unittest.TestCase):
    def test_explicit_policy_requires_both_devices_and_cannot_reinterpret_legacy(self):
        for expected in (False,True):
            r=dict(graphics(),schema=5,gpu_memory_usage_hint=expected,device_memory_usage_hints=[expected,expected])
            self.assertTrue(analyze(r,samples(),True,memory_usage=expected)['component_passed'])
            with self.assertRaises(ValueError):analyze(r,samples(),True,memory_usage=not expected)
            for hints in ([expected], [expected,not expected], [int(expected),expected], None):
                with self.assertRaises(ValueError):analyze(dict(r,device_memory_usage_hints=hints),samples(),True,memory_usage=expected)
        with self.assertRaises(ValueError):analyze(graphics(),samples(),True,memory_usage=False)

    def test_component_success_cannot_accept_full_window_c4_or_cpu_goal(self):
        for r,g in ((graphics(),True),(audio(),False)):
            v=analyze(r,samples(),g)
            self.assertTrue(v['component_passed']);self.assertFalse(v['c4_accepted']);self.assertFalse(v['p2_accepted'])
            self.assertIsNone(v['cpu_goal_met']);self.assertFalse(v['cpu_meter_independently_validated'])

    def test_missing_fault_scope_and_physical_claim_are_rejected(self):
        r=graphics();r['schema']=3
        with self.assertRaises(ValueError):analyze(r,samples(),True)
        r=graphics();r['physical_dpi_minimize']=True
        with self.assertRaises(ValueError):analyze(r,samples(),True)
        r=audio();r['physical_unplug']=True
        with self.assertRaises(ValueError):analyze(r,samples(),False)
        r['physical_unplug']=False;r['full_editor']=True
        with self.assertRaises(ValueError):analyze(r,samples(),False)

    def test_lost_draw_owner_key_reuse_and_failure_to_join_fail(self):
        for field,value in [('held_targets_after_loss',0),('frame_keys',[1,1]),('worker_closed',False),('rgba_final',1),('old_signal_rejected',False)]:
            r=graphics();r[field]=value;self.assertFalse(analyze(r,samples(),True)['component_passed'])

    def test_unknown_gpu_and_outside_phase_memory_peak_are_not_hidden(self):
        s=samples();s[0]['gpu_shared_bytes']=None
        with self.assertRaises(ValueError):analyze(graphics(),s,True)
        s=samples();s.append(dict(s[0],phase='setup',private_bytes=512*1024**2+1))
        self.assertFalse(analyze(graphics(),s,True)['component_passed'])

    def test_each_rebuild_channel_latency_and_missing_frames_remain_hard(self):
        r=audio()
        for p in r['rows'][1]:
            if p[1]==1:p[3]=21;p[4]=22
        self.assertFalse(analyze(r,samples(),False)['component_passed'])
        r=audio();r['stats'][0]['missing_frames']=1
        self.assertFalse(analyze(r,samples(),False)['component_passed'])

    def test_endpoint_release_anchor_and_exact_voice_evidence_cannot_be_substituted(self):
        for field,value in [('pcm_final',1),('old_reader_raw_final',1),('copy_anchors',[10,0]),('frozen_rebuild',False)]:
            r=audio();r[field]=value;self.assertFalse(analyze(r,samples(),False)['component_passed'])
        r=audio();r['stats'][0]['endpoint_returns_in_render']=1
        self.assertFalse(analyze(r,samples(),False)['component_passed'])
        r=audio();r['rows'][1][1]=r['rows'][1][0]
        with self.assertRaises(ValueError):analyze(r,samples(),False)


if __name__=='__main__':unittest.main()
