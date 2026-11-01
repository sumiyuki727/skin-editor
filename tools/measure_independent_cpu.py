"""Opt-in QPC scheduler accounting for owned Windows references, then product.

No ETL, stack collection, clock-frequency guess, counter correction or retries.
The collector needs Windows profiling permission; failure never runs the target.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time
from measure_audio_load import ROOT, WindowsSampler, observe, summarize
from measure_combined import source_identity
from measure_cpu_accounting import NativeReference, run_phase, PHASES, reference_checks
from measure_gui_candidate import evaluate_gui

MAX_EVENTS=1_000_000
SOURCE=ROOT/"tools/measure_cpu_trace.c"
BINARY=ROOT/"desktop/target/release/skin-editor-desktop.exe"


def stamp_api():
    kernel=c.WinDLL("kernel32",use_last_error=True)
    for name in ("QueryPerformanceCounter","QueryPerformanceFrequency"):
        f=getattr(kernel,name);f.argtypes=[c.POINTER(c.c_int64)];f.restype=w.BOOL
    def read(name="QueryPerformanceCounter"):
        value=c.c_int64()
        if not getattr(kernel,name)(c.byref(value)):raise OSError(c.get_last_error(),"QPC unavailable")
        return value.value
    return read,read("QueryPerformanceFrequency")


def scheduler_intervals(trace):
    schema=trace.get("schema")
    if (schema not in (1,2) or any(trace.get(k)!=0 for k in
            ("status","stop_status","trace_status","child_status","invalid","events_lost","buffers_lost"))
            or type(trace.get("frequency")) is not int or trace["frequency"]<=0
            or not 0 < trace.get("root_pid",0) or not 0 < trace.get("buffer_count",0)
            or not 0 < trace.get("buffers_allocated",0)<=128 or not 0 < trace.get("thread_count",0)<=512):
        raise ValueError("incomplete or lossy scheduler collection")
    events=trace.get("events")
    if not isinstance(events,list) or not 0 < len(events)<=MAX_EVENTS:
        raise ValueError("missing or unbounded owned scheduler events")
    lifetimes={}
    if schema==2:
        metadata=trace.get("thread_lifetimes")
        if not isinstance(metadata,list) or len(metadata)!=trace["thread_count"]:raise ValueError("missing lifetime metadata")
        for row in metadata:
            if (not isinstance(row,list) or len(row)!=3 or any(type(v) is not int or v<=0 for v in row)
                    or row[2] in lifetimes):raise ValueError("invalid lifetime metadata")
            lifetimes[row[2]]=tuple(row[:2])
    active={};cpus={};last={};intervals=[]
    # Timestamps may tie across CPUs. Out-before-in tie order permits migration;
    # a per-CPU reversal or an unmatched boundary fails instead of estimating.
    for row in events:
        if (not isinstance(row,list) or len(row)!=(6 if schema==2 else 5) or any(type(v) is not int or v<0 for v in row)
                or row[0]==0 or row[1]>=64 or row[2]==0 or row[3]==0 or row[4] not in (0,1)):
            raise ValueError("invalid scheduler row")
        if schema==2 and lifetimes.get(row[5])!=tuple(row[2:4]):raise ValueError("switch lifetime identity mismatch")
        if row[0]<last.get(row[1],0):raise ValueError("scheduler time reversal")
        last[row[1]]=row[0]
    for row in sorted(events,key=lambda e:(e[0],e[4])):
        at,cpu,pid,tid,enter=row[:5];key=(cpu,pid,tid,row[5] if schema==2 else 0)
        if enter:
            if key in active or cpu in cpus:raise ValueError("overlapping owned running identity")
            active[key]=(at,cpu);cpus[cpu]=key
        else:
            if key not in active:raise ValueError("unmatched owned switch out")
            begin,previous_cpu=active.pop(key)
            if previous_cpu!=cpu or cpus.pop(cpu,None)!=key or at<begin:
                raise ValueError("scheduler migration/identity mismatch")
            intervals.append((pid,tid,cpu,begin,at))
    if active:raise ValueError("unclosed owned scheduler interval")
    return intervals


def overlap_diagnostic(intervals,frequency):
    # Pair on each CPU. A migrated thread's switch-in can be timestamped just
    # before its previous CPU's switch-out. Preserve BOTH intervals, never trim
    # or deduct them. Bound/report this conservative measurement uncertainty.
    by_thread={};total=maximum=0
    for p,t,cpu,a,b in intervals:by_thread.setdefault((p,t),[]).append((a,b))
    for rows in by_thread.values():
        previous=0
        for a,b in sorted(rows):
            overlap=max(0,min(previous,b)-a);total+=overlap;maximum=max(maximum,overlap);previous=max(previous,b)
    return dict(total_overlap_ns=total*1e9/frequency,max_overlap_ns=maximum*1e9/frequency,
                conservative_intervals_preserved=True,valid=maximum*1_000_000<=100*frequency)


def scheduled_cost(intervals,pid,begin,end,frequency):
    if (any(type(v) is not int or v<=0 for v in (pid,begin,end,frequency)) or end<=begin):
        raise ValueError("invalid scheduled observation window")
    ticks=sum(max(0,min(b,end)-max(a,begin)) for p,t,cpu,a,b in intervals if p==pid)
    return dict(span_seconds=(end-begin)/frequency,scheduled_seconds=ticks/frequency,
                cpu_one_core_percent=100*ticks/(end-begin),qpc_begin=begin,qpc_end=end,
                scope="scheduled occupancy including unseparated ISR/DPC; no deduction")


def calibration(trace,report):
    intervals=scheduler_intervals(trace)
    overlap=overlap_diagnostic(intervals,trace["frequency"])
    if report.get("frequency")!=trace["frequency"]:raise ValueError("QPC clock mismatch")
    checks=reference_checks(report,report.get("pid"),report["phase_plan"])
    checks["cross_cpu_overlap_le_100_us"]=overlap["valid"]
    plan=[tuple(p) for p in report["phase_plan"]]
    if plan not in (list(PHASES),[PHASES[0],*reversed(PHASES[1:-1]),PHASES[-1]]):raise ValueError("reference plan changed")
    if report["pid"]!=trace["root_pid"] or not any(p==report["pid"] for p,t,cpu,a,b in intervals):raise ValueError("reference not owned by collector")
    costs=[]
    for row in report["rows"]:
        value=scheduled_cost(intervals,report["pid"],row["qpc_begin"],row["qpc_end"],trace["frequency"])
        actual=value["scheduled_seconds"]*1e9;busy=row["busy_wall_ns"];span=row["span_ns"]
        if row["phase"].startswith("idle"):
            checks[row["phase"]+"_quiet"]=value["cpu_one_core_percent"]<=.5
        elif row["phase"]=="dense_control":
            checks["continuous_control_90_to_101_percent"]=90<=value["cpu_one_core_percent"]<=101
        else:
            # Predeclared calibration band, not the product's 15% gate or a
            # correction factor. Allow timer/control work but reject tick alias.
            checks[row["phase"]+"_scheduled_work_consistent"]=.8*busy<=actual<=1.5*busy+2_000_000
            checks[row["phase"]+"_nominal_1_6_to_3_2_percent"]=1.6<=value["cpu_one_core_percent"]<=3.2
            checks[row["phase"]+"_work_wall_valid"]=busy/span<=.025 and row["missed"]==0
        costs.append(dict(phase=row["phase"],scheduled=value,raw=row["metrics"],pulses=row["pulses"]))
    return dict(checks=checks,costs=costs,overlap=overlap,passed=all(checks.values()))


def product_cost(trace,report):
    intervals=scheduler_intervals(trace);costs={}
    if trace["frequency"]!=report["frequency"]:raise ValueError("product QPC mismatch")
    if not any(p==report["pid"] for p,t,cpu,a,b in intervals):raise ValueError("product not observed by collector")
    for phase in report["metrics"]:
        samples=[s for s in report["samples"] if s["phase"]==phase]
        if len(samples)<2:raise ValueError("missing product sample window")
        costs[phase]=scheduled_cost(intervals,report["pid"],samples[0]["qpc_end"],samples[-1]["qpc_end"],trace["frequency"])
    checks=dict(report["original_checks"])
    overlap=overlap_diagnostic(intervals,trace["frequency"])
    checks["cross_cpu_overlap_le_100_us"]=overlap["valid"]
    checks["independent_cpu_all_stable_phases_le_15_one_core"]=bool(costs) and all(v["cpu_one_core_percent"]<=15 for v in costs.values())
    return dict(costs=costs,latency=report["latency"],checks=checks,overlap=overlap,
                observer_callback_wall_seconds=trace["callback_wall_qpc"]/trace["frequency"],
                observer_callback_count=trace["callback_count"],
                all_sampled_memory_max={k:max(s[k] for s in report["samples"]) for k in ("working_set_bytes","private_bytes")})


def fingerprint():
    return dict(source=source_identity(),collector_source_sha256=hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
                product_sha256=hashlib.sha256(BINARY.read_bytes()).hexdigest())


def build_collector(output):
    from check_core import linker_arguments
    env=dict(os.environ)
    linker_arguments(env)
    program=Path(env.get("ProgramFiles(x86)","C:/Program Files (x86)"))
    vswhere=program/"Microsoft Visual Studio/Installer/vswhere.exe"
    installation=Path(subprocess.check_output([str(vswhere),"-latest","-products","*","-requires","Microsoft.VisualStudio.Component.VC.Tools.x86.x64","-property","installationPath"],text=True).strip())
    toolsets=sorted(p for p in (installation/"VC/Tools/MSVC").iterdir() if (p/"bin/Hostx64/x64/cl.exe").is_file())
    if not toolsets:raise RuntimeError("MSVC x64 compiler unavailable; no tools installed")
    vc=toolsets[-1]
    sdks=sorted(p for p in (program/"Windows Kits/10/Include").iterdir() if (p/"shared/evntrace.h").is_file())
    if not sdks:raise RuntimeError("Windows SDK headers unavailable; no tools installed")
    sdk=sdks[-1]
    env["INCLUDE"]=";".join(str(p) for p in (vc/"include",sdk/"um",sdk/"shared",sdk/"ucrt"))
    env["PATH"]=str(vc/"bin/Hostx64/x64")+os.pathsep+env["PATH"]
    binary=output/"measure_cpu_trace.exe"
    with (output/"build.log").open("w",encoding="utf8") as log:
        subprocess.run([str(vc/"bin/Hostx64/x64/cl.exe"),"/nologo","/std:c11","/W4","/WX","/O2","/DUNICODE","/D_UNICODE",
                        "/Fe:"+str(binary),"/Fo:"+str(output/"trace.obj"),str(SOURCE)],cwd=output,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
    return binary


def reference_child(output,reverse):
    output.mkdir();api=NativeReference();stamp,frequency=stamp_api();rows=[];closed=False
    phases=(PHASES[0],*reversed(PHASES[1:-1]),PHASES[-1]) if reverse else PHASES
    try:
        for phase in phases:
            begin=stamp();row=run_phase(api,phase);end=stamp()
            row.update(qpc_begin=begin,qpc_end=end);rows.append(row)
            print("reference "+phase[0],flush=True)
    finally:
        try:api.close();closed=True
        finally:(output/"report.json").write_text(json.dumps(dict(schema=2,pid=os.getpid(),frequency=frequency,
            rows=rows,phase_plan=phases,timer_closed=closed),indent=2)+"\n",encoding="utf8")


def product_child(output):
    output.mkdir();stamp,frequency=stamp_api();pids=[]
    class Sampler(WindowsSampler):
        def __init__(self,child):super().__init__(child);pids.append(child.pid)
        def sample(self,phase):
            a=stamp();row=super().sample(phase);b=stamp()
            row.update(qpc_begin=a,qpc_end=b);return row
    report,samples=observe(BINARY,output/"gui","--verify-streamed-gui",timeout_seconds=45,sampler_factory=Sampler)
    phases=("idle","voices_1","voices_32","closed")
    spans=(("idle",.7),("voices_1",1.2),("voices_32",1.7),("closed",.7))
    metrics=summarize(samples,phases=phases,minimum_spans=spans)
    latency,other,checks=evaluate_gui(report,metrics)
    # Same stream/audit/cleanup checks as the existing product diagnostic.
    from measure_background_stream import stream_checks,audit_checks
    checks.update(stream_checks(report,False));checks.update(audit_checks(report))
    result=dict(pid=pids[0],frequency=frequency,report=report,samples=samples,metrics=metrics,latency=latency,
                joined_latency=other,original_checks=checks)
    (output/"report.json").write_text(json.dumps(result,indent=2)+"\n",encoding="utf8")


def collect(collector,output,mode,reverse=False):
    output.mkdir();destination=output/"target"
    command=[sys.executable,str(Path(__file__).resolve()),"--child",mode,"--output",str(destination)]
    if reverse:command.append("--reverse")
    with (output/"run.log").open("w",encoding="utf8") as log:
        process=subprocess.run([str(collector),str(output/"trace.json"),"65000",str(ROOT),subprocess.list2cmdline(command)],
                               cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,timeout=85)
    trace=json.loads((output/"trace.json").read_text(encoding="utf8"))
    if process.returncode:raise RuntimeError("scheduler collector failed: "+json.dumps({k:v for k,v in trace.items() if k!="events"})+"; evidence retained")
    return trace,json.loads((destination/"report.json").read_text(encoding="utf8"))


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("--output",required=True)
    parser.add_argument("--child",choices=("reference","product"));parser.add_argument("--reverse",action="store_true")
    parser.add_argument("--build-only",action="store_true");parser.add_argument("--run-product",action="store_true")
    args=parser.parse_args();output=(ROOT/args.output).resolve()
    if os.name!="nt" or output.exists() or ROOT/"artifacts" not in output.parents:raise RuntimeError("Windows and fresh artifacts directory required")
    if args.child:
        (reference_child(output,args.reverse) if args.child=="reference" else product_child(output));return
    output.mkdir();initial=fingerprint();collector=build_collector(output)
    if args.build_only:print(str(collector));return
    proof=dict(schema=1,identity=initial,platform=platform.platform(),reference_plan=["forward","reverse"],references=[],products=[],calibrated=False,product_accepted=False)
    try:
        for i,reverse in enumerate((False,True),1):
            print("Independent CPU reference %d/2"%i,flush=True)
            trace,report=collect(collector,output/("reference-%d"%i),"reference",reverse)
            result=calibration(trace,report);proof["references"].append(result)
            if not result["passed"]:raise RuntimeError("CPU calibration failed; product not started")
        proof["calibrated"]=True
        if args.run_product:
            for i in (1,2):
                print("Calibrated product %d/2"%i,flush=True)
                trace,report=collect(collector,output/("product-%d"%i),"product")
                # Keep the original raw CPU predicate visible. It is NOT repaired
                # or converted into the independent meter's value.
                proof["products"].append(product_cost(trace,report))
            proof["product_accepted"]=all(all(r["checks"].values()) for r in proof["products"])
    except (OSError,ValueError,RuntimeError,subprocess.SubprocessError) as error:
        proof["error"]=str(error)
        print(str(error),flush=True)
    finally:
        proof["unchanged"]=initial==fingerprint()
        proof["collector_sha256"]=hashlib.sha256(collector.read_bytes()).hexdigest()
        (output/"proof.json").write_text(json.dumps(proof,indent=2)+"\n",encoding="utf8")
    if not proof["calibrated"] or not proof["unchanged"] or args.run_product and not proof["product_accepted"]:raise SystemExit(1)


if __name__=="__main__":main()
