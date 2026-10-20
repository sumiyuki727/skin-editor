"""Opt-in real silent audio + asynchronous document handoff proof (Windows only)."""
import argparse
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
from check_p0_transport import digest, sources
from p0_process_metrics import WindowsProcessMetrics, environment_info

CHECKS = {"clock_advanced_before_pause", "paused_fresh_samples", "paused_span_zero", "paused_wall_observed",
          "resume_no_catchup", "playing_document_switch", "single_shared_clock", "fresh_input_only",
          "document_draft_cancelled", "single_document_commit", "cancel_keeps_old", "failed_keeps_old",
          "stale_gpu_discarded", "close_during_upload", "final_pause_preserved", "six_pixel_checks",
          "thirty_draft_updates", "no_fault"}


def validate(result):
    checks = result.get("checks", {})
    if (result.get("schema_version") != 1 or result.get("status") != "passed" or result.get("errors")
            or set(checks) != CHECKS or not all(v is True for v in checks.values())
            or result.get("accepted_inputs") != 2 or result.get("edit_count") != 30
            or result.get("nodes_per_document") != [1, 1, 2, 1, 1, 1, 1, 1, 1, 1]
            or result.get("accepted_documents") != 3 or result.get("stale_gpu") != 2
            or result.get("system_capture_used") is not False):
        raise RuntimeError("Incomplete combined clock/input/command/generation proof")
    frozen, resumed, final = (result[k] for k in ("frozen_us", "resume_us", "final_simulation_us"))
    if (frozen < 200000 or not frozen <= resumed <= frozen+50000 or final < resumed+200000
            or final != result["second_frozen_us"] or not math.isfinite(result["paused_wall_ms"])
            or result["paused_wall_ms"] < 300):
        raise RuntimeError("Switching reset the clock or resumed with paused-time catchup")
    rows = result.get("fresh_samples", [])
    if (not rows or any(r["device_us"] < 0 or r["device_us"] != r["simulation_us"] for r in rows)
            or any(b["sequence"] <= a["sequence"] or b["device_us"] < a["device_us"] for a, b in zip(rows, rows[1:]))):
        raise RuntimeError("Missing fresh monotonic common-clock observations")
    for request, position in [(2, frozen), (4, final)]:
        paused = [r for r in rows if r["request"] == request and r["phase"] == "Paused"]
        if len(paused) < 3 or any(r["device_us"] != position for r in paused):
            raise RuntimeError("Pause drift or insufficient device samples during handoff")
        if request == 2 and (result.get("paused_samples") != len(paused) or result.get("paused_span_us") != 0):
            raise RuntimeError("Declared pause observations differ from samples")
    playing_new = [r for r in rows if r["request"] == 3 and r["phase"] == "Playing" and r["document"] == 5]
    if not playing_new:
        raise RuntimeError("No completed document switch while device clock was playing")
    proofs = result.get("proofs", [])
    if (len(proofs) != 6 or [p["stage"] for p in proofs] != [0, 5, 1, 2, 3, 4]
            or [p["document"] for p in proofs] != [0, 0, 2, 2, 2, 5]
            or any(p["pixels"] != 1024*1024 or p["max_channel_error"] > 2 or p["pixels_outside_tolerance"] != 0
                   or p["display_max_channel_error"] > 2 or p["display_pixels_outside_tolerance"] != 0
                   or p["simulation_us"] != (final if p["stage"] == 4 else frozen)
                   or p["accepted_inputs"] != (2 if p["stage"] == 4 else 1) for p in proofs)):
        raise RuntimeError("Wrong document, input count, frozen time or composition/display pixels")
    uploads = result.get("uploads", [])
    if len(uploads) != 5 or any(u["uploaded_bytes"] != 40*1024*1024 or u["upload_frames"] != 5
                               or not 0 < u["max_source_bytes_per_frame"] <= 8*1024*1024
                               or not 0 < u["max_sources_per_frame"] <= 2 for u in uploads):
        raise RuntimeError("Combined source work exceeded bounded handoff coverage")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--repetitions", type=int, choices=range(1, 4), default=3)
    args = parser.parse_args()
    if os.name != "nt":
        raise RuntimeError("Real WASAPI combined proof requires Windows; no Linux audio result is implied")
    fixture = fixture_check(args.fixture)
    subprocess.run([sys.executable, str(ROOT / "tools/check_p0_gui.py"), "--device-audio"], check=True)
    exe = ROOT / "experiments/p0-egui/target/release/switch-audio-proof.exe"
    paths = sources() + [Path(__file__).resolve()]
    identity = {p.relative_to(ROOT).as_posix(): digest(p) for p in paths}
    executable_sha = digest(exe)
    for _ in range(args.repetitions):
        output = ROOT / "artifacts/p0" / ("switch-audio-"+uuid.uuid4().hex[:8])
        command = [str(exe), str(output), str(fixture)]
        samples = []
        log_path = ROOT / "artifacts/p0" / (output.name+".log")
        with log_path.open("wb") as log, subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT) as process:
            metrics = WindowsProcessMetrics(process)
            try:
                deadline = time.monotonic()+120
                while process.poll() is None:
                    if time.monotonic() > deadline:
                        process.kill(); process.wait()
                        raise RuntimeError("Combined proof timeout: "+str(output))
                    sample = metrics.sample()
                    if sample:
                        samples.append(sample)
                    time.sleep(.1)
            finally:
                metrics.close()
        if process.returncode:
            raise RuntimeError("Native combined proof failed; see "+str(log_path))
        shutil.copyfile(log_path, output / "run.log")
        result = json.loads((output / "result.json").read_text())
        validate(result)
        for kind in ("cpu", "gpu", "display"):
            for first, restored in [(0, 5), (2, 3)]:
                if digest(output / f"composition-{first}-{kind}.png") != digest(output / f"composition-{restored}-{kind}.png"):
                    raise RuntimeError("Interrupted draft or failed load changed old document pixels")
            if digest(output / f"composition-1-{kind}.png") == digest(output / f"composition-2-{kind}.png"):
                raise RuntimeError("Committed gesture produced unchanged pixels")
        if digest(exe) != executable_sha or any(digest(p) != identity[p.relative_to(ROOT).as_posix()] for p in paths):
            raise RuntimeError("Measured executable/source changed during run")
        evidence = {"schema_version": 1, "executable_sha256": executable_sha, "executable_bytes": exe.stat().st_size,
                    "source_sha256": identity, "fixture": fixture.relative_to(ROOT).as_posix(),
                    "inventory_sha256": digest(fixture / "inventory.json"), "samples": samples,
                    "output_sha256": {p.name: digest(p) for p in sorted(output.glob("*.png"))},
                    "environment": environment_info(), "note": "Real silent output, one shared transport and background GPU document handoff. Synthetic input, blocking paused pixel proofs; no full budget, acoustic or physical-device recovery claim."}
        for p in paths:
            dest = output / "sources" / p.relative_to(ROOT)
            dest.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(p, dest)
        rejected = ROOT / ("switch-audio-escape-"+uuid.uuid4().hex[:8])
        response = subprocess.run([str(exe), str(rejected), str(fixture)], cwd=ROOT, capture_output=True)
        if response.returncode == 0 or rejected.exists():
            raise RuntimeError("Output escape was accepted")
        response = subprocess.run(command, cwd=ROOT, capture_output=True)
        if response.returncode == 0 or any(digest(output / p) != h for p, h in evidence["output_sha256"].items()):
            raise RuntimeError("Existing combined evidence was overwritten")
        (output / "evidence.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
        print("Combined audio switch: "+str(output), flush=True)


if __name__ == "__main__":
    main()
