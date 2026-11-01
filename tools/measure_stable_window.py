"""Fixed B01/B05 windows; --product selects ordinary assembly, otherwise legacy control."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess

from measure_audio_load import ROOT, observe, summarize
from measure_combined import GpuSampler
from measure_cpu_threads import token_info
from measure_independent_cpu import (stamp_api, fingerprint, build_collector, calibration,
                                     scheduler_intervals, scheduled_cost, overlap_diagnostic)


def phases_for(mode):
    if mode == "stable-empty": return ("empty",)
    if mode == "stable-resources": return ("paused", "closed")
    raise ValueError("explicit stable mode required")


def launch_mode(mode, product):
    phases_for(mode)
    if type(product) is not bool:raise ValueError("explicit boolean product policy required")
    return ("product-empty" if mode=="stable-empty" else "product-stable") if product else mode


def report_checks(report, mode, product=None):
    phases = phases_for(mode)
    rows = report.get("rows")
    if (report.get("schema") != 1 or report.get("scope") != "full-editor-stable-windows"
            or report.get("completed") is not True or report.get("resources_case") is not (mode == "stable-resources")
            or not isinstance(rows, list) or any(not isinstance(r,dict) for r in rows)
            or tuple(r.get("phase") for r in rows) != phases):
        raise ValueError("incomplete stable protocol or extra/missing phase")
    for r in rows:
        if (any(type(r.get(k)) is not int or r[k] < 0 for k in ("span_us", "draws", "inputs"))
                or type(r.get("frozen")) is not bool or type(r.get("view_valid")) is not bool
                or not isinstance(r.get("geometry"),list) or len(r["geometry"])!=3
                or any(type(v) not in (int,float) or not math.isfinite(v) or v<=0 for v in r["geometry"])
                or any(not isinstance(r.get(k), list) or len(r[k]) != n
                       or any(type(v) is not int or v < 0 for v in r[k]) for k,n in (("before",5),("after",5),("shape",3)))):
            raise ValueError("invalid stable observation")
    checks = dict(full_65_seconds=all(r["span_us"] >= 65_000_000 for r in rows),
                  no_continuous_draws=all(r["draws"] <= 2 for r in rows),
                  no_physical_input=all(r["inputs"] == 0 for r in rows),
                  visible_unchanged_geometry=all(r["view_valid"] for r in rows),
                  simulation_and_background_frozen=all(r["frozen"] for r in rows))
    empty = next((r for r in rows if r["phase"] in ("empty", "closed")), None)
    checks["empty_or_closed_resources_released"] = empty is not None and empty["before"] == empty["after"] == [0]*5 and empty["shape"] == [0]*3
    if mode == "stable-resources":
        r = rows[0]
        checks["paused_keeps_standard_resources"] = report.get("shape") == r["shape"] == [100,10,500] and all(v>0 for v in r["before"][:4]) and r["before"] == r["after"]
        n = report["native"]
        if (any(type(n.get(k)) is not int or n[k]<0 for k in ("callbacks","missing_frames","render_errors","observation_overflows",
                "callback_resource_releases","endpoint_returns_in_render","endpoint_returns_outside_render"))
                or type(n.get("device_fault")) is not bool): raise ValueError("missing stopped native audit")
        checks["native_healthy_zero_missing"] = n["callbacks"]>0 and not n["device_fault"] and n["missing_frames"] == n["render_errors"] == n["observation_overflows"] == 0
        checks["native_ownership_and_source_released"] = n["callback_resource_releases"] == n["endpoint_returns_in_render"] == 0 and n["endpoint_returns_outside_render"] == 1 and report.get("source_raw_final") == 0
    else:
        checks["empty_project_has_no_sample"] = report.get("shape") == [0,0,0]
    if product is not None:
        launch_mode(mode,product)
        checks["requested_runtime_policy"] = (report.get("production_policy") is product
                                               and report.get("gpu_memory_usage_hint") is product)
    return checks


def observe_product(output, binary, mode, product=False):
    stamp, frequency = stamp_api()
    pids, tokens = [], []
    class Sampler(GpuSampler):
        def __init__(self, child):
            super().__init__(child)
            pids.append(child.pid)
            tokens.append(token_info(self.handle))
        def sample(self, phase):
            a = stamp()
            row = super().sample(phase)
            row.update(qpc_begin=a, qpc_end=stamp())
            return row
    report, samples = observe(binary, output/"gui", "--verify-"+launch_mode(mode,product),
                              timeout_seconds=215 if mode=="stable-resources" else 90, sampler_factory=Sampler)
    phases = phases_for(mode)
    metrics = summarize(samples, phases, tuple((p,60.) for p in phases))
    return dict(pid=pids[0], frequency=frequency, report=report, samples=samples, metrics=metrics,
                original_checks=report_checks(report,mode,product), product_token=tokens[0], mode=mode)


def analyze(trace, record, calibrated, product=None):
    intervals = scheduler_intervals(trace)
    if record["frequency"] != trace["frequency"]: raise ValueError("QPC frequency changed")
    if not any(p==record["pid"] for p,t,c,a,b in intervals): raise ValueError("owned process absent from trace")
    overlap = overlap_diagnostic(intervals, trace["frequency"])
    checks = dict(record["original_checks"])
    # Recheck the actual report against the parent request. Never let a child's
    # successful legacy-control predicates promote it into product acceptance.
    checks.update(report_checks(record["report"],record["mode"],product))
    checks.update(strict_references_passed=calibrated is True, scheduler_overlap_valid=overlap["valid"])
    costs, gpu = {}, {}
    for phase in phases_for(record["mode"]):
        selected = [s for s in record["samples"] if s["phase"]==phase]
        if len(selected)<2: raise ValueError("stable samples missing")
        a,b = selected[0]["qpc_end"], selected[-1]["qpc_end"]
        costs[phase] = scheduled_cost(intervals,record["pid"],a,b,trace["frequency"])
        checks[phase+"_independent_cpu_le_2_percent"] = costs[phase]["span_seconds"]>=60 and costs[phase]["cpu_one_core_percent"]<=2
        gpu[phase] = {}
        for kind,limit in (("dedicated",256),("shared",128)):
            values = [s.get("gpu_"+kind+"_bytes") for s in selected]
            valid = [v for v in values if type(v) is int and v>=0]
            gpu[phase][kind] = dict(valid_samples=len(valid),missing_samples=len(values)-len(valid),max_bytes=max(valid) if valid else None)
            checks[phase+"_gpu_"+kind+"_budget"] = len(valid)==len(values) and bool(valid) and max(valid)<=limit*1024**2
    samples = record["samples"]
    memory = {k:max(s[k] for s in samples) for k in ("working_set_bytes","private_bytes")}
    checks["process_memory_le_512_mib"] = all(0<v<=512*1024**2 for v in memory.values())
    if record["mode"]=="stable-empty": checks["empty_project_working_set_le_200_mib"] = 0<memory["working_set_bytes"]<=200*1024**2
    return dict(mode=record["mode"],costs=costs,gpu=gpu,all_lifecycle_sampled_memory=memory,
                raw_cpu=record["metrics"],report=record["report"],checks=checks,passed=all(checks.values()),
                scope="B01/B05 full stable windows; B06 is not the stable-idle 2% hard gate; B09 and normal 300s untested")


def main():
    from measure_main_work import Collector, run_child, ordinary, reference_workload_valid
    from measure_repaint import close_session, require_references
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--binary",required=True);p.add_argument("--output",required=True)
    p.add_argument("--product",action="store_true",help="Ordinary Windows assembly; require both product-policy markers")
    args=p.parse_args();output=(ROOT/args.output).resolve();binary=Path(args.binary).resolve()
    if (os.name!="nt" or not ordinary(token_info()) or output.exists() or ROOT/"artifacts" not in output.parents
            or ROOT/"artifacts" not in binary.parents or binary.name!="skin-editor-desktop.exe"):
        raise ValueError("ordinary runner, frozen owned binary and fresh artifacts output required")
    output.mkdir();initial=fingerprint();digest=hashlib.sha256(binary.read_bytes()).hexdigest()
    observer_binary=build_collector(output);runner=token_info();collector=None;refs=[]
    proof=dict(schema=2,identity=initial,binary_sha256=digest,references=[],products=[],calibrated=False,
               requested_production_policy=args.product,b05_product_accepted=False,
               plan=["stable-empty","stable-empty","stable-resources","stable-resources"],passed=False,p2_accepted=False)
    try:
        test=json.loads(subprocess.check_output([str(observer_binary),"--self-test"],text=True))
        if test.get("lifetime_self_test") is not True:raise ValueError("collector lifetime tests failed")
        proof["collector_self_test"]=test
        path=output/"references";path.mkdir()
        print("UAC 1/4: two strict references",flush=True);collector=Collector(observer_binary,path)
        for i,reverse in enumerate((False,True),1):
            print("strict reference %d/2"%i,flush=True)
            refs.append(run_child(path/("reference-%d"%i),"reference",reverse=reverse))
            if not reference_workload_valid(refs[-1]):raise ValueError("invalid reference; stable targets not admitted")
        trace,permissions=close_session(collector,path,runner,refs,2);collector=None
        proof["reference_permission"]=permissions
        proof["references"]=[calibration(dict(trace,root_pid=r["pid"]),r) for r in refs]
        require_references(proof["references"]);proof["calibrated"]=True
        for session,(mode,count) in enumerate((("stable-empty",2),("stable-resources",1),("stable-resources",1)),2):
            path=output/("session-%d"%session);path.mkdir();records=[]
            print("UAC %d/4: %s, %d fixed targets"%(session,mode,count),flush=True)
            collector=Collector(observer_binary,path)
            for i in range(count):
                if fingerprint()!=initial or hashlib.sha256(binary.read_bytes()).hexdigest()!=digest:raise ValueError("frozen source/binary changed")
                records.append(run_child(path/("product-%d"%(i+1)),"product",binary=binary,ui_mode=launch_mode(mode,args.product)))
            trace,permissions=close_session(collector,path,runner,records,count);collector=None
            for record in records:
                value=analyze(trace,record,proof["calibrated"],product=args.product);value["permission_checks"]=permissions
                proof["products"].append(value)
            # A hard failure ends this bounded plan. Never rerun for a quiet window.
            if any(not v["passed"] for v in proof["products"]):raise ValueError("stable hard check failed; no later targets or retry")
        proof["passed"]=len(proof["products"])==4 and all(v["passed"] for v in proof["products"])
    except (OSError,RuntimeError,ValueError,KeyError,subprocess.SubprocessError) as error:
        proof["error"]=str(error);print(str(error),flush=True)
    finally:
        if collector:
            try:collector.close()
            except (OSError,RuntimeError) as error:proof["cleanup_error"]=str(error);proof["passed"]=False
        proof["reference_report_paths"]=["references/reference-%d/report.json"%(i+1) for i in range(len(refs))]
        proof["unchanged"]=initial==fingerprint() and digest==hashlib.sha256(binary.read_bytes()).hexdigest()
        proof["b05_product_accepted"]=args.product and proof["passed"] and proof["unchanged"]
        (output/"proof.json").write_text(json.dumps(proof,indent=2)+"\n",encoding="utf8")
    if not proof["passed"] or not proof["unchanged"]:raise SystemExit(1)


if __name__=="__main__":main()
