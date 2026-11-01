"""Two ordinary native GUI text censuses; wall time is not CPU acceptance.

No alternate drawing, font replacement, UAC, ETW, retry or audio-policy change.
"""
import argparse
import json
import os
from pathlib import Path
from measure_audio_load import ROOT, observe, summarize, distribution
from measure_cpu_threads import token_info
from measure_independent_cpu import BINARY, fingerprint
from measure_main_work import ordinary
from measure_ui_wall import widget_wall, diagnostic_checks
from measure_gui_candidate import evaluate_gui
from measure_background_stream import stream_checks, audit_checks


def census(data,shutdown):
    if (not isinstance(data,dict) or data.get('schema')!=1 or data.get('invalid')!=0
            or not isinstance(shutdown,dict) or shutdown.get('schema')!=1 or shutdown.get('invalid')!=0 or shutdown.get('weak_identities')!=0):
        raise ValueError('incomplete text census/cleanup')
    rows=data.get('rows');previous=-1
    if not isinstance(rows,list) or not 128<=len(rows)<=1024 or not 0<=data.get('weak_identities',-1)<=512:
        raise ValueError('text census capacity')
    final_rows=shutdown.get('rows')
    if not isinstance(final_rows,list) or not len(rows)<=len(final_rows)<=1024 or final_rows[:len(rows)]!=rows:
        raise ValueError('cleanup does not match observed text census')
    for row in rows:
        if (not isinstance(row,list) or len(row)!=16 or any(type(v) is not int or v<0 for v in row)
                or row[0]<=previous or row[1]>17 or not 0<=row[3]<=row[2]<=512 or row[4]>row[2]
                or not 0<row[8]<=16384 or not 0<row[9]<=16384 or row[10]>1_000_000):
            raise ValueError('invalid text pass/count/atlas')
        previous=row[0]
    def summary(selected):
        text=sum(r[2] for r in selected)
        return dict(passes=len(selected),text_instances=text,reused_previous_pass=sum(r[3] for r in selected),
            reused_percent=100*sum(r[3] for r in selected)/text if text else 0,
            row_mesh_vertices=sum(r[5] for r in selected),row_mesh_indices=sum(r[6] for r in selected),
            cache_entries_range=[min(r[7] for r in selected),max(r[7] for r in selected)],
            atlas_sizes=sorted({(r[8],r[9]) for r in selected}),full_atlas_updates=sum(r[11] for r in selected),
            partial_atlas_updates=sum(r[12] for r in selected),survey_wall_ms=distribution([r[15]/1e6 for r in selected]))
    stable=[r for r in rows if r[1]==4]
    if len(stable)<100:raise ValueError('missing 32-voice text sample')
    return dict(scope='fresh pre-tessellation output; weak galley identity reuse, not cache hit count or final GPU mesh cost',
        all_passes=summary(rows),voices_32=summary(stable),weak_identities_final=0)


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',required=True)
    args=parser.parse_args();output=(ROOT/args.output).resolve()
    if os.name!='nt' or not ordinary(token_info()) or output.exists() or ROOT/'artifacts' not in output.parents:
        raise ValueError('ordinary Windows runner and fresh artifacts directory required; no elevation attempted')
    output.mkdir(parents=True);initial=fingerprint()
    proof=dict(schema=1,identity=initial,runner_token=token_info(),rounds=[],cpu_meter_validated=False,
        cpu_budget_passed=None,p2_a3_accepted=False,diagnostic_complete=False,uac_requested=False)
    for i in [1,2]:
        row=dict(round=i)
        try:
            if fingerprint()!=initial:raise ValueError('frozen source/binary changed before admission')
            print('Text work native %d/2; no CPU acceptance'%i,flush=True)
            report,samples=observe(BINARY,output/('run-%d'%i),'--verify-text-work-gui',timeout_seconds=45)
            metrics=summarize(samples,phases=('idle','voices_1','voices_32','closed'),
                minimum_spans=(('idle',.7),('voices_1',1.2),('voices_32',1.7),('closed',.7)))
            latency,joined,checks=evaluate_gui(report,metrics)
            checks.update(stream_checks(report,False));checks.update(audit_checks(report))
            row.update(latency=latency,joined_latency=joined,raw_metrics=metrics,original_checks=checks,
                diagnostic_checks=diagnostic_checks(checks),memory_peak={k:max(s[k] for s in samples) for k in ('working_set_bytes','private_bytes')},
                audio_health={k:report[k] for k in ('missing_frames','pcm_peak_bytes','pcm_final_bytes','source_raw_peak','source_raw_final',
                    'callback_resource_releases','endpoint_returns_in_render','endpoint_returns_outside_render','max_live_targets')})
            shutdown=json.loads((output/('run-%d'%i)/'text_shutdown.json').read_text(encoding='utf8'))
            row['text']=census(report.get('text_work'),shutdown)
            row['wall']=widget_wall(report.get('main_work'),minimum_calls=128)
        except (OSError,RuntimeError,ValueError,KeyError) as error:row['error']=str(error)
        proof['rounds'].append(row)
    proof['unchanged']=fingerprint()==initial
    proof['diagnostic_complete']=(proof['unchanged'] and len(proof['rounds'])==2
        and all('error' not in r and all(r['diagnostic_checks'].values()) for r in proof['rounds']))
    (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if not proof['diagnostic_complete']:raise SystemExit(1)


if __name__=='__main__':main()
