"""Owned Windows GUI cycles and thread-start modules; no stack capture or CPU discount."""
import argparse
from contextlib import contextmanager
import ctypes as c
from ctypes import wintypes as w
import json
import os
import platform
import re
import time
from measure_audio_load import ROOT, WindowsSampler, observe, summarize
from measure_gui_candidate import evaluate_gui
from measure_thread_sampling import CachedThreads, ObserverAudit, PHASES, additional_checks, identity, observer_cost
from measure_voice_cpu import thread_cost, work_cost

PLAN = ("process", "attributed", "attributed", "process")
BACKEND_PLAN = ("default", "primary", "primary", "default")
MODULE_LIMIT = 512


class ModuleEntry(c.Structure):
    _fields_ = [(n, w.DWORD) for n in ("size", "module_id", "pid", "global_count", "process_count")] + [
        ("base", c.c_void_p), ("bytes", w.DWORD), ("module", w.HMODULE),
        ("name", w.WCHAR*256), ("path", w.WCHAR*260)]


def origin(address, modules, status):
    if status != "ok": return dict(status=status, module=None, offset=None)
    hits = [m for m in modules if m["base"] <= address < m["base"]+m["bytes"]]
    if len(hits) != 1:
        return dict(status="unmapped-or-ambiguous", module=None, offset=None)
    m = hits[0]
    return dict(status="mapped-start-only", module=m["name"], offset=address-m["base"])


def configure_cycles(kernel):
    for name in ("QueryProcessCycleTime", "QueryThreadCycleTime"):
        f = getattr(kernel, name)
        f.argtypes = [w.HANDLE, c.POINTER(c.c_uint64)]; f.restype = w.BOOL


def read_cycles(function, handle):
    cycles = c.c_uint64()
    if not function(handle, c.byref(cycles)):
        raise OSError(c.get_last_error(), "owned cycle counter unavailable")
    return cycles.value


class CycleProcess(WindowsSampler):
    def __init__(self, child):
        super().__init__(child)
        configure_cycles(self.kernel)

    def sample(self, phase):
        result = super().sample(phase)
        result["process_cycles"] = read_cycles(self.kernel.QueryProcessCycleTime, self.handle)
        result["highres_time_ns"] = time.perf_counter_ns()
        return result


class AttributedThreads(CachedThreads):
    def __init__(self, child):
        super().__init__(child)
        configure_cycles(self.kernel)
        self.details, self.tids, self.starts = {}, {}, {}
        self.modules, self.module_status = [], "not-refreshed"
        for name in ("Module32FirstW", "Module32NextW"):
            f = getattr(self.kernel, name)
            f.argtypes = [w.HANDLE, c.POINTER(ModuleEntry)]; f.restype = w.BOOL
        # Documented optional internal diagnostic. No product/import-library dependency.
        try:
            self.ntdll = c.WinDLL("ntdll", use_last_error=True)
            self.query_start = self.ntdll.NtQueryInformationThread
            self.query_start.argtypes = [w.HANDLE, w.LONG, c.c_void_p, w.ULONG, c.POINTER(w.ULONG)]
            self.query_start.restype = w.LONG
        except (OSError, AttributeError): self.query_start = None

    def module_snapshot(self):
        k = self.kernel
        handle = k.CreateToolhelp32Snapshot(0x8 | 0x10, self.pid)
        if handle == w.HANDLE(-1).value:
            return [], "module-snapshot-error-%d" % c.get_last_error()
        rows = []
        try:
            entry = ModuleEntry(); entry.size = c.sizeof(entry)
            c.set_last_error(0)
            valid = k.Module32FirstW(handle, c.byref(entry))
            while valid:
                if entry.pid != self.pid: raise RuntimeError("foreign module PID rejected")
                if len(rows) >= MODULE_LIMIT: raise RuntimeError("module diagnostic limit exceeded")
                # Never persist the full path or read image/process memory.
                rows.append(dict(name=entry.name.lower(), base=entry.base or 0, bytes=entry.bytes))
                c.set_last_error(0)
                valid = k.Module32NextW(handle, c.byref(entry))
            error = c.get_last_error()
            return (rows, "ok") if error == 18 else ([], "module-enumeration-error-%d" % error)
        finally:
            if not k.CloseHandle(handle): raise OSError(c.get_last_error(), "module snapshot close failed")

    def open_thread(self, tid):
        handle = self.kernel.OpenThread(0x0840, False, tid)  # Read-only QUERY_INFORMATION + LIMITED.
        if handle: self.tids[handle] = tid
        return handle

    def times(self, handle):
        values = [w.FILETIME() for _ in range(4)]
        if not self.kernel.GetThreadTimes(handle, *(c.byref(t) for t in values)):
            raise OSError(c.get_last_error(), "owned thread times unavailable")
        ticks = [(v.dwHighDateTime << 32) | v.dwLowDateTime for v in values]
        self.details[handle] = dict(kernel_seconds=ticks[2]/1e7, user_seconds=ticks[3]/1e7,
                                    cycles=read_cycles(self.kernel.QueryThreadCycleTime, handle))
        return ticks[0], (ticks[2]+ticks[3])/1e7

    def description(self, handle):
        name = super().description(handle)
        created, _ = super().times(handle)
        key = "%d:%d" % (self.tids[handle], created)
        if self.query_start is None:
            address, status = 0, "start-query-unavailable"
        else:
            pointer, length = c.c_void_p(), w.ULONG()
            # THREADINFOCLASS=9; also verified against locked windows-sys 0.61.2.
            code = self.query_start(handle, 9, c.byref(pointer), c.sizeof(pointer), c.byref(length))
            address = pointer.value or 0
            status = "ok" if code >= 0 and address and length.value == c.sizeof(pointer) else "start-query-error-%08x" % (code & 0xffffffff)
        self.starts[key] = (address, status)
        return name

    def close_thread(self, handle):
        try: super().close_thread(handle)
        finally:
            self.details.pop(handle, None); self.tids.pop(handle, None)

    def sample(self, phase):
        if self.retained.last_phase != phase:
            self.modules, self.module_status = self.module_snapshot()
        result = super().sample(phase)
        for row in result["threads"]:
            row.update(self.details[self.retained.entries[row["key"]]["handle"]])
            address, status = self.starts[row["key"]]
            row["origin"] = origin(address, self.modules, status if status != "ok" else self.module_status)
        result["process_cycles"] = read_cycles(self.kernel.QueryProcessCycleTime, self.handle)
        result["highres_time_ns"] = time.perf_counter_ns()
        result["module_count"] = len(self.modules)
        result["module_status"] = self.module_status
        return result


class AttributionAudit(ObserverAudit):
    def __init__(self, child, mode):
        self.backend = AttributedThreads(child) if mode == "attributed" else CycleProcess(child)
        self.mode = "cached" if mode == "attributed" else "process"
        self.close_record = None


def cycle_cost(samples, phase="voices_32"):
    rows = [s for s in samples if s["phase"] == phase]
    if len(rows) < 2: raise ValueError("missing cycle phase")
    a, b = rows[0], rows[-1]
    cycles = b["process_cycles"]-a["process_cycles"]
    ns = b["highres_time_ns"]-a["highres_time_ns"]
    if cycles < 0 or ns <= 0: raise ValueError("invalid cycle/high-resolution interval")
    result = dict(process_cycles=cycles, highres_span_seconds=ns/1e9, units="cycles, never CPU percent or milliseconds")
    if "threads" in a:
        start = {t["key"]: t for t in a["threads"]}; end = {t["key"]: t for t in b["threads"]}
        threads = []
        for key in sorted(start.keys() & end.keys()):
            s, e = start[key], end[key]
            delta = {k:e[k]-s[k] for k in ("cycles", "kernel_seconds", "user_seconds")}
            if any(v < 0 for v in delta.values()) or e["group"] != s["group"]:
                raise ValueError("cycle/counter identity changed")
            threads.append(dict(group=e["group"], name=e["name"], origin=e["origin"], **delta))
        result.update(threads=threads, cycle_counter_gap=cycles-sum(t["cycles"] for t in threads),
                      entered=sorted(end.keys()-start.keys()), exited=sorted(start.keys()-end.keys()))
    return result


def gui_cost(report):
    data = report.get("gui_work")
    if (report.get("profile_work") is not True or not isinstance(data, dict) or type(data.get("invalid")) is not int or data["invalid"] != 0
            or data.get("scope") != "previous-framework-frame-wall-excluding-reported-vsync"):
        raise ValueError("missing valid GUI wall-clock scope")
    for rows in (data["logic"], data["frame_wall"], report["ui_cost"]):
        if len(rows) != 18 or any(len(r) != 3 or any(type(v) is not int or v < 0 for v in r)
                                  or r[2] > r[1] or r[0] == 0 and r[1:] != [0,0] for r in rows):
            raise ValueError("invalid GUI work rows")
        if rows[4][0] < 100: raise ValueError("insufficient 32 Voice GUI observations")
    return {key: dict(samples=rows[4][0], total_wall_ms=rows[4][1]/1e6,
                      mean_wall_ms=rows[4][1]/rows[4][0]/1e6, max_wall_ms=rows[4][2]/1e6)
            for key, rows in (("logic", data["logic"]), ("framework_previous_frame", data["frame_wall"]),
                              ("ui_body", report["ui_cost"]))}


def adapter_info(report):
    raw = report.get("adapter_utf8")
    if not isinstance(raw,list) or not 0 < len(raw) <= 4096 or any(type(v) is not int or not 0 <= v <= 255 for v in raw):
        raise ValueError("missing bounded actual adapter identity")
    text = bytes(raw).decode("utf8")
    backend = re.search(r"\bbackend: (\w+)\b",text)
    if not backend: raise ValueError("missing actual backend in locked wgpu AdapterInfo")
    return dict(info=text,backend=backend.group(1))


@contextmanager
def backend_override(value):
    previous = os.environ.get("WGPU_BACKEND")
    if value is not None: os.environ["WGPU_BACKEND"] = value
    try: yield
    finally:
        if previous is None: os.environ.pop("WGPU_BACKEND",None)
        else: os.environ["WGPU_BACKEND"] = previous


def main():
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument("--output", required=True)
    parser.add_argument("--backend-control",action="store_true",help="Owned Windows child default/primary/primary/default control; Vulkan and DX12 retained, GL excluded; product default unchanged")
    args = parser.parse_args(); output = (ROOT/args.output).resolve()
    binary = ROOT/"desktop/target/release/skin-editor-desktop.exe"
    if os.name != "nt" or output.exists() or ROOT/"artifacts" not in output.parents:
        raise RuntimeError("Windows and fresh artifacts directory required")
    output.mkdir(parents=True); initial = identity(binary); rounds = []
    plan = BACKEND_PLAN if args.backend_control else PLAN
    for i, mode in enumerate(plan, 1):
        print("Starting %d/%d %s" % (i, len(plan), mode), flush=True); audit = []
        sampler_mode = "attributed" if args.backend_control else mode
        def factory(child):
            value = AttributionAudit(child, sampler_mode); audit.append(value); return value
        try:
            with backend_override("vulkan,dx12" if args.backend_control and mode == "primary" else None):
                report, samples = observe(binary, output/("run-%02d"%i), "--verify-profiled-gui", timeout_seconds=40, sampler_factory=factory)
            metrics = summarize(samples, phases=PHASES, minimum_spans=(("idle",.7),("voices_1",1.2),("voices_32",1.7),("closed",.7)))
            latency, _, checks = evaluate_gui(report, metrics)
            checks.update(additional_checks(report, samples, audit[0].close_record))
            adapter = adapter_info(report)
            checks["requested_backend_selected"] = not args.backend_control or mode != "primary" or adapter["backend"] in ("Vulkan","Dx12")
            row = dict(completed=True, mode=mode, report=report, metrics=metrics, latency=latency, checks=checks,
                       adapter=adapter,
                       cycles=cycle_cost(samples), gui_work=gui_cost(report), callback_work=work_cost(report),
                       threads=thread_cost(samples) if sampler_mode == "attributed" else None,
                       observer=observer_cost(samples,"voices_32"), observer_close=audit[0].close_record,
                       all_sampled_memory_max={k:max(s[k] for s in samples) for k in ("working_set_bytes","private_bytes")})
        except (OSError,RuntimeError,ValueError,KeyError,TypeError) as error:
            row = dict(completed=False, mode=mode, error=str(error), checks=dict(completed=False))
        rounds.append(row); unchanged = initial == identity(binary)
        proof = dict(schema=1, identity=initial, platform=platform.platform(), plan=plan,
                     backend_control=args.backend_control,
                     clocks={k:vars(time.get_clock_info(k)) for k in ("monotonic","perf_counter","process_time")},
                     rounds=rounds, unchanged=unchanged,
                     passed=len(rounds)==len(plan) and unchanged and all(all(r["checks"].values()) for r in rounds))
        (output/"proof.json").write_text(json.dumps(proof,indent=2)+"\n",encoding="utf8")
        print(json.dumps(dict(round=i,mode=mode,completed=row["completed"],checks=row["checks"],
                              cpu=row.get("metrics",{}).get("voices_32"))),flush=True)
        if not row["completed"] or not unchanged: break
    if not proof["passed"]: raise RuntimeError("GUI CPU attribution plan failed; evidence retained")


if __name__ == "__main__": main()
