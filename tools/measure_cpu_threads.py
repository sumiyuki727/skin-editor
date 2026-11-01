"""Owned ETW thread identities and bracketed token/observer controls.

Only the collector needs elevation. Never changes the product, system policy,
desktop ACLs, global clock resolution or the original CPU/latency gates.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
import sys
import time
from measure_audio_load import ROOT,WindowsSampler,observe,summarize
from measure_independent_cpu import (BINARY,stamp_api,reference_child,fingerprint,build_collector,
                                     calibration,product_cost,scheduler_intervals)
from measure_gui_cpu import AttributedThreads,adapter_info
from measure_gui_candidate import evaluate_gui
from measure_background_stream import stream_checks,audit_checks

PLAN=((True,False),(True,True),(False,True),(False,True),(True,True),(True,False))
ELEVATED_PLAN=((False,False),(False,True),(False,True),(False,False))


def token_info(handle=None):
    k=c.WinDLL("kernel32",use_last_error=True);a=c.WinDLL("advapi32",use_last_error=True)
    k.GetCurrentProcess.argtypes=[];k.GetCurrentProcess.restype=w.HANDLE
    k.CloseHandle.argtypes=[w.HANDLE];k.CloseHandle.restype=w.BOOL
    a.OpenProcessToken.argtypes=[w.HANDLE,w.DWORD,c.POINTER(w.HANDLE)];a.OpenProcessToken.restype=w.BOOL
    a.GetTokenInformation.argtypes=[w.HANDLE,w.LONG,c.c_void_p,w.DWORD,c.POINTER(w.DWORD)];a.GetTokenInformation.restype=w.BOOL
    token=w.HANDLE();size=w.DWORD();result={}
    if not a.OpenProcessToken(handle if handle is not None else k.GetCurrentProcess(),8,c.byref(token)):
        raise OSError(c.get_last_error(),"owned token unavailable")
    try:
        for name,kind in (("elevated",20),("type",18)):
            value=w.DWORD()
            if not a.GetTokenInformation(token,kind,c.byref(value),c.sizeof(value),c.byref(size)):
                raise OSError(c.get_last_error(),"owned token summary unavailable")
            result[name]=value.value
        return result
    finally:
        if not k.CloseHandle(token):raise OSError(c.get_last_error(),"token close failed")


def permission_checks(trace,report,limited):
    expected=dict(elevated=0 if limited else 1,type=3 if limited else 2)
    return dict(explicit_permission_mode=trace.get("limited_requested") is limited,
                native_child_token=trace.get("child_elevated")==expected["elevated"] and trace.get("child_token_type")==expected["type"],
                python_child_token=report.get("token")==expected,
                product_token=(report.get("product_token")==expected) if "metrics" in report else True)


def pe_entry_rva(path):
    # Read only our already fingerprinted product's bounded PE headers.
    with path.open("rb") as f:
        header=f.read(64)
        if len(header)!=64 or header[:2]!=b"MZ":raise ValueError("product is not PE")
        offset=struct.unpack_from("<I",header,60)[0]
        if not 64<=offset<=1024*1024:raise ValueError("unbounded PE header offset")
        f.seek(offset);header=f.read(44)
    if len(header)!=44 or header[:4]!=b"PE\0\0" or struct.unpack_from("<H",header,24)[0] not in (0x10b,0x20b):
        raise ValueError("unsupported product PE header")
    return struct.unpack_from("<I",header,40)[0]


class PhaseIdentities:
    """Fakeable phase gate; expensive identity queries only on phase changes."""
    def __init__(self):self.phase=None;self.observations=[]
    def sample(self,phase,identify,ordinary,clock):
        if phase!=self.phase:
            if len(self.observations)>=24:raise RuntimeError("identity phase capacity exceeded")
            begin=clock();row=identify(phase);end=clock()
            self.observations.append(dict(phase=phase,begin=begin,end=end,threads=row["threads"]))
            self.phase=phase
        return ordinary(phase)


class IdentitySampler(AttributedThreads):
    def __init__(self,child):
        super().__init__(child);self.gate=PhaseIdentities();self.close_record=None
    def sample(self,phase):
        return self.gate.sample(phase,lambda p:AttributedThreads.sample(self,p),
                                lambda p:WindowsSampler.sample(self,p),time.perf_counter_ns)
    def close(self):
        count=len(self.retained.entries)
        try:super().close()
        finally:self.close_record=dict(before=count,after=len(self.retained.entries),closed=self.retained.closed)


def thread_costs(trace,report,phase="voices_32"):
    intervals=scheduler_intervals(trace);base=product_cost(trace,report);window=base["costs"][phase]
    begin,end=window["qpc_begin"],window["qpc_end"];by={};names={}
    counts={}
    for pid,tid,generation in trace.get("thread_lifetimes",[]):
        if pid==report["pid"]:counts[tid]=counts.get(tid,0)+1
    reused={tid for tid,count in counts.items() if count>1}
    observations=report["identities"]
    for item in [v for v in observations if v["phase"]!=phase]+[v for v in observations if v["phase"]==phase]:
        for row in item["threads"]:
            tid,created=map(int,row["key"].split(":"))
            if tid in names and names[tid]["created"]!=created and tid not in reused:raise ValueError("TID reused across retained identities")
            names[tid]=dict(created=created,name=row["name"],origin=row["origin"],identity_phase=item["phase"])
    for pid,tid,cpu,a,b in intervals:
        if pid==report["pid"]:by[tid]=by.get(tid,0)+max(0,min(b,end)-max(a,begin))
    rows=[]
    for tid,ticks in sorted(by.items(),key=lambda r:-r[1]):
        if not ticks:continue
        identity=names.get(tid,dict(created=None,name="",identity_phase=None,origin=dict(status="undiscovered",module=None,offset=None)))
        if tid in reused:
            identity=dict(created=None,name="",identity_phase=None,origin=dict(status="reused-tid-lifetimes-not-named",module=None,offset=None),lifetime_count=counts[tid])
        origin=identity["origin"]
        process_entry=origin["module"]==report.get("product_name",BINARY.name.lower()) and origin["offset"]==report["pe_entry_rva"]
        rows.append(dict(tid=tid,**identity,process_entry_start=process_entry,cpu_one_core_percent=ticks/(end-begin)*100))
    if abs(sum(r["cpu_one_core_percent"] for r in rows)-window["cpu_one_core_percent"])>1e-8:
        raise ValueError("thread sum differs from process scheduled occupancy")
    base.update(threads=rows,identity_queries=[dict(phase=v["phase"],wall_ms=(v["end"]-v["begin"])/1e6) for v in report["identities"]],
                observer_close=report["observer_close"],adapter=adapter_info(report["report"]),ui_work_wall=report["report"]["ui_cost"][4])
    base["checks"]["all_identity_handles_closed"]=report["observer_close"]["closed"] is True and report["observer_close"]["after"]==0
    return base


def product_child(output,attributed):
    stamp,frequency=stamp_api();observers=[];pids=[];tokens=[]
    class Sampler(IdentitySampler if attributed else WindowsSampler):
        def __init__(self,child):
            super().__init__(child);observers.append(self);pids.append(child.pid);tokens.append(token_info(self.handle))
        def sample(self,phase):
            a=stamp();row=super().sample(phase);b=stamp();row.update(qpc_begin=a,qpc_end=b);return row
    report,samples=observe(BINARY,output/"gui","--verify-streamed-gui",timeout_seconds=45,sampler_factory=Sampler)
    phases=("idle","voices_1","voices_32","closed");spans=(("idle",.7),("voices_1",1.2),("voices_32",1.7),("closed",.7))
    metrics=summarize(samples,phases=phases,minimum_spans=spans);latency,other,checks=evaluate_gui(report,metrics)
    checks.update(stream_checks(report,False));checks.update(audit_checks(report))
    return dict(pid=pids[0],frequency=frequency,report=report,samples=samples,metrics=metrics,latency=latency,
                joined_latency=other,original_checks=checks,identities=observers[0].gate.observations if attributed else [],
                observer_close=observers[0].close_record if attributed else dict(before=0,after=0,closed=True),
                product_token=tokens[0],pe_entry_rva=pe_entry_rva(BINARY))


def child(output,kind,reverse,attributed):
    os.chdir(ROOT)
    output.mkdir()
    with (output/"child.log").open("w",encoding="utf8",buffering=1) as log:
        sys.stdout=log;sys.stderr=log
        if kind=="reference":
            reference_child(output/"reference",reverse)
            result=json.loads((output/"reference/report.json").read_text(encoding="utf8"))
        else:result=product_child(output,attributed)
        result["token"]=token_info()
        (output/"report.json").write_text(json.dumps(result,indent=2)+"\n",encoding="utf8")


def collect(collector,output,kind,limited,reverse=False,attributed=True):
    output.mkdir();command=[sys.executable,str(Path(__file__).resolve()),"--child",kind,"--output",str(output/"target")]
    if reverse:command.append("--reverse")
    if not attributed:command.append("--plain")
    # The secondary-logon launch must resolve an initial directory before the
    # linked token starts. Both modes use the same system directory; the owned
    # Python child explicitly enters ROOT, and the actual product still uses ROOT.
    initial_directory=Path(os.environ["SystemRoot"])/"System32"
    args=[str(collector),str(output/"trace.json"),"65000",str(initial_directory),subprocess.list2cmdline(command)]
    if limited:args.append("--limited")
    with (output/"run.log").open("w",encoding="utf8") as log:
        process=subprocess.run(args,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,timeout=85)
    trace=json.loads((output/"trace.json").read_text(encoding="utf8"))
    if process.returncode:raise RuntimeError("collector rejected capture: "+json.dumps({k:v for k,v in trace.items() if k!="events"}))
    report=json.loads((output/"target/report.json").read_text(encoding="utf8"))
    checks=permission_checks(trace,report,limited)
    if not all(checks.values()):raise ValueError("requested token mode not verified")
    return trace,report,checks


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("--output",required=True)
    parser.add_argument("--child",choices=("reference","product"));parser.add_argument("--reverse",action="store_true")
    parser.add_argument("--plain",action="store_true")
    parser.add_argument("--elevated-only",action="store_true",help="Explicit separate diagnostic, never counted as a normal-token comparison")
    args=parser.parse_args();output=(ROOT/args.output).resolve()
    if os.name!="nt" or output.exists() or ROOT/"artifacts" not in output.parents:raise RuntimeError("Windows and fresh artifacts directory required")
    if args.child:child(output,args.child,args.reverse,not args.plain);return
    output.mkdir();initial=fingerprint();collector=build_collector(output)
    plan=ELEVATED_PLAN if args.elevated_only else PLAN
    reference_limited=not args.elevated_only
    proof=dict(schema=1,identity=initial,platform=platform.platform(),plan=plan,elevated_only=args.elevated_only,references=[],products=[],calibrated=False,passed=False)
    try:
        for i,reverse in enumerate((False,True),1):
            print("Reference %d/2 limited=%s"%(i,reference_limited),flush=True)
            t,r,checks=collect(collector,output/("reference-%d"%i),"reference",reference_limited,reverse)
            value=calibration(t,r);value["permission_checks"]=checks;proof["references"].append(value)
            if not value["passed"]:raise RuntimeError("limited reference failed; product not started")
        proof["calibrated"]=True
        for i,(limited,attributed) in enumerate(plan,1):
            print("Thread attribution %d/%d limited=%s identities=%s"%(i,len(plan),limited,attributed),flush=True)
            t,r,checks=collect(collector,output/("product-%d"%i),"product",limited,attributed=attributed)
            value=thread_costs(t,r);value.update(limited=limited,attributed=attributed,permission_checks=checks);proof["products"].append(value)
        proof["passed"]=all(all(r["checks"].values()) for r in proof["products"])
    except (OSError,RuntimeError,ValueError,KeyError,subprocess.SubprocessError) as error:
        proof["error"]=str(error);print(str(error),flush=True)
    finally:
        proof["unchanged"]=initial==fingerprint();proof["collector_sha256"]=hashlib.sha256(collector.read_bytes()).hexdigest()
        (output/"proof.json").write_text(json.dumps(proof,indent=2)+"\n",encoding="utf8")
    if not proof["passed"] or not proof["unchanged"]:raise SystemExit(1)


if __name__=="__main__":main()
