"""Check gesture command/widget tests; optionally verify the native renderer's own output."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import uuid

from check_p0 import ROOT
from check_p0_composition import check_cpu_boundary


def validate(result, position_tolerance=0):
    rows = result.get("observations", [])
    if (result.get("status") != "passed" or len(rows) != 4
            or not all(result.get(key) for key in ("before_equals_cancelled_canvas", "draft_changes_canvas", "commit_changes_canvas"))):
        raise RuntimeError("Missing or incorrect native gesture/cancel evidence")
    if ([r["stage"] for r in rows] != list(range(4))
            or [r["nodes"] for r in rows] != [1, 1, 1, 2]
            or [r["committed_node"] for r in rows] != [0, 0, 0, 1]
            or any(len(r["visible_position"]) != 2 or any(not math.isfinite(v) for v in r["visible_position"]) for r in rows)
            or any(abs(actual - expected) > position_tolerance for row, target in zip(rows, [[40, 40], [70, 60], [40, 40], [100, 70]])
                   for actual, expected in zip(row["visible_position"], target))
            or any(r["simulation_ms"] != 0 or r["accepted_game_inputs"] != 0 for r in rows)):
        raise RuntimeError("Gesture leaked into published nodes, simulation or game input")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-native", action="store_true")
    args = parser.parse_args()
    check_cpu_boundary((ROOT / "experiments/p0-egui/src/interaction.rs").read_text(encoding="utf-8"))
    subprocess.run([sys.executable, str(ROOT / "tools/check_p0_gui.py")], check=True)
    if not args.run_native:
        return
    exe = ROOT / "experiments/p0-egui/target/release" / ("interaction-proof.exe" if os.name == "nt" else "interaction-proof")
    output = ROOT / "artifacts/p0" / ("interaction-" + uuid.uuid4().hex[:8])
    rejected = ROOT / ("interaction-escape-" + uuid.uuid4().hex[:8])
    response = subprocess.run([str(exe), "--self-test", str(rejected)], cwd=ROOT, capture_output=True)
    if response.returncode == 0 or rejected.exists():
        raise RuntimeError("Interaction output boundary failed")
    subprocess.run([str(exe), "--self-test", str(output)], cwd=ROOT, check=True, timeout=35)
    validate(json.loads((output / "result.json").read_text(encoding="utf-8")))
    paths = [ROOT / "experiments/p0-egui" / p for p in ("Cargo.toml", "Cargo.lock", "src/lib.rs", "src/interaction.rs", "src/interaction_view.rs", "src/bin/interaction-proof.rs",
              "src/composition.rs", "src/composition_gpu.rs", "src/composition_fixture.rs", "src/composition_view.rs", "src/composition.wgsl", "src/composition_present.wgsl")]
    paths += [Path(__file__).resolve(), ROOT / "experiments/p0-preview-contract/src/lib.rs"]
    evidence = {"schema_version": 1, "executable_sha256": hashlib.sha256(exe.read_bytes()).hexdigest(),
                "source_sha256": {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
                "output_sha256": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(output.glob("*.png"))},
                "note": "Synthetic egui input and native renderer readback; no system capture, physical input or stable evidence."}
    for p in paths:
        dest = output / "sources" / p.relative_to(ROOT)
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(p, dest)
    response = subprocess.run([str(exe), "--self-test", str(output)], cwd=ROOT, capture_output=True)
    if response.returncode == 0 or any(hashlib.sha256((output / p).read_bytes()).hexdigest() != sha for p, sha in evidence["output_sha256"].items()):
        raise RuntimeError("Existing interaction output was overwritten")
    (output / "evidence.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
    print("Interaction proof: " + str(output))


if __name__ == "__main__":
    main()
