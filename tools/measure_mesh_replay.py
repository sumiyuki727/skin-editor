"""Two fixed native same-output CPU replays. No GPU upload or CPU acceptance.

Pure/framework tessellation and bounded CPU copies retain their own raw times;
no subtraction, invented framework remainder, retry or audio-policy change.
"""
import argparse
import json
import os
import struct
from measure_audio_load import ROOT,observe,summarize,distribution
from measure_cpu_threads import token_info
from measure_independent_cpu import BINARY,fingerprint
from measure_main_work import ordinary
from measure_ui_wall import widget_wall,diagnostic_checks
from measure_gui_candidate import evaluate_gui
from measure_background_stream import stream_checks,audit_checks

ORDINALS=(1,33,65,97)
PLAN=(0,1,2,3,2,1,0)
TIMES=dict(shape_clone_core=5,pure_tessellation=6,shape_clone_api=7,context_tessellation=8,
           fresh_allocate_and_copy=9,reserved_copy=10,temporary_release=16)

def replay(data,shutdown):
    if (not isinstance(data,dict) or data!=shutdown or data.get('schema')!=1 or data.get('invalid')!=0
            or data.get('scratch_active') is not False or data.get('error') is not None):
        raise ValueError('incomplete/rejected replay or scratch cleanup')
    samples=data.get('samples');rows=data.get('rows');stride=data.get('vertex_stride')
    if (not isinstance(samples,list) or len(samples)!=4 or not isinstance(rows,list) or len(rows)!=112
            or type(stride) is not int or not 0<stride<=256):raise ValueError('missing replay plan or stride')
    previous=-1
    for n,s in enumerate(samples):
        if (not isinstance(s,list) or len(s)!=11 or any(type(v) is not int or v<0 for v in s)
                or s[0]<=previous or s[1]!=ORDINALS[n] or not 0<s[2]<=2048 or not 0<s[9]<=2048
                or not 0<s[10]<=512 or not 1024<=s[7]<=16384 or s[8]<32 or s[7]*s[8]*4>4*1024**2
                or not 0<s[6]<=0xffffffff):raise ValueError('invalid fresh-output sample')
        scale=struct.unpack('<f',struct.pack('<I',s[6]))[0]
        if not 0<scale<100:raise ValueError('invalid output scale')
        previous=s[0]
        selected=rows[n*28:(n+1)*28]
        for k,row in enumerate(selected):
            repeat,position=divmod(k,7)
            if (not isinstance(row,list) or len(row)!=17 or any(type(v) is not int or v<0 for v in row)
                    or row[:5]!=[s[0],n,repeat,position,PLAN[position]] or row[11]>32768 or row[12]>131072
                    or row[13]>2048 or row[14]>row[13] or row[15]!=row[11]*stride+row[12]*4):
                raise ValueError('incomplete/reordered replay or invalid mesh packing')
            if PLAN[position]==3 and any(row[11:16]):raise ValueError('empty cohort produced geometry')
            if PLAN[position]==1 and row[14]!=0:raise ValueError('callback in text cohort')
            if position>=4 and row[11:16]!=selected[repeat*7+6-position][11:16]:
                raise ValueError('same-output bracket geometry changed')
        if s[5]<s[3]+s[4]+sum(sum(r[i] for i in TIMES.values()) for r in selected):
            raise ValueError('replay time partition exceeds whole observer wall')
    values={}
    for kind,name in enumerate(('all','text','other','empty')):
        selected=[r for r in rows if r[4]==kind]
        values[name]=dict(count=len(selected),timing_ms={label:distribution([r[i]/1e6 for r in selected]) for label,i in TIMES.items()},
            vertex_range=[min(r[11] for r in selected),max(r[11] for r in selected)],
            index_range=[min(r[12] for r in selected),max(r[12] for r in selected)],
            packed_bytes_range=[min(r[15] for r in selected),max(r[15] for r in selected)],
            paired_order={order:{label:distribution([r[i]/1e6 for r in selected if r[2]%2==parity]) for label,i in TIMES.items()}
                for parity,order in enumerate(('core_first','api_first'))})
    return dict(scope='same fresh output CPU WALL replay; not GPU upload, acoustic latency, CPU-budget correction or whole-framework decomposition',
        cohorts=values,sample_observer_wall_ms=distribution([s[5]/1e6 for s in samples]),
        atlas_metadata_wall_ms=distribution([s[4]/1e6 for s in samples]),
        exact_core_api_and_copy_checks=True,scratch_active_final=False)

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',required=True)
    args=parser.parse_args();output=(ROOT/args.output).resolve()
    if os.name!='nt' or not ordinary(token_info()) or output.exists() or ROOT/'artifacts' not in output.parents:
        raise ValueError('ordinary Windows runner and fresh artifacts required; no elevation attempted')
    output.mkdir(parents=True);initial=fingerprint()
    proof=dict(schema=1,identity=initial,runner_token=token_info(),rounds=[],cpu_meter_validated=False,cpu_budget_passed=None,
        p2_a3_accepted=False,diagnostic_complete=False,uac_requested=False)
    for i in [1,2]:
        row=dict(round=i)
        try:
            if fingerprint()!=initial:raise ValueError('frozen source/binary changed')
            print('Same-output mesh replay %d/2; no CPU acceptance'%i,flush=True)
            report,samples=observe(BINARY,output/('run-%d'%i),'--verify-mesh-replay-gui',timeout_seconds=45)
            metrics=summarize(samples,phases=('idle','voices_1','voices_32','closed'),minimum_spans=(('idle',.7),('voices_1',1.2),('voices_32',1.7),('closed',.7)))
            latency,joined,checks=evaluate_gui(report,metrics);checks.update(stream_checks(report,False));checks.update(audit_checks(report))
            row.update(latency=latency,joined_latency=joined,raw_metrics=metrics,original_checks=checks,diagnostic_checks=diagnostic_checks(checks),
                memory_peak={k:max(s[k] for s in samples) for k in ('working_set_bytes','private_bytes')},
                audio_health={k:report[k] for k in ('missing_frames','pcm_peak_bytes','pcm_final_bytes','source_raw_peak','source_raw_final',
                    'callback_resource_releases','endpoint_returns_in_render','endpoint_returns_outside_render','max_live_targets')})
            shutdown=json.loads((output/('run-%d'%i)/'mesh_shutdown.json').read_text(encoding='utf8'))
            row['replay']=replay(report.get('mesh_replay'),shutdown)
            row['wall']=widget_wall(report.get('main_work'),minimum_calls=128)
        except (OSError,ValueError,KeyError,RuntimeError) as e:row['error']=str(e)
        proof['rounds'].append(row)
    proof['unchanged']=fingerprint()==initial
    proof['diagnostic_complete']=proof['unchanged'] and len(proof['rounds'])==2 and all('error' not in r and all(r['diagnostic_checks'].values()) for r in proof['rounds'])
    (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if not proof['diagnostic_complete']:raise SystemExit(1)

if __name__=='__main__':main()
