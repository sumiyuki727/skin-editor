"""Unprivileged process counters; diagnostic prototype costs, not P2 CPU proof."""

import os
import sys
import time


def snapshot():
    result = {"wall": time.perf_counter(), "cpu": time.process_time(), "memory_source": None}
    if os.name == "nt":
        import ctypes
        from ctypes import wintypes

        class Memory(ctypes.Structure):
            _fields_ = [("cb", wintypes.DWORD), ("faults", wintypes.DWORD)] + [
                (name, ctypes.c_size_t) for name in ("peak_working", "working", "peak_paged", "paged",
                                                   "peak_nonpaged", "nonpaged", "commit", "peak_commit", "private")]

        class Io(ctypes.Structure):
            _fields_ = [(name, ctypes.c_ulonglong) for name in
                        ("reads", "writes", "others", "read_bytes", "write_bytes", "other_bytes")]

        kernel, psapi = ctypes.WinDLL("kernel32", use_last_error=True), ctypes.WinDLL("psapi", use_last_error=True)
        kernel.GetCurrentProcess.restype = wintypes.HANDLE
        handle = kernel.GetCurrentProcess()
        psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Memory), wintypes.DWORD]
        kernel.GetProcessIoCounters.argtypes = [wintypes.HANDLE, ctypes.POINTER(Io)]
        memory, io = Memory(), Io()
        memory.cb = ctypes.sizeof(memory)
        if not psapi.GetProcessMemoryInfo(handle, ctypes.byref(memory), memory.cb):
            raise OSError(ctypes.get_last_error(), "GetProcessMemoryInfo")
        if not kernel.GetProcessIoCounters(handle, ctypes.byref(io)):
            raise OSError(ctypes.get_last_error(), "GetProcessIoCounters")
        result.update(working=memory.working, private=memory.private,
                      lifetime_peak_working=memory.peak_working, lifetime_peak_commit=memory.peak_commit,
                      read_bytes=io.read_bytes, write_bytes=io.write_bytes,
                      read_operations=io.reads, write_operations=io.writes,
                      memory_source="GetProcessMemoryInfo; lifetime peaks, not sampled phase peaks",
                      io_source="GetProcessIoCounters; this process including SQLite journal I/O")
    else:
        import resource
        peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        result.update(working=None, private=None,
                      lifetime_peak_working=peak if sys.platform == "darwin" else peak * 1024,
                      lifetime_peak_commit=None, read_bytes=None, write_bytes=None,
                      memory_source="getrusage lifetime RSS only; private commit unavailable",
                      io_source="unavailable in this prototype adapter")
    return result


def difference(before, after):
    return {"wall_ms": (after["wall"] - before["wall"]) * 1000,
            "cpu_ms": (after["cpu"] - before["cpu"]) * 1000,
            "read_bytes": None if after["read_bytes"] is None else after["read_bytes"] - before["read_bytes"],
            "write_bytes": None if after["write_bytes"] is None else after["write_bytes"] - before["write_bytes"],
            "private_after": after["private"], "working_after": after["working"],
            "lifetime_peak_working": after["lifetime_peak_working"],
            "lifetime_peak_commit": after["lifetime_peak_commit"]}
