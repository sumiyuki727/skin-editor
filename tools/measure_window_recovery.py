"""Owned eframe windows only: resize, cross-monitor attempt and minimize/restore.

No screen capture, global settings, UAC or arbitrary process/window control.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import math
import os
from pathlib import Path
import queue
import subprocess
import threading
import time
from measure_audio_load import ROOT, distribution
from measure_combined import GpuSampler, source_identity
from measure_cpu_threads import token_info
from measure_main_work import ordinary


def analyze(report,samples,actions,*,retired_instance=False):
    if (report.get('schema')!=1 or report.get('scope')!='same-process-eframe-window-replacement'
            or report.get('full_editor') is not True or report.get('memory_usage') is not True
            or report.get('in_place_device_reset') is not False or len(report.get('windows',[]))!=2
            or len(report.get('retirements',[]))!=2 or not samples):
        raise ValueError('wrong window recovery scope or incomplete evidence')
    if any(type(s.get(k)) is not int or s[k]<=0 for s in samples for k in ('private_bytes','working_set_bytes')):
        raise ValueError('unknown process memory is not zero')
    checks=dict(process_budget=all(s[k]<=512*1024**2 for s in samples for k in ('private_bytes','working_set_bytes')),
        pixels_and_project=report['pixels_exact'] is True and report['project_unchanged'] is True,
        final_leases=report['raw_final']==report['rgba_final']==report['pcm_final']==0 and report['raw_peak']<=128*1024**2 and report['rgba_peak']<=64*1024**2 and report['pcm_peak']<=32*1024**2)
    latencies=[]
    for index,window in enumerate(report['windows']):
        rows=window['rows']
        if len(rows)!=64 or any(len(p)!=9 for p in rows) or len({p[0] for p in rows})!=64 or len({(p[1],p[2]) for p in rows})!=64:
            raise ValueError('missing exact request/voice rows')
        for p in rows:
            if (any(type(v) not in (int,float) or not math.isfinite(v) or v<0 for v in p)
                    or p[1] not in (0,1) or not 0<=p[3]<=p[4] or not 0<=p[7]<p[8] or min(p[0],p[2],p[5],p[6])<=0):
                raise ValueError('invalid exact callback evidence')
        lanes=[[p[3] for p in rows if p[1]==i] for i in (0,1)]
        if any(len(v)!=32 for v in lanes): raise ValueError('missing audio channel')
        latency=[distribution(v) for v in lanes];latencies.append(latency)
        checks['window_%d'%index]=(window['phase']==14 and window['issued']==64 and window['full_ui_passes']>8
            and window['software_scale'] is True and len(window['screens'])==4 and all(s[2]==s[0]*s[1] and s[3]>=16 for s in window['screens'])
            and window['notice']==1 and window['pending_prepare'] is True and window['old_voices']>0 and window['pixel_error']<=2
            and window['missing_frames']==window['render_errors']==window['observation_overflows']==window['pcm_after_fault']==0
            and window['callbacks']>0 and window['endpoint_returns']==window['endpoint_returns_outside_render']==1
            and window['callback_resource_releases']==window['endpoint_returns_in_render']==0 and all(v['p95']<=20 for v in latency))
        retirement=report['retirements'][index]
        empty=retirement['device_resources_empty'] is True
        if retired_instance and not empty:
            empty=(retirement.get('instance_retired') is True and retirement.get('other_registries_empty') is True
                and retirement.get('texture_kept')==0 and type(retirement.get('texture_ids')) is int
                and retirement['texture_ids']==retirement.get('surface_failures',0)>0
                and retirement['surface_failures']==report['windows'][index].get('fault',{}).get('surface_faults'))
        checks['retirement_%d'%index]=empty and retirement['targets']==0 and all(retirement[k] is True for k in ('renderer_released','surfaces_empty','worker_closed','importer_closed','held_draw_blocked'))
        live=[s for s in samples if s['phase']==('window_before_play' if index==0 else 'window_after_play')]
        if len(live)<2:raise ValueError('no live GPU observations')
        for k,limit in (('dedicated',256),('shared',128)):
            values=[s.get('gpu_'+k+'_bytes') for s in live]
            if any(type(v) is not int or v<0 for v in values):raise ValueError('unknown live GPU is not zero')
            checks['live_%d_%s'%(index,k)]=max(values)<=limit*1024**2
    checks['background_anchor']=report['windows'][1]['restore_anchor']>=report['windows'][0]['pause_anchor']>0
    checks['native_window_actions']=(len(actions)==4 and {(a['action'],a['window']) for a in actions}=={(k,i) for k in ('dpi_move','minimize_restore') for i in (0,1)}
        and sum(a['action']=='minimize_restore' and a['minimized'] and a['restored'] for a in actions)==2)
    checks['dpi_binding']=all(a['scale_changed'] is report['windows'][a['window']]['physical_dpi']
        and abs(report['windows'][a['window']]['native_before']-a['before']/96)<.001
        and abs(report['windows'][a['window']]['native_after']-a['after']/96)<.001 for a in actions if a['action']=='dpi_move')
    gpu={}
    for k,limit in (('dedicated',256),('shared',128)):
        raw=[s.get('gpu_'+k+'_bytes') for s in samples];values=[v for v in raw if type(v) is int and v>=0]
        gpu[k]=dict(max_bytes=max(values) if values else None,unknown=len(raw)-len(values))
        checks['all_known_gpu_'+k]=bool(values) and max(values)<=limit*1024**2
    return dict(checks=checks,component_passed=all(checks.values()),latency_ms=latencies,gpu=gpu,
        memory={k:max(s[k] for s in samples) for k in ('private_bytes','working_set_bytes','lifetime_peak_working_set_bytes','lifetime_peak_pagefile_bytes')},
        physical_dpi_exercised=all(v['physical_dpi'] is True for v in report['windows']),
        c4_accepted=False,p2_accepted=False,cpu_goal_met=None)


def analyze_fault(report,samples,actions,plan,initialization=None):
    expected={'prepare':1,'paint':2,'rebuild-failure':3,'init-consumed':4,'init-prepared':5,'init-audio':6,'cleanup-held':7,'endpoint-held':8,'draft-gpu':9,'draft-audio':10,'draft-both':11}.get(plan)
    if expected is None or report.get('fault_plan')!=expected:
        raise ValueError('fault injection plan missing or mismatched')
    result=analyze(report,samples,actions,retired_instance=True)
    first,second=report['windows'];fault=first.get('fault')
    if not isinstance(fault,dict):raise ValueError('missing real callback fault evidence')
    result['checks']['present_boundary']=(first.get('fault_plan')==expected and fault.get('fired')==1
        and fault.get('prepares',0)>=1 and (plan!='paint' or fault.get('paints',0)>=1)
        and fault.get('noticed',0)>=1 and fault.get('logic_observed')==1
        and fault.get('audio_closed_before_logic')==1 and fault.get('premature_errors')==0
        and fault.get('surface_faults',0)>=1 and fault.get('device_notices',0)>=1)
    result['checks']['fault_errors_classified']=(fault.get('out_of_memory')==0 and fault.get('errors')==
        fault.get('validation_errors',-1)+fault.get('internal_errors',-1)+fault.get('out_of_memory',-1))
    result['checks']['old_draw_rejected']=all(w.get('late_frame_rejected') is True for w in (first,second))
    result['checks']['new_window_healthy']=second.get('fault_plan')==0 and second.get('fault') is None and second.get('previous_frame_rejected') is True
    result['checks']['failed_creation_scope']=report.get('failed_creation_retired') is (plan=='rebuild-failure')
    if expected>=4:
        if not isinstance(initialization,dict) or initialization.get('schema')!=1:
            raise ValueError('missing post-consumption initialization evidence')
        stage=3 if expected>=7 else expected-3
        result['checks']['post_consumption_stage']=(initialization.get('stage')==stage
            and all(initialization.get(k) is True for k in ('consumed','editor_constructed','project_exact','history_exact','observer_closed','audio_closed','renderer_released','registries_empty','instance_retired'))
            and initialization.get('nodes',0)>=3 and initialization.get('branches',0)>=1 and initialization.get('remembered_paths')==1
            and (stage==1 or (initialization.get('prepared') is True and initialization.get('held_draw_blocked') is True)))
        result['checks']['history_path_after_retry']=report.get('history_navigation_verified') is True
        result['checks']['failed_attempt_kept_copy_anchor']=initialization.get('anchor')==first['pause_anchor']
        result['checks']['failed_frame_key_evidence']=(initialization.get('last_frame_key')==0 if stage==1
            else type(initialization.get('last_frame_key')) is int and initialization['last_frame_key']>0)
        if stage==3:
            stats=initialization.get('audio_stats')
            if not isinstance(stats,dict):raise ValueError('failed initialization omitted native audio receipt')
            result['checks']['failed_attempt_audio_health']=(stats.get('callbacks',0)>0 and
                all(stats.get(k)==0 for k in ('missing_frames','render_errors','observation_overflows','endpoint_returns_in_render','callback_resource_releases'))
                and stats.get('endpoint_returns')==stats.get('endpoint_returns_outside_render')==1)
        if expected==7:
            cleanup=initialization.get('cleanup')
            if not isinstance(cleanup,dict):raise ValueError('missing retained native cleanup evidence')
            result['checks']['cleanup_quarantine']=(all(cleanup.get(k) is True for k in (
                'retry_blocked','repeated_refusal','project_exact_while_blocked','history_exact_while_blocked',
                'renderer_released_before_cleanup','runtime_live_before_release','paused','audio_closed','framework_gate_after_runtime','retry_ready_after_release'))
                and cleanup.get('targets_retained',0)>0 and cleanup.get('raw_bytes_retained',0)>0)
        if expected==8:
            endpoint=initialization.get('endpoint_cleanup')
            if not isinstance(endpoint,dict):raise ValueError('missing retained endpoint-return failure evidence')
            result['checks']['endpoint_return_quarantine']=(all(endpoint.get(k) is True for k in (
                'pending_detected','supply_retained','receiver_retained','new_audio_rejected','new_backend_rejected',
                'project_exact_while_pending','history_exact_while_pending','actual_late_delivery','framework_gate_after_runtime'))
                and endpoint.get('repeated_attempts')==3 and endpoint.get('pcm_retained_bytes',0)>0
                and endpoint.get('raw_retained_bytes',0)>0 and endpoint.get('returns_before_delivery')==0
                and endpoint.get('pcm_after_delivery')==0)
        if expected>=9:
            activity=initialization.get('activity')
            if not isinstance(activity,dict):raise ValueError('missing active draft/fault evidence')
            result['checks']['active_edit_interleaving']=(all(activity.get(k) is True for k in (
                'draft_active_before','visual_and_audio_changed','scene_frozen_before','pending_draft_preparation',
                'owned_draft_cancelled','interrupt_idempotent','late_release_rejected','old_draft_frames_rejected',
                'committed_project_exact','history_exact','paused'))
                and activity.get('input_path')=='egui-canvas-drag'
                and activity.get('gpu_fault') is (expected in (9,11))
                and activity.get('audio_fault') is (expected in (10,11))
                and activity.get('audition_voices_before',0)>0
                and activity.get('requests_added',0)>=3
                and initialization.get('draft_request_growth')==activity['requests_added'])
    result['component_passed']=all(result['checks'].values())
    return result


class Windows:
    def __init__(self,pid):
        self.pid=pid;self.u=c.WinDLL('user32',use_last_error=True)
        self.callback=c.WINFUNCTYPE(w.BOOL,w.HWND,w.LPARAM)
        self.u.EnumWindows.argtypes=[self.callback,w.LPARAM]
        self.u.GetWindowThreadProcessId.argtypes=[w.HWND,c.POINTER(w.DWORD)]
        self.u.GetWindowTextW.argtypes=[w.HWND,w.LPWSTR,c.c_int]
        self.u.IsWindow.argtypes=[w.HWND];self.u.IsIconic.argtypes=[w.HWND]
        self.u.GetWindowRect.argtypes=[w.HWND,c.POINTER(w.RECT)]
        self.u.GetDpiForWindow.argtypes=[w.HWND];self.u.GetDpiForWindow.restype=w.UINT
        self.u.SetWindowPos.argtypes=[w.HWND,w.HWND,c.c_int,c.c_int,c.c_int,c.c_int,w.UINT]
        self.u.ShowWindow.argtypes=[w.HWND,c.c_int]

    def owned(self,hwnd):
        pid=w.DWORD();self.u.GetWindowThreadProcessId(hwnd,c.byref(pid))
        if not self.u.IsWindow(hwnd) or pid.value!=self.pid:raise ValueError('window is not owned by this test child')

    def find(self,index):
        found=[];count=[0]
        def visit(hwnd,_):
            count[0]+=1
            if count[0]>4096:return False
            pid=w.DWORD();self.u.GetWindowThreadProcessId(hwnd,c.byref(pid))
            if pid.value==self.pid:
                title=c.create_unicode_buffer(256);self.u.GetWindowTextW(hwnd,title,256)
                if title.value=='Skin Editor - C4 recovery %d'%index:found.append(hwnd)
            return True
        self.u.EnumWindows(self.callback(visit),0)
        if count[0]>4096 or len(found)!=1:raise ValueError('unique owned recovery window not found')
        self.owned(found[0]);return found[0]

    def move_monitor(self,hwnd):
        self.owned(hwnd);rect=w.RECT()
        if not self.u.GetWindowRect(hwnd,c.byref(rect)):raise OSError(c.get_last_error(),'window rect unavailable')
        monitors=[];callback=c.WINFUNCTYPE(w.BOOL,w.HANDLE,w.HDC,c.POINTER(w.RECT),w.LPARAM)
        self.u.EnumDisplayMonitors.argtypes=[w.HDC,c.POINTER(w.RECT),callback,w.LPARAM]
        def monitor(_,__,r,___):
            monitors.append((r.contents.left,r.contents.top,r.contents.right,r.contents.bottom));return len(monitors)<16
        if not self.u.EnumDisplayMonitors(None,None,callback(monitor),0):raise OSError(c.get_last_error(),'monitor metadata unavailable')
        before=self.u.GetDpiForWindow(hwnd)
        target=next((r for r in monitors if not r[0]<=rect.left+20<r[2] or not r[1]<=rect.top+20<r[3]),None)
        if target:
            self.owned(hwnd)
            if not self.u.SetWindowPos(hwnd,None,target[0]+30,target[1]+30,0,0,0x15):raise OSError(c.get_last_error(),'owned window move failed')
            time.sleep(.3) # fixed native event observation; no invented DPI event
        self.owned(hwnd);after=self.u.GetDpiForWindow(hwnd)
        if not before or not after:raise ValueError('native window DPI unavailable')
        return dict(action='dpi_move',before=before,after=after,moved=bool(target),scale_changed=before!=after)

    def restore(self,hwnd,sampler,samples):
        deadline=time.monotonic()+5
        while True:
            self.owned(hwnd)
            if self.u.IsIconic(hwnd):break
            if time.monotonic()>deadline:raise ValueError('native minimization not observed')
            time.sleep(.02)
        for _ in range(3):
            self.owned(hwnd)
            if not self.u.IsIconic(hwnd):raise ValueError('minimized observation interrupted')
            samples.append(sampler.sample('window_minimized'));time.sleep(.1)
        self.owned(hwnd);self.u.ShowWindow(hwnd,4) # restore without activating the test window
        while self.u.IsIconic(hwnd):
            self.owned(hwnd)
            if time.monotonic()>deadline:raise ValueError('native restoration not observed')
            time.sleep(.02)
        return dict(action='minimize_restore',minimized=True,restored=True)


def acknowledge(path,value):
    if path.exists():raise ValueError('checkpoint acknowledgement already exists')
    temporary=path.with_suffix('.tmp')
    with temporary.open('xb') as f:f.write(value)
    temporary.rename(path)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',required=True);parser.add_argument('--build-receipt',required=True);parser.add_argument('--out',required=True)
    parser.add_argument('--fault-plan',choices=('prepare','paint','rebuild-failure','init-consumed','init-prepared','init-audio','cleanup-held','endpoint-held','draft-gpu','draft-audio','draft-both'))
    args=parser.parse_args();binary=Path(args.binary).resolve();out=Path(args.out).resolve();identity=source_identity()
    build=json.loads(Path(args.build_receipt).read_text());sha=hashlib.sha256(binary.read_bytes()).hexdigest()
    if (os.name!='nt' or not ordinary(token_info()) or out.exists() or ROOT/'artifacts' not in out.parents
        or build.get('source')!=identity or build.get('repeat_sha256')!=[sha,sha] or build.get('profile')!='release' or build.get('requested_features')!=[]):
        raise ValueError('fresh ordinary frozen release required')
    out.mkdir();directory=out/'process';messages=queue.Queue(maxsize=64);overflow=threading.Event();samples=[];actions=[]
    proof=dict(schema=1,source=identity,binary_sha256=sha,c4_accepted=False,p2_accepted=False)
    with (out/'stderr.log').open('x',encoding='utf8') as errors:
        command=([str(binary),'--verify-present-fault',str(directory),args.fault_plan] if args.fault_plan
            else [str(binary),'--verify-window-recovery',str(directory)])
        child=subprocess.Popen(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=errors,text=True,encoding='utf8')
        def read():
            for line in child.stdout:
                try:messages.put_nowait(json.loads(line))
                except (ValueError,queue.Full):overflow.set()
        reader=threading.Thread(target=read,daemon=True);reader.start();sampler=None;phase='starting';deadline=time.monotonic()+150
        try:
            if not ordinary(token_info(w.HANDLE(int(child._handle)))):raise ValueError('child unexpectedly elevated')
            sampler=GpuSampler(child);windows=Windows(child.pid)
            while child.poll() is None:
                try:
                    message=messages.get(timeout=.1);phase=message['phase']
                    if phase in ('window_dpi','window_minimize'):
                        index=message['window']
                        if type(index) is not int or index not in (0,1):raise ValueError('wrong window index')
                        hwnd=windows.find(index)
                        if phase=='window_dpi':
                            action=windows.move_monitor(hwnd);acknowledge(directory/('dpi-%d.ok'%index),b'moved' if action['scale_changed'] else b'skipped')
                        else:
                            action=windows.restore(hwnd,sampler,samples);acknowledge(directory/('restore-%d.ok'%index),str(index).encode('ascii'))
                        actions.append(dict(action,window=index))
                except queue.Empty:pass
                try:samples.append(sampler.sample(phase))
                except OSError:
                    if child.poll() is None:raise
                if time.monotonic()>deadline:raise RuntimeError('bounded window recovery timed out')
            reader.join(timeout=2)
            if reader.is_alive() or overflow.is_set() or child.returncode:raise RuntimeError('window recovery failed; inspect owned stderr/failure')
            report=json.loads((directory/'report.json').read_text())
            initialization=json.loads((directory/'initialization.json').read_text()) if args.fault_plan and args.fault_plan.startswith(('init-','cleanup-','endpoint-','draft-')) else None
            result=analyze_fault(report,samples,actions,args.fault_plan,initialization) if args.fault_plan else analyze(report,samples,actions)
            proof.update(report=report,analysis=result,initialization=initialization)
        except (OSError,ValueError,RuntimeError,KeyError) as e:proof['error']=str(e)
        finally:
            if sampler:sampler.close()
            if child.poll() is None:child.kill();child.wait(timeout=5)
            reader.join(timeout=2);child.stdout.close()
            proof.update(samples=samples,actions=actions,returncode=child.returncode,unchanged=source_identity()==identity and hashlib.sha256(binary.read_bytes()).hexdigest()==sha)
            (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if proof.get('error') or not proof['unchanged'] or not proof.get('analysis',{}).get('component_passed'):raise SystemExit(1)


if __name__=='__main__':main()
