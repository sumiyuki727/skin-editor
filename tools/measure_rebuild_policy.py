"""Fixed default/MemoryUsage fault-rebuild controls; owned ordinary Windows PIDs.

Checkpoint diagnostics and uncheckpointed budgets stay separate. No CPU acceptance.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
from measure_audio_load import ROOT, WindowsSampler, observe
from measure_combined import GpuSampler, source_identity
from measure_cpu_threads import token_info
from measure_main_work import ordinary
from measure_fault_rebuild import analyze
from measure_rebuild_attribution import run as stages, STAGES

PLAN=(False,True,True,False)


def pixels(directory):
    a=(directory/'first.rgba').read_bytes();b=(directory/'rebuilt.rgba').read_bytes()
    if len(a)!=16384 or a!=b: raise ValueError('missing or changed fault-rebuild pixels')
    return hashlib.sha256(a).hexdigest()


def pool_analysis(run, memory_usage):
    rows=run['checkpoints'];r=run['report']
    if (run['error'] or r.get('schema')!=5 or r.get('gpu_memory_usage_hint') is not memory_usage
            or r.get('device_memory_usage_hints')!=[memory_usage,memory_usage]
            or any(type(v) is not bool for v in r.get('device_memory_usage_hints',[]))
            or len(rows)!=len(STAGES) or [p['stage'] for p in rows]!=list(STAGES)
            or r.get('full_editor') is not False or r.get('pixels_exact') is not True
            or r.get('worker_closed') is not True or r.get('shutdown_ok') is not True
            or any(r.get(k)!=0 for k in ('raw_final','rgba_final','final_targets'))):
        raise ValueError('incomplete staged policy, pixels or retirement evidence')
    result=[]
    for before,during,after in ((5,6,9),(16,17,20)):
        values=[rows[n]['allocator'] for n in (before,during,after)]
        if any(not isinstance(a,dict) or a.get('supported') is not True for a in values):
            raise ValueError('unknown allocator capacity is not zero')
        for a in values:
            if any(type(a.get(k)) is not int or a[k]<0 for k in ('allocated_bytes','reserved_bytes')):
                raise ValueError('invalid allocator capacity')
        b,d,a=values
        names=lambda value:[bytes(p['name_utf8']).decode('utf8') for p in value['allocations']]
        result.append(dict(before_reserved=b['reserved_bytes'],during_reserved=d['reserved_bytes'],
                           after_reserved=a['reserved_bytes'],reserve_growth=d['reserved_bytes']-b['reserved_bytes'],
                           live_readback=names(d).count('own frame readback')==1,
                           readback_released='own frame readback' not in names(a),
                           empty_capacity_retained=a['reserved_bytes']==d['reserved_bytes'],
                           allocated_returned=a['allocated_bytes']==b['allocated_bytes'],
                           private_growth=rows[during]['metric']['private_bytes']-rows[before]['metric']['private_bytes']))
    return result


def compare_pools(runs):
    if len(runs)!=4 or [r['memory_usage'] for r in runs]!=list(PLAN):
        raise ValueError('fixed four controls required')
    controls=[r['pools'] for r in runs if not r['memory_usage']]
    candidates=[r['pools'] for r in runs if r['memory_usage']]
    if any(len(v)!=2 for v in controls+candidates):raise ValueError('both device incarnations required')
    return dict(
        live_then_released=all(p['live_readback'] and p['readback_released'] and p['allocated_returned'] for r in runs for p in r['pools']),
        retained_capacity=all(p['empty_capacity_retained'] for r in runs for p in r['pools']),
        smaller_first_and_rebuilt_pool=all(max(v[i]['reserve_growth'] for v in candidates)<min(v[i]['reserve_growth'] for v in controls) for i in (0,1)))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',required=True);parser.add_argument('--build-receipt',required=True);parser.add_argument('--out',required=True)
    args=parser.parse_args();binary=Path(args.binary).resolve();out=Path(args.out).resolve()
    build=json.loads(Path(args.build_receipt).read_text());identity=source_identity();sha=hashlib.sha256(binary.read_bytes()).hexdigest()
    if (os.name!='nt' or not ordinary(token_info()) or out.exists() or ROOT/'artifacts' not in out.parents
            or ROOT/'artifacts' not in binary.parents or build.get('source')!=identity
            or build.get('profile')!='release' or build.get('requested_features')!=[] or build.get('repeat_sha256')!=[sha,sha]):
        raise ValueError('ordinary frozen default release and fresh artifact output required')
    out.mkdir();proof=dict(schema=1,scope='offscreen fault-rebuild memory policy controls; not full editor',
        plan=PLAN,source=identity,binary_sha256=sha,staged=[],uncheckpointed=[],pixel_controls=[],
        c4_accepted=False,p2_accepted=False,cpu_goal_met=None,default_policy_changed=False)
    try:
        reference=None
        for i,enabled in enumerate(PLAN,1):
            directory=out/('stage-%d'%i)
            r=stages(binary,directory,'default',True,enabled)
            row=dict(memory_usage=enabled,run=r)
            proof['staged'].append(row)
            row['pools']=pool_analysis(r,enabled);row['pixel_sha256']=pixels(directory/'native')
            reference=reference or row['pixel_sha256']
            if row['pixel_sha256']!=reference:raise ValueError('cross-policy staged pixel mismatch')
        proof['pool_checks']=compare_pools(proof['staged'])
        for i,enabled in enumerate(PLAN,1):
            directory=out/('native-%d'%i)
            report,samples=observe(binary,directory,'--verify-graphics-faults'+('-memory' if enabled else ''),timeout_seconds=90,sampler_factory=GpuSampler)
            result=analyze(report,samples,True,memory_usage=enabled)
            digest=pixels(directory)
            proof['uncheckpointed'].append(dict(memory_usage=enabled,report=report,samples=samples,analysis=result,pixel_sha256=digest))
            if digest!=reference:raise ValueError('cross-policy uncheckpointed pixel mismatch')
            # Budget failures stay in fixed controls. They do not cancel later planned controls.
        for enabled in (False,True):
            directory=out/('pixels-memory' if enabled else 'pixels-default')
            with directory.with_suffix('.log').open('x',encoding='utf8') as f:
                subprocess.run([str(binary),'--verify-gpu-memory' if enabled else '--verify-gpu',str(directory)],cwd=ROOT,stdout=f,stderr=subprocess.STDOUT,check=True,timeout=120)
            report=json.loads((directory/'report.json').read_text());data=(directory/'pixel_trace.rgba').read_bytes()
            proof['pixel_controls'].append(dict(memory_usage=enabled,report=report,trace_sha256=hashlib.sha256(data).hexdigest()))
        traces=proof['pixel_controls'];proof['pixel_controls_exact']=traces[0]['trace_sha256']==traces[1]['trace_sha256']
        proof['pixel_controls_passed']=proof['pixel_controls_exact'] and all(t['report']['max_channel_error']<=2 and t['report']['gpu_memory_usage_hint'] is t['memory_usage'] for t in traces)
        report,samples=observe(binary,out/'audio','--verify-audio-faults',timeout_seconds=90,sampler_factory=WindowsSampler)
        proof['audio']=dict(report=report,samples=samples,analysis=analyze(report,samples,False))
        proof['candidate_component_passed']=(all(proof['pool_checks'].values()) and proof['pixel_controls_passed']
            and all(r['analysis']['component_passed'] for r in proof['uncheckpointed'] if r['memory_usage'])
            and proof['audio']['analysis']['component_passed'])
    except (OSError,ValueError,RuntimeError,KeyError,subprocess.SubprocessError) as e:proof['error']=str(e)
    finally:
        proof['unchanged']=identity==source_identity() and hashlib.sha256(binary.read_bytes()).hexdigest()==sha
        (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if proof.get('error') or not proof['unchanged'] or not proof.get('candidate_component_passed'):raise SystemExit(1)


if __name__=='__main__':main()
