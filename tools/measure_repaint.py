"""Strict ordinary-token references, drained/calibrated BEFORE product admission.

Two separate owned ETW sessions; only collectors ask for UAC. No retries,
reference threshold changes, sound policy changes, or CPU corrections.
"""
import argparse
from collections import Counter
import hashlib
import json
import os
import platform
from pathlib import Path
import subprocess
from measure_audio_load import ROOT
from measure_cpu_threads import token_info,thread_costs
from measure_independent_cpu import fingerprint,build_collector,calibration
from measure_main_work import (PLAN,Collector,run_child,ordinary,reference_workload_valid,
                               main_costs,clock_mapping)


def permission(trace,runner,created,reports,expected):
    return dict(ordinary_runner=ordinary(runner),observer_identity=trace.get("observed_runner") is True
        and trace.get("root_pid")==os.getpid() and trace.get("runner_created")==created,
        native_token=trace.get("child_elevated")==0 and trace.get("child_token_type")==runner.get("type"),
        own_targets=len(reports)==expected and len({r.get("runner_pid") for r in reports})==expected and len({r.get("pid") for r in reports})==expected
        and all(r.get("runner_pid",0)>0 and r["runner_pid"]==r.get("owner",{}).get("pid")
                and ordinary(r.get("token")) and ordinary(r.get("owner",{}).get("token"))
                and ("product_token" not in r or ordinary(r["product_token"])) for r in reports))


def require_references(references):
    if len(references)!=2 or not all(v.get("passed") is True for v in references):
        raise ValueError("strict independent references failed; no product admission")


def repaint_costs(trace,report,base):
    data=report["report"].get("repaints")
    if not isinstance(data,dict) or data.get("schema")!=1 or any(data.get(k)!=0 for k in ("invalid","overflow","pending")):
        raise ValueError("incomplete repaint observation")
    requests,passes,inputs,sources=(data[k] for k in ("requests","passes","inputs","sources"))
    if not (isinstance(requests,list) and 0<len(requests)<=32768 and isinstance(passes,list) and 0<len(passes)<=4096
            and isinstance(inputs,list) and 0<len(inputs)<=4096 and isinstance(sources,list) and 0<len(sources)<=128):
        raise ValueError("repaint observation capacity")
    offset,clock=clock_mapping(report["clock_pairs"],trace["frequency"])
    w=base["costs"]["voices_32"];a,b=w["qpc_begin"],w["qpc_end"]
    def inside(epoch):
        if type(epoch) is not int or epoch<=0:raise ValueError("invalid repaint clock")
        at=(epoch*trace["frequency"]-offset)//1_000_000_000
        return a<=at<b
    delays=Counter();request_passes=set();all_passes=set();causes=Counter();events=[0]*7;input_calls=pointer_calls=empty_calls=0
    for row in requests:
        if not isinstance(row,list) or len(row)!=3 or any(type(v) is not int or v<0 for v in row):raise ValueError("invalid repaint request")
        if inside(row[0]):delays[row[2]]+=1;request_passes.add(row[1])
    for row in passes:
        if not isinstance(row,dict) or type(row.get("pass")) is not int or row["pass"] in all_passes:raise ValueError("duplicate or bad cause pass")
        all_passes.add(row["pass"])
        if not isinstance(row.get("causes"),list) or len(row["causes"])>64:raise ValueError("cause capacity")
        for i in row["causes"]:
            if type(i) is not int or not 0<=i<len(sources):raise ValueError("unknown repaint source")
        if inside(row["at"]):causes.update(row["causes"])
    for row in inputs:
        if not isinstance(row,list) or len(row)!=9 or any(type(v) is not int or v<0 for v in row):raise ValueError("invalid input counters")
        if inside(row[0]):
            input_calls+=1
            for i,v in enumerate(row[2:]):events[i]+=v
            pointer_calls+=bool(row[2] or row[3]);empty_calls+=not any(row[2:8])
    locations=[]
    for i,n in causes.most_common():
        s=sources[i]
        file=bytes(s["file_utf8"]).decode("utf8")
        if len(file)>160 or file.startswith(("/","\\")) or ":" in file or type(s["line"]) is not int:raise ValueError("invalid source location")
        locations.append(dict(file=file,line=s["line"],count=n))
    return dict(clock=clock,requests=sum(delays.values()),immediate=delays[0],delays_ns=dict(sorted(delays.items())),
                requesting_passes=len(request_passes),previous_pass_cause_counts=locations,
                input_calls=input_calls,pointer_input_calls=pointer_calls,empty_input_calls=empty_calls,
                input_events=dict(zip(("pointer_moved","mouse_moved","wheel","key_or_button","focus","other","focused_calls"),events)),
                scope="all egui requests (not only winning); previous-pass causes are not individual request causes; input counts contain no values")


def close_session(collector,path,runner,reports,expected):
    created=collector.created;collector.close()
    trace=json.loads((path/"trace.json").read_text())
    checks=permission(trace,runner,created,reports,expected)
    if not all(checks.values()):raise ValueError("ordinary session permission or ownership mismatch")
    return trace,checks


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("--output",required=True);args=p.parse_args()
    output=(ROOT/args.output).resolve()
    if os.name!="nt" or ROOT/"artifacts" not in output.parents or output.exists() or not ordinary(token_info()):
        raise ValueError("Windows ordinary runner and fresh artifacts directory required")
    output.mkdir();initial=fingerprint();binary=build_collector(output);runner=token_info();collector=None
    self_test=json.loads(subprocess.check_output([str(binary),"--self-test"],text=True))
    if self_test.get("lifetime_self_test") is not True:raise ValueError("collector lifetime self-test failed")
    refpath=output/"references";refs=[]
    proof=dict(schema=1,identity=initial,plan=PLAN,runner_token=runner,platform=platform.platform(),collector_self_test=self_test,
               references=[],products=[],calibrated=False,passed=False,products_started=False)
    try:
        refpath.mkdir()
        print("UAC 1/2: strict reference collector only",flush=True);collector=Collector(binary,refpath)
        for i,reverse in enumerate((False,True),1):
            print("strict ordinary reference %d/2"%i,flush=True)
            r=run_child(refpath/("reference-%d"%i),"reference",reverse=reverse);refs.append(r)
            if not reference_workload_valid(r):raise ValueError("strict reference workload failed; products not started")
        trace,checks=close_session(collector,refpath,runner,refs,2);collector=None
        proof["reference_permission"]=checks
        proof["reference_targets"]=[dict(pid=r["pid"],runner_pid=r["runner_pid"],owner=r["owner"],token=r["token"]) for r in refs]
        proof["references"]=[calibration(dict(trace,root_pid=r["pid"]),r) for r in refs]
        require_references(proof["references"]);proof["calibrated"]=True
        # No GUI can start until the reference logger is stopped, drained and
        # every strict calibration predicate has passed. Second UAC is explicit.
        prodpath=output/"products";prodpath.mkdir();reports=[]
        print("References strictly passed; UAC 2/2: product collector only",flush=True);collector=Collector(binary,prodpath)
        for i,profile in enumerate(PLAN,1):
            print("ordinary GUI %d/4 repaint observation=%s"%(i,profile),flush=True);proof["products_started"]=True
            reports.append(run_child(prodpath/("product-%d"%i),"product",profile=profile))
        trace,checks=close_session(collector,prodpath,runner,reports,4);collector=None;proof["product_permission"]=checks
        proof["product_targets"]=[dict(pid=r["pid"],runner_pid=r["runner_pid"],owner=r["owner"],token=r["token"],product_token=r["product_token"]) for r in reports]
        for profile,r in zip(PLAN,reports):
            v=thread_costs(trace,r);v.update(profile=profile)
            if profile:
                try:
                    v["main_categories"]=main_costs(trace,r,v);v["repaint_sources"]=repaint_costs(trace,r,v)
                    v["checks"]["repaint_observation_complete"]=True
                except (ValueError,KeyError) as error:v["attribution_error"]=str(error);v["checks"]["repaint_observation_complete"]=False
            v["checks"]["strict_reference_calibration_passed"]=True;proof["products"].append(v)
        proof["passed"]=all(all(v["checks"].values()) for v in proof["products"])
    except (OSError,RuntimeError,ValueError,KeyError,subprocess.SubprocessError) as error:
        proof["error"]=str(error);print(str(error),flush=True)
    finally:
        if collector:
            try:collector.close()
            except (OSError,RuntimeError) as error:proof["cleanup_error"]=str(error)
        if not proof["references"] and refs and (refpath/"trace.json").exists():
            try:
                t=json.loads((refpath/"trace.json").read_text())
                proof["references"]=[calibration(dict(t,root_pid=r["pid"]),r) for r in refs]
            except (ValueError,KeyError) as error:proof["reference_analysis_error"]=str(error)
        proof["unchanged"]=initial==fingerprint();proof["collector_sha256"]=hashlib.sha256(binary.read_bytes()).hexdigest()
        (output/"proof.json").write_text(json.dumps(proof,indent=2)+"\n")
    if not proof["passed"] or not proof["unchanged"]:raise SystemExit(1)


if __name__=="__main__":main()
