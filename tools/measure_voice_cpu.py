"""Read CPU counters of runner-owned Windows child threads; never sample other PIDs."""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import os
from pathlib import Path
from measure_audio_load import ROOT, WindowsSampler, observe, summarize
from measure_gui_candidate import evaluate_gui
from measure_combined import source_identity

LIMIT = 128


def work_cost(report):
    edges = report.get("work_edges", [])
    if (report.get("profile_work") is not True or len(edges) != 2
            or any(len(row) != 6 or any(type(v) is not int or v < 0 for v in row) for row in edges)):
        raise ValueError("missing coherent callback work edges")
    delta = [b-a for a,b in zip(*edges)]
    if any(v < 0 for v in delta) or delta[0] == 0 or delta[5] == 0 or sum(delta[2:5]) > delta[1]:
        raise ValueError("invalid callback work interval")
    span = (report["lifecycle"][4][1]-report["lifecycle"][3][1])/1000.
    if span <= 0: raise ValueError("invalid GUI work span")
    return dict(callbacks=delta[0], frames=delta[5], gui_span_seconds=span,
                audio_span_seconds=delta[5]/report["rate"],
                mean_callback_work_us=delta[1]/delta[0]/1000,
                wall_time_fraction_percent=dict(total=delta[1]/1e9/span*100, background=delta[2]/1e9/span*100,
                                                voice_prepare=delta[3]/1e9/span*100, final_filter_copy=delta[4]/1e9/span*100))


class ThreadEntry(c.Structure):
    _fields_ = [(name, w.DWORD) for name in ("size", "usage", "tid", "pid")] + [("base", w.LONG), ("delta", w.LONG), ("flags", w.DWORD)]


def group(name, main):
    if main: return "gui_main"
    return {"background-wav-reader": "background_read", "skin-audio-supply": "supply",
            "skin-audio-decode": "short_decode", "cpal_wasapi_out": "native_output",
            "gui-copy-observer": "copy_observer"}.get(name, "other")


def thread_cost(samples, phase="voices_32"):
    selected = [s for s in samples if s["phase"] == phase]
    if len(selected) < 2: raise ValueError("missing thread phase")
    a, b = selected[0], selected[-1]
    span = b["time"]-a["time"]
    if span <= 0: raise ValueError("invalid thread span")
    start = {t["key"]: t for t in a["threads"]}
    end = {t["key"]: t for t in b["threads"]}
    stable = start.keys() & end.keys()
    rows = []
    totals = {}
    for key in stable:
        t = start[key]
        e = end[key]
        delta = e["cpu_seconds"]-t["cpu_seconds"]
        if e["group"] != t["group"] or delta < 0: raise ValueError("thread identity/counter changed")
        totals[t["group"]] = totals.get(t["group"], 0.) + delta/span*100
        rows.append(dict(key=key, name=t["name"], group=t["group"], cpu_seconds=delta, one_core_percent=delta/span*100))
    return dict(span_seconds=span, groups_one_core_percent=totals, threads=rows,
                complete=start.keys() == end.keys(), entered=sorted(end.keys()-start.keys()), exited=sorted(start.keys()-end.keys()),
                process_one_core_percent=(b["cpu_seconds"]-a["cpu_seconds"])/span*100,
                counter_gap_seconds=(b["cpu_seconds"]-a["cpu_seconds"])-sum(r["cpu_seconds"] for r in rows))


class ThreadSampler(WindowsSampler):
    def __init__(self, child):
        super().__init__(child)
        self.pid = child.pid
        self.handles = {}
        self.main = None
        k = self.kernel
        k.CreateToolhelp32Snapshot.argtypes = [w.DWORD, w.DWORD]; k.CreateToolhelp32Snapshot.restype = w.HANDLE
        for name in ("Thread32First", "Thread32Next"):
            f = getattr(k, name); f.argtypes = [w.HANDLE, c.POINTER(ThreadEntry)]; f.restype = w.BOOL
        k.OpenThread.argtypes = [w.DWORD, w.BOOL, w.DWORD]; k.OpenThread.restype = w.HANDLE
        k.GetProcessIdOfThread.argtypes = [w.HANDLE]; k.GetProcessIdOfThread.restype = w.DWORD
        k.CloseHandle.argtypes = [w.HANDLE]; k.CloseHandle.restype = w.BOOL
        k.GetThreadTimes.argtypes = [w.HANDLE] + [c.POINTER(w.FILETIME)]*4; k.GetThreadTimes.restype = w.BOOL
        k.GetThreadDescription.argtypes = [w.HANDLE, c.POINTER(c.c_void_p)]; k.GetThreadDescription.restype = w.LONG
        k.LocalFree.argtypes = [c.c_void_p]; k.LocalFree.restype = c.c_void_p

    def times(self, handle):
        values = [w.FILETIME() for _ in range(4)]
        if not self.kernel.GetThreadTimes(handle, *(c.byref(t) for t in values)):
            raise OSError(c.get_last_error(), "owned child thread times unavailable")
        ticks = [(v.dwHighDateTime << 32) | v.dwLowDateTime for v in values]
        return ticks[0], (ticks[2]+ticks[3])/10**7

    def live_ids(self):
        k = self.kernel
        snapshot = k.CreateToolhelp32Snapshot(4, 0)
        if snapshot == w.HANDLE(-1).value: raise OSError(c.get_last_error(), "thread snapshot failed")
        live = []
        try:
            entry = ThreadEntry(); entry.size = c.sizeof(entry)
            valid = k.Thread32First(snapshot, c.byref(entry))
            while valid:
                if entry.pid == self.pid: live.append(entry.tid)
                valid = k.Thread32Next(snapshot, c.byref(entry))
        finally: k.CloseHandle(snapshot)
        if len(live) > LIMIT: raise RuntimeError("owned thread diagnostic limit exceeded")
        return live

    def open_thread(self, tid):
        return self.kernel.OpenThread(0x0800, False, tid)

    def owner(self, handle):
        return self.kernel.GetProcessIdOfThread(handle)

    def close_thread(self, handle):
        if not self.kernel.CloseHandle(handle):
            raise OSError(c.get_last_error(), "owned thread handle close failed")

    def description(self, handle):
        ptr = c.c_void_p()
        if self.kernel.GetThreadDescription(handle, c.byref(ptr)) < 0:
            raise OSError("owned thread description unavailable")
        try: return c.wstring_at(ptr.value)[:256] if ptr.value else ""
        finally:
            if ptr.value: self.kernel.LocalFree(ptr)

    def thread_rows(self, live):
        rows = []
        for tid in live:
            h = self.open_thread(tid)  # QUERY_LIMITED_INFORMATION, only filtered owner PID.
            if not h: continue  # A target thread can exit between snapshot and query.
            try:
                if self.owner(h) != self.pid: continue
                created, seconds = self.times(h)
                key = "%d:%d" % (tid, created)
                if key not in self.handles:
                    if len(self.handles) == LIMIT: raise RuntimeError("thread lifetime diagnostic limit exceeded")
                    self.handles[key] = self.description(h)
                rows.append(dict(key=key, created=created, name=self.handles[key], cpu_seconds=seconds))
            finally: self.close_thread(h)
        return rows

    def sample(self, phase):
        rows = self.thread_rows(self.live_ids())
        if self.main is None and rows: self.main = min(rows, key=lambda t: t["created"])["key"]
        for row in rows: row["group"] = group(row["name"], row["key"] == self.main)
        sample = super().sample(phase); sample["threads"] = rows
        return sample


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True)
    parser.add_argument("--instrumented", action="store_true")
    args = parser.parse_args()
    output = (ROOT/args.output).resolve()
    binary = ROOT/"desktop/target/release/skin-editor-desktop.exe"
    if os.name != "nt" or output.exists() or ROOT/"artifacts" not in output.parents:
        raise RuntimeError("Windows and fresh artifacts directory required")
    output.mkdir(parents=True)
    identity = dict(source=source_identity(), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
    rounds = []
    for i in range(2):
        report, samples = observe(binary, output/("run-%02d" % (i+1)), "--verify-profiled-gui" if args.instrumented else "--verify-streamed-gui", timeout_seconds=40, sampler_factory=ThreadSampler)
        metrics = summarize(samples, phases=("idle", "voices_1", "voices_32", "closed"), minimum_spans=(("voices_32", 1.7),))
        latency, _, checks = evaluate_gui(report, metrics)
        costs = {p: thread_cost(samples, p) for p in ("idle", "voices_1", "voices_32", "closed")}
        work = work_cost(report) if args.instrumented else None
        rounds.append(dict(report=report, metrics=metrics, latency=latency, checks=checks, costs=costs, work=work))
        unchanged = identity["source"] == source_identity() and identity["binary_sha256"] == hashlib.sha256(binary.read_bytes()).hexdigest()
        (output/"proof.json").write_text(json.dumps(dict(schema=1, identity=identity, rounds=rounds, unchanged=unchanged), indent=2), encoding="utf8")
        print(json.dumps(dict(round=i+1, cpu=costs["voices_32"]["groups_one_core_percent"], checks=checks)), flush=True)
        if not unchanged: raise RuntimeError("profile build changed")


if __name__ == "__main__": main()
