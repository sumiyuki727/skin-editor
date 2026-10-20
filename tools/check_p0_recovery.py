"""Opt-in Windows stream retirement/reopen + frozen editor proof; no unplug claim."""
import argparse
import json
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

CHECKS = {"two_recoveries", "clock_progress_each_stream", "recovered_paused_samples", "old_epoch_faults_ignored",
          "anchored_clock", "fresh_input_only", "fault_edit_one_commit", "four_pixel_checks", "final_pause", "no_unexpected_error"}


def validate(result):
    checks = result.get("checks", {})
    if (result.get("schema_version") != 1 or result.get("status") != "passed" or result.get("errors")
            or set(checks) != CHECKS or not all(v is True for v in checks.values())
            or result.get("accepted_inputs") != 2 or result.get("nodes") != 2 or result.get("edit_count") != 30
            or result.get("stale_rejected") != 2 or result.get("system_capture_used") is not False):
        raise RuntimeError("Incomplete recovery/input/editor coverage")
    frozen = result.get("frozen_us", [])
    if (len(frozen) != 2 or result.get("recovered_us") != frozen or frozen[0] < 200000
            or frozen[1] < frozen[0]+200000 or result["final_simulation_us"] < frozen[1]+200000):
        raise RuntimeError("Recovery changed frozen time or did not resume each stream")
    rows = result.get("fresh_samples", [])
    if (not rows or set(r["epoch"] for r in rows) != {0, 1, 2}
            or any(r["device_us"] < r["origin_us"] or r["simulation_us"] != r["base_us"]+r["device_us"]-r["origin_us"] for r in rows)
            or any(b["epoch"] < a["epoch"] or b["simulation_us"] < a["simulation_us"]
                   or (b["epoch"] == a["epoch"] and b["sequence"] <= a["sequence"]) for a, b in zip(rows, rows[1:]))):
        raise RuntimeError("Nonmonotonic, stale or incorrectly anchored device observations")
    for epoch in (1, 2):
        epoch_rows = [r for r in rows if r["epoch"] == epoch]
        paused = [r for r in epoch_rows if r["request"] == 0 and r["phase"] == "Paused"]
        playing = [r for r in epoch_rows if r["phase"] == "Playing" and r["request"] == 1]
        if (len(paused) < 3 or epoch_rows[0] != paused[0] or not playing
                or any(r["base_us"] != frozen[epoch-1] or r["origin_us"] != epoch_rows[0]["device_us"] for r in epoch_rows)
                or any(r["simulation_us"] != frozen[epoch-1] for r in paused)
                or playing[-1]["simulation_us"] < frozen[epoch-1]+200000):
            raise RuntimeError("Reopened stream did not remain paused until explicit resume")
    final_paused = [r for r in rows if r["epoch"] == 2 and r["request"] == 2 and r["phase"] == "Paused"]
    if len(final_paused) < 3 or any(r["simulation_us"] != result["final_simulation_us"] for r in final_paused):
        raise RuntimeError("Final recovered stream lacks fresh stable pause samples")
    checkpoints = result.get("checkpoints", [])
    if (len(checkpoints) != 4 or [c["stage"] for c in checkpoints] != [0, 1, 2, 3]
            or [c["epoch"] for c in checkpoints] != [0, 0, 1, 2]
            or [c["phase"] for c in checkpoints] != ["Faulted", "Faulted", "Paused", "Paused"]
            or [c["nodes"] for c in checkpoints] != [1, 2, 2, 2]
            or [c["inputs"] for c in checkpoints] != [1, 1, 1, 2]
            or [c["simulation_us"] for c in checkpoints] != [*([frozen[0]]*3), frozen[1]]):
        raise RuntimeError("Fault-time edit or recovery changed history, inputs or time")
    proofs = result.get("proofs", [])
    if (len(proofs) != 4 or [p["stage"] for p in proofs] != list(range(4))
            or any(p["pixels"] != 1024*1024 or not 0 <= p["max_channel_error"] <= 2 or p["pixels_outside_tolerance"] != 0
                   or not 0 <= p["display_max_channel_error"] <= 2 or p["display_pixels_outside_tolerance"] != 0 for p in proofs)):
        raise RuntimeError("Missing or incorrect recovery composition/display comparison")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--repetitions", type=int, choices=range(1, 4), default=3)
    args = parser.parse_args()
    if os.name != "nt":
        raise RuntimeError("Windows recovery proof only; Linux and macOS are unverified")
    fixture = fixture_check(args.fixture)
    subprocess.run([sys.executable, str(ROOT / "tools/check_p0_gui.py"), "--device-audio"], check=True)
    exe = ROOT / "experiments/p0-egui/target/release/recovery-proof.exe"
    paths = sources()+[Path(__file__).resolve()]
    identity = {p.relative_to(ROOT).as_posix(): digest(p) for p in paths}
    executable_sha = digest(exe)
    for _ in range(args.repetitions):
        output = ROOT / "artifacts/p0" / ("recovery-"+uuid.uuid4().hex[:8])
        command = [str(exe), str(output), str(fixture)]
        samples = []
        log_path = output.with_suffix(".log")
        with log_path.open("wb") as log, subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT) as process:
            metrics = WindowsProcessMetrics(process)
            try:
                deadline = time.monotonic()+80
                while process.poll() is None:
                    if time.monotonic()>deadline:
                        process.kill(); process.wait()
                        raise RuntimeError("Recovery proof timeout: "+str(output))
                    sample = metrics.sample()
                    if sample:
                        samples.append(sample)
                    time.sleep(.1)
            finally:
                metrics.close()
        if process.returncode:
            raise RuntimeError("Native recovery failed: "+str(log_path))
        shutil.copyfile(log_path, output/"run.log")
        result = json.loads((output/"result.json").read_text(encoding="utf-8"))
        validate(result)
        for kind in ("cpu", "gpu", "display"):
            hashes = [digest(output/f"composition-{n}-{kind}.png") for n in range(4)]
            if hashes[0] == hashes[1] or len(set(hashes[1:])) != 1:
                raise RuntimeError("Fault edit was invisible or rebuilding changed edited pixels")
        if digest(exe) != executable_sha or any(digest(p) != identity[p.relative_to(ROOT).as_posix()] for p in paths):
            raise RuntimeError("Measured sources/executable changed during proof")
        evidence = {"schema_version": 1, "executable_sha256": executable_sha, "executable_bytes": exe.stat().st_size,
                    "source_sha256": identity, "fixture": fixture.relative_to(ROOT).as_posix(),
                    "inventory_sha256": digest(fixture/"inventory.json"), "samples": samples,
                    "output_sha256": {p.name: digest(p) for p in sorted(output.glob("*.png"))},
                    "environment": environment_info(), "note": "Two explicit retire/reopen cycles of our silent shared stream. No physical unplug, default-device switch, acoustic latency, budget or stable compatibility claim."}
        for p in paths:
            dest = output/"sources"/p.relative_to(ROOT)
            dest.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(p, dest)
        rejected = ROOT/("recovery-escape-"+uuid.uuid4().hex[:8])
        response = subprocess.run([str(exe), str(rejected), str(fixture)], cwd=ROOT, capture_output=True)
        if response.returncode == 0 or rejected.exists():
            raise RuntimeError("Output escape accepted")
        response = subprocess.run(command, cwd=ROOT, capture_output=True)
        if response.returncode == 0 or any(digest(output/p) != sha for p, sha in evidence["output_sha256"].items()):
            raise RuntimeError("Existing recovery output was overwritten")
        (output/"evidence.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
        print("Output recovery: "+str(output), flush=True)


if __name__ == "__main__":
    main()
