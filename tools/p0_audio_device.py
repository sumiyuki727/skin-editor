"""Windows-only WASAPI silent render/clock diagnostic; no capture or product backend."""
import argparse
import ctypes as c
import hashlib
import json
import math
import os
from pathlib import Path
import time
import uuid

from check_p0 import ROOT


def analyze(samples):
    phases = {name: [s for s in samples if s["phase"] == name] for name in ("playing", "paused", "resumed")}
    if any(len(rows) < 10 for rows in phases.values()):
        raise RuntimeError("Insufficient device clock samples")
    if [s["phase"] for s in samples] != [p for name in ("playing", "paused", "resumed") for p in [name] * len(phases[name])]:
        raise RuntimeError("Clock phases must be contiguous and ordered")
    if (any(not math.isfinite(s["position_ms"]) or not math.isfinite(s["wall_seconds"]) for s in samples)
            or any(b["wall_seconds"] < a["wall_seconds"] or b["position_ms"] < a["position_ms"] for a, b in zip(samples, samples[1:]))):
        raise RuntimeError("Nonfinite or backwards device clock")
    playing, paused, resumed = (phases[n] for n in ("playing", "paused", "resumed"))
    spread = max(s["position_ms"] for s in paused) - min(s["position_ms"] for s in paused)
    gap = resumed[0]["position_ms"] - paused[-1]["position_ms"]
    wall_gap_ms = (resumed[0]["wall_seconds"] - paused[-1]["wall_seconds"]) * 1000
    before = playing[-1]["position_ms"] - playing[0]["position_ms"]
    after = resumed[-1]["position_ms"] - resumed[0]["position_ms"]
    active_wall = (resumed[-1]["wall_seconds"] - resumed[0]["wall_seconds"]) * 1000
    checks = {"advances_before_pause": before >= 200, "paused_clock_spread_le_2_ms": spread <= 2,
              "position_readings_accurate": all(s.get("position_hresult", 0) == 0 for s in samples),
              "paused_observation_ge_300_ms": (paused[-1]["wall_seconds"] - paused[0]["wall_seconds"]) >= .3,
              "advances_after_resume": after >= 200, "resume_has_no_pause_catchup": gap <= wall_gap_ms + 50,
              "resumed_advance_le_active_wall_plus_50_ms": after <= active_wall + 50}
    return {"status": "passed" if all(checks.values()) else "failed", "checks": checks,
            "paused_clock_spread_ms": spread, "before_advance_ms": before, "after_advance_ms": after,
            "resume_first_gap_ms": gap, "resume_first_wall_gap_ms": wall_gap_ms}


class Guid(c.Structure):
    _fields_ = [("bytes", c.c_ubyte * 16)]

    def __init__(self, value):
        super().__init__((c.c_ubyte * 16).from_buffer_copy(uuid.UUID(value).bytes_le))


class WaveFormat(c.Structure):
    _pack_ = 1
    _fields_ = [("tag", c.c_uint16), ("channels", c.c_uint16), ("rate", c.c_uint32),
                ("bytes_per_second", c.c_uint32), ("block_align", c.c_uint16),
                ("bits", c.c_uint16), ("extra_bytes", c.c_uint16)]


def checked(hr, operation):
    if hr < 0:
        raise RuntimeError(f"{operation}: HRESULT 0x{hr & 0xffffffff:08x}")
    return hr


def invoke(pointer, index, args, *values):
    table = c.cast(pointer, c.POINTER(c.POINTER(c.c_void_p))).contents
    method = c.WINFUNCTYPE(c.c_int32, c.c_void_p, *args)(table[index])
    return checked(method(pointer, *values), "COM method " + str(index))


def probe():
    if os.name != "nt" or c.sizeof(c.c_void_p) != 8:
        raise RuntimeError("Requires 64-bit Windows Python; other platforms untested")
    ole = c.OleDLL("ole32")
    ole.CoInitializeEx.argtypes = [c.c_void_p, c.c_uint32]
    ole.CoInitializeEx.restype = c.c_int32
    ole.CoCreateInstance.argtypes = [c.POINTER(Guid), c.c_void_p, c.c_uint32, c.POINTER(Guid), c.POINTER(c.c_void_p)]
    ole.CoCreateInstance.restype = c.c_int32
    ole.CoTaskMemFree.argtypes = [c.c_void_p]
    ole.CoTaskMemFree.restype = None
    ole.CoUninitialize.argtypes = []
    ole.CoUninitialize.restype = None
    checked(ole.CoInitializeEx(None, 0), "CoInitializeEx")
    owned = []
    mix = c.c_void_p()
    client = None
    running = False
    cleanup = []
    def obtain(parent, index, args, *values):
        out = c.c_void_p()
        invoke(parent, index, [*args, c.POINTER(c.c_void_p)], *values, c.byref(out))
        owned.append(out)
        return out
    try:
        enumerator = c.c_void_p()
        checked(ole.CoCreateInstance(c.byref(Guid("bcde0395-e52f-467c-8e3d-c4579291692e")), None, 1,
                                     c.byref(Guid("a95664d2-9614-4f35-a746-de8db63617e6")), c.byref(enumerator)), "Create endpoint enumerator")
        owned.append(enumerator)
        device = obtain(enumerator, 4, [c.c_int32, c.c_int32], 0, 1)  # render, multimedia role
        iid_client = Guid("1cb9ad4c-dbfa-4c32-b178-c2f568a703b2")
        client = obtain(device, 3, [c.POINTER(Guid), c.c_uint32, c.c_void_p], c.byref(iid_client), 1, None)
        invoke(client, 8, [c.POINTER(c.c_void_p)], c.byref(mix))
        fmt = c.cast(mix, c.POINTER(WaveFormat)).contents
        metadata = {"tag": fmt.tag, "channels": fmt.channels, "sample_rate": fmt.rate,
                    "block_align": fmt.block_align, "bits": fmt.bits, "extra_bytes": fmt.extra_bytes}
        if not 1 <= fmt.channels <= 32 or not 8000 <= fmt.rate <= 384000 or not 1 <= fmt.block_align <= 256:
            raise RuntimeError("Unexpected endpoint mix format")
        invoke(client, 3, [c.c_int32, c.c_uint32, c.c_int64, c.c_int64, c.c_void_p, c.c_void_p],
               0, 0, 1000000, 0, mix, None)  # shared mode, requested 100 ms, no global settings
        buffer_frames = c.c_uint32()
        latency = c.c_int64()
        invoke(client, 4, [c.POINTER(c.c_uint32)], c.byref(buffer_frames))
        invoke(client, 5, [c.POINTER(c.c_int64)], c.byref(latency))
        render = obtain(client, 14, [c.POINTER(Guid)], c.byref(Guid("f294acfc-3146-4483-a7bf-addca7c260e2")))
        clock = obtain(client, 14, [c.POINTER(Guid)], c.byref(Guid("cd63314f-3fba-4a1b-812c-ef96358728e7")))
        frequency = c.c_uint64()
        invoke(clock, 3, [c.POINTER(c.c_uint64)], c.byref(frequency))
        if not frequency.value:
            raise RuntimeError("Zero audio clock frequency")
        samples = []
        submitted = 0
        def feed():
            nonlocal submitted
            padding = c.c_uint32()
            invoke(client, 6, [c.POINTER(c.c_uint32)], c.byref(padding))
            count = buffer_frames.value - padding.value
            if count < 0:
                raise RuntimeError("Endpoint padding exceeds buffer size")
            if count:
                data = c.c_void_p()
                invoke(render, 3, [c.c_uint32, c.POINTER(c.c_void_p)], count, c.byref(data))
                invoke(render, 4, [c.c_uint32, c.c_uint32], count, 2)  # SILENT; no audio or microphone data read
                submitted += count
        def observe(phase, duration, fill):
            start = time.perf_counter()
            while True:
                if fill:
                    feed()
                position, qpc = c.c_uint64(), c.c_uint64()
                hr = invoke(clock, 4, [c.POINTER(c.c_uint64), c.POINTER(c.c_uint64)], c.byref(position), c.byref(qpc))
                samples.append({"phase": phase, "wall_seconds": time.perf_counter(), "position_units": position.value,
                                "position_ms": position.value / frequency.value * 1000, "qpc_100ns": qpc.value,
                                "position_hresult": hr})
                if time.perf_counter() - start >= duration:
                    break
                time.sleep(.01)
        feed()
        invoke(client, 10, [])
        running = True
        observe("playing", .45, True)
        invoke(client, 11, [])
        running = False
        observe("paused", .35, False)
        invoke(client, 10, [])
        running = True
        observe("resumed", .45, True)
        invoke(client, 11, [])
        running = False
        return {"schema_version": 1, **analyze(samples), "api": "WASAPI shared render / IAudioClock",
                "default_role": "multimedia render", "format": metadata, "buffer_frames": buffer_frames.value,
                "reported_stream_latency_ms": latency.value / 10000, "clock_frequency": frequency.value,
                "silent_frames_submitted": submitted, "samples": samples, "cleanup": cleanup,
                "limits": ["Silent device stream, not audible output or microphone recording", "Reported stream latency is not measured acoustic/input-to-sound latency",
                           "100 ms requested shared buffer, polling diagnostic, not a selected low-latency backend", "Separate from GUI/game clock; no device-switch, underrun or cross-platform validation"]}
    finally:
        if client:
            try:
                if running:
                    invoke(client, 11, [])
                invoke(client, 12, [])
                cleanup.append("stream stopped and reset")
            except Exception as error:
                cleanup.append(str(error))
        for pointer in reversed(owned):
            invoke(pointer, 2, [])
        if mix:
            ole.CoTaskMemFree(mix)
        ole.CoUninitialize()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repetitions", type=int, choices=range(1, 6), default=3)
    args = parser.parse_args()
    output = ROOT / "artifacts/p0" / ("audio-device-" + uuid.uuid4().hex[:8])
    output.mkdir()
    source = Path(__file__).resolve()
    (output / source.name).write_bytes(source.read_bytes())
    runs = []
    for _ in range(args.repetitions):
        try:
            runs.append(probe())
        except Exception as error:
            runs.append({"status": "failed", "error": str(error)})
    result = {"schema_version": 1, "status": "passed" if all(r["status"] == "passed" for r in runs) else "failed",
              "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(), "runs": runs}
    (output / "result.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print("Audio device proof: " + str(output))
    if result["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
