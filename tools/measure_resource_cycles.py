"""Current separated audio / full Editor resource cycles, owned Windows PID only.

Twenty fixed cycles; a two-cycle smoke plan never accepts C3. No CPU deductions,
leak allowance, identical-run retry, or automatic trend acceptance.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import statistics
import re

from measure_audio_load import ROOT, distribution, observe, summarize
from measure_combined import GpuSampler, source_identity
from measure_cpu_threads import token_info
from measure_main_work import ordinary

def read_phase(path):
    try:
        with path.open('rb') as f: data=f.read(257)
        if len(data)>256:return None
        value=json.loads(data)
        if not isinstance(value,dict):return None
        if (type(value.get('step')) is not int or not 1<=value['step']<=256 or not isinstance(value.get('phase'),str)
                or not re.fullmatch(r'(initial|final)_(settle|closed)|cycle_\d{2}_(loading|playing|tail|closing|closed_settle|closed)|finished',value['phase'])):
            return None
        return value
    except (OSError,ValueError,TypeError):return None

def label_sample(sample,before,after):
    # Preserve every counter sample for lifecycle budgets. Only an unchanged
    # native epoch contributes to a particular playback/closed-baseline window.
    valid=before is not None and before==after
    return dict(sample,phase=before['phase'] if valid else 'transition',phase_epoch_valid=valid,
                phase_before=before,phase_after=after)

class CycleSampler(GpuSampler):
    def __init__(self,child,directory):
        super().__init__(child);self.phase_file=directory/'phase.json'
    def sample(self,phase):
        before=read_phase(self.phase_file)
        result=super().sample(phase)
        return label_sample(result,before,read_phase(self.phase_file))


def numeric(values):
    return all(type(v) in (int,float) and math.isfinite(v) and v >= 0 for v in values)


def cycle_checks(r, index, smoke, rate):
    counts = [4,4,2] if smoke else [32,32,16]
    if (r.get('schema') != 1 or r.get('index') != index or r.get('shape') != [100,10,500,16]
            or r.get('target') != (index-1)%10 or r.get('counts') != counts
            or not isinstance(r.get('session'),str) or not r['session'].isdecimal()):
        raise ValueError('incomplete cycle identity, fixture or schedule')
    probes=r['probes']
    if (len(probes) != sum(counts[:2])+2 or any(len(p)!=10 for p in probes)
            or len({p[0] for p in probes}) != len(probes) or len({(p[1],p[2]) for p in probes}) != len(probes)):
        raise ValueError('missing or duplicate exact request/Voice observations')
    for p in probes:
        if (not numeric(p[:9]) or p[1] not in (0,1) or type(p[9]) is not bool or not 0<=p[7]<p[8]
                or p[3]>p[4] or any(type(p[i]) is not int or p[i]<=0 for i in (0,2,5,6,8))):
            raise ValueError('invalid callback or request observation')
    lanes=[[p[3] for p in probes if p[9] and p[1]==lane] for lane in (0,1)]
    if [len(v) for v in lanes] != counts[:2]:raise ValueError('wrong per-channel sample counts')
    latency=[distribution(v) for v in lanes]
    if not (10 if smoke else 120)<=len(r['cpu_frame_ms'])<=1500:raise ValueError('missing/unbounded frame samples')
    frames=distribution(r['cpu_frame_ms'])
    peak,final=r['resources_peak'],r['resources_final']
    if len(peak)!=5 or len(final)!=5 or any(type(v) is not int or v<0 for v in peak+final):raise ValueError('invalid lease accounting')
    seconds=1.2 if smoke else 6
    checks=dict(
        complete_playback=r['playing_us']>=seconds*1e6 and r['native_play_frames']>=seconds*rate
            and r['background_end']-r['background_start']>=seconds*rate,
        zero_missing_including_startup=r['missing_frames']==0,
        native_healthy=r['callbacks']>0 and not r['device_fault'] and r['render_errors']==r['observation_overflows']==0,
        both_channels_copy_p95_le_20_ms=all(v['p95']<=20 for v in latency),
        cpu_frame_p95_le_16_7_ms=frames['p95']<=16.7,
        actual_lease_release=final==[0]*5 and r['post_drop_confirmed'] is True and r['staging_final_bytes']==0,
        all_owned_workers_joined=r['workers_joined']==[True]*3,
        close_exercised_inflight_work=r['pending_graphics_at_close'] is True and min(r['voices_at_close'])>0,
        native_callback_ownership=r['endpoint_returns']==1 and r['callback_resource_releases']==r['endpoint_returns_in_render']==0
            and r['endpoint_returns_outside_render']==1,
        bounded_managed_resources=all(v<=lim for v,lim in zip(peak,(32*1024**2,128*1024**2,80*1024**2,8,80*1024**2)))
            and 0<r['peak_voices']<=32,
        background_source_closed=not r['background_fault'] and r['background_raw_peak']==8192 and r['background_raw_final']==0
            and r['background_read_bytes']>=seconds*rate*4,
        graphics_published_and_sources_reused=r['images_published']==counts[2] and r['source_allocations']<=1 and r['source_uploads']<=1,
        unchanged_geometry=r['view_valid'] is True and len(r['geometry'])==3 and numeric(r['geometry']) and min(r['geometry'])>0,
        bounded_schedule=r['max_trigger_lateness_us']<=80_000,
    )
    return dict(checks=checks,passed=all(checks.values()),latency_ms=latency,cpu_frame_ms=frames,
                long_frames=sum(v>16.7 for v in r['cpu_frame_ms']),ready_ms=r['ready_ms'],close_ms=r['close_ms'])


def slope(values):
    centre=(len(values)-1)/2
    denom=sum((i-centre)**2 for i in range(len(values)))
    return sum((i-centre)*v for i,v in enumerate(values))/denom if denom else 0.


def trend(values):
    if len(values)!=20 or not numeric(values):raise ValueError('twenty closed baselines required')
    result=dict(first=values[0],last=values[-1],first_last_delta=values[-1]-values[0],
                first_five_median=statistics.median(values[:5]),last_five_median=statistics.median(values[-5:]),
                median_delta=statistics.median(values[-5:])-statistics.median(values[:5]),
                slope_per_cycle=slope(values),last_ten_slope=slope(values[-10:]),last_five_slope=slope(values[-5:]),
                last_ten_min=min(values[-10:]),last_ten_max=max(values[-10:]),values=values)
    result['interpretation']='Raw trend only; positive tail growth requires attribution/review, not a numerical leak allowance'
    return result


def analyze(report, cycles, samples, smoke=False, product=False):
    rounds=2 if smoke else 20
    if (report.get('schema')!=1 or report.get('scope')!='current-separated-full-resource-cycles'
            or report.get('smoke') is not smoke or report.get('completed') is not True
            or report.get('rounds')!=rounds or report.get('expected_rounds')!=rounds or len(cycles)!=rounds
            or report.get('production_policy',False) is not product
            or report.get('gpu_memory_usage_hint') is not product or report.get('reusable_staging') is not False
            or report.get('rate')!=48000 or report.get('channels')!=2 or report.get('controlled_focus') is not True
            or report.get('physical_input_tested') is not False or len(report.get('cycles',[]))!=rounds):
        raise ValueError('wrong route, fixed cycle plan, device format or incomplete report')
    if len({r['session'] for r in cycles})!=rounds:raise ValueError('old project session reused as a new cycle')
    last=0;seen={}
    for s in samples:
        if s['phase']=='transition':continue
        before,after=s.get('phase_before'),s.get('phase_after')
        if s.get('phase_epoch_valid') is not True or before!=after or not before or before['phase']!=s['phase'] or before['step']<last:
            raise ValueError('unstable/reversed native phase epoch cannot define a baseline')
        if before['step'] in seen and seen[before['step']]!=s['phase']:raise ValueError('reused phase epoch')
        last=before['step'];seen[last]=s['phase']
    values=[cycle_checks(r,i+1,smoke,report['rate']) for i,r in enumerate(cycles)]
    for i,(r,s) in enumerate(zip(cycles,report['cycles']),1):
        if s['index']!=i or s['session']!=r['session'] or s['resources_after_drop']!=[0]*3 or s['workers_joined']!=[True]*3:
            raise ValueError('cycle archive differs from closed owner proof')
    phases=['initial_closed']
    for i in range(1,rounds+1):phases.extend(('cycle_%02d_playing'%i,'cycle_%02d_closed'%i))
    phases.append('final_closed')
    minimum=[(p,(.1 if smoke else 3.5) if p.endswith('_closed') and p!='final_closed' else (.3 if smoke else 60) if p=='final_closed' else (.8 if smoke else 5.5)) for p in phases]
    metrics=summarize(samples,phases,minimum)
    gpu={}
    for phase in phases:
        rows=[r for r in samples if r['phase']==phase];gpu[phase]={}
        for name in ('dedicated','shared'):
            raw=[r.get('gpu_'+name+'_bytes') for r in rows];valid=[v for v in raw if type(v) is int and v>=0]
            gpu[phase][name]=dict(valid_samples=len(valid),missing_samples=len(raw)-len(valid),max_bytes=max(valid) if valid else None)
    lifecycle_gpu={}
    for name in ('dedicated','shared'):
        raw=[s.get('gpu_'+name+'_bytes') for s in samples];valid=[v for v in raw if type(v) is int and v>=0]
        lifecycle_gpu[name]=dict(valid_samples=len(valid),missing_samples=len(raw)-len(valid),max_bytes=max(valid) if valid else None)
    baselines=[]
    for i in range(1,rounds+1):
        phase='cycle_%02d_closed'%i;rows=[s for s in samples if s['phase']==phase]
        chosen=[s for s in rows if s['time']>=rows[0]['time']+(0 if smoke else .5)]
        if len(chosen)<2 or chosen[-1]['time']-chosen[0]['time']<(.1 if smoke else 2.5):raise ValueError('insufficient fixed closed baseline')
        baseline=dict(index=i,span_seconds=chosen[-1]['time']-chosen[0]['time'])
        for key in ('private_bytes','working_set_bytes','gpu_dedicated_bytes','gpu_shared_bytes'):
            v=[s.get(key) for s in chosen]
            if any(type(x) is not int or x<0 for x in v):raise ValueError('unknown closed memory/GPU is not zero')
            baseline[key]=dict(first=v[0],last=v[-1],min=min(v),max=max(v),median=statistics.median(v))
        baselines.append(baseline)
    checks=dict(
        all_cycle_behavior=all(v['passed'] for v in values),
        all_editor_owners_dropped=report['closed_editor_owners'] is True and report['all_post_drop_bytes_zero'] is True,
        lifecycle_memory_budget=all(0<s[k]<=512*1024**2 for s in samples for k in ('private_bytes','working_set_bytes')),
        complete_gpu_budget=all(gpu[p][n]['missing_samples']==0 and gpu[p][n]['max_bytes'] is not None and gpu[p][n]['max_bytes']<=limit*1024**2
                                for p in phases for n,limit in (('dedicated',256),('shared',128))),
        all_known_gpu_peaks_budget=all(lifecycle_gpu[n]['max_bytes'] is not None and lifecycle_gpu[n]['max_bytes']<=limit*1024**2 for n,limit in (('dedicated',256),('shared',128))),
        bounded_retained_summaries=report['retained_summary_capacity_bytes']<=32*1024,
    )
    raw_play=sum(metrics['cycle_%02d_playing'%i]['cpu_seconds'] for i in range(1,rounds+1))
    wall_play=sum(metrics['cycle_%02d_playing'%i]['sample_span_seconds'] for i in range(1,rounds+1))
    return dict(checks=checks,behavior_and_budget_passed=all(checks.values()),cycles=values,phases_raw=metrics,gpu=gpu,lifecycle_gpu=lifecycle_gpu,
                closed_baselines=baselines,private_trend=None if smoke else trend([b['private_bytes']['median'] for b in baselines]),
                working_set_trend=None if smoke else trend([b['working_set_bytes']['median'] for b in baselines]),
                raw_playing_one_core_percent=raw_play/wall_play*100,cpu_meter_independently_validated=False,cpu_goal_met=None,
                cpu_note='Raw GetProcessTimes only. CPU15% goal not accepted; 4s and diagnostic shell65s are not full Editor B05 proof',
                peak_memory={k:max(s[k] for s in samples) for k in ('private_bytes','working_set_bytes','lifetime_peak_working_set_bytes','lifetime_peak_pagefile_bytes')},
                c3_accepted=False,trend_review_required=not smoke,p2_accepted=False)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',required=True);p.add_argument('--build-receipt',required=True);p.add_argument('--output',required=True);p.add_argument('--smoke',action='store_true');p.add_argument('--product',action='store_true')
    args=p.parse_args();binary=Path(args.binary).resolve();output=(ROOT/args.output).resolve()
    receipt=json.loads(Path(args.build_receipt).read_text(encoding='utf-8'));identity=source_identity();sha=hashlib.sha256(binary.read_bytes()).hexdigest()
    if (os.name!='nt' or not ordinary(token_info()) or output.exists() or ROOT/'artifacts' not in output.parents or ROOT/'artifacts' not in binary.parents
            or receipt.get('source')!=identity or receipt.get('repeat_sha256')!=[sha,sha] or receipt.get('profile')!='release' or receipt.get('requested_features')!=[]):
        raise ValueError('ordinary frozen default-release build and fresh artifacts output required')
    output.mkdir(parents=True);proof=dict(schema=1,source=identity,binary_sha256=sha,production_policy=args.product,smoke=args.smoke,expected_rounds=2 if args.smoke else 20,c3_accepted=False,p2_accepted=False)
    try:
        directory=output/'process'
        report,samples=observe(binary,directory,('--verify-product-cycles-smoke' if args.smoke else '--verify-product-cycles') if args.product else ('--verify-resource-cycles-smoke' if args.smoke else '--verify-resource-cycles'),timeout_seconds=120 if args.smoke else 900,
                               sampler_factory=lambda child:CycleSampler(child,directory))
        cycles=[json.loads((output/'process'/('cycle-%02d.json'%i)).read_text(encoding='utf-8')) for i in range(1,proof['expected_rounds']+1)]
        proof.update(report=report,cycles=cycles,samples=samples,analysis=analyze(report,cycles,samples,args.smoke,args.product))
    except (OSError,RuntimeError,ValueError,KeyError) as e:proof['error']=str(e)
    finally:
        proof['unchanged']=identity==source_identity() and sha==hashlib.sha256(binary.read_bytes()).hexdigest()
        (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf-8')
    if not proof.get('analysis',{}).get('behavior_and_budget_passed') or not proof['unchanged']:raise SystemExit(1)


if __name__=='__main__':main()
