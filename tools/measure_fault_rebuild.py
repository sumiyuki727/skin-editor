"""Bounded opt-in C4 component fault/reopen; not full window recovery acceptance."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
from measure_audio_load import ROOT, WindowsSampler, observe, distribution
from measure_combined import GpuSampler, source_identity
from measure_cpu_threads import token_info
from measure_main_work import ordinary


def analyze(report, samples, graphics, *, memory_usage=None):
    if report.get('schema') not in ((4,5) if graphics else (1,)) or report.get('full_editor') is not False or not samples:
        raise ValueError('wrong scope or absent owned PID samples')
    if memory_usage is not None and (not graphics or type(memory_usage) is not bool):
        raise ValueError('invalid explicit graphics policy')
    if graphics:
        if report['schema']==4:
            if memory_usage is not None or 'gpu_memory_usage_hint' in report:
                raise ValueError('legacy schema cannot prove an explicit device policy')
        else:
            expected=False if memory_usage is None else memory_usage
            hints=report.get('device_memory_usage_hints')
            if (report.get('gpu_memory_usage_hint') is not expected or not isinstance(hints,list)
                    or len(hints)!=2 or any(value is not expected for value in hints)):
                raise ValueError('first/rebuilt device policy does not match the requested control')
    if any(type(s.get(k)) is not int or s[k] <= 0 for s in samples for k in ('working_set_bytes','private_bytes')):
        raise ValueError('unknown process memory is not zero')
    checks = dict(process_budget=all(s[k] <= 512*1024**2 for s in samples for k in ('working_set_bytes','private_bytes')))
    detail = {}
    if graphics:
        if report.get('scope') != 'offscreen-own-device-destroy-rebuild' or report.get('physical_dpi_minimize') is not False:
            raise ValueError('offscreen result cannot accept physical window behavior')
        name=report.get('adapter_utf8')
        if not isinstance(name,list) or not 1<=len(name)<=256 or any(type(v) is not int or not 0<=v<=255 for v in name):
            raise ValueError('missing bounded adapter identity')
        detail['adapter']=bytes(name).decode('utf-8')
        keys=report['frame_keys']
        checks.update(confirmed_destroy=report['device_lost_reason']=='Destroyed',
                      late_result_and_draw_fenced=all(report[k] is True for k in ('inflight_prepare','old_cpu_output_discarded','device_owner_released_before_open','parent_context_released_before_open','late_draw_request_rejected','old_signal_rejected')),
                      held_owner_and_key_identity=report['held_targets_after_loss']>0 and len(keys)==2 and 0<keys[0]<keys[1],
                      pixels_and_latest_viewport=report['pixels_exact'] is True and report['viewport_requests']==32,
                      actual_release=report['final_targets']==report['raw_final']==report['rgba_final']==0 and report['worker_closed'] is True and report['shutdown_ok'] is True)
        gpu={}
        for phase in ('graphics_active','graphics_rebuilt'):
            rows=[s for s in samples if s['phase']==phase]
            if len(rows)<2: raise ValueError('missing fixed live-resource GPU observation')
            for name,limit in (('dedicated',256),('shared',128)):
                values=[s.get('gpu_'+name+'_bytes') for s in rows]
                if any(type(v) is not int or v<0 for v in values): raise ValueError('unknown live GPU sample')
                checks[phase+'_'+name+'_budget']=max(values)<=limit*1024**2
        for name,limit in (('dedicated',256),('shared',128)):
            raw=[s.get('gpu_'+name+'_bytes') for s in samples];values=[v for v in raw if type(v) is int and v>=0]
            gpu[name]=dict(max_bytes=max(values) if values else None,missing_samples=len(raw)-len(values))
            checks['all_known_'+name+'_budget']=bool(values) and max(values)<=limit*1024**2
        detail['gpu']=gpu
    else:
        if (report.get('scope')!='controlled-native-device-error-reopen' or report.get('physical_unplug') is not False
                or report.get('rounds')!=2 or (report.get('rate'),report.get('channels'))!=(48000,2)
                or len(report.get('stats',[]))!=2 or len(report.get('rows',[]))!=2):
            raise ValueError('wrong fixed audio fault plan')
        latency=[]
        for rows in report['rows']:
            if len(rows)!=64 or any(len(p)!=9 for p in rows) or len({p[0] for p in rows})!=64 or len({(p[1],p[2]) for p in rows})!=64:
                raise ValueError('missing or duplicated request/Voice evidence')
            for p in rows:
                if (any(type(v) not in (int,float) or not math.isfinite(v) or v<0 for v in p)
                        or p[1] not in (0,1) or p[3]>p[4] or not 0<=p[7]<p[8]
                        or any(type(p[i]) is not int or p[i]<=0 for i in (0,2,5,6,8))):
                    raise ValueError('invalid exact callback evidence')
            lanes=[[p[3] for p in rows if p[1]==i] for i in (0,1)]
            if any(len(v)!=32 for v in lanes): raise ValueError('missing channel samples')
            latency.append([distribution(v) for v in lanes])
        checks.update(zero_missing=all(s['missing_frames']==0 for s in report['stats']),
                      planned_fault_only=all(s['device_fault'] is True and s['render_errors']==s['observation_overflows']==0 and s['callbacks']>0 for s in report['stats']),
                      each_stage_lane_p95_le_20_ms=all(s['p95']<=20 for stage in latency for s in stage),
                      actual_release=report['pcm_final']==report['raw_final']==report['old_reader_raw_final']==0 and report['pcm_peak']<=32*1024**2 and report['raw_peak']<=128*1024**2,
                      callback_ownership=all(s['endpoint_returns']==s['endpoint_returns_outside_render']==1 and s['callback_resource_releases']==s['endpoint_returns_in_render']==0 for s in report['stats']),
                      frozen_and_old_generation=report['old_signal_rejected'] is True and report['frozen_rebuild'] is True and len(report['copy_anchors'])==2 and 0<report['copy_anchors'][0]<report['copy_anchors'][1])
        detail['latency_ms']=latency
    return dict(checks=checks,component_passed=all(checks.values()),c4_accepted=False,p2_accepted=False,
                cpu_goal_met=None,cpu_meter_independently_validated=False,
                memory={k:max(s[k] for s in samples) for k in ('working_set_bytes','private_bytes','lifetime_peak_working_set_bytes','lifetime_peak_pagefile_bytes')},**detail)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',required=True);parser.add_argument('--build-receipt',required=True);parser.add_argument('--output',required=True)
    args=parser.parse_args();binary=Path(args.binary).resolve();directory=(ROOT/args.output).resolve()
    receipt=json.loads(Path(args.build_receipt).read_text(encoding='utf-8'));identity=source_identity();sha=hashlib.sha256(binary.read_bytes()).hexdigest()
    if (os.name!='nt' or not ordinary(token_info()) or directory.exists() or ROOT/'artifacts' not in directory.parents
            or ROOT/'artifacts' not in binary.parents or receipt.get('source')!=identity or receipt.get('repeat_sha256')!=[sha,sha]
            or receipt.get('profile')!='release' or receipt.get('requested_features')!=[]):
        raise ValueError('fresh output and ordinary frozen default release required')
    directory.mkdir(parents=True);proof=dict(schema=1,source=identity,binary_sha256=sha,c4_accepted=False,p2_accepted=False,runs=[])
    try:
        for graphics,name in ((True,'graphics'),(False,'audio')):
            report,samples=observe(binary,directory/name,'--verify-'+name+'-faults',timeout_seconds=90,sampler_factory=GpuSampler if graphics else WindowsSampler)
            result=analyze(report,samples,graphics);proof['runs'].append(dict(name=name,report=report,samples=samples,analysis=result))
            if not result['component_passed']: raise ValueError(name+' component checks failed')
    except (OSError,RuntimeError,ValueError,KeyError) as e: proof['error']=str(e)
    finally:
        proof['unchanged']=source_identity()==identity and hashlib.sha256(binary.read_bytes()).hexdigest()==sha
        (directory/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf-8')
    if proof.get('error') or len(proof['runs'])!=2 or not proof['unchanged']: raise SystemExit(1)


if __name__=='__main__':main()
