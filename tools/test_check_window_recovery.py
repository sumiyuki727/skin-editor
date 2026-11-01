import copy
from pathlib import Path
import tempfile
import unittest
from measure_window_recovery import analyze,analyze_fault,acknowledge


def fixture():
    rows=[[i+1,i%2,i+1,12.,14.,100+i,i+1,0,480] for i in range(64)]
    window=dict(phase=14,issued=64,full_ui_passes=200,software_scale=True,screens=[[1180,800,944000,64],[980,700,686000,64]]*2,
        notice=1,pending_prepare=True,old_voices=1,pixel_error=1,missing_frames=0,render_errors=0,observation_overflows=0,
        pcm_after_fault=0,callbacks=100,endpoint_returns=1,endpoint_returns_outside_render=1,endpoint_returns_in_render=0,
        callback_resource_releases=0,rows=rows,pause_anchor=100,restore_anchor=None,physical_dpi=False,native_before=1.,native_after=1.)
    retirement=dict(targets=0,renderer_released=True,surfaces_empty=True,device_resources_empty=True,worker_closed=True,importer_closed=True,held_draw_blocked=True)
    report=dict(schema=1,scope='same-process-eframe-window-replacement',full_editor=True,memory_usage=True,in_place_device_reset=False,
        windows=[window,dict(copy.deepcopy(window),restore_anchor=100,pause_anchor=200)],retirements=[retirement,copy.deepcopy(retirement)],
        pixels_exact=True,project_unchanged=True,raw_final=0,rgba_final=0,pcm_final=0,raw_peak=100,rgba_peak=100,pcm_peak=100)
    samples=[dict(phase=p,private_bytes=100,working_set_bytes=100,lifetime_peak_working_set_bytes=100,lifetime_peak_pagefile_bytes=100,gpu_dedicated_bytes=100,gpu_shared_bytes=100)
        for p in ('window_before_play','window_before_play','window_after_play','window_after_play')]
    actions=[dict(action='dpi_move',window=i,before=96,after=96,scale_changed=False) for i in (0,1)]+[dict(action='minimize_restore',window=i,minimized=True,restored=True) for i in (0,1)]
    return report,samples,actions


def initialization_fixture(plan='init-audio'):
    r,s,a=fixture();code={'init-consumed':4,'init-prepared':5,'init-audio':6}[plan]
    r.update(fault_plan=code,failed_creation_retired=False,history_navigation_verified=True)
    for w in r['windows']:w.update(late_frame_rejected=True,fault=None,fault_plan=0,previous_frame_rejected=True)
    r['windows'][0].update(fault_plan=code,fault=dict(fired=1,prepares=1,paints=0,noticed=4,logic_observed=1,audio_closed_before_logic=1,
        premature_errors=0,errors=0,validation_errors=0,internal_errors=0,out_of_memory=0,surface_faults=3,device_notices=1))
    init=dict(schema=1,stage=code-3,consumed=True,editor_constructed=True,project_exact=True,history_exact=True,observer_closed=True,
        audio_closed=True,renderer_released=True,registries_empty=True,instance_retired=True,nodes=6,branches=1,remembered_paths=1,
        prepared=code>=5,held_draw_blocked=code>=5,anchor=100,last_frame_key=50 if code>=5 else 0,
        audio_stats=dict(callbacks=2,missing_frames=0,render_errors=0,observation_overflows=0,endpoint_returns_in_render=0,callback_resource_releases=0,endpoint_returns=1,endpoint_returns_outside_render=1) if code==6 else None)
    return r,s,a,plan,init


class WindowRecovery(unittest.TestCase):
    def test_active_edit_requires_real_input_cancel_and_each_declared_fault_order(self):
        for plan,code in [('draft-gpu',9),('draft-audio',10),('draft-both',11)]:
            r,s,a,_,i=initialization_fixture();r['fault_plan']=code;r['windows'][0]['fault_plan']=code
            i['draft_request_growth']=4
            i['activity']=dict(input_path='egui-canvas-drag',gpu_fault=code in (9,11),audio_fault=code in (10,11),
                draft_active_before=True,visual_and_audio_changed=True,scene_frozen_before=True,audition_voices_before=1,
                pending_draft_preparation=True,owned_draft_cancelled=True,interrupt_idempotent=True,late_release_rejected=True,
                old_draft_frames_rejected=True,committed_project_exact=True,history_exact=True,requests_added=4,paused=True)
            self.assertTrue(analyze_fault(r,s,a,plan,i)['component_passed'])
            for key,value in i['activity'].items():
                bad=copy.deepcopy(i);bad['activity'][key]=not value if type(value) is bool else (0 if type(value) is int else 'wrong')
                self.assertFalse(analyze_fault(r,s,a,plan,bad)['component_passed'],key)
            i['draft_request_growth']=0
            self.assertFalse(analyze_fault(r,s,a,plan,i)['component_passed'])
    def test_endpoint_failure_requires_actual_retained_owners_gating_and_late_delivery(self):
        r,s,a,_,i=initialization_fixture();r['fault_plan']=8;r['windows'][0]['fault_plan']=8
        i['endpoint_cleanup']=dict(pending_detected=True,supply_retained=True,receiver_retained=True,new_audio_rejected=True,
            new_backend_rejected=True,repeated_attempts=3,project_exact_while_pending=True,history_exact_while_pending=True,
            pcm_retained_bytes=100,raw_retained_bytes=100,returns_before_delivery=0,actual_late_delivery=True,
            pcm_after_delivery=0,framework_gate_after_runtime=True)
        self.assertTrue(analyze_fault(r,s,a,'endpoint-held',i)['component_passed'])
        for key,value in i['endpoint_cleanup'].items():
            bad=copy.deepcopy(i)
            bad['endpoint_cleanup'][key]=False if type(value) is bool else (1 if value==0 else 0)
            self.assertFalse(analyze_fault(r,s,a,'endpoint-held',bad)['component_passed'],key)
        i.pop('endpoint_cleanup')
        with self.assertRaises(ValueError):analyze_fault(r,s,a,'endpoint-held',i)
    def test_cleanup_cannot_unlock_retry_until_real_retained_owners_are_gone(self):
        r,s,a,_,i=initialization_fixture();r['fault_plan']=7;r['windows'][0]['fault_plan']=7
        i['cleanup']=dict(retry_blocked=True,repeated_refusal=True,project_exact_while_blocked=True,history_exact_while_blocked=True,
            targets_retained=1,raw_bytes_retained=100,rgba_bytes_while_blocked=0,renderer_released_before_cleanup=True,
            runtime_live_before_release=True,paused=True,audio_closed=True,framework_gate_after_runtime=True,retry_ready_after_release=True)
        self.assertTrue(analyze_fault(r,s,a,'cleanup-held',i)['component_passed'])
        for key in i['cleanup']:
            if key=='rgba_bytes_while_blocked':continue
            bad=copy.deepcopy(i);bad['cleanup'][key]=False if type(bad['cleanup'][key]) is bool else 0
            self.assertFalse(analyze_fault(r,s,a,'cleanup-held',bad)['component_passed'],key)
        i.pop('cleanup')
        with self.assertRaises(ValueError):analyze_fault(r,s,a,'cleanup-held',i)
    def test_post_consumption_requires_exact_history_and_each_cleanup_owner(self):
        r,s,a,p,i=initialization_fixture();self.assertTrue(analyze_fault(r,s,a,p,i)['component_passed'])
        for key in ('consumed','editor_constructed','project_exact','history_exact','observer_closed','audio_closed','renderer_released','registries_empty','instance_retired','prepared','held_draw_blocked'):
            bad=copy.deepcopy(i);bad[key]=False;self.assertFalse(analyze_fault(r,s,a,p,bad)['component_passed'])
        with self.assertRaises(ValueError):analyze_fault(r,s,a,p)

    def test_initialization_stage_copy_anchor_and_redo_path_are_independent_requirements(self):
        for plan in ('init-consumed','init-prepared','init-audio'):
            r,s,a,p,i=initialization_fixture(plan);self.assertTrue(analyze_fault(r,s,a,p,i)['component_passed'])
            bad=copy.deepcopy(i);bad['stage']=9;self.assertFalse(analyze_fault(r,s,a,p,bad)['component_passed'])
            bad=copy.deepcopy(i);bad['anchor']=0;self.assertFalse(analyze_fault(r,s,a,p,bad)['component_passed'])
            r['history_navigation_verified']=False;self.assertFalse(analyze_fault(r,s,a,p,i)['component_passed'])
        r,s,a,p,i=initialization_fixture();i['audio_stats']['missing_frames']=1
        self.assertFalse(analyze_fault(r,s,a,p,i)['component_passed'])
    def test_injection_requires_actual_boundary_notice_and_pre_gui_audio_gate(self):
        r,s,a=fixture();r['fault_plan']=2;r['failed_creation_retired']=False
        for w in r['windows']:w.update(late_frame_rejected=True,fault=None,fault_plan=0,previous_frame_rejected=True)
        f=dict(fired=1,prepares=1,paints=1,noticed=1,logic_observed=1,audio_closed_before_logic=1,premature_errors=0,errors=2,validation_errors=2,internal_errors=0,out_of_memory=0,surface_faults=1,device_notices=1)
        r['windows'][0].update(fault_plan=2,fault=f)
        self.assertTrue(analyze_fault(r,s,a,'paint')['component_passed'])
        for key in ('fired','prepares','paints','noticed','logic_observed','audio_closed_before_logic','surface_faults','device_notices'):
            copy_r=copy.deepcopy(r);copy_r['windows'][0]['fault'][key]=0
            self.assertFalse(analyze_fault(copy_r,s,a,'paint')['component_passed'])
        r['windows'][0]['fault']['premature_errors']=1
        self.assertFalse(analyze_fault(r,s,a,'paint')['component_passed'])

    def test_failed_creation_and_late_draw_cannot_be_inferred_from_regular_reopen(self):
        r,s,a=fixture();r['fault_plan']=3;r['failed_creation_retired']=False
        for w in r['windows']:w.update(late_frame_rejected=True,fault=None,fault_plan=0,previous_frame_rejected=True)
        r['windows'][0].update(fault_plan=3,fault=dict(fired=1,prepares=1,noticed=1,logic_observed=1,audio_closed_before_logic=1,premature_errors=0,errors=2,validation_errors=2,internal_errors=0,out_of_memory=0,surface_faults=1,device_notices=1))
        self.assertFalse(analyze_fault(r,s,a,'rebuild-failure')['component_passed'])
        r['failed_creation_retired']=True;self.assertTrue(analyze_fault(r,s,a,'rebuild-failure')['component_passed'])
        r['windows'][1]['late_frame_rejected']=False;self.assertFalse(analyze_fault(r,s,a,'rebuild-failure')['component_passed'])
        with self.assertRaises(ValueError):analyze_fault(r,s,a,'paint')

    def test_surface_id_slots_are_not_live_textures_or_a_blanket_retirement_exception(self):
        r,s,a=fixture();r['fault_plan']=1;r['failed_creation_retired']=False
        for w in r['windows']:w.update(late_frame_rejected=True,fault=None,fault_plan=0,previous_frame_rejected=True)
        r['windows'][0].update(fault_plan=1,fault=dict(fired=1,prepares=1,noticed=1,logic_observed=1,audio_closed_before_logic=1,premature_errors=0,errors=0,validation_errors=0,internal_errors=0,out_of_memory=0,surface_faults=3,device_notices=1))
        q=r['retirements'][0];q.update(device_resources_empty=False,instance_retired=True,other_registries_empty=True,texture_kept=0,texture_ids=3,surface_failures=3)
        self.assertTrue(analyze_fault(r,s,a,'prepare')['component_passed'])
        self.assertFalse(analyze(r,s,a)['component_passed'])
        for key,value in [('instance_retired',False),('other_registries_empty',False),('texture_kept',1),('texture_ids',4),('surface_failures',2)]:
            bad=copy.deepcopy(r);bad['retirements'][0][key]=value
            self.assertFalse(analyze_fault(bad,s,a,'prepare')['component_passed'])

    def test_full_component_success_does_not_accept_physical_dpi_or_in_place_reset(self):
        result=analyze(*fixture());self.assertTrue(result['component_passed']);self.assertFalse(result['physical_dpi_exercised'])
        self.assertFalse(result['c4_accepted']);self.assertFalse(result['p2_accepted']);self.assertIsNone(result['cpu_goal_met'])

    def test_each_retirement_owner_and_final_resource_is_required(self):
        for key in ('renderer_released','surfaces_empty','device_resources_empty','worker_closed','importer_closed','held_draw_blocked'):
            r,s,a=fixture();r['retirements'][0][key]=False;self.assertFalse(analyze(r,s,a)['component_passed'])
        for key in ('raw_final','rgba_final','pcm_final'):
            r,s,a=fixture();r[key]=1;self.assertFalse(analyze(r,s,a)['component_passed'])

    def test_latency_underrun_memory_and_unknown_gpu_remain_hard(self):
        r,s,a=fixture();r['windows'][1]['rows'][0][3:5]=[21.,22.]
        # More than one tail sample is needed to exceed a p95 gate.
        for p in r['windows'][1]['rows']:
            if p[1]==0:p[3:5]=[21.,22.]
        self.assertFalse(analyze(r,s,a)['component_passed'])
        r,s,a=fixture();r['windows'][0]['missing_frames']=1;self.assertFalse(analyze(r,s,a)['component_passed'])
        r,s,a=fixture();s.append(dict(s[0],phase='teardown',private_bytes=512*1024**2+1));self.assertFalse(analyze(r,s,a)['component_passed'])
        r,s,a=fixture();s[0]['gpu_shared_bytes']=None
        with self.assertRaises(ValueError):analyze(r,s,a)

    def test_old_receipts_duplicate_actions_or_unproven_dpi_do_not_pass(self):
        r,s,a=fixture();r['windows'][0]['rows'][1]=r['windows'][0]['rows'][0]
        with self.assertRaises(ValueError):analyze(r,s,a)
        r,s,a=fixture();a[-1]=dict(a[-2]);self.assertFalse(analyze(r,s,a)['component_passed'])
        r,s,a=fixture();r['windows'][0]['physical_dpi']=True;self.assertFalse(analyze(r,s,a)['component_passed'])

    def test_ack_creation_is_atomic_and_never_overwrites_existing_ack(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'restore-0.ok';acknowledge(p,b'0');self.assertEqual(p.read_bytes(),b'0')
            with self.assertRaises(ValueError):acknowledge(p,b'wrong')
            self.assertEqual(p.read_bytes(),b'0')
            self.assertFalse(p.with_suffix('.tmp').exists())


if __name__=='__main__':unittest.main()
