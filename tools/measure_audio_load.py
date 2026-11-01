"""Opt-in metrics for a runner-owned native audio child; Windows, no window APIs."""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import math
import os
import platform
from pathlib import Path
import queue
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
PHASES = ("idle", "latency", "voices_1", "voices_32", "isolation", "closed")


def distribution(values):
    if not values or any(not math.isfinite(v) or v < 0 for v in values):
        raise ValueError("missing/invalid latency samples")
    values = sorted(values)
    return {"count": len(values), "p50": values[math.ceil(len(values)*.5)-1],
            "p95": values[math.ceil(len(values)*.95)-1],
            "p99": values[math.ceil(len(values)*.99)-1], "max": values[-1]}


def summarize(samples, phases=PHASES, minimum_spans=None):
    result = {}
    for phase in phases:
        selected = [s for s in samples if s["phase"] == phase]
        if len(selected) < 2:
            raise ValueError("missing phase samples: " + phase)
        first, last = selected[0], selected[-1]
        span = last["time"] - first["time"]
        cpu = last["cpu_seconds"] - first["cpu_seconds"]
        if span <= 0 or cpu < 0:
            raise ValueError("invalid phase clock/cpu")
        for s in selected:
            if any(s[k] <= 0 for k in ("working_set_bytes", "private_bytes")):
                raise ValueError("missing memory counters")
        result[phase] = {"sample_count": len(selected), "sample_span_seconds": span,
                         "cpu_seconds": cpu, "cpu_one_logical_core_percent": cpu/span*100,
                         "working_set_bytes_max": max(s["working_set_bytes"] for s in selected),
                         "private_bytes_max": max(s["private_bytes"] for s in selected)}
    spans = minimum_spans if minimum_spans is not None else (("idle", 2), ("voices_1", 5), ("voices_32", 5), ("closed", 1))
    if any(result[p]["sample_span_seconds"] < seconds for p, seconds in spans):
        raise ValueError("insufficient stable-phase span")
    return result


def checks(report, metrics):
    lat = report["latency_upper_bound_ms"]
    if len(lat) != 2 or any(len(v) != 64 for v in lat):
        raise ValueError("missing per-lane native observations")
    return {"native_submission_observed_upper_bound_p95_le_20_ms": all(distribution(v)["p95"] <= 20 for v in lat),
            "no_startup_underflow": report["startup_empty_frames"] == 0,
            "idle_retiming_exercised": report["retimed_idle_frames"] > 0,
            "no_steady_underflow": report["steady_empty_frames"] == 0,
            "thirty_two_voices_and_refuse_33rd": report["peak_voices"] == 32 and report["capacity_refusals"] == 4,
            "dual_channel_isolation": report["dual_channel_isolation"],
            "managed_pcm_released": report["pcm_final_bytes"] == 0 and report["pcm_peak_bytes"] <= 32*1024**2,
            "audio_only_process_memory_le_512_mib": all(m["working_set_bytes_max"] <= 512*1024**2 and m["private_bytes_max"] <= 512*1024**2 for m in metrics.values())}


class Memory(c.Structure):
    _fields_ = [("cb", w.DWORD), ("faults", w.DWORD)] + [(n, c.c_size_t) for n in
                ("peak_ws", "ws", "peak_pool", "pool", "peak_nonpaged", "nonpaged", "pagefile", "peak_pagefile", "private")]


class WindowsSampler:
    def __init__(self, child):
        if os.name != "nt":
            raise RuntimeError("process metrics adapter not implemented for this platform")
        self.handle = w.HANDLE(int(child._handle))  # Only this runner's own Popen child.
        self.kernel = c.WinDLL("kernel32", use_last_error=True)
        self.psapi = c.WinDLL("psapi", use_last_error=True)
        self.kernel.GetProcessTimes.argtypes = [w.HANDLE] + [c.POINTER(w.FILETIME)]*4
        self.kernel.GetProcessTimes.restype = w.BOOL
        self.psapi.GetProcessMemoryInfo.argtypes = [w.HANDLE, c.POINTER(Memory), w.DWORD]
        self.psapi.GetProcessMemoryInfo.restype = w.BOOL

    def sample(self, phase):
        memory = Memory(); memory.cb = c.sizeof(memory)
        times = [w.FILETIME() for _ in range(4)]
        if not self.kernel.GetProcessTimes(self.handle, *(c.byref(t) for t in times)) or not self.psapi.GetProcessMemoryInfo(self.handle, c.byref(memory), memory.cb):
            raise OSError(c.get_last_error(), "runner-owned process counters unavailable")
        ticks = sum((t.dwHighDateTime << 32) | t.dwLowDateTime for t in times[2:])
        return dict(phase=phase, time=time.monotonic(), cpu_seconds=ticks/10**7,
                    working_set_bytes=memory.ws, private_bytes=memory.private,
                    lifetime_peak_working_set_bytes=memory.peak_ws, lifetime_peak_pagefile_bytes=memory.peak_pagefile)


def observe(binary, directory, argument, timeout_seconds=90, sampler_factory=WindowsSampler):
    if not 0 < timeout_seconds <= 900:
        raise ValueError("invalid bounded observation timeout")
    messages = queue.Queue(maxsize=32)
    overflow = threading.Event()
    with (directory.parent / (directory.name + "-stderr.log")).open("w", encoding="utf-8") as errors:
        child = subprocess.Popen([str(binary), argument, str(directory)], cwd=ROOT,
                                 stdout=subprocess.PIPE, stderr=errors, text=True, encoding="utf-8")
        def read():
            for line in child.stdout:
                try:
                    messages.put_nowait(json.loads(line))
                except (ValueError, queue.Full):
                    overflow.set()
        reader = threading.Thread(target=read, daemon=True); reader.start()
        samples = []; phase = "setup"; deadline = time.monotonic()+timeout_seconds
        sampler = None
        try:
            sampler = sampler_factory(child)
            while child.poll() is None:
                try:
                    phase = messages.get(timeout=.1)["phase"]
                except queue.Empty:
                    pass
                try:
                    samples.append(sampler.sample(phase))
                except OSError:
                    if child.poll() is None:
                        raise
                    break
                if time.monotonic() >= deadline:
                    raise RuntimeError("native audio load timed out")
            reader.join(timeout=2)
            with (directory.parent / (directory.name + "-observation.json")).open("x", encoding="utf-8") as recording:
                json.dump(dict(samples=samples, last_phase=phase, returncode=child.returncode, protocol_overflow=overflow.is_set()), recording, indent=2)
            if reader.is_alive() or overflow.is_set() or child.returncode:
                raise RuntimeError("native audio load failed or protocol overflowed; inspect owned stderr log")
        finally:
            if sampler is not None and hasattr(sampler, "close"):
                sampler.close()
            if child.poll() is None:
                child.kill(); child.wait(timeout=5)
            reader.join(timeout=2)
            observation_path = directory.parent / (directory.name + "-observation.json")
            if not observation_path.exists():
                with observation_path.open("x", encoding="utf-8") as recording:
                    json.dump(dict(samples=samples, last_phase=phase, returncode=child.returncode, protocol_overflow=overflow.is_set()), recording, indent=2)
            child.stdout.close()
    report = json.loads((directory / "report.json").read_text(encoding="utf-8"))
    return report, samples


def run(binary, directory):
    report, samples = observe(binary, directory, "--verify-audio-load")
    metrics = summarize(samples)
    return {"report": report, "phases": metrics, "samples": samples,
            "latency_ms": [distribution(v) for v in report["latency_upper_bound_ms"]],
            "receipt_ms": [distribution(v) for v in report["receipt_ms"]], "checks": checks(report, metrics)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    binary = Path(args.binary).resolve(); output = (ROOT / args.output).resolve()
    expected = (ROOT / "desktop/target/release/skin-editor-desktop.exe").resolve()
    if os.name != "nt" or binary != expected or not binary.is_file():
        raise RuntimeError("requires the explicit Windows release desktop binary")
    if output.exists() or ROOT / "artifacts" not in output.parents:
        raise RuntimeError("output must be a new directory in artifacts")
    output.mkdir(parents=True)
    fingerprint = hashlib.sha256(binary.read_bytes()).hexdigest()
    runs = [run(binary, output / ("run-%02d" % (i+1))) for i in range(2)]
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == fingerprint
    result = dict(schema=1, binary_sha256=fingerprint, builds="release", platform="Windows",
                  logical_processors=os.cpu_count(), os_build=platform.platform(), processor_identifier=os.environ.get("PROCESSOR_IDENTIFIER"), sample_interval_ms=100, runs=runs,
                  passed=all(all(r["checks"].values()) for r in runs),
                  scope="native callback observation upper bound includes observer scheduling; CPU one logical core, audio-only process; device consumption/acoustic and full graphics budgets unmeasured")
    (output / "metrics.json").write_text(json.dumps(result, indent=2)+"\n", encoding="utf-8")
    print(json.dumps({"passed": result["passed"], "runs": [{"latency_ms": r["latency_ms"], "checks": r["checks"], "phases": r["phases"]} for r in runs]}, indent=2))
    if not result["passed"]:
        raise RuntimeError("scoped audio load acceptance failed; evidence retained")


if __name__ == "__main__":
    main()
