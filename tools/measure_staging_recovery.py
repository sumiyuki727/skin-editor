"""Opt-in same-device 20-cycle staging recovery; two owned Windows processes."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import statistics
from measure_audio_load import ROOT, observe, summarize, distribution
from measure_combined import (GpuSampler, source_identity, identity_matches, analyze,
                              staging_analysis, allocator_analysis, causal_analysis)


class RecoverySampler(GpuSampler):
    def sample(self, phase):
        result=super().sample(phase)
        if getattr(self,'last_phase',None)!=phase:
            print(json.dumps(dict(phase=phase,time=result['time'],private_bytes=result['private_bytes'],
                                  working_set_bytes=result['working_set_bytes'])),flush=True)
            self.last_phase=phase
        return result


def trend(values):
    """Describe retention, without inventing a permitted-leak threshold."""
    if len(values) != 20 or any(not math.isfinite(v) or v < 0 for v in values):
        raise ValueError('twenty valid closing observations required')
    center = statistics.mean(values)
    return dict(first=values[0],last=values[-1],first_five_median=statistics.median(values[:5]),
                last_five_median=statistics.median(values[-5:]),
                median_change=statistics.median(values[-5:])-statistics.median(values[:5]),
                slope_per_cycle=sum((i-9.5)*(v-center) for i,v in enumerate(values))/665,
                consecutive_increases=sum(b>a for a,b in zip(values,values[1:])),values=values)


def gpu_summary(samples):
    result={}
    for name in ('dedicated','shared'):
        values=[s.get('gpu_'+name+'_bytes') for s in samples]
        valid=[v for v in values if type(v) is int and v>=0]
        result[name]=dict(valid=len(valid),missing=len(values)-len(valid),max=max(valid) if valid else None)
    return result


def analyze_recovery(report, cycles, samples):
    if (report.get('cycles')!=20 or len(cycles)!=20 or report.get('gpu_memory_usage_hint') is not True or
            report.get('reusable_staging') is not True or report.get('settle_seconds')!=10 or report.get('stable_seconds')!=65):
        raise ValueError('incomplete lifecycle or altered observation schedule')
    snapshots=report['gpu_allocator_snapshots']
    expected=['cycle_01_paused_start','cycle_01_paused_end']+[
        'cycle_%02d_%s'%(n,suffix) for n in range(1,21) for suffix in ('closed_start','closed_end')]
    if len(snapshots)!=42 or [s['stage'] for s in snapshots]!=expected:
        raise ValueError('incomplete or unbounded lifecycle allocator trace')
    allocator=[allocator_analysis([s])[0] for s in snapshots]
    frames=report.get('stable_ui_frames')
    if not isinstance(frames,list) or len(frames)!=3 or any(type(v) is not int or v<1 for v in frames):
        raise ValueError('missing stable UI frame observation')
    phases=('paused','closed_initial','closed_final')
    stable=summarize(samples,phases,[(p,60) for p in phases])
    stable_gpu={p:gpu_summary([s for s in samples if s['phase']==p]) for p in phases}
    # The ordinary 300 s workload still uses its existing gates. Do not fold
    # later short cycles into Playing, or use their idle time to lower CPU.
    first_samples=[dict(s,phase='closed') if s['phase']=='closed_initial' else s
                   for s in samples if s['phase'] in ('playing','closed_initial')]
    standard=analyze(cycles[0],first_samples,memory_usage=True,staging=True)
    causal=causal_analysis(cycles[0])
    per_cycle=[];closing={k:[] for k in ('working_set_bytes','private_bytes','gpu_dedicated_bytes','gpu_shared_bytes')}
    for index,r in enumerate(cycles):
        if (r.get('gpu_memory_usage_hint') is not True or r.get('reusable_staging') is not True or
                (r['original_pngs'],r['documents'],r['layers'])!=(100,10,500)):
            raise ValueError('cycle lacks the standard resources')
        if index and (r['seconds'],r['triggers'],r['edits'],r['switches'])!=(3,24,3,1):
            raise ValueError('incomplete short-cycle activity')
        # Compare equal-age closing windows. The extra 10+65 s first/last
        # observations are reported separately, never mixed into cycle trends.
        phase='cycle_%02d_settling'%(index+1) if index in (0,19) else 'cycle_%02d_closed'%(index+1)
        selected=[s for s in samples if s['phase']==phase]
        if len(selected)<5 or selected[-1]['time']-selected[0]['time']<1:
            raise ValueError('missing closing baseline')
        tail=[s for s in selected if 0.8<=s['time']-selected[0]['time']<=1.8]
        if len(tail)<5:raise ValueError('missing comparable closing window')
        for key in closing:
            values=[s.get(key) for s in tail]
            if any(type(v) is not int or v<0 for v in values):raise ValueError('missing closing memory counter')
            closing[key].append(statistics.median(values))
        counts=staging_analysis(r,True)
        cpu=distribution(r['cpu_frame_ms'])
        gates=dict(counts,
            no_underflow=r['empty_frames']==r['empty_callbacks']==0 and not r['underrun_observations'],
            managed_resources_released=r['resources_released'] is True and
                all(r[k]==0 for k in ('pcm_final_bytes','raw_final_bytes','rgba_final_bytes','final_targets','source_cache_final_bytes')),
            managed_peaks_bounded=r['pcm_peak_bytes']<=32*1024**2 and r['raw_peak_bytes']<=128*1024**2 and
                r['rgba_peak_bytes']<=80*1024**2 and r['max_targets']<=8 and r['peak_voices']<=32,
            old_audio_cleaned=r['dual_channel_isolation'] is True and r['draft_history_cleanup'] is True,
            cpu_frame_p95_le_16_7_ms=cpu['p95']<=16.7,cpu_frame_max_le_16_7_ms=cpu['max']<=16.7,
            trace_bounded=r['trace_overflow'] is False and len(r['preparation_trace'])<=512)
        per_cycle.append(dict(checks=gates,cpu_frame_ms=cpu,closing_phase=phase))
    # Ignore initial driver-counter discovery only before Playing. Missing data
    # in any active/closing/stable phase fails, rather than becoming zero.
    measured=[s for s in samples if s['phase'] in ('playing',*phases) or
              (s['phase'].startswith('cycle_') and s['phase'].endswith(('_playing','_closed')))]
    gpu=gpu_summary(measured)
    lifetime_gpu=gpu_summary(samples)
    checks=dict(standard['checks'],
        stable_cpu_one_core_le_2_percent=all(v['cpu_one_logical_core_percent']<=2 for v in stable.values()),
        stable_ui_frames_le_2=all(v<=2 for v in frames),
        all_cycles_cleanup_and_bounds=all(all(v['checks'].values()) for v in per_cycle),
        all_process_memory_le_512_mib=all(s['working_set_bytes']<=512*1024**2 and s['private_bytes']<=512*1024**2 for s in samples),
        valid_loading_gpu_peaks_bounded=all(lifetime_gpu[n]['max'] is not None and lifetime_gpu[n]['max']<=limit*1024**2
                                           for n,limit in (('dedicated',256),('shared',128))),
        all_measured_gpu_budget=all(gpu[n]['missing']==0 and gpu[n]['max'] is not None and gpu[n]['max']<=limit*1024**2
                                    for n,limit in (('dedicated',256),('shared',128))))
    return dict(checks=checks,standard=standard,causal=causal,stable=stable,stable_gpu=stable_gpu,gpu=gpu,lifetime_gpu=lifetime_gpu,
                per_cycle=per_cycle,closing_trends={k:trend(v) for k,v in closing.items()},
                allocator=allocator,
                retention_status='requires_review_no_permitted_leak_allowance')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',required=True);parser.add_argument('--output',required=True)
    args=parser.parse_args();binary=Path(args.binary).resolve();output=(ROOT/args.output).resolve()
    if os.name!='nt' or binary!=(ROOT/'desktop/target/release/skin-editor-desktop.exe').resolve() or not binary.is_file():
        raise RuntimeError('requires explicit Windows release desktop binary')
    if output.exists() or ROOT/'artifacts' not in output.parents:raise RuntimeError('new artifacts directory required')
    output.mkdir(parents=True)
    identity=dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),source_sha256=source_identity())
    (output/'identity.json').write_text(json.dumps(identity,indent=2)+'\n',encoding='utf-8')
    runs=[]
    for index in range(2):
        path=output/('run-%02d'%(index+1));print('Starting recovery %d/2'%(index+1),flush=True)
        try:
            report,samples=observe(binary,path,'--verify-staging-recovery',timeout_seconds=900,sampler_factory=RecoverySampler)
            cycles=[json.loads((path/('cycle-%02d'%(n+1))/'report.json').read_text(encoding='utf-8')) for n in range(20)]
            result=analyze_recovery(report,cycles,samples)
            runs.append(dict(completed=True,**result))
            print(json.dumps(dict(run=index+1,checks=result['checks'],stable=result['stable'],trends=result['closing_trends'])),flush=True)
        except (OSError,ValueError,RuntimeError,KeyError,TypeError) as e:
            runs.append(dict(completed=False,error=str(e),checks=dict(completed=False)))
            print(str(e),flush=True)
            break
    unchanged=identity_matches(identity['source_sha256']) and hashlib.sha256(binary.read_bytes()).hexdigest()==identity['binary_sha256']
    passed=len(runs)==2 and unchanged and all(all(r['checks'].values()) for r in runs)
    (output/'metrics.json').write_text(json.dumps(dict(schema=1,sources_unchanged=unchanged,checks_passed=passed,
        retention_requires_review=True,runs=runs),indent=2)+'\n',encoding='utf-8')
    if not passed:raise RuntimeError('recovery gates failed; observations retained')


if __name__=='__main__':main()
