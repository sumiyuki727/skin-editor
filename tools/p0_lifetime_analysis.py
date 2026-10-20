"""Align owned-process samples to observed release events; never invent missing samples."""
from bisect import bisect_left


def release_samples(samples, released_at, offsets=(0, .1, .5, 1, 3, 10), maximum_lag=.25):
    if maximum_lag < 0 or any(offset < 0 for offset in offsets):
        raise ValueError("Nonnegative timing bounds required")
    ordered = sorted(samples, key=lambda row: row["elapsed_seconds"])
    times = [row["elapsed_seconds"] for row in ordered]
    result = []
    for offset in offsets:
        index = bisect_left(times, released_at + offset)
        found = index < len(times) and times[index] <= released_at + offset + maximum_lag
        result.append(dict(requested_offset_seconds=offset,
                           observed_offset_seconds=times[index] - released_at if found else None,
                           sample=ordered[index] if found else None))
    return result


def upload_receipts(events):
    pending, latencies, maximum = {}, [], 0
    for row in events:
        if row["kind"] == "UploadQueued":
            ticket = row["ticket"]
            start, size = pending.get(ticket, (row["elapsed_seconds"], 0))
            pending[ticket] = (start, size + row["bytes"])
            maximum = max(maximum, sum(value[1] for value in pending.values()))
        elif row["kind"] == "UploadComplete":
            detail = row["detail"]
            ticket = detail["ticket"]
            if ticket not in pending:
                raise ValueError("Duplicate or unknown GPU completion receipt")
            start, size = pending.pop(ticket)
            if detail["bytes"] != size or detail["callback_elapsed_seconds"] < start:
                raise ValueError("Upload receipt bytes/time mismatch")
            latencies.append((detail["callback_elapsed_seconds"] - start) * 1000)
    return dict(peak_payload_bytes=maximum, remaining_payload_bytes=sum(v[1] for v in pending.values()),
                queued_to_callback_wall_ms=latencies,
                note="Callback time includes frame submission, GPU work and poll scheduling; not GPU execution duration")
