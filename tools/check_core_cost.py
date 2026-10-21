"""Build an independent diagnostic and measure only its Windows child process.

No P0 imports, UI, image bytes, private directories or new Python dependencies.
The core remains free of clocks, OS APIs and allocator instrumentation.
"""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import platform
import queue
import subprocess
import threading
import time

from check_core import ROOT, linker_arguments, tool_environment

PHASES = ("ready", "initialized", "transactions_a", "fork_checkout", "transactions_b", "cross_branch", "draft_cancel", "released")
PROFILES = {"minimal": (0, 0, 0), "standard": (100, 10, 500), "pressure": (1000, 100, 5000)}


def validate_phase(value, expected):
    if value.get("phase") != expected:
        raise ValueError("Unexpected diagnostic phase")
    for field in ("wall_ns", "operations", "p50_ns", "p95_ns", "p99_ns", "max_ns", "nodes", "retained_blobs", "retained_requests"):
        if type(value.get(field)) is not int or value[field] < 0:
            raise ValueError("Invalid diagnostic field: " + field)
    if not value["p50_ns"] <= value["p95_ns"] <= value["p99_ns"] <= value["max_ns"]:
        raise ValueError("Invalid latency order")
    if expected in ("transactions_a", "transactions_b") and value["operations"] != 5000:
        raise ValueError("Incomplete transaction workload")
    if expected == "cross_branch" and value["operations"] != 2:
        raise ValueError("Incomplete branch round trip")
    if expected == "draft_cancel" and value["operations"] != 300:
        raise ValueError("Incomplete draft workload")
    return value


def safe_output(value):
    target = Path(value).resolve()
    artifacts = (ROOT / "artifacts").resolve()
    if target == artifacts or artifacts not in target.parents:
        raise ValueError("Diagnostic output must be a new directory inside artifacts/")
    if target.exists():
        raise ValueError("Output directory already exists; choose a new name")
    return target


def built_executable(output):
    """Use Cargo's actual artifact, including a configured target directory."""
    artifacts = [message["executable"] for message in map(json.loads, output.splitlines())
                 if message.get("reason") == "compiler-artifact"
                 and message.get("target", {}).get("name") == "skin-editor-core-cost"
                 and message.get("executable")]
    if len(artifacts) != 1:
        raise ValueError("Expected exactly one diagnostic executable from Cargo")
    return Path(artifacts[0])


class MemoryCounters(ctypes.Structure):
    _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
        (name, ctypes.c_size_t) for name in (
            "PeakWorkingSetSize", "WorkingSetSize", "QuotaPeakPagedPoolUsage", "QuotaPagedPoolUsage",
            "QuotaPeakNonPagedPoolUsage", "QuotaNonPagedPoolUsage", "PagefileUsage", "PeakPagefileUsage", "PrivateUsage",
        )
    ]


class WindowsProcess:
    def __init__(self, pid):
        self.kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        self.psapi = ctypes.WinDLL("psapi", use_last_error=True)
        self.kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
        self.kernel.OpenProcess.restype = wintypes.HANDLE
        self.kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        self.kernel.CloseHandle.restype = wintypes.BOOL
        self.kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        self.kernel.GetProcessTimes.restype = wintypes.BOOL
        self.psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(MemoryCounters), wintypes.DWORD]
        self.psapi.GetProcessMemoryInfo.restype = wintypes.BOOL
        self.handle = self.kernel.OpenProcess(0x1000 | 0x0010, False, pid)
        if not self.handle:
            raise ctypes.WinError(ctypes.get_last_error())

    def sample(self):
        values = MemoryCounters()
        values.cb = ctypes.sizeof(values)
        times = [wintypes.FILETIME() for _ in range(4)]
        if not self.psapi.GetProcessMemoryInfo(self.handle, ctypes.byref(values), values.cb):
            raise ctypes.WinError(ctypes.get_last_error())
        if not self.kernel.GetProcessTimes(self.handle, *(ctypes.byref(t) for t in times)):
            raise ctypes.WinError(ctypes.get_last_error())
        cpu_ns = sum((t.dwHighDateTime << 32 | t.dwLowDateTime) * 100 for t in times[2:])
        return {"working_set_bytes": values.WorkingSetSize, "private_commit_bytes": values.PrivateUsage,
                "os_peak_working_set_bytes": values.PeakWorkingSetSize, "os_peak_commit_bytes": values.PeakPagefileUsage,
                "cpu_ns": cpu_ns}

    def close(self):
        if self.handle:
            self.kernel.CloseHandle(self.handle)
            self.handle = None


def run_probe(executable, profile, repetition, output):
    stderr_path = output / (profile + "-" + str(repetition) + ".stderr.log")
    with stderr_path.open("x", encoding="utf-8") as stderr:
        process = subprocess.Popen([str(executable), profile], cwd=ROOT, stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=stderr, text=True, encoding="utf-8")
        monitor = None
        messages = queue.Queue()

        def read_lines():
            try:
                for line in process.stdout:
                    messages.put(line)
            finally:
                messages.put(None)

        reader = threading.Thread(target=read_lines, daemon=True)
        reader.start()
        try:
            monitor = WindowsProcess(process.pid)
            phases = []
            previous_cpu = None
            for phase in PHASES:
                deadline = time.monotonic() + 600
                count, maximum_ws, maximum_commit = 0, 0, 0
                while True:
                    point = monitor.sample()
                    count += 1
                    maximum_ws = max(maximum_ws, point["working_set_bytes"])
                    maximum_commit = max(maximum_commit, point["private_commit_bytes"])
                    if time.monotonic() > deadline:
                        raise RuntimeError("Diagnostic phase timed out: " + phase)
                    try:
                        line = messages.get(timeout=0.010)
                    except queue.Empty:
                        continue
                    if line is None:
                        raise RuntimeError("Diagnostic exited before completing " + phase)
                    value = validate_phase(json.loads(line), phase)
                    # The child is waiting for an acknowledgement at each boundary.
                    point = monitor.sample()
                    maximum_ws = max(maximum_ws, point["working_set_bytes"])
                    maximum_commit = max(maximum_commit, point["private_commit_bytes"])
                    cpu_ns = 0 if previous_cpu is None else point["cpu_ns"] - previous_cpu
                    previous_cpu = point["cpu_ns"]
                    value.update(boundary=point, sampled_peak_working_set_bytes=maximum_ws,
                                 sampled_peak_private_commit_bytes=maximum_commit, samples=count + 1, cpu_ns=cpu_ns,
                                 single_core_cpu_percent=100 * cpu_ns / value["wall_ns"] if value["wall_ns"] else None)
                    phases.append(value)
                    process.stdin.write("continue\n")
                    process.stdin.flush()
                    print(profile, repetition, phase, "wall_ms=%.2f" % (value["wall_ns"] / 1e6),
                          "private_MiB=%.2f" % (point["private_commit_bytes"] / 1048576), flush=True)
                    break
            process.stdin.close()
            if process.wait(timeout=10):
                raise RuntimeError("Diagnostic failed; see its stderr log")
            reader.join(timeout=5)
            initialized = phases[1]["nodes"]
            if phases[4]["nodes"] != initialized + 10000 or phases[5]["nodes"] != phases[6]["nodes"]:
                raise ValueError("History node count changed unexpectedly")
            if phases[6]["retained_blobs"] != PROFILES[profile][0]:
                raise ValueError("Resource retention count changed")
            return {"profile": profile, "repetition": repetition, "phases": phases}
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
            process.stdout.close()
            if not process.stdin.closed:
                process.stdin.close()
            if monitor:
                monitor.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, help="New directory inside artifacts/")
    parser.add_argument("--profiles", nargs="+", choices=tuple(PROFILES), default=list(PROFILES))
    parser.add_argument("--repetitions", type=int, choices=range(1, 6), default=2)
    parser.add_argument("--toolchain-dir")
    args = parser.parse_args()
    if os.name != "nt":
        raise RuntimeError("Only the Windows monitor is implemented; other platforms are unverified")
    output = safe_output(args.output)
    cargo, environment = tool_environment(args.toolchain_dir)
    linker = linker_arguments(environment)
    manifest = ROOT / "tools/core-cost/Cargo.toml"
    common = ["--manifest-path", str(manifest), "--offline", "--locked"]
    subprocess.run([cargo, "fmt", "--manifest-path", str(manifest), "--", "--check"], cwd=ROOT, env=environment, check=True)
    subprocess.run([cargo, *linker, "clippy", *common, "--all-targets", "--", "-D", "warnings"], cwd=ROOT, env=environment, check=True)
    build = subprocess.check_output([cargo, *linker, "build", *common, "--release", "--message-format=json"],
                                    cwd=ROOT, env=environment, text=True)
    executable = built_executable(build)
    source = sorted(list((ROOT / "crates").rglob("*.rs")) + list((ROOT / "crates").rglob("Cargo.toml")) +
                    [ROOT / n for n in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "tools/core-cost/Cargo.toml",
                                      "tools/core-cost/Cargo.lock", "tools/core-cost/src/main.rs", "tools/check_core_cost.py")])
    output.mkdir(parents=True, exist_ok=False)
    report = {"schema_version": 1, "api_version": 4, "platform": platform.platform(), "logical_cpus": os.cpu_count(),
              "scope": "release metadata-only core child process; diagnostic parent excluded; no GUI/GPU/audio/image bytes",
              "sampling_target_ms": 10, "peak_semantics": "sampled peaks may miss transient allocations; os_peak_* are cumulative process-lifetime high water marks",
              "cpu_semantics": "GetProcessTimes deltas include diagnostic bookkeeping; percentage is one logical core; wall latency uses child Instant",
              "source_sha256": {str(p.relative_to(ROOT)).replace('\\', '/'): hashlib.sha256(p.read_bytes()).hexdigest() for p in source},
              "executable_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(), "profiles": PROFILES,
              "transactions_per_process": 10000, "draft_replacements": 300, "branch_round_trips": 1, "runs": []}
    for profile in args.profiles:
        for repetition in range(1, args.repetitions + 1):
            report["runs"].append(run_probe(executable, profile, repetition, output))
            with (output / "report.json").open("w", encoding="utf-8", newline="\n") as result:
                json.dump(report, result, ensure_ascii=False, indent=2)
                result.write("\n")
    print("PASS: diagnostic completed; report:", output / "report.json", flush=True)


if __name__ == "__main__":
    main()
