"""Bounded owned-process CPU reference: periodic busy/wait work, no GUI or audio."""
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
from measure_audio_load import ROOT
from measure_combined import source_identity
from measure_gui_cpu import configure_cycles, read_cycles

PERIOD_NS = 10_000_000
BUSY_NS = 200_000
MAX_PULSES = 1400
PHASES = (("idle_before", 2100, 0), ("pulse_0", 2100, 0),
          ("pulse_2", 2100, 2_000_000), ("pulse_long", 12600, 0),
          ("dense_control", 2100, 0), ("pulse_5", 2100, 5_000_000), ("idle_after", 2100, 0))


def next_deadline(previous, now, period=PERIOD_NS):
    if period <= 0: raise ValueError("invalid reference period")
    deadline = previous+period
    skipped = max(0,(now-deadline)//period+1)
    return deadline+skipped*period, skipped


def reference_cost(row):
    required = ("span_ns","busy_wall_ns","awake_wall_ns","pulses","missed","cycles","thread_kernel_ticks",
                "thread_user_ticks","process_kernel_ticks","process_user_ticks","iterations")
    if any(type(row.get(k)) is not int or row[k] < 0 for k in required) or row["span_ns"] <= 0:
        raise ValueError("invalid reference counters")
    if not row["busy_wall_ns"] <= row["awake_wall_ns"] <= row["span_ns"] or row["pulses"] > MAX_PULSES:
        raise ValueError("invalid bounded reference work")
    seconds=row["span_ns"]/1e9
    return dict(span_seconds=seconds,
                thread_cpu_seconds=(row["thread_kernel_ticks"]+row["thread_user_ticks"])/1e7,
                thread_cpu_one_core_percent=(row["thread_kernel_ticks"]+row["thread_user_ticks"])/1e7/seconds*100,
                process_cpu_one_core_percent=(row["process_kernel_ticks"]+row["process_user_ticks"])/1e7/seconds*100,
                busy_wall_percent=row["busy_wall_ns"]/row["span_ns"]*100,
                own_awake_wall_percent=row["awake_wall_ns"]/row["span_ns"]*100,
                cycles=row["cycles"],cycle_units="cycles; never converted to CPU percent or time")


def sampled_tick_model(span_ns, offset_ns, period_ns=PERIOD_NS, busy_ns=BUSY_NS, quantum_ns=15_625_000):
    """Illustration of a hypothetical periodic accountant, not an OS implementation claim."""
    values=(span_ns,offset_ns,period_ns,busy_ns,quantum_ns)
    if any(type(v) is not int or v < 0 for v in values) or not span_ns or not period_ns or not quantum_ns:
        raise ValueError("invalid model interval")
    if busy_ns > period_ns or offset_ns >= period_ns or span_ns//quantum_ns > 100000 or span_ns//period_ns > 100000:
        raise ValueError("unbounded or invalid model")
    busy=sum(min(busy_ns,span_ns-t) for t in range(offset_ns,span_ns,period_ns))
    hits=sum((t-offset_ns)%period_ns < busy_ns and t >= offset_ns for t in range(quantum_ns,span_ns,quantum_ns))
    return dict(assumption="periodic sampling model only; not verified Windows kernel internals",
                span_ns=span_ns,offset_ns=offset_ns,quantum_ns=quantum_ns,
                work_wall_ns=busy,charged_ns=hits*quantum_ns,
                work_wall_percent=busy/span_ns*100,charged_percent=hits*quantum_ns/span_ns*100)


def reference_checks(report, expected_pid, phases):
    rows=report.get("rows",[])
    if len(rows)!=len(phases):raise ValueError("missing reference phases")
    exercised=True
    for row,(name,millis,offset) in zip(rows,phases):
        reference_cost(row)
        if (row["phase"],row["requested_ms"],row["offset_ns"]) != (name,millis,offset):
            raise ValueError("reference phase identity changed")
        exercised &= row["span_ns"] >= millis*1_000_000
        if name.startswith("idle"):
            exercised &= row["pulses"]==row["iterations"]==row["busy_wall_ns"]==0
        else:
            exercised &= row["pulses"]>0 and row["iterations"]>0 and row["cycles"]>0
    return dict(owned_process=report.get("pid")==expected_pid,
                versioned_reference=report.get("schema")==2,
                local_timer_closed=report.get("timer_closed") is True,
                requested_work_exercised=bool(exercised))


class NativeReference:
    def __init__(self):
        self.k=c.WinDLL("kernel32",use_last_error=True)
        k=self.k
        for name in ("GetCurrentProcess","GetCurrentThread"):
            f=getattr(k,name);f.argtypes=[];f.restype=w.HANDLE
        for name in ("GetProcessTimes","GetThreadTimes"):
            f=getattr(k,name);f.argtypes=[w.HANDLE]+[c.POINTER(w.FILETIME)]*4;f.restype=w.BOOL
        configure_cycles(k)
        k.CreateWaitableTimerExW.argtypes=[c.c_void_p,w.LPCWSTR,w.DWORD,w.DWORD];k.CreateWaitableTimerExW.restype=w.HANDLE
        k.SetWaitableTimer.argtypes=[w.HANDLE,c.POINTER(c.c_int64),w.LONG,c.c_void_p,c.c_void_p,w.BOOL];k.SetWaitableTimer.restype=w.BOOL
        k.WaitForSingleObject.argtypes=[w.HANDLE,w.DWORD];k.WaitForSingleObject.restype=w.DWORD
        for name in ("CloseHandle","CancelWaitableTimer"):
            f=getattr(k,name);f.argtypes=[w.HANDLE];f.restype=w.BOOL
        self.process=k.GetCurrentProcess();self.thread=k.GetCurrentThread()
        # Anonymous, non-inherited, local high-resolution timer; no system timer-resolution change.
        self.timer=k.CreateWaitableTimerExW(None,None,2,0x100002)
        if not self.timer:raise OSError(c.get_last_error(),"local high-resolution timer unavailable")

    def counters(self):
        result={}
        for prefix,function,handle in (("thread",self.k.GetThreadTimes,self.thread),("process",self.k.GetProcessTimes,self.process)):
            values=[w.FILETIME() for _ in range(4)]
            if not function(handle,*(c.byref(v) for v in values)):raise OSError(c.get_last_error(),"self CPU counters unavailable")
            ticks=[(v.dwHighDateTime<<32)|v.dwLowDateTime for v in values]
            result[prefix+"_kernel_ticks"],result[prefix+"_user_ticks"]=ticks[2:]
        result["cycles"]=read_cycles(self.k.QueryThreadCycleTime,self.thread)
        return result

    def wait_until(self, deadline):
        remaining=deadline-time.perf_counter_ns()
        if remaining <= 0:return
        due=c.c_int64(-max(1,(remaining+99)//100))
        if not self.k.SetWaitableTimer(self.timer,c.byref(due),0,None,None,False):
            raise OSError(c.get_last_error(),"reference timer arm failed")
        timeout=min(5000,(remaining+999999)//1000000+1000)
        if self.k.WaitForSingleObject(self.timer,timeout) != 0:
            raise RuntimeError("reference timer wait failed or timed out")

    def close(self):
        if self.timer is None:return
        handle=self.timer;self.timer=None
        try:
            if not self.k.CancelWaitableTimer(handle):raise OSError(c.get_last_error(),"reference timer cancel failed")
        finally:
            if not self.k.CloseHandle(handle):raise OSError(c.get_last_error(),"reference timer close failed")


def run_phase(api, phase, clock=time.perf_counter_ns):
    name,millis,offset=phase
    counters=api.counters();begin=clock();end=begin+millis*1_000_000
    pulses=missed=busy=awake=checksum=iterations=0
    gaps=[]
    if name.startswith("idle"):
        api.wait_until(end)
    elif name=="dense_control":
        at=clock()
        while clock() < end:
            checksum=(checksum*1664525+1013904223)&0xffffffff
            iterations+=1
        busy=clock()-at;awake=clock()-at;pulses=1
    else:
        deadline=begin+offset;previous=None
        while deadline < end:
            api.wait_until(deadline)
            at=clock()
            if at >= end:break
            if previous is not None:gaps.append(at-previous)
            previous=at
            stop=min(at+BUSY_NS,end)
            while clock() < stop:
                checksum=(checksum*1664525+1013904223)&0xffffffff
                iterations+=1
            after=clock();busy+=after-at;pulses+=1
            deadline,skipped=next_deadline(deadline,after);missed+=skipped
            awake+=clock()-at
            if pulses >= MAX_PULSES:raise RuntimeError("reference pulse bound exceeded")
        api.wait_until(end)
    finish=clock();last=api.counters()
    row=dict(phase=name,requested_ms=millis,offset_ns=offset,span_ns=finish-begin,pulses=pulses,
             busy_wall_ns=busy,awake_wall_ns=awake,missed=missed,checksum=checksum,
             iterations=iterations,
             wake_gaps_ns=gaps,**{k:last[k]-counters[k] for k in last})
    row["metrics"]=reference_cost(row)
    return row


def output_directory(value):
    path=(ROOT/value).resolve()
    if os.name != "nt" or path.exists() or ROOT/"artifacts" not in path.parents:
        raise RuntimeError("Windows and fresh artifacts directory required")
    path.mkdir(parents=True)
    return path


def child(output, reverse):
    api=NativeReference();rows=[];closed=False
    phases=(PHASES[0],*reversed(PHASES[1:-1]),PHASES[-1]) if reverse else PHASES
    try:
        for phase in phases:
            row=run_phase(api,phase);rows.append(row)
            print(json.dumps(dict(phase=phase[0],metrics=row["metrics"])),flush=True)
    finally:
        try:api.close();closed=True
        finally:
            (output/"report.json").write_text(json.dumps(dict(schema=2,pid=os.getpid(),phase_plan=phases,rows=rows,
                timer_closed=closed,clocks={k:vars(time.get_clock_info(k)) for k in ("monotonic","perf_counter","process_time")},
                scope="own Python main thread, local high-resolution timer; no GUI/audio; awake wall omits timer rearm/wait and kernel overhead, not a strict CPU upper bound"),indent=2)+"\n",encoding="utf8")


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("--output",required=True)
    parser.add_argument("--reference",action="store_true");parser.add_argument("--reverse",action="store_true")
    args=parser.parse_args();output=output_directory(args.output)
    if args.reference:
        child(output,args.reverse);return
    initial=source_identity();rounds=[]
    for i,reverse in enumerate((False,True),1):
        destination=output/("run-%02d"%i)
        command=[sys.executable,str(Path(__file__).resolve()),"--reference","--output",str(destination)]
        if reverse:command.append("--reverse")
        print("Starting reference %d/2"%i,flush=True)
        process=subprocess.Popen(command,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,encoding="utf8")
        timed_out=False
        try:
            try:stdout,stderr=process.communicate(timeout=40)
            except subprocess.TimeoutExpired:
                timed_out=True;process.kill();stdout,stderr=process.communicate(timeout=5)
        finally:
            if process.poll() is None:process.kill();process.wait(timeout=5)
        (output/("run-%02d-stdout.log"%i)).write_text(stdout,encoding="utf8")
        (output/("run-%02d-stderr.log"%i)).write_text(stderr,encoding="utf8")
        if timed_out or process.returncode:raise RuntimeError("owned reference failed or timed out; logs retained")
        report=json.loads((destination/"report.json").read_text(encoding="utf8"))
        phases=(PHASES[0],*reversed(PHASES[1:-1]),PHASES[-1]) if reverse else PHASES
        checks=reference_checks(report,process.pid,phases)
        if not all(checks.values()):
            raise RuntimeError("owned reference incomplete; evidence retained")
        report["reference_checks"]=checks
        rounds.append(report)
        unchanged=initial==source_identity()
        (output/"proof.json").write_text(json.dumps(dict(schema=2,identity=initial,platform=platform.platform(),
            rounds=rounds,unchanged=unchanged,completed=len(rounds)==2 and unchanged,product_acceptance=False,
            illustrative_models=[sampled_tick_model(span,offset) for span in (2_100_000_000,12_600_000_000) for offset in (0,200000)]),indent=2)+"\n",encoding="utf8")
        print(json.dumps(dict(round=i,rows=[dict(phase=r["phase"],metrics=r["metrics"]) for r in report["rows"]])),flush=True)
        if not unchanged:raise RuntimeError("reference source changed")


if __name__=="__main__":main()
