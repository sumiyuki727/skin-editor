"""Module metadata for a Popen-owned Windows child; no window or page-content reads."""
import ctypes
from ctypes import wintypes
import os
import time


def graphic_modules(snapshot):
    names = {item["name"].lower() for item in snapshot.get("modules", [])}
    known = ("graphics-hook64.dll", "nvspcap64.dll", "nvcameraallowlisting64.dll",
             "nvoglv64.dll", "nvgpucomp64.dll", "nvwgf2umx.dll")
    return {name: True if name in names else (False if snapshot["complete"] else None) for name in known}


def module_snapshot(process):
    if os.name != "nt" or ctypes.sizeof(ctypes.c_void_p) != 8:
        raise RuntimeError("Module diagnostics require 64-bit Windows Python")
    start = time.monotonic()
    api = ctypes.WinDLL("psapi", use_last_error=True)
    api.EnumProcessModulesEx.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.HMODULE),
                                       wintypes.DWORD, ctypes.POINTER(wintypes.DWORD), wintypes.DWORD]
    api.EnumProcessModulesEx.restype = wintypes.BOOL

    class Info(ctypes.Structure):
        _fields_ = [("base", ctypes.c_void_p), ("size", wintypes.DWORD), ("entry", ctypes.c_void_p)]

    api.GetModuleInformation.argtypes = [wintypes.HANDLE, wintypes.HMODULE, ctypes.POINTER(Info), wintypes.DWORD]
    api.GetModuleInformation.restype = wintypes.BOOL
    api.GetModuleBaseNameW.argtypes = [wintypes.HANDLE, wintypes.HMODULE, wintypes.LPWSTR, wintypes.DWORD]
    api.GetModuleBaseNameW.restype = wintypes.DWORD
    handle = wintypes.HANDLE(int(process._handle))
    modules, errors, complete, capacity = [], [], False, 256
    for _ in range(3):
        buffer = (wintypes.HMODULE * capacity)()
        needed = wintypes.DWORD()
        if not api.EnumProcessModulesEx(handle, buffer, ctypes.sizeof(buffer), ctypes.byref(needed), 3):
            errors.append({"operation": "EnumProcessModulesEx", "error": ctypes.get_last_error()})
            break
        if needed.value > ctypes.sizeof(buffer):
            capacity = (needed.value // ctypes.sizeof(wintypes.HMODULE)) + 32
            if capacity > 4096:
                errors.append({"operation": "module_bound", "needed": capacity})
                break
            continue
        complete = True
        for module in buffer[:needed.value // ctypes.sizeof(wintypes.HMODULE)]:
            info, name = Info(), ctypes.create_unicode_buffer(512)
            length = api.GetModuleBaseNameW(handle, module, name, len(name))
            if not length or length >= len(name) - 1 or not api.GetModuleInformation(handle, module, ctypes.byref(info), ctypes.sizeof(info)):
                complete = False
                errors.append({"operation": "module_metadata", "error": ctypes.get_last_error()})
                continue
            modules.append({"name": name.value, "base": info.base, "size": info.size})
        break
    if not modules:
        complete = False
    result = {"schema_version": 1, "pid": process.pid, "complete": complete, "errors": errors,
              "modules": modules, "scan_seconds": time.monotonic() - start,
              "note": "Non-atomic PSAPI module metadata only; missing modules are unknown after any failure"}
    result["graphic_modules"] = graphic_modules(result)
    return result
