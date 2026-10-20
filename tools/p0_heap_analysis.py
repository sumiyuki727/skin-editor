"""Summarize a PID-scoped xperf heap CSV, without claiming full process attribution."""
import csv
from collections import Counter


SYSTEM = {"ntdll.dll", "kernel32.dll", "kernelbase.dll", "ucrtbase.dll", "msvcrt.dll",
          "vcruntime140.dll", "vcruntime140_1.dll"}


def summarize_heap_lines(lines, modules=(), executable="normal.exe"):
    """The caller must establish that the input contains only its owned PID's heap events.

    Stack rows follow their trigger, including xperf's unshifted timestamps. An
    allocation seen before tracing began is unknown, never invented as a leak.
    Heap segment counters are kept separate from allocation requests/PrivateUsage.
    """
    blocks, heaps = {}, {}
    origins, counts = Counter(), Counter()
    live = peak = unmatched = collisions = failed = coalesced = 0
    pending, samples, next_sample = None, [], 0

    def origin_for(address):
        for module in modules:
            if module["base"] <= address < module["base"] + module["size"]:
                name = module["name"].lower()
                if name in SYSTEM:
                    return None
                if name == executable.lower():
                    return "experiment-executable"
                if name in {"nvoglv64.dll", "nvwgf2umx.dll", "nvgpucomp64.dll", "nvapi64.dll"}:
                    return "nvidia-driver"
                return "native:" + name
        return None

    def remove(key):
        nonlocal live, unmatched
        old = blocks.pop(key, None)
        if old is None:
            unmatched += 1
        else:
            size, origin = old
            live -= size
            origins[origin] -= size

    def flush():
        nonlocal pending, live, peak, collisions, failed, next_sample, coalesced
        if pending is None:
            return
        kind, fields, origin = pending
        pending = None
        heap = fields[4]
        address = int(fields[5], 16)
        if not address:
            failed += 1
            return  # A failed realloc preserves the previous block.
        key = (heap, address)
        size = int(fields[7 if kind == "HeapRealloc" else 6], 16)
        if kind == "HeapRealloc":
            old_key = (heap, int(fields[6], 16))
            # A moving Windows heap realloc can emit its internal Alloc/Free
            # before the completion event. Count the block once and retain its
            # allocation stack when the completion has no resolved stack.
            if old_key != key and key in blocks and blocks[key][0] == size:
                coalesced += 1
                if old_key in blocks:
                    remove(old_key)
                previous_origin = blocks[key][1]
                if origin and origin != previous_origin:
                    origins[previous_origin] -= size
                    origins[origin] += size
                    blocks[key] = (size, origin)
                return
            remove(old_key)
        if key in blocks:
            collisions += 1
            remove(key)
        origin = origin or "unknown"
        blocks[key] = (size, origin)
        live += size
        origins[origin] += size
        peak = max(peak, live)
        time = int(fields[1])
        if time >= next_sample:
            next_sample = time + 100_000
            samples.append({"trace_seconds": time / 1e6, "observed_live_bytes": live,
                            "observed_live_by_origin": dict(origins),
                            "reported_heap_segments": {key: dict(value) for key, value in heaps.items()}})

    for line in lines:
        stripped = line.lstrip()
        if not stripped.startswith(("Heap", "Stack,")):
            continue
        fields = next(csv.reader([line], skipinitialspace=True))
        kind = fields[0].strip()
        if kind == "Stack":
            if pending and fields[2].strip() == pending[1][3].strip() and pending[2] is None:
                origin = origin_for(int(fields[4], 16))
                if origin:
                    pending = (pending[0], pending[1], origin)
            continue
        try:
            int(fields[1])
        except (ValueError, IndexError):
            continue
        flush()
        counts[kind] += 1
        if kind in ("HeapAlloc", "HeapRealloc"):
            pending = (kind, fields, None)
        elif kind == "HeapFree":
            remove((fields[4], int(fields[5], 16)))
        elif kind in ("HeapExtend", "HeapContract"):
            heaps[fields[4]] = {"committed_bytes": int(fields[8]), "reserved_bytes": int(fields[9]),
                                "allocated_counter_bytes": int(fields[11]), "free_counter_bytes": int(fields[7])}
        elif kind == "HeapDestroy":
            for key in [key for key in blocks if key[0] == fields[4]]:
                remove(key)
            heaps.pop(fields[4], None)
    flush()
    return {"schema_version": 2, "event_counts": dict(counts), "observed_requested_peak_bytes": peak,
            "observed_requested_last_bytes": live, "observed_last_by_origin": dict(origins),
            "unmatched_free_or_realloc": unmatched, "address_collisions": collisions,
            "failed_allocations": failed, "samples": samples,
            "coalesced_moving_realloc_completions": coalesced,
            "limits": ["PID scope must be established by the trace runner, not inferred from Unknown PID labels",
                       "Attach misses earlier allocations; observed live is incomplete",
                       "Module origin is the first known non-system stack frame; executable includes statically linked code",
                       "Heap segment counters exclude unreported heaps and direct virtual allocations; do not subtract them from PrivateUsage",
                       "No symbol download or driver-internal attribution"]}
