"""Opt-in native background composition switching; Windows process measurements."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import uuid

from check_p0 import ROOT
from check_p0_edit_composition import fixture_check
from p0_process_metrics import WindowsProcessMetrics, environment_info
from p0_frame_attribution import attribute

CHECKS = {"stale_gpu_discarded", "failed_keeps_old", "cancel_keeps_old", "twenty_switches", "old_view_retained",
          "all_views_paused", "closed_without_active_or_pending", "bounded_source_upload", "pixel_proofs", "no_errors"}


def validate(result, pixels, observe):
    checks = result.get("checks", {})
    if (result.get("schema_version") != 1 or result.get("status") != "passed" or result.get("errors")
            or set(checks) != CHECKS or not all(v is True for v in checks.values())
            or result.get("pixel_diagnostic") is not pixels or result.get("long_observation") is not observe
            or result.get("simulation_ms") != 0 or result.get("accepted_game_inputs") != 0
            or result.get("accepted_documents") != 22 or result.get("stale_gpu", 0) < 1
            or result.get("retained_frames", 0) < 1 or result.get("system_capture_used") is not False):
        raise RuntimeError("Missing switching, pause or generation evidence")
    for key, count in [("switch_ms", 20), ("request_cpu_ms", 25)]:
        values = result.get(key, [])
        if len(values) != count or any(not math.isfinite(v) or v < 0 for v in values):
            raise RuntimeError("Missing/nonfinite request or completion timings")
    uploads = result.get("uploads", [])
    if len(uploads) != 23 or any(u["uploaded_bytes"] != 40 * 1024 * 1024 or u["upload_frames"] != 5
                                or not 0 < u["max_source_bytes_per_frame"] <= 8 * 1024 * 1024
                                or not 0 < u["max_sources_per_frame"] <= 2 for u in uploads):
        raise RuntimeError("Source uploads were incomplete or exceeded frame bounds")
    proofs = result.get("proofs", [])
    if pixels:
        if (len(proofs) != 4 or [p["document"] for p in proofs] != [0, 2, 2, 9]
                or [p["stage"] for p in proofs] != [0, 1, 3, 2]
                or any(p["pixels"] != 1024 * 1024 or p["max_channel_error"] > 2
                       or p["pixels_outside_tolerance"] != 0 or p["display_max_channel_error"] > 2
                       or p["display_pixels_outside_tolerance"] != 0 for p in proofs)):
            raise RuntimeError("Missing/different document, source or display pixel proof")
    elif proofs:
        raise RuntimeError("Performance run unexpectedly included pixel diagnostics")


def statistics(values):
    if not values:
        return None
    values = sorted(values)
    return {"count": len(values), "p50": values[(len(values)-1)//2],
            "p95": values[math.ceil(len(values)*.95)-1], "max": values[-1]}


def phase_summary(samples):
    rows = []
    # Only samples labelled within the same native phase; no startup/transition delta.
    for phase in ("Loading", "Cycle", "IdleSettle", "Idle", "ClosedSettle", "Closed"):
        selected = [s for s in samples if s.get("phase") == phase]
        row = {"phase": phase, "sample_count": len(selected)}
        if len(selected) > 1:
            first, last = selected[0], selected[-1]
            span = last["monotonic_seconds"] - first["monotonic_seconds"]
            row.update(sample_span_seconds=span, cpu_one_logical_core_percent=(last["cpu_seconds"]-first["cpu_seconds"])/span*100)
        for key in ("working_set_bytes", "private_bytes", "gpu_dedicated_bytes", "gpu_shared_bytes"):
            values = [s[key] for s in selected if s[key] is not None]
            row[key+"_max"] = max(values) if values else None
            row[key+"_last"] = values[-1] if values else None
        rows.append(row)
    return rows


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", required=True, type=Path)
    parser.add_argument("--smoke", action="store_true", help="Only pixel diagnostic; no idle/close budget claim")
    parser.add_argument("--profile", action="store_true", help="Two short frame-attribution diagnostics; no budgets")
    parser.add_argument("--warmup", action="store_true", help="Prepare/reuse compute pipelines on a one-shot worker")
    args = parser.parse_args()
    if args.smoke and args.profile:
        raise RuntimeError("Choose pixel smoke or frame diagnostics")
    if os.name != "nt":
        raise RuntimeError("This measurement runner is Windows-only; Linux native validation remains separate")
    fixture = fixture_check(args.fixture)
    subprocess.run([sys.executable, str(ROOT / "tools/check_p0_gui.py")], check=True)
    exe = ROOT / "experiments/p0-egui/target/release/switch-proof.exe"
    paths = [ROOT / "experiments/p0-egui/Cargo.toml", ROOT / "experiments/p0-egui/Cargo.lock",
             *sorted((ROOT / "experiments/p0-egui/src").rglob("*.rs")),
             *sorted((ROOT / "experiments/p0-egui/src").glob("*.wgsl")), Path(__file__).resolve(),
             ROOT / "tools/check_p0_gui.py", ROOT / "tools/p0_process_metrics.py", ROOT / "tools/p0_frame_attribution.py"]
    executable_sha = digest(exe)
    source_identity = {p.relative_to(ROOT).as_posix(): digest(p) for p in paths}
    for pixels, observe in ([(False, False)] * 2 if args.profile else [(True, False)] + ([] if args.smoke else [(False, True)] * 2)):
        output = ROOT / "artifacts/p0" / ("composition-switch-"+uuid.uuid4().hex[:8])
        command = [str(exe), str(output), str(fixture)] + (["--profile"] if args.profile else ["--pixels" if pixels else "--observe"])
        if args.warmup:
            command.append("--warmup")
        samples = []
        # Log directly to disk to avoid a blocked stdout pipe during long observations.
        log_path = ROOT / "artifacts/p0" / (output.name+".log")
        with log_path.open("wb") as log, subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT) as process:
            metrics = WindowsProcessMetrics(process)
            try:
                deadline = time.monotonic()+210
                while process.poll() is None:
                    if time.monotonic() > deadline:
                        process.kill(); process.wait()
                        raise RuntimeError("Switch proof timeout")
                    sample = metrics.sample()
                    if sample:
                        try:
                            sample["phase"] = json.loads((output / "phase.json").read_text())["phase"]
                        except (OSError, ValueError):
                            sample["phase"] = "Unknown"
                        samples.append(sample)
                    time.sleep(.1)
            finally:
                metrics.close()
        if not output.is_dir():
            raise RuntimeError("No native output; see "+str(log_path))
        shutil.copyfile(log_path, output / "run.log")
        if process.returncode:
            raise RuntimeError("Native switching failed: "+str(output))
        result = json.loads((output / "result.json").read_text())
        validate(result, pixels, observe)
        warmup_ms = result.get("pipeline_warmup_ms")
        if args.warmup != (warmup_ms is not None) or (warmup_ms is not None and (not math.isfinite(warmup_ms) or warmup_ms <= 0)):
            raise RuntimeError("Pipeline preparation strategy differs from request")
        if pixels:
            for kind in ("cpu", "gpu", "display"):
                if digest(output / f"composition-1-{kind}.png") != digest(output / f"composition-3-{kind}.png"):
                    raise RuntimeError("Cancel/error changed previous document pixels")
            if digest(output / "composition-0-cpu.png") == digest(output / "composition-1-cpu.png"):
                raise RuntimeError("Different documents produced unchanged proof")
        phases = phase_summary(samples)
        idle = next(p for p in phases if p["phase"] == "Idle")
        closed = next(p for p in phases if p["phase"] == "Closed")
        budgets = None if not observe else {
            "idle_60_seconds": idle.get("sample_span_seconds", 0) >= 59,
            "closed_60_seconds": closed.get("sample_span_seconds", 0) >= 59,
            "idle_cpu_le_2_percent_one_core": idle.get("cpu_one_logical_core_percent", 100) <= 2,
            "closed_cpu_le_2_percent_one_core": closed.get("cpu_one_logical_core_percent", 100) <= 2,
            "idle_no_continuous_frames": result["idle_ui_frames"] <= 2,
            "closed_no_continuous_frames": result["closed_ui_frames"] <= 2,
            "loading_eframe_cpu_p95_le_16_7_ms": statistics(result["eframe_cpu_frame_ms_loading"])["p95"] <= 16.7}
        if digest(exe) != executable_sha or any(digest(p) != source_identity[p.relative_to(ROOT).as_posix()] for p in paths):
            raise RuntimeError("Measured executable or source changed during the run; preserve as unverified")
        evidence = {"schema_version": 1, "executable_sha256": executable_sha, "executable_bytes": exe.stat().st_size,
                    "source_sha256": source_identity,
                    "fixture": fixture.relative_to(ROOT).as_posix(), "inventory_sha256": digest(fixture / "inventory.json"),
                    "output_sha256": {p.name: digest(p) for p in sorted(output.glob("*.png"))},
                    "samples": samples, "phase_summary": phases, "budgets": budgets,
                    "warmup": args.warmup, "pipeline_warmup_ms": result.get("pipeline_warmup_ms"), "first_document_wall_ms": result.get("first_document_wall_ms"),
                    "switch_ms": statistics(result["switch_ms"]), "request_cpu_ms": statistics(result["request_cpu_ms"]),
                    "loading_eframe_cpu_ms": statistics(result["eframe_cpu_frame_ms_loading"]), "environment": environment_info(),
                    "note": "Frame budget counts source bytes only, not allocations or driver costs. Phase labels sampled from native marker, transition samples may be excluded. 59 s sample span is required within native 60 s idle/closed windows. Pixel run is diagnostic, not a performance budget."}
        evidence["frame_trace_sha256"] = digest(output / "frame-trace.json") if args.profile else None
        evidence["frame_attribution"] = attribute(json.loads((output / "frame-trace.json").read_text()))[:8] if args.profile else None
        for p in paths:
            dest = output / "sources" / p.relative_to(ROOT)
            dest.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(p, dest)
        rejected = ROOT / ("switch-escape-"+uuid.uuid4().hex[:8])
        response = subprocess.run([str(exe), str(rejected), str(fixture)], cwd=ROOT, capture_output=True)
        if response.returncode == 0 or rejected.exists():
            raise RuntimeError("Output boundary failed")
        response = subprocess.run(command, cwd=ROOT, capture_output=True)
        if response.returncode == 0 or any(digest(output / p) != h for p, h in evidence["output_sha256"].items()):
            raise RuntimeError("Existing output was overwritten")
        (output / "evidence.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
        print("Background composition switch: "+str(output), flush=True)


if __name__ == "__main__":
    main()
