"""Opt-in same-process silent output + simulation/input + GPU editor proof."""
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
from check_p0_composition import check_cpu_boundary
from check_p0_edit_composition import fixture_check
from p0_process_metrics import WindowsProcessMetrics, environment_info

CHECKS = {"audio_clock_was_advancing", "cancel_restores_canvas", "commit_changes_canvas", "draft_changes_canvas",
          "four_composition_proofs", "fresh_paused_device_samples", "fresh_press_after_resume_only", "no_fault",
          "paused_device_span_is_zero", "paused_inputs_discarded", "paused_wall_at_least_300_ms", "resume_does_not_catch_up",
          "simulation_advances_after_resume", "simulation_uses_device_clock", "single_commit", "three_hundred_edits"}


def validate(result):
    checks = result.get("checks", {})
    if (result.get("schema_version") != 1 or result.get("status") != "passed"
            or set(checks) != CHECKS or not all(v is True for v in checks.values())
            or result.get("errors") or result.get("edit_count") != 300 or result.get("accepted_inputs") != 2):
        raise RuntimeError("Transport checks or coverage failed")
    frozen, resumed, final = (result[k] for k in ("frozen_us", "resume_us", "final_simulation_us"))
    wall = result["paused_wall_ms"]
    if (frozen < 200000 or not frozen <= resumed <= frozen + 50000 or final < resumed + 200000
            or not math.isfinite(wall) or wall < 300):
        raise RuntimeError("Device clock did not freeze/resume independently of wall time")
    rows = result.get("fresh_samples", [])
    if (not rows or any(r["device_us"] < 0 or r["device_us"] != r["simulation_us"] for r in rows)
            or any(b["sequence"] <= a["sequence"] or b["device_us"] < a["device_us"] for a, b in zip(rows, rows[1:]))):
        raise RuntimeError("Simulation is not following fresh monotonic device samples")
    paused = [s for s in rows if s["phase"] == "Paused" and s["request"] == 2]
    if (len(paused) < 3 or any(s["device_us"] != frozen for s in paused)
            or result.get("paused_device_samples") != len(paused) or result.get("paused_device_span_us") != 0):
        raise RuntimeError("Paused position proof is missing or drifting")
    captures = result.get("captures", [])
    if (len(captures) != 4 or [c["stage"] for c in captures] != list(range(4))
            or [c["nodes"] for c in captures] != [1, 1, 1, 2]
            or any(c["accepted_inputs"] != 1 or c["simulation_us"] != frozen for c in captures)):
        raise RuntimeError("Paused input/history/render checkpoints differ")
    proofs = result.get("composition_pixel_checks", [])
    if (len(proofs) != 4 or any(p["pixels"] != 1024 * 1024 or p["max_channel_error"] > 2 or p["pixels_outside_tolerance"] != 0
                              or p["display_max_channel_error"] > 2 or p["display_pixels_outside_tolerance"] != 0 for p in proofs)):
        raise RuntimeError("Composition/display proof failed")
    c = result.get("composition", {})
    if (c.get("source_upload_batches") != 1 or c.get("source_upload_bytes") != 40 * 1024 * 1024
            or c.get("updates") != 303 or c.get("operation_upload_bytes") != 303 * 4480
            or c.get("normal_path_cpu_pixel_readbacks") != 0 or c.get("normal_path_cpu_texture_uploads") != 0):
        raise RuntimeError("Unexpected source/upload lifetime during pause")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sources():
    paths = []
    for package in ("p0-egui", "p0-wasapi", "p0-preview-contract", "p0-media"):
        root = ROOT / "experiments" / package
        paths += [root / "Cargo.toml", root / "Cargo.lock", *sorted((root / "src").rglob("*.rs"))]
    paths += sorted((ROOT / "experiments/p0-egui/src").glob("*.wgsl"))
    paths += [Path(__file__).resolve(), ROOT / "tools/check_p0_gui.py", ROOT / "tools/p0_process_metrics.py"]
    return paths


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", required=True, type=Path)
    parser.add_argument("--repetitions", type=int, choices=range(1, 4), default=3)
    args = parser.parse_args()
    if os.name != "nt":
        raise RuntimeError("Real output integration is currently Windows-only")
    fixture = fixture_check(args.fixture)
    check_cpu_boundary((ROOT / "experiments/p0-egui/src/audio_transport.rs").read_text(encoding="utf-8"))
    subprocess.run([sys.executable, str(ROOT / "tools/check_p0_gui.py"), "--device-audio"], check=True)
    exe = ROOT / "experiments/p0-egui/target/release/transport-proof.exe"
    for _ in range(args.repetitions):
        output = ROOT / "artifacts/p0" / ("transport-" + uuid.uuid4().hex[:8])
        command = [str(exe), str(output), str(fixture)]
        samples = []
        with subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT) as process:
            metrics = WindowsProcessMetrics(process)
            try:
                deadline = time.monotonic() + 110
                while process.poll() is None:
                    if time.monotonic() > deadline:
                        process.kill()
                        process.wait()
                        raise RuntimeError("Transport proof exceeded timeout")
                    sample = metrics.sample()
                    if sample:
                        samples.append(sample)
                    time.sleep(.1)
            finally:
                metrics.close()
            log = process.stdout.read().decode("utf-8", errors="replace")
            (output / "run.log").write_text(log, encoding="utf-8")
            if process.returncode:
                raise RuntimeError("Native transport failed: " + str(output))
        validate(json.loads((output / "result.json").read_text(encoding="utf-8")))
        evidence = {"schema_version": 1, "executable_sha256": digest(exe), "executable_bytes": exe.stat().st_size,
                    "source_sha256": {p.relative_to(ROOT).as_posix(): digest(p) for p in sources()},
                    "fixture": fixture.relative_to(ROOT).as_posix(), "inventory_sha256": digest(fixture / "inventory.json"),
                    "output_sha256": {p.name: digest(p) for p in sorted(output.glob("*.png"))}, "samples": samples,
                    "environment": environment_info(), "note": "Real same-process silent WASAPI output. Synthetic egui keys/pointer. Diagnostic CPU pixel proofs and regular test repaints are not latency or idle-CPU budgets."}
        for p in sources():
            dest = output / "sources" / p.relative_to(ROOT)
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(p, dest)
        rejected = ROOT / ("transport-escape-" + uuid.uuid4().hex[:8])
        response = subprocess.run([str(exe), str(rejected), str(fixture)], cwd=ROOT, capture_output=True)
        if response.returncode == 0 or rejected.exists():
            raise RuntimeError("Output escape was accepted")
        response = subprocess.run(command, cwd=ROOT, capture_output=True)
        if response.returncode == 0 or any(digest(output / p) != sha for p, sha in evidence["output_sha256"].items()):
            raise RuntimeError("Existing transport output was overwritten")
        (output / "evidence.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
        print("Transport proof: " + str(output), flush=True)


if __name__ == "__main__":
    main()
