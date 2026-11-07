"""Observe only a handle opened for the caller's owned child PID; Windows v1."""
import ctypes
from ctypes import wintypes
import os


class MemoryCounters(ctypes.Structure):
    _fields_ = [('cb', wintypes.DWORD), ('PageFaultCount', wintypes.DWORD)] + [
        (name, ctypes.c_size_t) for name in ('PeakWorkingSetSize', 'WorkingSetSize', 'QuotaPeakPagedPoolUsage',
                                           'QuotaPagedPoolUsage', 'QuotaPeakNonPagedPoolUsage', 'QuotaNonPagedPoolUsage',
                                           'PagefileUsage', 'PeakPagefileUsage', 'PrivateUsage')]


class OwnedProcess:
    def __init__(self, pid):
        if os.name != 'nt':
            raise OSError('Windows process observations unavailable on this platform')
        self.kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        self.psapi = ctypes.WinDLL('psapi', use_last_error=True)
        k = self.kernel
        k.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
        k.OpenProcess.restype = wintypes.HANDLE
        k.CloseHandle.argtypes = [wintypes.HANDLE]
        k.CloseHandle.restype = wintypes.BOOL
        k.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)]*4
        k.GetProcessTimes.restype = wintypes.BOOL
        self.psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(MemoryCounters), wintypes.DWORD]
        self.psapi.GetProcessMemoryInfo.restype = wintypes.BOOL
        self.handle = k.OpenProcess(0x1000 | 0x0010, False, pid)
        if not self.handle:
            raise ctypes.WinError(ctypes.get_last_error())

    def sample(self):
        values = MemoryCounters()
        values.cb = ctypes.sizeof(values)
        times = [wintypes.FILETIME() for _ in range(4)]
        if not self.psapi.GetProcessMemoryInfo(self.handle, ctypes.byref(values), values.cb):
            raise ctypes.WinError(ctypes.get_last_error())
        if not self.kernel.GetProcessTimes(self.handle, *(ctypes.byref(value) for value in times)):
            raise ctypes.WinError(ctypes.get_last_error())
        ticks = [(value.dwHighDateTime << 32 | value.dwLowDateTime)*100 for value in times]
        return dict(user_ns=ticks[3], kernel_ns=ticks[2], working_set_bytes=values.WorkingSetSize,
                    private_commit_bytes=values.PrivateUsage, os_peak_working_set_bytes=values.PeakWorkingSetSize,
                    os_peak_commit_bytes=values.PeakPagefileUsage)

    def close(self):
        if self.handle:
            handle, self.handle = self.handle, None
            if not self.kernel.CloseHandle(handle):
                raise ctypes.WinError(ctypes.get_last_error())
