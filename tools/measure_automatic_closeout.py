"""Fixed ordinary-permission P2 product-policy checks, with no human procedures.

Does not invoke ETW/UAC, manipulate external clients, accept physical inputs, or
claim B05 CPU validity. Failures are retained; no automatic repeat is attempted.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

from measure_audio_load import ROOT, observe, summarize
from measure_background_stream import stream_checks, audit_checks
from measure_combined import GpuSampler, source_identity
from measure_cpu_threads import token_info
from measure_gui_candidate import evaluate_gui
from measure_main_work import ordinary
from measure_stable_window import report_checks


def process_boundary(sample, before, after):
    return dict(sample, process_alive_before=before, process_alive_after=after)


class LifecycleSampler(GpuSampler):
    def __init__(self, child):
        self.child = child
        super().__init__(child)

    def sample(self, phase):
        before = self.child.poll() is None
        value = super().sample(phase)
        return process_boundary(value, before, self.child.poll() is None)


def gpu_budget(samples):
    result = {}
    for name, limit in (("dedicated", 256), ("shared", 128)):
        values = [s.get("gpu_"+name+"_bytes") for s in samples]
        valid = [v for v in values if type(v) is int and v >= 0]
        # observe() can enter its final sample before wait(timeout=.1), then
        # sample after the owned PID has exited. Retain that unknown observation
        # as an exit boundary, never assign it zero or call it a live closed span.
        unknown_exit=0;unknown_initial=0;seen_known=False
        for s,v in zip(samples,values):
            if type(v) is int and v>=0:
                seen_known=True
            elif s.get('process_alive_before') is False or s.get('process_alive_after') is False:
                unknown_exit+=1
            elif not seen_known and s.get('phase')=='loading':
                # A bounded contiguous pre-observation loading prefix. As in
                # N/C3, this stays unknown; no peak or zero is inferred here.
                unknown_initial+=1
        unknown_live = len(values)-len(valid)-unknown_exit-unknown_initial
        result[name] = dict(known=len(valid), missing=len(values)-len(valid),
                            unknown_exit_boundary=unknown_exit, unknown_live=unknown_live,
                            unknown_initial_loading=unknown_initial,
                            max_bytes=max(valid) if valid else None,
                            passed=bool(valid) and unknown_live==0 and max(valid)<=limit*1024**2)
    return result


def stable_result(report, samples, resources):
    if report.get('production_policy') is not True or report.get('gpu_memory_usage_hint') is not True:
        raise ValueError('ordinary Windows product policy missing')
    checks = report_checks(report, 'stable-resources' if resources else 'stable-empty')
    phases = ('paused','closed') if resources else ('empty',)
    metrics = summarize(samples, phases, tuple((p,60) for p in phases))
    peaks = {k:max(s[k] for s in samples) for k in ('private_bytes','working_set_bytes')}
    checks['lifecycle_memory_budget'] = all(0<v<=512*1024**2 for v in peaks.values())
    if not resources:
        checks['empty_working_set_le_200_mib'] = peaks['working_set_bytes']<=200*1024**2
    gpu = gpu_budget(samples)
    checks['known_complete_gpu_budget'] = all(v['passed'] for v in gpu.values())
    return dict(checks=checks, behavior_and_memory_passed=all(checks.values()), gpu=gpu,
                memory_peak=peaks, raw_cpu=metrics, b05_cpu_accepted=False,
                independent_cpu_validated=False, physical_input_tested=False,
                remaining='B05 independent CPU and physical acceptance require separately confirmed procedures')


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',required=True);p.add_argument('--build-receipt',required=True)
    p.add_argument('--output',required=True)
    p.add_argument('--continue-after',help='Reviewed interrupted plan; retain its failed Q and reuse only accepted identical-runtime N/C3')
    args=p.parse_args();binary=Path(args.binary).resolve();out=(ROOT/args.output).resolve()
    receipt=json.loads(Path(args.build_receipt).read_text(encoding='utf8'))
    sha=hashlib.sha256(binary.read_bytes()).hexdigest();identity=source_identity()
    if (os.name!='nt' or not ordinary(token_info()) or out.exists() or ROOT/'artifacts' not in out.parents
            or ROOT/'artifacts' not in binary.parents or receipt.get('profile')!='release'
            or receipt.get('requested_features')!=[] or receipt.get('source')!=identity
            or receipt.get('repeat_sha256')!=[sha,sha]):
        raise ValueError('ordinary frozen default-feature release and fresh artifact directory required')
    out.mkdir(parents=True)
    proof=dict(schema=1,source=identity,binary_sha256=sha,production_policy=True,
               plan=['two-300s-standard','twenty-full-cycles','two-streamed-Q',
                     'two-empty-two-paused-closed','eleven-window-faults','two-pixel-controls','offscreen-fault'],
               steps=[],automatic_passed=False,p2_accepted=False,human_acceptance_started=False)
    def save():
        proof['unchanged']=identity==source_identity() and sha==hashlib.sha256(binary.read_bytes()).hexdigest()
        (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    def unchanged():
        if identity!=source_identity() or sha!=hashlib.sha256(binary.read_bytes()).hexdigest():
            raise ValueError('source/binary changed during fixed plan')
    def command(name, command, timeout):
        unchanged();print('Starting '+name,flush=True)
        with (out/(name+'.log')).open('x',encoding='utf8') as log:
            result=subprocess.run(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,timeout=timeout)
        proof['steps'].append(dict(name=name,returncode=result.returncode));save()
        if result.returncode:raise ValueError(name+' failed; no retry')
    common=['--binary',str(binary),'--build-receipt',args.build_receipt]
    try:
        if args.continue_after:
            prior=Path(args.continue_after).resolve()
            old=json.loads((prior/'proof.json').read_text(encoding='utf8'))
            changed={k for k in set(identity)|set(old['source']) if identity.get(k)!=old['source'].get(k)}
            if (ROOT/'artifacts' not in prior.parents or old['binary_sha256']!=sha or not old['unchanged']
                    or old.get('error')!='streamed Q failed; no retry' or len(old['steps'])!=3
                    or [(s['name'],s.get('returncode')) for s in old['steps'][:2]]!=[('standard',0),('cycles',0)]
                    or {k for k,v in old['steps'][2]['hard_checks'].items() if not v}!={'known_complete_gpu_budget'}
                    or not changed or not changed<={'tools/measure_automatic_closeout.py','tools/test_check_automatic_closeout.py'}):
                raise ValueError('continuation may only retain reviewed same-binary N/C3 across observer-only changes')
            n=json.loads((prior/'standard/proof.json').read_text(encoding='utf8'))
            c=json.loads((prior/'cycles/proof.json').read_text(encoding='utf8'))
            if not n['standard_hard_accepted'] or not c['analysis']['behavior_and_budget_passed']:
                raise ValueError('previous N/C3 not accepted')
            proof['retained_accepted_steps']=dict(directory=str(prior.relative_to(ROOT)),
                proof_sha256=hashlib.sha256((prior/'proof.json').read_bytes()).hexdigest(),
                observer_only_changes=sorted(changed),failed_Q_retained=True,failed_Q_reclassified=False)
        else:
            command('standard',[sys.executable,'tools/measure_current_combined.py',*common,'--product','--output',str(out/'standard')],900)
            command('cycles',[sys.executable,'tools/measure_resource_cycles.py',*common,'--product','--output',str(out/'cycles')],950)
        for i in (1,2):
            unchanged();print('Starting streamed Q %d/2'%i,flush=True)
            report,samples=observe(binary,out/('Q-%d'%i),'--verify-streamed-gui',timeout_seconds=50,sampler_factory=LifecycleSampler)
            metrics=summarize(samples,('idle','voices_1','voices_32','closed'),(('idle',.7),('voices_1',1.2),('voices_32',1.7),('closed',.7)))
            latency,secondary,checks=evaluate_gui(report,metrics)
            checks.update(stream_checks(report,False));checks.update(audit_checks(report))
            checks['lifecycle_memory_budget']=all(0<s[k]<=512*1024**2 for s in samples for k in ('private_bytes','working_set_bytes'))
            gpu=gpu_budget(samples);checks['known_complete_gpu_budget']=all(v['passed'] for v in gpu.values())
            # v2 explicitly changes this one legacy predicate to a goal. Keep
            # the original reading and every other predicate; no prefix waiver.
            hard={k:v for k,v in checks.items() if k!='cpu_average_le_15_one_core'}
            proof['steps'].append(dict(name='Q-%d'%i,report=report,samples=samples,metrics_raw=metrics,
                latency=latency,joined_latency=secondary,original_checks=checks,hard_checks=hard,gpu=gpu,
                passed=all(hard.values()),cpu_raw_numeric_le_15=checks['cpu_average_le_15_one_core'],cpu_goal_met=None))
            save()
            if not all(hard.values()):raise ValueError('streamed Q failed; no retry')
        for i,resources in enumerate((False,False,True,True),1):
            unchanged();print('Starting nonprivileged stability %d/4'%i,flush=True)
            report,samples=observe(binary,out/('stable-%d'%i),'--verify-product-stable' if resources else '--verify-product-empty',
                                   timeout_seconds=230 if resources else 105,sampler_factory=LifecycleSampler)
            result=stable_result(report,samples,resources)
            proof['steps'].append(dict(name='stable-%d'%i,report=report,samples=samples,result=result));save()
            if not result['behavior_and_memory_passed']:raise ValueError('stable behavior/memory failed; no retry')
        for flag,name in [('--active-edit','active-edit'),('--endpoint-return','endpoint'),('--cleanup','cleanup'),('--initialization','initialization'),('', 'present')]:
            command(name,[sys.executable,'tools/measure_present_fault.py',*([flag] if flag else []),*common,'--out',str(out/name)],600)
        pixels=[]
        for flag,name in [('--verify-gpu','pixels-default'),('--verify-gpu-memory','pixels-memory')]:
            command(name,[str(binary),flag,str(out/name)],120)
            r=json.loads((out/name/'report.json').read_text(encoding='utf8'))
            if r['max_channel_error']>2:raise ValueError('CPU/GPU pixel mismatch')
            pixels.append(hashlib.sha256((out/name/'pixel_trace.rgba').read_bytes()).hexdigest())
        if len(set(pixels))!=1:raise ValueError('allocator policy changed pixel output')
        from measure_fault_rebuild import analyze
        unchanged();print('Starting offscreen fault',flush=True)
        report,samples=observe(binary,out/'offscreen','--verify-graphics-faults-memory',timeout_seconds=100,sampler_factory=GpuSampler)
        analysis=analyze(report,samples,True,memory_usage=True)
        proof['steps'].append(dict(name='offscreen',report=report,samples=samples,analysis=analysis));save()
        if not analysis['component_passed']:raise ValueError('offscreen fault failed')
        proof.update(automatic_passed=True,pixel_trace_sha256=pixels[0],c3_trend_review_required=True)
    except (OSError,ValueError,RuntimeError,KeyError,subprocess.SubprocessError) as e:
        proof['error']=str(e);print(str(e),flush=True)
    finally:save()
    if not proof['automatic_passed'] or not proof['unchanged']:raise SystemExit(1)


if __name__=='__main__':main()
