"""Join explicitly consecutive diagnostic frames to eframe's previous-frame CPU report."""
import math


def attribute(trace):
    frames = trace.get("frames", [])
    steps = trace.get("switch_steps", [])
    if not frames or any(b["frame"] != a["frame"] + 1 for a, b in zip(frames, frames[1:])):
        raise RuntimeError("Repeated, missing or unordered frame IDs are ambiguous")
    by_id = {}
    for step in steps:
        if step["frame"] in by_id:
            raise RuntimeError("Repeated switch step cannot be joined unambiguously")
        by_id[step["frame"]] = step
    output = []
    for previous, reporter in zip(frames, frames[1:]):
        cpu = reporter["previous_eframe_cpu_ms"]
        values = [cpu, previous["logic_ms"], previous["ui_ms"]]
        if any(v is None or not math.isfinite(v) or v < 0 for v in values):
            raise RuntimeError("Missing or invalid CPU span")
        row = {"frame": previous["frame"], "reporter_frame": reporter["frame"], "eframe_cpu_ms": cpu,
               "logic_ms": previous["logic_ms"], "ui_ms": previous["ui_ms"],
               "outside_app_spans_ms": cpu - previous["logic_ms"] - previous["ui_ms"],
               "switch": by_id.get(previous["frame"])}
        output.append(row)
    return sorted(output, key=lambda row: row["eframe_cpu_ms"], reverse=True)
