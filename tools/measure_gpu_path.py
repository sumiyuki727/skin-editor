"""Explicit actual-frame markers and separate own-buffer upload-only probes.

No CPU/GPU budget acceptance, retries, timing subtraction or presentation claim.
The ordinary product CPU control is measured separately by measure_repaint.py.
"""
import argparse
import json
import os
from measure_audio_load import ROOT,observe,summarize,distribution
from measure_cpu_threads import token_info
from measure_independent_cpu import BINARY,fingerprint
from measure_main_work import ordinary
from measure_ui_wall import diagnostic_checks
from measure_gui_candidate import evaluate_gui
from measure_background_stream import stream_checks,audit_checks

ORDINALS=(1,33,65,97)
INTERVALS=('output_to_after_tessellation','painter_entry_to_after_upload',
           'callback_prepare_and_finish','surface_acquire_and_pass_setup',
           'draw_encoding','post_draw_to_gpu_completion_observation')

def integer_row(row,length):
    if not isinstance(row,list) or len(row)!=length or any(type(v) is not int or v<0 for v in row):
        raise ValueError('invalid monotonic observation row')

def path(data,shutdown,mode,minimum_frames=97):
    for v in (data,shutdown):
        if (not isinstance(v,dict) or v.get('schema')!=1 or v.get('mode')!=mode or v.get('invalid')!=0
                or v.get('error') is not None or type(v.get('discarded')) is not int or v['discarded']<0):
            raise ValueError('rejected/incomplete GPU path observation')
    if shutdown.get('released') is not True or data.get('released') is not False:
        raise ValueError('missing explicit probe release')
    for key in ('frames','samples','uploads','discarded'):
        if data.get(key)!=shutdown.get(key):raise ValueError('late/missing GPU observation after report')
    frames,samples,uploads=(data.get(k) for k in ('frames','samples','uploads'))
    if not all(isinstance(v,list) for v in (frames,samples,uploads)):raise ValueError('missing bounded arrays')
    if mode=='markers':
        if not minimum_frames<=len(frames)<=512 or samples or uploads:
            raise ValueError('incomplete actual-frame plan')
        previous=0
        for row in frames:
            integer_row(row,8)
            if row[0]<=previous or not all(row[i]>0 and row[i]<=row[i+1] for i in range(1,7)):
                raise ValueError('missing, reversed or duplicate actual-frame edges')
            previous=row[0]
        return dict(scope='actual-frame CPU wall regions; completion includes GPU/queue/poll/scheduling, not CPU submit, present or device consumption',
            count=len(frames),discarded=data['discarded'],interval_ms={label:distribution([(r[i+2]-r[i+1])/1e6 for r in frames]) for i,label in enumerate(INTERVALS)},
            released=True)
    if mode!='upload' or frames or len(samples)!=4 or len(uploads)!=16:
        raise ValueError('incomplete isolated upload plan')
    previous=0
    for n,s in enumerate(samples):
        integer_row(s,7)
        if (s[0]<=previous or s[1]!=ORDINALS[n] or not 0<s[3]<=2048 or not 0<s[4]<=32768 or not 0<s[5]<=131072 or not 0<=s[6]<=2048):
            raise ValueError('invalid fresh upload sample or geometry capacity')
        previous=s[0]
    for i,row in enumerate(uploads):
        integer_row(row,8)
        if row[:2]!=list(divmod(i,4)) or row[5]==0 or row[7]<row[5]:
            raise ValueError('invalid upload sequence or missing submission completion')
        # Completion may execute before queue.submit returns. Do not invent
        # GPU or present duration by subtracting submit CPU end from it.
    fields=dict(create_encoder=2,real_update_buffers=3,finish_encoder=4,real_queue_submit=6)
    timing=lambda selected:{name:distribution([r[index]/1e6 for r in selected]) for name,index in fields.items()}
    snapshots=shutdown.get('snapshots')
    if (not isinstance(snapshots,list) or len(snapshots)!=3
            or [s.get('stage') for s in snapshots]!=['before_probe','created_probe','released_probe']):
        raise ValueError('missing upload allocator lifecycle observations')
    return dict(scope='separate renderer on same queue, real upload-only API calls; extra workload not normal drawing/submit/present or GPU total budget',
        timings_ms=timing(uploads),first_call_ms=timing(uploads[:1]),warm_calls_ms=timing(uploads[1:]),
        completion_from_submit_begin_ms=distribution([(r[7]-r[5])/1e6 for r in uploads]),
        geometry=[s[3:] for s in samples],source_clone_and_tessellation_ms=distribution([s[2]/1e6 for s in samples]),
        snapshots=snapshots,released=True)

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True);args=p.parse_args()
    output=(ROOT/args.output).resolve()
    if os.name!='nt' or not ordinary(token_info()) or output.exists() or ROOT/'artifacts' not in output.parents:
        raise ValueError('ordinary Windows runner and fresh artifacts required; no elevation attempted')
    output.mkdir(parents=True);initial=fingerprint()
    proof=dict(schema=1,identity=initial,runner_token=token_info(),rounds=[],cpu_meter_validated=False,
        p2_a3_accepted=False,diagnostic_complete=False,uac_requested=False)
    for mode,flag in [('markers','--verify-gpu-path-gui'),('upload','--verify-upload-api-gui')]:
        for i in [1,2]:
            row=dict(mode=mode,round=i);destination=output/('%s-%d'%(mode,i))
            try:
                if fingerprint()!=initial:raise ValueError('frozen source/binary changed')
                print('GPU path %s %d/2; diagnostic only'%(mode,i),flush=True)
                report,samples=observe(BINARY,destination,flag,timeout_seconds=45)
                metrics=summarize(samples,phases=('idle','voices_1','voices_32','closed'),minimum_spans=(('idle',.7),('voices_1',1.2),('voices_32',1.7),('closed',.7)))
                latency,joined,checks=evaluate_gui(report,metrics);checks.update(stream_checks(report,False));checks.update(audit_checks(report))
                row.update(latency=latency,joined_latency=joined,raw_metrics=metrics,original_checks=checks,diagnostic_checks=diagnostic_checks(checks),
                    memory_peak={k:max(s[k] for s in samples) for k in ('working_set_bytes','private_bytes')},
                    audio_health={k:report[k] for k in ('missing_frames','pcm_peak_bytes','pcm_final_bytes','source_raw_peak','source_raw_final',
                        'callback_resource_releases','endpoint_returns_in_render','endpoint_returns_outside_render','max_live_targets')})
                shutdown=json.loads((destination/'gpu_shutdown.json').read_text(encoding='utf8'))
                row['path']=path(report.get('gpu_path'),shutdown,mode)
            except (OSError,ValueError,KeyError,RuntimeError) as e:row['error']=str(e)
            proof['rounds'].append(row)
    proof['unchanged']=fingerprint()==initial
    proof['diagnostic_complete']=proof['unchanged'] and all('error' not in r and all(r['diagnostic_checks'].values()) for r in proof['rounds'])
    (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if not proof['diagnostic_complete']:raise SystemExit(1)

if __name__=='__main__':main()
