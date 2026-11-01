"""Ordinary runner owns all targets; elevated ETW collector only observes.

Fixed off/on/on/off main-boundary plan. No counter corrections or retries.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid
from measure_audio_load import ROOT, WindowsSampler, observe, summarize
from measure_cpu_threads import token_info, IdentitySampler, pe_entry_rva, thread_costs
from measure_independent_cpu import (BINARY, stamp_api, reference_child, fingerprint,
    build_collector, calibration, scheduler_intervals)
from measure_gui_candidate import evaluate_gui
from measure_background_stream import stream_checks, audit_checks
from measure_cpu_accounting import reference_checks

PLAN=(False,True,True,False)
CATEGORIES=("framework_ui_and_tessellation","application_logic","application_ui",
            "diagnostic_step","presentation_and_event_loop")
DETAIL_CATEGORIES=CATEGORIES+("editor_widgets","graphics_poll","preview_request","host_ui_dispatch")
PANEL_CATEGORIES=DETAIL_CATEGORIES+("scene_controls","layers_assets","properties_history","canvas",
                                   "audio_panel","settings_diagnostics","project_snapshot")
TEXT_CATEGORIES=PANEL_CATEGORIES+("scene_widgets","scene_readout","audio_device","audio_status",
    "audio_library","audio_components","audio_properties","audio_readiness","audio_actions")
MAX_ROWS=32768
MAX_CLOCK_NS=50_000  # Attribution uncertainty gate, never deducted from CPU.


def categories_for_schema(schema):
    if type(schema) is not int or schema not in (1,2,3,4):raise ValueError("unsupported main boundary schema")
    return {1:CATEGORIES,2:DETAIL_CATEGORIES,3:PANEL_CATEGORIES,4:TEXT_CATEGORIES}[schema]

def row_limit(schema):
    categories_for_schema(schema)
    return MAX_ROWS if schema==4 else 16384


def ordinary(token):
    return isinstance(token,dict) and token.get("elevated")==0 and token.get("type") in (1,3)


def reference_workload_valid(report):
    # Check workload quality BEFORE releasing later targets. Final independent
    # accounting still waits for the collector's complete, drained session.
    return all(reference_checks(report,report["pid"],report["phase_plan"]).values()) and all(
        row["missed"]==0 and row["busy_wall_ns"]/row["span_ns"]<=.025
        for row in report["rows"] if row["phase"].startswith("pulse_"))


def evaluate_reports(trace,references,products):
    if len(references)!=2 or len(products)!=len(PLAN):raise ValueError("incomplete or extra capture reports")
    refs=[calibration(dict(trace,root_pid=r["pid"]),r) for r in references]
    calibrated=len(refs)==2 and all(v["passed"] for v in refs)
    values=[]
    for profile,r in zip(PLAN,products):
        v=thread_costs(trace,r);v.update(profile=profile,diagnostic_only=not calibrated)
        v["checks"]["reference_calibration_passed"]=calibrated
        if profile:
            try:
                v["main_categories"]=main_costs(trace,r,v);v["checks"]["main_boundaries_valid"]=True
            except (ValueError,KeyError) as error:
                v["attribution_error"]=str(error);v["checks"]["main_boundaries_valid"]=False
        values.append(v)
    return dict(references=refs,products=values,calibrated=calibrated,
                passed=calibrated and len(values)==len(PLAN) and all(all(v["checks"].values()) for v in values))


def permission_checks(trace,runner,reports,created):
    return dict(observe_only=trace.get("observed_runner") is True and trace.get("launch_api")=="ordinary-runner-owned-children",
                runner_identity=trace.get("root_pid")==os.getpid() and trace.get("runner_created")==created,
                runner_ordinary=ordinary(runner),native_ordinary=trace.get("child_elevated")==0 and trace.get("child_token_type")==runner.get("type") and ordinary(runner),
                all_targets_ordinary=len(reports)==6 and len({r.get("pid") for r in reports})==6 and len({r.get("runner_pid") for r in reports})==6
                    and all(ordinary(r.get("token")) and ordinary(r.get("owner",{}).get("token"))
                    and r.get("runner_pid")==r.get("owner",{}).get("pid") and r.get("runner_pid",0)>0
                    and ("product_token" not in r or ordinary(r["product_token"])) for r in reports))


def precise_pair(stamp,kernel):
    ft=c.c_uint64();a=stamp();kernel.GetSystemTimePreciseAsFileTime(c.byref(ft));b=stamp()
    return [a,b,(ft.value-116444736000000000)*100]


def clock_mapping(pairs,frequency):
    if not isinstance(pairs,list) or not 2<=len(pairs)<=1024:raise ValueError("clock calibration missing or unbounded")
    offsets=[];lower=[];upper=[];width=0
    for a,b,epoch in pairs:
        if any(type(v) is not int or v<=0 for v in (a,b,epoch)) or b<a:raise ValueError("invalid clock pair")
        # Keep large epoch math integral until subtracting. Units ns * frequency.
        offsets.append(epoch*frequency-(a+b)*500_000_000)
        lower.append(epoch*frequency-b*1_000_000_000-100*frequency)
        upper.append(epoch*frequency-a*1_000_000_000+100*frequency)
        width=max(width,(b-a)*1e9/frequency)
    spread=(max(offsets)-min(offsets))/frequency
    if width>MAX_CLOCK_NS or spread>MAX_CLOCK_NS:raise ValueError("clock calibration too uncertain or changed")
    # ALL brackets constrain one constant offset, including slow/preempted
    # pairs. No best-pair selection or dropping inconvenient clock samples.
    lo,hi=max(lower),min(upper)
    if lo>hi:raise ValueError("clock brackets have no common offset")
    offset=(lo+hi)//2
    return offset,dict(max_pair_ns=width,offset_spread_ns=spread,feasible_offset_width_ns=(hi-lo)/frequency,
                       bound_ns=(hi-lo)/(2*frequency)+1000)


def main_costs(trace,report,base):
    work=report["report"].get("main_work")
    if not isinstance(work,dict) or work.get("invalid")!=0:raise ValueError("main boundaries invalid")
    categories=categories_for_schema(work.get("schema"))
    rows=work.get("rows")
    if not isinstance(rows,list) or not 2<=len(rows)<=row_limit(work.get('schema')):raise ValueError("main boundary capacity")
    offset,clock=clock_mapping(report["clock_pairs"],trace["frequency"])
    mapped=[];drift=[]
    for row in rows:
        if (not isinstance(row,list) or len(row)!=3 or any(type(v) is not int or v<0 for v in row)
                or row[2]>=len(categories)):raise ValueError("bad main boundary")
        epoch,elapsed,kind=row
        if mapped and (epoch<rows[len(mapped)-1][0] or elapsed<rows[len(mapped)-1][1]):raise ValueError("main clock reversal")
        drift.append((epoch-rows[0][0])-(elapsed-rows[0][1]))
        mapped.append(((epoch*trace["frequency"]-offset)//1_000_000_000,kind))
    if max(drift)-min(drift)>MAX_CLOCK_NS:raise ValueError("SystemTime disagrees with monotonic boundary clock")
    mains=[r for r in base["threads"] if r["process_entry_start"]]
    if len(mains)!=1:raise ValueError("ambiguous product main identity")
    tid=mains[0]["tid"];window=base["costs"]["voices_32"];a,b=window["qpc_begin"],window["qpc_end"]
    if mapped[0][0]>a or mapped[-1][0]<b:raise ValueError("main boundaries do not cover measurement window")
    intervals=[(x,y) for p,t,cpu,x,y in scheduler_intervals(trace) if p==report["pid"] and t==tid]
    totals=[0]*len(categories);edges=0
    for (x,kind),(y,_) in zip(mapped,mapped[1:]):
        x=max(x,a);y=min(y,b)
        if y<=x:continue
        edges+=1
        totals[kind]+=sum(max(0,min(y,v)-max(x,u)) for u,v in intervals)
    percent=[t*100/(b-a) for t in totals]
    if abs(sum(percent)-mains[0]["cpu_one_core_percent"])>1e-6:raise ValueError("main categories do not partition main occupancy")
    counts=[sum(1 for at,kind in mapped if a<=at<b and kind==i) for i in range(len(categories))]
    rollup=percent[:len(CATEGORIES)]
    if work["schema"]>=2:
        rollup[1]+=percent[6]
        rollup[2]+=percent[5]+percent[7]+percent[8]
    if work["schema"]>=3:rollup[2]+=sum(percent[9:])
    return dict(tid=tid,schema=work["schema"],clock=clock,monotonic_disagreement_ns=max(drift)-min(drift),edges=edges,
                boundary_counts=dict(zip(categories,counts)),ui_calls_per_second=counts[2]*trace["frequency"]/(b-a),
                present_edges_per_second=counts[4]*trace["frequency"]/(b-a),
                boundary_uncertainty_percent_upper=edges*2*clock["bound_ns"]*100/((b-a)*1e9/trace["frequency"]),
                cpu_one_core_percent=dict(zip(categories,percent)),total=sum(percent),
                rollup_cpu_one_core_percent=dict(zip(CATEGORIES,rollup)),
                scope="boundary intersection with conservative scheduled occupancy; presentation includes waits and events; no pure driver attribution")


def product_argument(profile,ui_mode=None):
    if ui_mode is None:return '--verify-main-work-gui' if profile else '--verify-streamed-gui'
    if not profile and ui_mode in ('stable-empty','stable-resources','product-empty','product-stable'):
        return '--verify-'+ui_mode
    if ui_mode=='repaint' and not profile:return '--verify-repaint-only-gui'
    if ui_mode=='dispatch-repaint' and not profile:return '--verify-dispatch-repaint-gui'
    if not profile or ui_mode not in ('framework','presentation','editor','dispatch'):
        raise ValueError('explicit observed diagnostic control mode required')
    return dict(framework='--verify-framework-audio-gui',presentation='--verify-presentation-audio-gui',
                editor='--verify-editor-baseline-gui',dispatch='--verify-dispatch-gui')[ui_mode]


def product(output,profile,binary=BINARY,ui_mode=None):
    argument=product_argument(profile,ui_mode)
    if ui_mode in ('stable-empty','stable-resources','product-empty','product-stable'):
        from measure_stable_window import observe_product
        if ui_mode in ('product-empty','product-stable'):
            mode='stable-empty' if ui_mode=='product-empty' else 'stable-resources'
            return observe_product(output,binary,mode,product=True)
        return observe_product(output,binary,ui_mode)
    stamp,frequency=stamp_api();observers=[];pids=[];tokens=[];pairs=[]
    kernel=c.WinDLL("kernel32",use_last_error=True)
    kernel.GetSystemTimePreciseAsFileTime.argtypes=[c.POINTER(c.c_uint64)];kernel.GetSystemTimePreciseAsFileTime.restype=None
    class Sampler(IdentitySampler):
        def __init__(self,child):
            super().__init__(child);observers.append(self);pids.append(child.pid);tokens.append(token_info(self.handle))
        def sample(self,phase):
            a=stamp();pairs.append(precise_pair(stamp,kernel));row=super().sample(phase);b=stamp()
            row.update(qpc_begin=a,qpc_end=b);return row
    report,samples=observe(binary,output/"gui",argument,timeout_seconds=45,sampler_factory=Sampler)
    metrics=summarize(samples,phases=("idle","voices_1","voices_32","closed"),minimum_spans=(("idle",.7),("voices_1",1.2),("voices_32",1.7),("closed",.7)))
    latency,other,checks=evaluate_gui(report,metrics);checks.update(stream_checks(report,False));checks.update(audit_checks(report))
    return dict(pid=pids[0],frequency=frequency,report=report,samples=samples,metrics=metrics,latency=latency,
                joined_latency=other,original_checks=checks,identities=observers[0].gate.observations,
                observer_close=observers[0].close_record,product_token=tokens[0],product_name=binary.name.lower(),pe_entry_rva=pe_entry_rva(binary),clock_pairs=pairs)


class Collector:
    """One newly elevated read-only observer; runner retains and owns targets."""
    def __init__(self,binary,output):
        self.k=c.WinDLL("kernel32",use_last_error=True);self.handles=[];self.process=None
        k=self.k
        for name,args,restype in (("GetCurrentProcess",[],w.HANDLE),("CloseHandle",[w.HANDLE],w.BOOL),
            ("CreateEventW",[c.c_void_p,w.BOOL,w.BOOL,w.LPCWSTR],w.HANDLE),
            ("SetEvent",[w.HANDLE],w.BOOL),("WaitForSingleObject",[w.HANDLE,w.DWORD],w.DWORD),
            ("GetProcessTimes",[w.HANDLE,*([c.POINTER(c.c_uint64)]*4)],w.BOOL)):
            f=getattr(k,name);f.argtypes=args;f.restype=restype
        values=[c.c_uint64() for _ in range(4)]
        if not k.GetProcessTimes(k.GetCurrentProcess(),*[c.byref(v) for v in values]):raise OSError(c.get_last_error(),"runner identity")
        self.created=values[0].value
        prefix="Local\\SkinEditorCpu-%d-%d-%s"%(os.getpid(),self.created,uuid.uuid4().hex)
        names=[prefix+"-ready",prefix+"-done"]
        try:
            for n in names:
                h=k.CreateEventW(None,True,False,n)
                if not h or c.get_last_error()==183:raise OSError(c.get_last_error(),"fresh collector gate required")
                self.handles.append(h)
            # Only the frozen collector is elevated; no shell, command server or
            # token borrowing. ShellExecute returns a retained own process handle.
            class Execute(c.Structure):
                _fields_=[("size",w.DWORD),("mask",w.ULONG),("window",w.HWND),("verb",w.LPCWSTR),("file",w.LPCWSTR),
                    ("parameters",w.LPCWSTR),("directory",w.LPCWSTR),("show",c.c_int),("instance",w.HINSTANCE),
                    ("idlist",c.c_void_p),("class_name",w.LPCWSTR),("class_key",w.HKEY),("hotkey",w.DWORD),
                    ("icon",w.HANDLE),("process",w.HANDLE)]
            shell=c.WinDLL("shell32",use_last_error=True);shell.ShellExecuteExW.argtypes=[c.POINTER(Execute)];shell.ShellExecuteExW.restype=w.BOOL
            e=Execute();e.size=c.sizeof(e);e.mask=0x40|0x100;e.verb="runas";e.file=str(binary);e.directory=str(ROOT);e.show=0
            e.parameters=subprocess.list2cmdline([str(output/"trace.json"),"240000","--observe",str(os.getpid()),str(self.created),*names])
            if not shell.ShellExecuteExW(c.byref(e)):raise OSError(c.get_last_error(),"collector elevation canceled or failed")
            self.process=e.process
            # Wait on readiness OR exit, not a guessed startup delay.
            k.WaitForMultipleObjects.argtypes=[w.DWORD,c.POINTER(w.HANDLE),w.BOOL,w.DWORD];k.WaitForMultipleObjects.restype=w.DWORD
            if k.WaitForMultipleObjects(2,(w.HANDLE*2)(self.handles[0],self.process),False,60000)!=0:
                raise RuntimeError("collector not ready; no target started")
        except BaseException:
            self.close();raise
    def close(self):
        error=None
        if self.handles:
            if not self.k.SetEvent(self.handles[-1]):error=RuntimeError("collector done gate failed")
        if self.process:
            if self.k.WaitForSingleObject(self.process,20000)!=0:error=RuntimeError("collector did not stop in bound")
            self.k.CloseHandle(self.process);self.process=None
        for h in self.handles:self.k.CloseHandle(h)
        self.handles=[]
        if error:raise error


def run_child(output,kind,profile=False,reverse=False,binary=None,ui_mode=None):
    if ui_mode is not None:
        if kind!='product':raise ValueError('UI control is product diagnostic only')
        product_argument(profile,ui_mode)
    output.mkdir();args=[sys.executable,str(Path(__file__).resolve()),"--child",kind,"--output",str(output)]
    if profile:args.append("--profile")
    if reverse:args.append("--reverse")
    if binary is not None:args.extend(["--binary",str(binary)])
    if ui_mode is not None:args.extend(["--ui-mode",ui_mode])
    k=c.WinDLL("kernel32",use_last_error=True)
    k.CreateEventW.argtypes=[c.c_void_p,w.BOOL,w.BOOL,w.LPCWSTR];k.CreateEventW.restype=w.HANDLE
    k.CreateJobObjectW.argtypes=[c.c_void_p,w.LPCWSTR];k.CreateJobObjectW.restype=w.HANDLE
    k.SetInformationJobObject.argtypes=[w.HANDLE,c.c_int,c.c_void_p,w.DWORD];k.SetInformationJobObject.restype=w.BOOL
    k.AssignProcessToJobObject.argtypes=[w.HANDLE,w.HANDLE];k.AssignProcessToJobObject.restype=w.BOOL
    k.CloseHandle.argtypes=[w.HANDLE];k.CloseHandle.restype=w.BOOL
    k.SetEvent.argtypes=[w.HANDLE];k.SetEvent.restype=w.BOOL
    # JOBOBJECT_EXTENDED_LIMIT_INFORMATION: standard Win64 layout; zero all
    # optional fields, set only BasicLimitInformation.LimitFlags. No priority,
    # CPU restriction, breakaway, or foreign process assignment.
    class Basic(c.Structure):
        _fields_=[("process_time",c.c_int64),("job_time",c.c_int64),("flags",w.DWORD),("min_ws",c.c_size_t),
                  ("max_ws",c.c_size_t),("active",w.DWORD),("affinity",c.c_size_t),("priority",w.DWORD),("scheduling",w.DWORD)]
    class Limits(c.Structure):
        _fields_=[("basic",Basic),("io",c.c_uint64*6),("process_memory",c.c_size_t),("job_memory",c.c_size_t),
                  ("peak_process",c.c_size_t),("peak_job",c.c_size_t)]
    name="Local\\SkinEditorCpuTarget-%d-%s"%(os.getpid(),uuid.uuid4().hex)
    gate=k.CreateEventW(None,True,False,name);job=k.CreateJobObjectW(None,None);limits=Limits();limits.basic.flags=0x2000
    if not gate or not job:
        if gate:k.CloseHandle(gate)
        if job:k.CloseHandle(job)
        raise OSError(c.get_last_error(),"owned target gate/job")
    args.extend(["--gate",name])
    try:
        if not k.SetInformationJobObject(job,9,c.byref(limits),c.sizeof(limits)):raise OSError(c.get_last_error(),"owned job limit")
        timeout=220 if ui_mode in ('stable-resources','product-stable') else 95 if ui_mode in ('stable-empty','product-empty') else 55
        return launch_owned(k,gate,job,args,output,timeout=timeout)
    finally:
        k.CloseHandle(job);k.CloseHandle(gate)


def launch_owned(k,gate,job,args,output,timeout=55):
    if type(timeout) is not int or not 1<=timeout<=220:raise ValueError('bounded owned target timeout required')
    with (output/"child.log").open("x",encoding="utf8") as log:
        # Popen ownership is retained until wait; targets never originate at the
        # elevated collector. Bounded failure kills only this own child.
        child=subprocess.Popen(args,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
        owner=None
        try:
            owner=dict(pid=child.pid,token=token_info(w.HANDLE(int(child._handle))))
            if not ordinary(owner["token"]):raise RuntimeError("own target elevated before release")
            if not k.AssignProcessToJobObject(job,w.HANDLE(int(child._handle))):raise OSError(c.get_last_error(),"own child job assignment")
            if not k.SetEvent(gate):raise OSError(c.get_last_error(),"own child release")
            if child.wait(timeout=timeout):raise RuntimeError("ordinary child failed")
        finally:
            if child.poll() is None:child.kill();child.wait(timeout=5)
    report=json.loads((output/"report.json").read_text(encoding="utf8"))
    if report.get("runner_pid")!=child.pid:raise ValueError("owned child report PID mismatch")
    report["owner"]=owner
    return report


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("--output",required=True);p.add_argument("--child",choices=("reference","product"))
    p.add_argument("--profile",action="store_true");p.add_argument("--reverse",action="store_true");p.add_argument("--gate");p.add_argument("--binary")
    p.add_argument('--ui-mode',choices=('framework','presentation','editor','dispatch','repaint','dispatch-repaint','stable-empty','stable-resources','product-empty','product-stable'))
    args=p.parse_args();output=(ROOT/args.output).resolve()
    if args.ui_mode is not None and args.child!='product':raise ValueError('UI control requires product child')
    if os.name!="nt" or ROOT/"artifacts" not in output.parents:raise RuntimeError("Windows artifacts directory required")
    if args.child:
        if not ordinary(token_info()):raise RuntimeError("target unexpectedly elevated")
        if not args.gate or not args.gate.startswith("Local\\SkinEditorCpuTarget-"):raise ValueError("own runner gate required")
        k=c.WinDLL("kernel32",use_last_error=True)
        k.OpenEventW.argtypes=[w.DWORD,w.BOOL,w.LPCWSTR];k.OpenEventW.restype=w.HANDLE
        k.WaitForSingleObject.argtypes=[w.HANDLE,w.DWORD];k.WaitForSingleObject.restype=w.DWORD;k.CloseHandle.argtypes=[w.HANDLE]
        gate=k.OpenEventW(0x100000,False,args.gate)
        if not gate:raise OSError(c.get_last_error(),"own target gate unavailable")
        try:
            if k.WaitForSingleObject(gate,30000)!=0:raise RuntimeError("own target admission timed out")
        finally:k.CloseHandle(gate)
        if args.child=="reference":
            reference_child(output/"reference",args.reverse);report=json.loads((output/"reference/report.json").read_text(encoding="utf8"))
        else:
            binary=Path(args.binary).resolve() if args.binary else BINARY
            if binary!=BINARY and ROOT/"artifacts" not in binary.parents:raise ValueError("owned product binary required")
            report=product(output,args.profile,binary,args.ui_mode)
        report.update(token=token_info(),runner_pid=os.getpid());(output/"report.json").write_text(json.dumps(report,indent=2)+"\n",encoding="utf8");return
    if output.exists() or not ordinary(token_info()):raise RuntimeError("fresh output and ordinary runner required")
    output.mkdir();initial=fingerprint();binary=build_collector(output);references=[];products=[];collector=None
    self_test=json.loads(subprocess.check_output([str(binary),"--self-test"],text=True))
    if self_test.get("lifetime_self_test") is not True:raise ValueError("native lifetime tests failed; no UAC or target")
    proof=dict(schema=1,identity=initial,plan=PLAN,runner_token=token_info(),collector_self_test=self_test,references=[],products=[],passed=False)
    try:
        print("UAC: only the scheduler collector; ordinary runner owns all targets",flush=True)
        collector=Collector(binary,output)
        for i,reverse in enumerate((False,True),1):
            print("ordinary reference %d/2"%i,flush=True);references.append(run_child(output/("reference-%d"%i),"reference",reverse=reverse))
            r=references[-1]
            if not reference_workload_valid(r):raise ValueError("reference workload invalid; products not started")
        for i,profile in enumerate(PLAN,1):
            print("ordinary product %d/4 boundaries=%s"%(i,profile),flush=True);products.append(run_child(output/("product-%d"%i),"product",profile=profile))
        created=collector.created;collector.close();collector=None
        trace=json.loads((output/"trace.json").read_text(encoding="utf8"));proof["permission_checks"]=permission_checks(trace,proof["runner_token"],references+products,created)
        proof["runner_created"]=created
        proof["targets"]=[dict(pid=r["pid"],runner_pid=r["runner_pid"],owner=r["owner"],token=r["token"],product_token=r.get("product_token")) for r in references+products]
        if not all(proof["permission_checks"].values()):raise ValueError("ordinary permission or ownership mismatch")
        proof.update(evaluate_reports(trace,references,products))
    except (OSError,RuntimeError,ValueError,KeyError,subprocess.SubprocessError) as error:
        proof["error"]=str(error);print(str(error),flush=True)
    finally:
        if collector:
            try:collector.close()
            except (OSError,RuntimeError) as error:proof["cleanup_error"]=str(error);proof["passed"]=False
        proof["unchanged"]=initial==fingerprint();proof["collector_sha256"]=hashlib.sha256(binary.read_bytes()).hexdigest()
        (output/"proof.json").write_text(json.dumps(proof,indent=2)+"\n",encoding="utf8")
    if not proof["passed"] or not proof["unchanged"]:raise SystemExit(1)


if __name__=="__main__":main()
