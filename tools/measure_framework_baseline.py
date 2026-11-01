"""Fixed F/P/E/E/P/F controls, same audio workload, window and repaint policy.

F includes common audio/project host work: it is not a pure framework process.
No difference is subtracted from the editor's unchanged acceptance predicates.
"""
import argparse
import hashlib
import json
import math
import os
import subprocess
from measure_audio_load import ROOT
from measure_cpu_threads import token_info, thread_costs
from measure_independent_cpu import fingerprint, build_collector, calibration
from measure_main_work import (Collector, run_child, ordinary, reference_workload_valid,
                               main_costs)
from measure_repaint import close_session, require_references, repaint_costs

PLAN = ('framework', 'presentation', 'editor', 'editor', 'presentation', 'framework')
MAX_RATE_RATIO = 1.05  # Comparability only; never a CPU correction or frame limiter.


def control_checks(report, role):
    c = report.get('gui_control')
    if role not in PLAN or not isinstance(c,dict):raise ValueError('missing control mode')
    geometry = c.get('geometry', {})
    stages = c.get('stages', [])
    if (c.get('schema') != 3 or c.get('inactive_on_create') is not True or c.get('mouse_passthrough') is not True
            or c.get('mode') != role or c.get('geometry_changed') is not False
            or c.get('audio_policy') != 'unchanged-streamed-runtime'
            or c.get('nominal_playing_repaint_ms') != 16
            or not isinstance(geometry, dict) or geometry.get('window') != [1180., 800.]
            or not isinstance(stages,list) or len(stages) != 18 or any(not isinstance(r,list) or len(r)!=4
                or any(type(v) is not int or v<0 for v in r) or r[0]!=r[1]+r[3] or r[2]>r[0] for r in stages)):
        raise ValueError('invalid or changed control metadata')
    calls, full, paints, minimal = stages[4]
    ppp=geometry.get('pixels_per_point');preview=geometry.get('preview',[]);size=geometry.get('size',[])
    if (not isinstance(ppp,(int,float)) or not math.isfinite(ppp) or ppp<=0
            or not isinstance(preview,list) or len(preview)!=4 or any(not isinstance(v,(int,float)) or not math.isfinite(v) for v in preview)
            or not (0<=preview[0]<preview[2]<=1180 and 0<=preview[1]<preview[3]<=800)
            or not isinstance(size,list) or len(size)!=2 or any(type(v) is not int or not 0<v<=1024 for v in size)
            or type(geometry.get('generation')) is not int or geometry['generation']<=0
            or calls!=report.get('ui_cost',[[0]]*18)[4][0]
            or sum(s[0] for s in stages)!=report.get('gui_passes')):
        raise ValueError('invalid control geometry/counts')
    if calls < 100 or stages[0][0] == 0 or stages[0][0] != stages[0][1]:
        raise ValueError('control not primed or missing 32-voice frame work')
    if role == 'editor': valid = (full == calls and minimal == 0 and paints == calls)
    else: valid = (full == 0 and minimal == calls and paints == (calls if role == 'presentation' else 0))
    if not valid: raise ValueError('measured mode silently rebuilt editor or skipped intended draw')
    return dict(control_path_exercised=True, geometry_stable=True)


def comparison_checks(values, reports):
    if len(values)!=6 or len(reports)!=6 or tuple(v.get('role') for v in values)!=PLAN:
        raise ValueError('incomplete or reordered baseline plan')
    signatures=[];rates=[];focus=[]
    for value, report in zip(values, reports):
        r=report['report'];control_checks(r,value['role'])
        signatures.append((r['gui_control']['geometry'],value['adapter'],r['target_format_utf8'],
                           r['rate'],r['channels'],r['period_frames'],r['source_frames']))
        rates.extend((value['main_categories']['ui_calls_per_second'],
                      value['main_categories']['present_edges_per_second']))
        repaint=value['repaint_sources'];events=repaint['input_events'];n=repaint['input_calls']
        focus.append(None if n<=0 else False if events['focused_calls']==0 else True if events['focused_calls']==n else None)
    return dict(same_geometry_device_format_audio=all(s==signatures[0] for s in signatures),
                same_nominal_16ms_actual_rate_band=all(55<=r<=65 for r in rates) and max(rates)/min(rates)<=MAX_RATE_RATIO,
                no_immediate_repaint=all(v['repaint_sources']['immediate']==0 for v in values),
                no_physical_input=all(not any(n for k,n in v['repaint_sources']['input_events'].items() if k!='focused_calls') for v in values),
                real_window_focus_matches=all(f is False for f in focus))


def editor_passed(values):
    # Control-only success cannot authorize the product. The full editor must
    # pass every unchanged original and independent predicate in BOTH rounds.
    required={'independent_cpu_all_stable_phases_le_15_one_core','zero_missing_frames',
              'gui_request_copy_observed_upper_p95_le_20_ms'}
    return (len(values)==6 and tuple(v.get('role') for v in values)==PLAN
            and all(required<=v['checks'].keys() and all(v['checks'].values()) for v in values if v['role']=='editor'))


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',required=True)
    args=parser.parse_args();output=(ROOT/args.output).resolve()
    if os.name!='nt' or ROOT/'artifacts' not in output.parents or output.exists() or not ordinary(token_info()):
        raise ValueError('ordinary Windows runner and fresh artifacts output required')
    output.mkdir();initial=fingerprint();binary=build_collector(output);runner=token_info()
    self_test=json.loads(subprocess.check_output([str(binary),'--self-test'],text=True))
    if self_test.get('lifetime_self_test') is not True:raise ValueError('collector self test failed')
    proof=dict(schema=1,identity=initial,plan=PLAN,runner_token=runner,collector_self_test=self_test,
               references=[],products=[],calibrated=False,comparison_valid=False,editor_passed=False,
               p2_a3_accepted=False,products_started=False)
    collector=None;refs=[];refpath=output/'references'
    try:
        refpath.mkdir();print('UAC 1/2: strict references',flush=True);collector=Collector(binary,refpath)
        for i,reverse in enumerate((False,True),1):
            print('Strict reference %d/2'%i,flush=True);r=run_child(refpath/('reference-%d'%i),'reference',reverse=reverse);refs.append(r)
            if not reference_workload_valid(r):raise ValueError('reference workload failed; no GUI admission')
        trace,checks=close_session(collector,refpath,runner,refs,2);collector=None;proof['reference_permission']=checks
        proof['references']=[calibration(dict(trace,root_pid=r['pid']),r) for r in refs]
        require_references(proof['references']);proof['calibrated']=True
        prodpath=output/'products';prodpath.mkdir();reports=[]
        print('References passed; UAC 2/2: fixed F/P/E/E/P/F',flush=True);collector=Collector(binary,prodpath)
        for i,role in enumerate(PLAN,1):
            if fingerprint()!=initial:raise ValueError('frozen source/binary changed before admission')
            print('GUI %d/6 %s'%(i,role),flush=True);proof['products_started']=True
            reports.append(run_child(prodpath/('product-%d'%i),'product',profile=True,ui_mode=role))
        trace,checks=close_session(collector,prodpath,runner,reports,6);collector=None;proof['product_permission']=checks
        proof['product_targets']=[dict(pid=r['pid'],runner_pid=r['runner_pid'],owner=r['owner'],token=r['token'],product_token=r['product_token']) for r in reports]
        for role,r in zip(PLAN,reports):
            value=thread_costs(trace,r);value['role']=role;value['checks']['strict_references_passed']=True
            try:
                value['main_categories']=main_costs(trace,r,value);value['repaint_sources']=repaint_costs(trace,r,value)
                value['checks'].update(control_checks(r['report'],role));value['checks']['observations_complete']=True
            except (ValueError,KeyError) as error:
                value['attribution_error']=str(error);value['checks']['observations_complete']=False
            proof['products'].append(value)
        proof['comparison_checks']=comparison_checks(proof['products'],reports)
        proof['comparison_valid']=all(proof['comparison_checks'].values())
        proof['editor_passed']=proof['comparison_valid'] and editor_passed(proof['products'])
    except (OSError,RuntimeError,ValueError,KeyError,subprocess.SubprocessError) as error:
        proof['error']=str(error);print(str(error),flush=True)
    finally:
        if collector:
            try:collector.close()
            except (OSError,RuntimeError) as error:proof['cleanup_error']=str(error)
        proof['unchanged']=initial==fingerprint();proof['collector_sha256']=hashlib.sha256(binary.read_bytes()).hexdigest()
        (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if not proof['comparison_valid'] or not proof['editor_passed'] or not proof['unchanged']:raise SystemExit(1)


if __name__=='__main__':main()
