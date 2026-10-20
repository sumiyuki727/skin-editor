"""Native P0 continuous gesture + resident GPU composition/presentation proof."""
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
from check_p0_interaction import validate as validate_gesture
from p0_process_metrics import WindowsProcessMetrics, environment_info


def validate(result, readback):
    validate_gesture(result, position_tolerance=0.0001)
    c = result.get("composition") or {}
    if (result.get("edit_count") != 300 or len(result.get("edit_ui_host_ms", [])) != 300
            or result.get("per_edit_readback") != readback
            or len(result.get("edit_readback_ms", [])) != (300 if readback else 0)
            or c.get("source_upload_batches") != 1 or c.get("source_upload_bytes") != 40 * 1024 * 1024
            or c.get("updates") != 303 or c.get("operation_upload_bytes") != 303 * 4480
            or c.get("normal_path_cpu_pixel_readbacks") != 0 or c.get("normal_path_cpu_texture_uploads") != 0):
        raise RuntimeError("Incomplete continuous-edit coverage or unexpected source/pixel uploads")
    if any(not math.isfinite(v) or v < 0 for v in result["edit_ui_host_ms"] + result["edit_readback_ms"]):
        raise RuntimeError("Invalid edit timing samples")
    checks = result.get("composition_pixel_checks", [])
    if (len(checks) != 4 or [v["stage"] for v in checks] != list(range(4))
            or any(v["pixels"] != 1024 * 1024 or v["max_channel_error"] > 2
                   or v["pixels_outside_tolerance"] != 0 or v.get("display_max_channel_error", 256) > 2
                   or v.get("display_pixels_outside_tolerance", 1) != 0 for v in checks)):
        raise RuntimeError("Missing or incorrect composition CPU/GPU spot checks")


def fixture_check(fixture):
    fixture = fixture.resolve(strict=True)
    if (ROOT / "artifacts/p0").resolve() not in fixture.parents:
        raise RuntimeError("Use an independent fixture under artifacts/p0")
    inventory = json.loads((fixture / "inventory.json").read_text(encoding="utf-8"))
    if (inventory["asset_count"], inventory["document_count"], inventory["layer_count"], inventory["side"]) != (100, 10, 500, 1024):
        raise RuntimeError("Not a standard fixture")
    for item in inventory["files"]:
        path = (fixture / item["path"]).resolve(strict=True)
        if fixture not in path.parents or digest(path) != item["sha256"]:
            raise RuntimeError("Fixture boundary or SHA-256 mismatch")
    return fixture


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sources():
    return [ROOT / "experiments/p0-egui" / p for p in (
        "Cargo.toml", "Cargo.lock", "src/lib.rs", "src/composition.rs", "src/composition_gpu.rs",
        "src/composition_fixture.rs", "src/composition_view.rs", "src/composition.wgsl", "src/composition_present.wgsl",
        "src/interaction.rs", "src/interaction_view.rs", "src/bin/interaction-proof.rs")] + [
            Path(__file__).resolve(), ROOT / "tools/check_p0_interaction.py", ROOT / "tools/p0_process_metrics.py",
            ROOT / "experiments/p0-preview-contract/src/lib.rs"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", required=True, type=Path)
    parser.add_argument("--repetitions", type=int, choices=range(1, 4), default=2)
    args = parser.parse_args()
    fixture = fixture_check(args.fixture)
    subprocess.run([sys.executable, str(ROOT / "tools/check_p0_composition.py"), "--run-native"], check=True)
    exe = ROOT / "experiments/p0-egui/target/release" / ("interaction-proof.exe" if os.name == "nt" else "interaction-proof")
    for repetition in range(args.repetitions):
        for readback in (False, True):
            output = ROOT / "artifacts/p0" / ("edit-composition-" + uuid.uuid4().hex[:8])
            command = [str(exe), "--self-test", str(output), "--fixture", str(fixture)] + (["--edit-readback"] if readback else [])
            samples = []
            with subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT) as process:
                metrics = WindowsProcessMetrics(process) if os.name == "nt" else None
                try:
                    deadline = time.monotonic() + 110
                    while process.poll() is None:
                        if time.monotonic() > deadline:
                            process.kill()
                            process.wait()
                            raise RuntimeError("Continuous edit proof exceeded timeout")
                        if metrics:
                            sample = metrics.sample()
                            if sample:
                                samples.append(sample)
                        time.sleep(0.1)
                finally:
                    if metrics:
                        metrics.close()
                log = process.stdout.read().decode("utf-8", errors="replace")
                (output / "run.log").write_text(log, encoding="utf-8")
                if process.returncode:
                    raise RuntimeError("Native edit proof failed: " + str(output))
            result = json.loads((output / "result.json").read_text(encoding="utf-8"))
            validate(result, readback)
            evidence = {"schema_version": 1, "repetition": repetition, "per_edit_readback": readback,
                        "executable_sha256": digest(exe), "source_sha256": {p.relative_to(ROOT).as_posix(): digest(p) for p in sources()},
                        "fixture": fixture.relative_to(ROOT).as_posix(), "inventory_sha256": digest(fixture / "inventory.json"),
                        "output_sha256": {p.name: digest(p) for p in sorted(output.glob("*.png"))},
                        "samples": samples, "environment": environment_info(),
                        "note": "Normal GPU presentation has no CPU pixel transfer. Four CPU/GPU spot checks and file saves are outside the 300-edit phase. Extra per-edit UI screenshots run in separate processes; they do not measure physical screen latency or total application budgets."}
            for p in sources():
                dest = output / "sources" / p.relative_to(ROOT)
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(p, dest)
            rejected = ROOT / ("edit-composition-escape-" + uuid.uuid4().hex[:8])
            response = subprocess.run([str(exe), "--self-test", str(rejected), "--fixture", str(fixture)], cwd=ROOT, capture_output=True)
            if response.returncode == 0 or rejected.exists():
                raise RuntimeError("Output escape was accepted")
            response = subprocess.run(command, cwd=ROOT, capture_output=True)
            if response.returncode == 0 or any(digest(output / p) != sha for p, sha in evidence["output_sha256"].items()):
                raise RuntimeError("Existing output was overwritten")
            (output / "evidence.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
            print("Continuous edit proof: " + str(output), flush=True)


if __name__ == "__main__":
    main()
