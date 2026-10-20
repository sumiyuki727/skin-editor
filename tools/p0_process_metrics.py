"""Read metrics only for a runner-owned Windows process; no window APIs."""

import ctypes
from ctypes import wintypes
import os
import platform
import time


class MemoryCounters(ctypes.Structure):
    _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
        (name, ctypes.c_size_t) for name in (
            "PeakWorkingSetSize", "WorkingSetSize", "QuotaPeakPagedPoolUsage", "QuotaPagedPoolUsage",
            "QuotaPeakNonPagedPoolUsage", "QuotaNonPagedPoolUsage", "PagefileUsage", "PeakPagefileUsage", "PrivateUsage")]


class CounterUnion(ctypes.Union):
    _fields_ = [("doubleValue", ctypes.c_double), ("largeValue", ctypes.c_longlong)]


class CounterValue(ctypes.Structure):
    _fields_ = [("CStatus", wintypes.DWORD), ("value", CounterUnion)]


class CounterItem(ctypes.Structure):
    _fields_ = [("szName", wintypes.LPWSTR), ("FmtValue", CounterValue)]


class MemoryRegion(ctypes.Structure):
    _fields_ = [("BaseAddress", ctypes.c_void_p), ("AllocationBase", ctypes.c_void_p),
                ("AllocationProtect", wintypes.DWORD), ("PartitionId", wintypes.WORD),
                ("RegionSize", ctypes.c_size_t), ("State", wintypes.DWORD),
                ("Protect", wintypes.DWORD), ("Type", wintypes.DWORD)]


def summarize_regions(regions):
    """Virtual page classification, not allocation ownership or PrivateUsage."""
    totals = {"private": 0, "mapped": 0, "image": 0, "other": 0}
    allocations = {}
    for region in regions:
        if region["state"] != 0x1000:
            continue
        kind = {0x20000: "private", 0x40000: "mapped", 0x1000000: "image"}.get(region["type"], "other")
        totals[kind] += region["size"]
        key = (region["allocation_base"], kind)
        allocations[key] = allocations.get(key, 0) + region["size"]
    largest = sorted(allocations.items(), key=lambda entry: entry[1], reverse=True)[:32]
    return {"committed_region_bytes": totals,
            "largest_committed_allocations": [{"allocation_base": hex(key[0]), "kind": key[1], "bytes": size}
                                               for key, size in largest]}


class WindowsProcessMetrics:
    def __init__(self, process):
        if os.name != "nt":
            raise RuntimeError("This metrics implementation requires Windows; do not fabricate other-platform samples")
        self.process = process
        self.psapi = ctypes.WinDLL("psapi", use_last_error=True)
        self.kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        self.psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(MemoryCounters), wintypes.DWORD]
        self.kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        self.pdh = ctypes.WinDLL("pdh")
        self.pdh.PdhOpenQueryW.argtypes = [wintypes.LPCWSTR, ctypes.c_size_t, ctypes.POINTER(wintypes.HANDLE)]
        self.pdh.PdhAddEnglishCounterW.argtypes = [wintypes.HANDLE, wintypes.LPCWSTR, ctypes.c_size_t, ctypes.POINTER(wintypes.HANDLE)]
        self.pdh.PdhCollectQueryData.argtypes = [wintypes.HANDLE]
        self.pdh.PdhGetFormattedCounterArrayW.argtypes = [wintypes.HANDLE, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD), ctypes.POINTER(wintypes.DWORD), ctypes.c_void_p]
        self.pdh.PdhCloseQuery.argtypes = [wintypes.HANDLE]
        self.query = wintypes.HANDLE()
        self.counters = {}
        self.gpu_errors = []
        status = self.pdh.PdhOpenQueryW(None, 0, ctypes.byref(self.query))
        if status == 0:
            for label, counter in (("dedicated", "Dedicated Usage"), ("shared", "Shared Usage")):
                handle = wintypes.HANDLE()
                path = f"\\GPU Process Memory(pid_{process.pid}_*)\\{counter}"
                status = self.pdh.PdhAddEnglishCounterW(self.query, path, 0, ctypes.byref(handle))
                if status == 0:
                    self.counters[label] = handle
                else:
                    self.gpu_errors.append({"operation": "add " + label, "status": hex(status & 0xffffffff)})
        else:
            self.gpu_errors.append({"operation": "open query", "status": hex(status & 0xffffffff)})

    def sample(self):
        memory = MemoryCounters()
        memory.cb = ctypes.sizeof(memory)
        handle = wintypes.HANDLE(int(self.process._handle))
        if not self.psapi.GetProcessMemoryInfo(handle, ctypes.byref(memory), memory.cb):
            return None
        times = [wintypes.FILETIME() for _ in range(4)]
        if not self.kernel.GetProcessTimes(handle, *(ctypes.byref(t) for t in times)):
            return None
        cpu = sum((t.dwHighDateTime << 32) + t.dwLowDateTime for t in times[2:]) / 10000000
        result = {"monotonic_seconds": time.monotonic(), "working_set_bytes": memory.WorkingSetSize,
                  "peak_working_set_bytes": memory.PeakWorkingSetSize, "private_bytes": memory.PrivateUsage,
                  "peak_commit_bytes": memory.PeakPagefileUsage,
                  "cpu_seconds": cpu, "gpu_dedicated_bytes": None, "gpu_shared_bytes": None}
        if self.counters and self.pdh.PdhCollectQueryData(self.query) == 0:
            for name, counter in self.counters.items():
                size, count = wintypes.DWORD(), wintypes.DWORD()
                self.pdh.PdhGetFormattedCounterArrayW(counter, 0x400, ctypes.byref(size), ctypes.byref(count), None)
                if size.value:
                    data = ctypes.create_string_buffer(size.value)
                    status = self.pdh.PdhGetFormattedCounterArrayW(counter, 0x400, ctypes.byref(size), ctypes.byref(count), data)
                    if status == 0:
                        items = ctypes.cast(data, ctypes.POINTER(CounterItem))
                        values = [items[i].FmtValue.value.largeValue for i in range(count.value)
                                  if items[i].FmtValue.CStatus in (0, 1)
                                  and items[i].szName.startswith(f"pid_{self.process.pid}_")]
                        if values:
                            result[f"gpu_{name}_bytes"] = sum(values)
        return result

    def virtual_memory_snapshot(self):
        """Query only this runner-owned PID's page metadata; never read page contents."""
        if ctypes.sizeof(ctypes.c_void_p) != 8:
            raise RuntimeError("Virtual-memory diagnostics require a 64-bit Python host")
        query = self.kernel.VirtualQueryEx
        query.argtypes = [wintypes.HANDLE, ctypes.c_void_p, ctypes.POINTER(MemoryRegion), ctypes.c_size_t]
        query.restype = ctypes.c_size_t
        assert ctypes.sizeof(MemoryRegion) == 48
        address, regions, complete, error = 0, [], False, None
        start = time.monotonic()
        before = MemoryCounters()
        before.cb = ctypes.sizeof(before)
        before_ok = self.psapi.GetProcessMemoryInfo(wintypes.HANDLE(int(self.process._handle)), ctypes.byref(before), before.cb)
        while len(regions) < 200000:
            region = MemoryRegion()
            count = query(wintypes.HANDLE(int(self.process._handle)), address, ctypes.byref(region), ctypes.sizeof(region))
            if not count:
                error = ctypes.get_last_error()
                complete = error == 87  # ERROR_INVALID_PARAMETER at the end of the address space.
                break
            if count != ctypes.sizeof(region) or (region.BaseAddress or 0) + region.RegionSize <= address:
                error = "Invalid/non-advancing VirtualQueryEx result"
                break
            regions.append({"allocation_base": region.AllocationBase or 0, "size": region.RegionSize,
                            "state": region.State, "type": region.Type})
            address = (region.BaseAddress or 0) + region.RegionSize
        result = summarize_regions(regions)
        after = MemoryCounters()
        after.cb = ctypes.sizeof(after)
        after_ok = self.psapi.GetProcessMemoryInfo(wintypes.HANDLE(int(self.process._handle)), ctypes.byref(after), after.cb)
        result.update(monotonic_seconds=time.monotonic(), scan_seconds=time.monotonic()-start,
                      region_count=len(regions), complete=complete, terminal_error=error,
                      private_usage_before_bytes=before.PrivateUsage if before_ok else None,
                      private_usage_after_bytes=after.PrivateUsage if after_ok else None)
        return result

    def close(self):
        if self.query:
            self.pdh.PdhCloseQuery(self.query)


def environment_info():
    result = {"os": platform.platform(), "architecture": platform.machine(), "logical_processors": os.cpu_count()}
    if os.name == "nt":
        import winreg
        with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"HARDWARE\DESCRIPTION\System\CentralProcessor\0") as key:
            result["cpu"] = winreg.QueryValueEx(key, "ProcessorNameString")[0].strip()
        class MemoryStatus(ctypes.Structure):
            _fields_ = [("length", wintypes.DWORD), ("load", wintypes.DWORD)] + [
                (n, ctypes.c_ulonglong) for n in ("physical", "available", "pagefile", "available_pagefile", "virtual", "available_virtual", "extended")]
        status = MemoryStatus()
        status.length = ctypes.sizeof(status)
        if ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status)):
            result["physical_memory_bytes"] = status.physical
    return result


def summarize(samples, phases):
    """Use in-phase samples only; intervals never straddle loading/idle transitions."""
    rows = []
    for index, phase in enumerate(phases[:-1]):
        end = phases[index + 1]["elapsed_seconds"]
        selected = [s for s in samples if phase["elapsed_seconds"] <= s["elapsed_seconds"] < end]
        row = {"phase": phase["phase"], "cycle": phase.get("cycle"), "sample_count": len(selected),
               "duration_seconds": end - phase["elapsed_seconds"]}
        for key in ("working_set_bytes", "private_bytes", "gpu_dedicated_bytes", "gpu_shared_bytes"):
            values = [s[key] for s in selected if s[key] is not None]
            row[key + "_max"] = max(values) if values else None
            row[key + "_last"] = values[-1] if values else None
        if len(selected) >= 2:
            first, last = selected[0], selected[-1]
            duration = last["elapsed_seconds"] - first["elapsed_seconds"]
            row["sample_span_seconds"] = duration
            row["cpu_one_logical_core_percent"] = (last["cpu_seconds"] - first["cpu_seconds"]) / duration * 100
        rows.append(row)
    return rows


def cycle_statistics(samples, phases):
    """Sampled CPU work for the entire repeated cycle interval, including RoundEnd."""
    begin = next((p for p in phases if p["phase"] == "Cycle"), None)
    end = next((p for p in phases if p["phase"] == "ClosedSettle"), None)
    if begin is None or end is None:
        return None
    selected = [s for s in samples if begin["elapsed_seconds"] <= s["elapsed_seconds"] < end["elapsed_seconds"]]
    if len(selected) < 2:
        return None
    span = selected[-1]["elapsed_seconds"] - selected[0]["elapsed_seconds"]
    cpu = selected[-1]["cpu_seconds"] - selected[0]["cpu_seconds"]
    documents = end["cycle"] - begin["cycle"]
    if span <= 0 or cpu < 0 or documents <= 0:
        return None
    return {"phase_wall_seconds": end["elapsed_seconds"]-begin["elapsed_seconds"],
            "sample_span_seconds": span, "sampled_cpu_seconds": cpu,
            "cpu_one_logical_core_percent": cpu/span*100, "documents": documents,
            "sampled_cpu_ms_per_document": cpu*1000/documents,
            "ui_frames": end["ui_frames"]-begin["ui_frames"], "sample_count": len(selected)}
