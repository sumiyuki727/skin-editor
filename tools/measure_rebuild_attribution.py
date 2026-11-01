"""Opt-in own-child rebuild checkpoints; ABBA implicit-layer diagnostic, no UAC.

Metadata only, not allocation stacks, peak acceptance or corrected budget scores.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import os
from pathlib import Path
import queue
import subprocess
import threading
import time
from measure_audio_load import ROOT, WindowsSampler
from measure_combined import GpuSampler, source_identity
from measure_cpu_threads import token_info
from measure_main_work import ordinary
from measure_cpu_accounting import NativeReference

STAGES = ('before_instance','instance_adapter','device','model','pipelines','upload_dispatch',
          'readback_buffer','readback_submitted','readback_mapped','readback_released',
          'pending_upload','host_retired','parents_retired','rebuilt_instance_adapter',
          'rebuilt_device','rebuilt_pipelines','rebuilt_upload_dispatch','readback_buffer',
          'readback_submitted','readback_mapped','rebuilt_readback_released','viewport_storm',
          'shutdown','final_host_retired','final_parents_retired','final_leases_released')
PLAN = ('default','no_implicit','no_implicit','default')


def validate_checkpoint(message, count):
    if (type(message.get('checkpoint')) is not int or message['checkpoint'] != count+1
            or not 0 <= count < len(STAGES) or message.get('stage') != STAGES[count]):
        raise ValueError('stale, reordered or unknown checkpoint')
    allocator = message.get('allocator')
    if allocator is not None:
        if not isinstance(allocator,dict) or type(allocator.get('supported')) is not bool:
            raise ValueError('missing allocator availability')
        if allocator['supported']:
            for key in ('allocated_bytes','reserved_bytes'):
                if type(allocator.get(key)) is not int or allocator[key] < 0:
                    raise ValueError('unknown allocator is not zero')


def region_summary(rows):
    result = {}
    for r in rows:
        key = hex(r['protect'])
        result[key] = result.get(key,0) + r['bytes']
    return result


def region_delta(before, after):
    def identity(r): return (r['base'],r['allocation'],r['bytes'],r['protect'])
    old = {identity(r) for r in before}; new = {identity(r) for r in after}
    return dict(retained_bytes=sum(r[2] for r in old & new),
                new_or_changed_bytes=sum(r[2] for r in new-old),
                gone_or_changed_bytes=sum(r[2] for r in old-new))


def timeline_interval(rows, phase, read, clock):
    """Own the whole observation bracket; never assign a query to its end only."""
    if len(rows)>=65536: raise ValueError('timeline bound exceeded')
    start=clock(); before=phase(); metric=read(before); after=phase(); end=clock()
    if end<start or any(type(metric.get(k)) is not int or metric[k]<=0 for k in ('private_bytes','working_set_bytes')):
        raise ValueError('invalid timeline clock or unknown memory')
    rows.append(dict(start=start,end=end,before=before,after=after,metric=metric))


class MBI(c.Structure):
    _fields_=[('base',c.c_void_p),('allocation',c.c_void_p),('allocation_protect',w.DWORD),
              ('partition',c.c_ushort),('size',c.c_size_t),('state',w.DWORD),('protect',w.DWORD),('kind',w.DWORD)]


class Regions(GpuSampler):
    def __init__(self,child):
        super().__init__(child)
        self.kernel.VirtualQueryEx.argtypes=[w.HANDLE,c.c_void_p,c.POINTER(MBI),c.c_size_t]
        self.kernel.VirtualQueryEx.restype=c.c_size_t
        self.psapi.EnumProcessModulesEx.argtypes=[w.HANDLE,c.POINTER(c.c_void_p),w.DWORD,c.POINTER(w.DWORD),w.DWORD]
        self.psapi.EnumProcessModulesEx.restype=w.BOOL
        self.psapi.GetModuleBaseNameW.argtypes=[w.HANDLE,c.c_void_p,w.LPWSTR,w.DWORD]
        self.psapi.GetModuleBaseNameW.restype=w.DWORD

    def snapshot(self):
        start=time.monotonic(); address=0; rows=[]; count=0
        while address < 0x7fffffffffff:
            if count >= 16384: raise ValueError('region query bound exceeded')
            v=MBI();count+=1
            if not self.kernel.VirtualQueryEx(self.handle,c.c_void_p(address),c.byref(v),c.sizeof(v)):
                # End of application address space; a mid-map failure is unknown.
                if c.get_last_error() != 87: raise OSError(c.get_last_error(),'region query failed')
                break
            if v.state==0x1000 and v.kind==0x20000:
                rows.append(dict(base=v.base or 0,allocation=v.allocation or 0,bytes=v.size,protect=v.protect))
            next_address=(v.base or 0)+v.size
            if next_address<=address: raise ValueError('non advancing address map')
            address=next_address
        slots=(c.c_void_p*1024)();needed=w.DWORD()
        if not self.psapi.EnumProcessModulesEx(self.handle,slots,c.sizeof(slots),c.byref(needed),3):
            raise OSError(c.get_last_error(),'module metadata unavailable')
        if needed.value>c.sizeof(slots): raise ValueError('module count bound exceeded')
        names=[]
        for i in range(needed.value//c.sizeof(c.c_void_p)):
            name=c.create_unicode_buffer(260)
            if not self.psapi.GetModuleBaseNameW(self.handle,slots[i],name,260):
                raise OSError(c.get_last_error(),'module basename unavailable')
            names.append(name.value)
        return dict(region_calls=count,capture_ms=(time.monotonic()-start)*1000,rows=rows,
                    private_committed_sum=sum(r['bytes'] for r in rows),by_protection=region_summary(rows),modules=names)


def run(binary, directory, mode, timeline=False, memory_usage=False):
    environment=dict(os.environ)
    if any(environment.get(k) for k in ('VK_LOADER_LAYERS_DISABLE','VK_LOADER_LAYERS_ENABLE','VK_LOADER_LAYERS_ALLOW','VK_INSTANCE_LAYERS')):
        raise ValueError('inherited layer overrides require an explicit separate plan')
    if mode=='no_implicit': environment['VK_LOADER_LAYERS_DISABLE']='~implicit~'
    elif mode!='default': raise ValueError('unknown layer plan')
    messages=queue.Queue(maxsize=64); overflow=threading.Event(); checkpoints=[]; samples=[]
    directory.parent.mkdir(parents=True,exist_ok=True)
    if directory.exists(): raise ValueError('fresh checkpoint output required')
    with directory.with_suffix('.stderr.log').open('x',encoding='utf8') as errors:
        argument='--verify-graphics-stages-memory' if memory_usage else '--verify-graphics-stages'
        child=subprocess.Popen([str(binary),argument,str(directory)],cwd=ROOT,
                               env=environment,stdout=subprocess.PIPE,stderr=errors,text=True,encoding='utf8')
        def read():
            for line in child.stdout:
                try: messages.put_nowait(json.loads(line))
                except (ValueError,queue.Full): overflow.set()
        reader=threading.Thread(target=read,daemon=True);reader.start()
        sampler=None; failure=None; phase='startup'; deadline=time.monotonic()+120
        stop=threading.Event(); timeline_rows=[]; timeline_errors=[]; timeline_thread=None
        state=['startup']; state_lock=threading.Lock()
        def set_stage(value):
            with state_lock: state[0]=value
        def get_stage():
            with state_lock: return state[0]
        try:
            sampler=Regions(child)
            if not ordinary(token_info(w.HANDLE(int(child._handle)))): raise ValueError('child must remain ordinary')
            if timeline:
                fast=WindowsSampler(child)
                def collect():
                    timer=None
                    try:
                        timer=NativeReference() # existing local high-resolution timer; no global timer setting
                        while not stop.is_set() and child.poll() is None:
                            timeline_interval(timeline_rows,get_stage,fast.sample,time.perf_counter)
                            timer.wait_until(time.perf_counter_ns()+2_000_000)
                    except (OSError,ValueError,RuntimeError) as e: timeline_errors.append(str(e))
                    finally:
                        if timer:
                            try: timer.close()
                            except OSError as e: timeline_errors.append(str(e))
                timeline_thread=threading.Thread(target=collect,daemon=True);timeline_thread.start()
            while child.poll() is None or not messages.empty():
                if time.monotonic()>deadline: raise RuntimeError('bounded stage run timed out')
                try: message=messages.get(timeout=.1)
                except queue.Empty:
                    if child.poll() is None: samples.append(sampler.sample(phase))
                    continue
                if 'checkpoint' not in message:
                    phase=message['phase'];continue
                validate_checkpoint(message,len(checkpoints));phase=message['stage']
                set_stage('held:'+phase)
                if child.poll() is not None: raise RuntimeError('child exited before checkpoint acknowledgement')
                start=time.monotonic()
                metric=sampler.sample(phase); regions=sampler.snapshot(); after=sampler.sample(phase)
                checkpoints.append(dict(message,metric=metric,metric_after=after,regions=regions,
                                        parent_capture_ms=(time.monotonic()-start)*1000))
                # Atomic create: child cannot observe the parent's empty/partial write.
                temporary=directory/'ack.tmp'
                with temporary.open('x',encoding='ascii') as f: f.write(str(message['checkpoint']))
                next_stage=STAGES[len(checkpoints)] if len(checkpoints)<len(STAGES) else 'exit'
                set_stage('between:'+phase+'->'+next_stage)
                temporary.rename(directory/('ack-%d'%message['checkpoint']))
            reader.join(timeout=2)
            if reader.is_alive() or overflow.is_set() or child.returncode or len(checkpoints)!=len(STAGES):
                raise RuntimeError('incomplete stage run or protocol overflow')
        except (OSError,RuntimeError,ValueError,KeyError) as e: failure=str(e)
        finally:
            stop.set()
            if timeline_thread:
                timeline_thread.join(timeout=2)
                if timeline_thread.is_alive(): timeline_errors.append('timeline thread failed to exit')
            if sampler: sampler.close()
            if child.poll() is None: child.kill();child.wait(timeout=5)
            reader.join(timeout=2);child.stdout.close()
    report_path=directory/'native/report.json'
    if timeline_errors and failure is None: failure='; '.join(timeline_errors)
    return dict(mode=mode,checkpoints=checkpoints,samples=samples,error=failure,timeline=timeline_rows,timeline_errors=timeline_errors,
                report=json.loads(report_path.read_text()) if report_path.exists() else None,
                returncode=child.returncode,protocol_overflow=overflow.is_set())


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',required=True);parser.add_argument('--build-receipt',required=True);parser.add_argument('--out',required=True)
    parser.add_argument('--timeline',action='store_true',help='Additional 2 ms own-child memory sampling; diagnostic only')
    args=parser.parse_args();binary=Path(args.binary).resolve();out=Path(args.out).resolve()
    build=json.loads(Path(args.build_receipt).read_text());identity=source_identity()
    sha=hashlib.sha256(binary.read_bytes()).hexdigest()
    if build.get('source')!=identity or build.get('profile')!='release' or build.get('requested_features')!=[] or build.get('repeat_sha256')!=[sha,sha]:
        raise ValueError('current repeatable default release required')
    if not ordinary(token_info()): raise ValueError('ordinary runner required')
    out.mkdir();proof=dict(schema=1,scope='diagnostic stage metadata; not peak acceptance',source=identity,binary_sha256=sha,
                           plan=PLAN,timeline_requested=args.timeline,
                           timeline_clock=dict(implementation=time.get_clock_info('perf_counter').implementation,
                                               resolution=time.get_clock_info('perf_counter').resolution),
                           runs=[],c4_accepted=False,p2_accepted=False,cpu_goal_met=None)
    try:
        for i,mode in enumerate(PLAN):
            result=run(binary,out/str(i+1),mode,args.timeline);proof['runs'].append(result)
            if result['error']: raise RuntimeError(result['error'])
    except (OSError,RuntimeError,ValueError) as e: proof['error']=str(e)
    finally:
        proof['unchanged']=identity==source_identity() and hashlib.sha256(binary.read_bytes()).hexdigest()==sha
        (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if proof.get('error') or not proof['unchanged']: raise SystemExit(1)


if __name__=='__main__':main()
