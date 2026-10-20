"""Opt-in generated low-amplitude PCM + pause/seek/recovery editor proof (Windows)."""
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

CHECKS = {"four_content_epochs", "all_epochs_nonzero_pcm", "paused_submission_frozen", "fresh_input_only",
          "thirty_updates_one_commit", "four_pixel_proofs", "old_epochs_rejected", "normal_resume_no_catchup",
          "clock_from_device", "final_paused", "no_error"}


def reference_sample(frame, rate):
    seconds = frame/rate
    local = seconds % .5
    if seconds >= 6 or local >= .2:
        return 0.
    envelope = max(0., min(1., local/.005, (.2-local)/.005))
    return .04*envelope*math.sin(2*math.pi*[440, 660, 880][int(seconds/.5) % 3]*seconds)


def validate(result):
    checks = result.get("checks", {})
    if (result.get("schema_version") != 1 or result.get("status") != "passed" or result.get("errors")
            or set(checks) != CHECKS or not all(v is True for v in checks.values())
            or result.get("accepted_inputs") != 2 or result.get("nodes") != 2 or result.get("edit_count") != 30
            or result.get("system_capture_used") is not False or result.get("microphone_or_loopback_used") is not False
            or result.get("human_audibility") != "unconfirmed" or result.get("stale_rejected") != 3):
        raise RuntimeError("Incomplete PCM/pause/seek/editor proof, or unsupported hearing claim")
    fmt = result["audio_format"]
    rate, channels = fmt["rate"], fmt["channels"]
    if not 8000 <= rate <= 384000 or channels not in (1, 2) or fmt["bits"] != 32:
        raise RuntimeError("Unsupported PCM stream format")
    frozen, resumed = result["frozen_us"], result["normal_resume_us"]
    if frozen < 200000 or not frozen <= resumed <= frozen+50000:
        raise RuntimeError("Normal resume caught up paused wall time")
    bases = {0: 0, 1: 1050000, 2: 50000, 3: result["recovery_frozen_us"]}
    rows = result.get("fresh_samples", [])
    if not rows or set(r["epoch"] for r in rows) != set(bases):
        raise RuntimeError("Missing PCM stream epochs")
    for epoch, base in bases.items():
        epoch_rows = [r for r in rows if r["epoch"] == epoch]
        for r in epoch_rows:
            if r["base_us"] != base or r["simulation_us"] != base+r["device_us"]:
                raise RuntimeError("Simulation followed submission counter or wrong seek base")
            pcm = r["pcm"]
            if (pcm["requested_us"] != base or pcm["source_origin_frame"] != base*rate//1000000
                    or pcm["submitted_frames"] < 0 or pcm["packets"] < 0):
                raise RuntimeError("Wrong content origin or submission frontier")
            for key in ("first", "last"):
                packet = pcm[key]
                if pcm["packets"] == 0:
                    if packet is not None or pcm["submitted_frames"] != 0:
                        raise RuntimeError("Unplayed stream contains submitted PCM")
                    continue
                expected = pcm["source_origin_frame"] if key == "first" else pcm["source_origin_frame"]+pcm["submitted_frames"]-packet["frames"]
                if (packet["source_frame"] != expected or not 0 < packet["frames"] <= fmt["buffer_frames"]
                        or not 0 <= packet["peak"] <= .040001 or not math.isfinite(packet["peak"])
                        or not 0 <= packet["nonzero_samples"] <= packet["frames"]*channels
                        or len(packet["probe"]) != 8):
                    raise RuntimeError("PCM packet geometry, peak or source cursor differs")
                for n, value in enumerate(packet["probe"][:min(packet["frames"], 8)]):
                    if not math.isfinite(value) or abs(value-reference_sample(expected+n, rate)) > 2e-7:
                        raise RuntimeError("Submitted PCM probes differ from independent tone reference")
        for a, b in zip(epoch_rows, epoch_rows[1:]):
            if (b["sequence"] <= a["sequence"] or b["device_us"] < a["device_us"]
                    or b["pcm"]["submitted_frames"] < a["pcm"]["submitted_frames"]
                    or b["pcm"]["packets"] < a["pcm"]["packets"]):
                raise RuntimeError("Stale samples or reset content within the same stream")
        if epoch_rows[-1]["pcm"]["nonzero_samples"] <= 0:
            raise RuntimeError("Stream never submitted nonzero content")
    for epoch, request in [(0, 2), (0, 4), (1, 0), (1, 2), (2, 0), (3, 0), (3, 2)]:
        paused = [r for r in rows if r["epoch"] == epoch and r["request"] == request and r["phase"] == "Paused"]
        if (len(paused) < 3 or len({r["device_us"] for r in paused}) != 1
                or len({r["pcm"]["submitted_frames"] for r in paused}) != 1):
            raise RuntimeError("Pause did not freeze both device clock and PCM submissions")
        if request == 0 and (paused[0]["simulation_us"] != bases[epoch] or paused[-1]["pcm"]["submitted_frames"] != 0):
            raise RuntimeError("Replacement stream started playing before explicit resume")
    if (any(b["epoch"] < a["epoch"] for a, b in zip(rows, rows[1:]))
            or result["stale_rejected"] != 3 or result["final_simulation_us"] < bases[3]+200000):
        raise RuntimeError("Missing stale-epoch rejection or post-recovery playback")
    checkpoints = result.get("checkpoints", [])
    if (len(checkpoints) != 4 or [c["epoch"] for c in checkpoints] != [0, 0, 1, 3]
            or [c["nodes"] for c in checkpoints] != [1, 2, 2, 2]
            or [c["accepted_inputs"] for c in checkpoints] != [1, 1, 2, 2]
            or [c["simulation_us"] for c in checkpoints] != [frozen, frozen, bases[1], bases[3]]
            or checkpoints[0]["pcm"]["submitted_frames"] != checkpoints[1]["pcm"]["submitted_frames"]):
        raise RuntimeError("Paused edit, seek or recovery modified history/input/PCM state")
    proofs = result.get("proofs", [])
    if (len(proofs) != 4 or [p["stage"] for p in proofs] != list(range(4))
            or any(p["pixels"] != 1024*1024 or not 0 <= p["max_channel_error"] <= 2 or p["pixels_outside_tolerance"] != 0
                   or not 0 <= p["display_max_channel_error"] <= 2 or p["display_pixels_outside_tolerance"] != 0 for p in proofs)):
        raise RuntimeError("PCM integration composition/display comparison failed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--repetitions", type=int, choices=range(1, 4), default=3)
    args = parser.parse_args()
    if os.name != "nt":
        raise RuntimeError("PCM device proof currently requires Windows")
    fixture = fixture_check(args.fixture)
    subprocess.run([sys.executable, str(ROOT/"tools/check_p0_gui.py"), "--device-audio"], check=True)
    exe = ROOT/"experiments/p0-egui/target/release/pcm-proof.exe"
    paths = sources()+[Path(__file__).resolve()]
    identity = {p.relative_to(ROOT).as_posix(): digest(p) for p in paths}
    executable_sha = digest(exe)
    for _ in range(args.repetitions):
        output = ROOT/"artifacts/p0"/("pcm-"+uuid.uuid4().hex[:8])
        command = [str(exe), str(output), str(fixture)]
        log_path = output.with_suffix(".log")
        samples = []
        with log_path.open("wb") as log, subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT) as process:
            metrics = WindowsProcessMetrics(process)
            try:
                deadline = time.monotonic()+80
                while process.poll() is None:
                    if time.monotonic()>deadline:
                        process.kill(); process.wait()
                        raise RuntimeError("PCM timeout: "+str(output))
                    sample = metrics.sample()
                    if sample:
                        samples.append(sample)
                    time.sleep(.1)
            finally:
                metrics.close()
        if process.returncode:
            raise RuntimeError("Native PCM failed: "+str(log_path))
        shutil.copyfile(log_path, output/"run.log")
        result = json.loads((output/"result.json").read_text(encoding="utf-8"))
        validate(result)
        for kind in ("cpu", "gpu", "display"):
            hashes = [digest(output/f"composition-{n}-{kind}.png") for n in range(4)]
            if hashes[0] == hashes[1] or len(set(hashes[1:])) != 1:
                raise RuntimeError("Edit invisible or relocation changed pixels")
        if digest(exe) != executable_sha or any(digest(p) != identity[p.relative_to(ROOT).as_posix()] for p in paths):
            raise RuntimeError("PCM measurement source/executable changed")
        evidence = {"schema_version": 1, "executable_sha256": executable_sha, "executable_bytes": exe.stat().st_size,
                    "source_sha256": identity, "fixture": fixture.relative_to(ROOT).as_posix(), "inventory_sha256": digest(fixture/"inventory.json"),
                    "samples": samples, "output_sha256": {p.name: digest(p) for p in sorted(output.glob("*.png"))}, "environment": environment_info(),
                    "note": "Actual generated low-amplitude float PCM submission; no microphone/loopback or hearing/acoustic-latency claim. Paused pixel diagnostics are not resource budgets."}
        for p in paths:
            dest = output/"sources"/p.relative_to(ROOT)
            dest.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(p, dest)
        rejected = ROOT/("pcm-escape-"+uuid.uuid4().hex[:8])
        response = subprocess.run([str(exe), str(rejected), str(fixture)], cwd=ROOT, capture_output=True)
        if response.returncode == 0 or rejected.exists():
            raise RuntimeError("Output escape accepted")
        response = subprocess.run(command, cwd=ROOT, capture_output=True)
        if response.returncode == 0 or any(digest(output/p) != sha for p, sha in evidence["output_sha256"].items()):
            raise RuntimeError("Existing PCM evidence overwritten")
        (output/"evidence.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
        print("Generated PCM proof: "+str(output), flush=True)


if __name__ == "__main__":
    main()
