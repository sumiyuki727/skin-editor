"""Fixed baseline/candidate/candidate/baseline, all with identical observations.
Strict references stop/drain/calibrate before product admission. No retries.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
from measure_audio_load import ROOT
from measure_cpu_threads import token_info,thread_costs
from measure_independent_cpu import BINARY,fingerprint,build_collector,calibration
from measure_main_work import Collector,run_child,ordinary,reference_workload_valid,main_costs
from measure_repaint import close_session,require_references,repaint_costs

PLAN=("baseline","candidate","candidate","baseline")
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def candidate_passed(rows):
    selected=[r for r in rows if r.get("role")=="candidate"]
    required={'independent_cpu_all_stable_phases_le_15_one_core','zero_missing_frames','gui_request_copy_observed_upper_p95_le_20_ms'}
    return len(rows)==4 and tuple(r.get("role") for r in rows)==PLAN and len(selected)==2 and all(required<=r['checks'].keys() and all(r["checks"].values()) for r in selected)

def comparison_checks(values,reports):
    """Ordinary-window ABBA. No input suppression or measured-value correction.

    The old binary records target size/generation, not actual window/DPI; do
    not advertise those absent fields as newly measured geometry evidence.
    """
    if len(values)!=4 or len(reports)!=4 or tuple(v.get('role') for v in values)!=PLAN:
        raise ValueError('incomplete or reordered ABBA plan')
    if any(not all(k in v for k in ('main_categories','repaint_sources','adapter')) for v in values):
        return dict(observations_complete_for_comparison=False)
    signatures=[];rates=[];focus=[]
    for value,report in zip(values,reports):
        r=report['report']
        if r.get('gui_control','missing') is not None:raise ValueError('ordinary editor path required')
        visual=r['visual'][4]
        if not isinstance(visual,list) or len(visual)!=10 or visual[0]!=5 or any(type(v) is not int or v<=0 for v in visual[5:8]):
            raise ValueError('missing preview target identity')
        signatures.append((value['adapter'],r['target_format_utf8'],r['rate'],r['channels'],r['period_frames'],r['source_frames'],visual[5:8]))
        rates.extend((value['main_categories']['ui_calls_per_second'],value['main_categories']['present_edges_per_second']))
        repaint=value['repaint_sources'];events=repaint['input_events'];n=repaint['input_calls']
        focus.append(None if n<=0 else False if events['focused_calls']==0 else True if events['focused_calls']==n else None)
    return dict(same_device_format_audio_target=all(s==signatures[0] for s in signatures),
        same_actual_refresh_band=all(math.isfinite(r) and 55<=r<=65 for r in rates) and max(rates)/min(rates)<=1.05,
        no_immediate_repaint=all(v['repaint_sources']['immediate']==0 for v in values),
        no_physical_input=all(not any(n for k,n in v['repaint_sources']['input_events'].items() if k!='focused_calls') for v in values),
        real_window_focus_matches=focus[0] is not None and all(f is focus[0] for f in focus))

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("--output",required=True);p.add_argument("--baseline",required=True);args=p.parse_args()
    output=(ROOT/args.output).resolve();baseline=(ROOT/args.baseline).resolve()
    if os.name!="nt" or output.exists() or ROOT/"artifacts" not in output.parents or ROOT/"artifacts" not in baseline.parents or not ordinary(token_info()):
        raise ValueError("ordinary Windows runner, owned baseline and fresh artifacts required")
    output.mkdir();initial=fingerprint();baseline_sha=digest(baseline);binary=build_collector(output);collector=None;refs=[];refpath=output/"references"
    self_test=json.loads(subprocess.check_output([str(binary),"--self-test"],text=True))
    if self_test.get("lifetime_self_test") is not True:raise ValueError("collector self test failed")
    runner=token_info();proof=dict(schema=2,identity=initial,baseline_sha256=baseline_sha,plan=PLAN,runner_token=runner,
        references=[],products=[],calibrated=False,comparison_valid=False,passed=False,products_started=False,collector_self_test=self_test)
    try:
        refpath.mkdir();print("UAC 1/2: strict references",flush=True);collector=Collector(binary,refpath)
        for i,reverse in enumerate((False,True),1):
            print("Strict reference %d/2"%i,flush=True);r=run_child(refpath/("reference-%d"%i),"reference",reverse=reverse);refs.append(r)
            if not reference_workload_valid(r):raise ValueError("reference workload invalid; no products")
        trace,checks=close_session(collector,refpath,runner,refs,2);collector=None;proof["reference_permission"]=checks
        proof["reference_targets"]=[dict(pid=r["pid"],runner_pid=r["runner_pid"],owner=r["owner"],token=r["token"]) for r in refs]
        proof["references"]=[calibration(dict(trace,root_pid=r["pid"]),r) for r in refs]
        require_references(proof["references"]);proof["calibrated"]=True
        prodpath=output/"products";prodpath.mkdir();reports=[]
        print("References passed; UAC 2/2: fixed A/B/B/A",flush=True);collector=Collector(binary,prodpath)
        for i,role in enumerate(PLAN,1):
            selected=baseline if role=="baseline" else BINARY
            expected=baseline_sha if role=="baseline" else initial["product_sha256"]
            if digest(selected)!=expected or fingerprint()!=initial:raise ValueError("frozen source/binary changed before admission")
            print("GUI %d/4 %s (all observations enabled)"%(i,role),flush=True);proof["products_started"]=True
            reports.append(run_child(prodpath/("product-%d"%i),"product",profile=True,binary=selected))
        trace,checks=close_session(collector,prodpath,runner,reports,4);collector=None;proof["product_permission"]=checks
        proof["product_targets"]=[dict(pid=r["pid"],runner_pid=r["runner_pid"],owner=r["owner"],token=r["token"],product_token=r["product_token"],product_name=r["product_name"]) for r in reports]
        for role,r in zip(PLAN,reports):
            v=thread_costs(trace,r);v["role"]=role;v["checks"]["strict_references_passed"]=True
            try:
                v["main_categories"]=main_costs(trace,r,v);v["repaint_sources"]=repaint_costs(trace,r,v);v["checks"]["observations_complete"]=True
            except (ValueError,KeyError) as error:v["attribution_error"]=str(error);v["checks"]["observations_complete"]=False
            proof["products"].append(v)
        proof['comparison_checks']=comparison_checks(proof['products'],reports)
        proof['comparison_valid']=all(proof['comparison_checks'].values())
        proof["passed"]=proof['comparison_valid'] and candidate_passed(proof["products"])
    except (OSError,RuntimeError,ValueError,KeyError,subprocess.SubprocessError) as error:
        proof["error"]=str(error);print(str(error),flush=True)
    finally:
        if collector:
            try:collector.close()
            except (OSError,RuntimeError) as error:proof["cleanup_error"]=str(error);proof["passed"]=False
        if not proof["references"] and refs and (refpath/"trace.json").exists():
            try:
                t=json.loads((refpath/"trace.json").read_text());proof["references"]=[calibration(dict(t,root_pid=r["pid"]),r) for r in refs]
            except (ValueError,KeyError) as error:proof["reference_analysis_error"]=str(error)
        proof["unchanged"]=initial==fingerprint() and baseline_sha==digest(baseline)
        proof["collector_sha256"]=digest(binary)
        (output/"proof.json").write_text(json.dumps(proof,indent=2)+"\n")
    if not proof["passed"] or not proof["unchanged"]:raise SystemExit(1)

if __name__=="__main__":main()
