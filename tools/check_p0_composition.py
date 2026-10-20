"""Build/check CPU composition semantics; GPU pixel proof is explicitly opt-in."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import uuid

from check_p0 import ROOT
from p0_process_metrics import WindowsProcessMetrics, environment_info


def validate_scale(result):
    cases = result.get("cases", [])
    if (result.get("status") != "passed" or len(cases) != 10
            or [c["document"] for c in cases] != list(range(10))
            or result.get("warm_dispatches") != 300):
        raise RuntimeError("Incomplete standard composition proof")
    for case in cases:
        if (case["layers"] != 50 or case["ops"] != 70 or case["groups"] != 10
                or case["pixels"] != 1024 * 1024 or case["pixels_outside_tolerance"] != 0
                or case["max_channel_error"] > 2 or len(case["source_ids"]) != 10
                or len(case["warm_gpu_ms"]) != 30 or len(case["warm_submit_wait_readback_ms"]) != 30):
            raise RuntimeError("Incorrect standard composition coverage or pixels")


def source_paths():
    return [ROOT / "experiments/p0-egui" / name for name in (
        "Cargo.toml", "Cargo.lock", "src/composition.rs", "src/composition.wgsl",
        "src/composition_gpu.rs", "src/composition_fixture.rs", "src/lib.rs", "src/bin/composition-proof.rs")] + [Path(__file__).resolve()]


def run_scale(exe, fixture, repetitions):
    fixture = fixture.resolve(strict=True)
    if (ROOT / "artifacts/p0").resolve() not in fixture.parents:
        raise RuntimeError("Use a generated fixture under artifacts/p0")
    # Check all declared fixture hashes before taking measurements.
    inventory = json.loads((fixture / "inventory.json").read_text(encoding="utf-8"))
    if (inventory["asset_count"], inventory["document_count"], inventory["layer_count"], inventory["side"]) != (100, 10, 500, 1024):
        raise RuntimeError("Not the standard fixture")
    for item in inventory["files"]:
        path = (fixture / item["path"]).resolve(strict=True)
        if fixture not in path.parents or hashlib.sha256(path.read_bytes()).hexdigest() != item["sha256"]:
            raise RuntimeError("Fixture hash/boundary mismatch")
    outputs = []
    for _ in range(repetitions):
        output = ROOT / "artifacts/p0" / ("composition-scale-" + uuid.uuid4().hex[:8])
        samples = []
        # The binary owns creation and existing-output rejection.
        with subprocess.Popen([str(exe), str(output), "--scale", str(fixture)], cwd=ROOT,
                              stdout=subprocess.PIPE, stderr=subprocess.STDOUT) as process:
            metrics = WindowsProcessMetrics(process) if os.name == "nt" else None
            try:
                deadline = time.monotonic() + 300
                while process.poll() is None:
                    if time.monotonic() > deadline:
                        process.kill()
                        process.wait()
                        raise RuntimeError("Standard composition exceeded 300 seconds")
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
                raise RuntimeError("Standard composition failed: " + str(output))
        result = json.loads((output / "result.json").read_text(encoding="utf-8"))
        validate_scale(result)
        evidence = {"schema_version": 1, "executable_sha256": hashlib.sha256(exe.read_bytes()).hexdigest(),
                    "fixture": str(fixture.relative_to(ROOT)),
                    "inventory_sha256": hashlib.sha256((fixture / "inventory.json").read_bytes()).hexdigest(),
                    "source_sha256": {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths()},
                    "output_sha256": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(output.glob("*.png"))},
                    "samples": samples, "environment": environment_info(),
                    "note": "Own process only. 100 ms sparse memory samples include CPU reference and PNG saves; GPU timestamps exclude host work. Not full editor budgets."}
        for p in source_paths():
            dest = output / "sources" / p.relative_to(ROOT)
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(p, dest)
        rejection = subprocess.run([str(exe), str(output), "--scale", str(fixture)], cwd=ROOT, capture_output=True)
        if rejection.returncode == 0 or any(hashlib.sha256((output / name).read_bytes()).hexdigest() != sha
                                           for name, sha in evidence["output_sha256"].items()):
            raise RuntimeError("Existing scale output was overwritten")
        (output / "evidence.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
        outputs.append(str(output))
        print("Standard composition proof: " + str(output), flush=True)
    return outputs


def check_cpu_boundary(source):
    # A small import guard, paired with Cargo's reviewed dependency whitelist.
    code = re.sub(r"//[^\n]*", "", source.split("#[cfg(test)]", 1)[0])
    forbidden = r"\b(?:eframe|egui|wgpu)\s*::|\bstd\s*::\s*(?:fs|net|thread)\b|use\s+std\s*::\s*\{[^;]*\b(?:fs|net|thread)\b"
    if re.search(forbidden, code, re.DOTALL):
        raise RuntimeError("CPU composition must not import UI, GPU, file/network or worker APIs")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-native", action="store_true")
    parser.add_argument("--fixture", type=Path, help="Also measure all standard documents; implies --run-native")
    parser.add_argument("--repetitions", type=int, choices=range(1, 4), default=2)
    args = parser.parse_args()
    check_cpu_boundary((ROOT / "experiments/p0-egui/src/composition.rs").read_text(encoding="utf-8"))
    # This also checks the unchanged candidate's module/dependency boundary.
    subprocess.run([sys.executable, str(ROOT / "tools/check_p0_gui.py")], check=True)
    if not args.run_native and args.fixture is None:
        return
    output = ROOT / "artifacts/p0" / ("composition-" + uuid.uuid4().hex[:8])
    exe = ROOT / "experiments/p0-egui/target/release" / ("composition-proof.exe" if os.name == "nt" else "composition-proof")
    rejected = ROOT / ("composition-escape-" + uuid.uuid4().hex[:8])
    response = subprocess.run([str(exe), str(rejected)], cwd=ROOT, capture_output=True)
    if response.returncode == 0 or rejected.exists():
        raise RuntimeError("Composition output boundary failed")
    subprocess.run([str(exe), str(output)], cwd=ROOT, check=True, timeout=90)
    result = json.loads((output / "result.json").read_text(encoding="utf-8"))
    if result["status"] != "passed" or len(result["cases"]) != 9:
        raise RuntimeError("Composition pixel proof failed")
    paths = source_paths()
    evidence = dict(schema_version=1, executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),
                    source_sha256={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
                    output_sha256={p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(output.glob("*.png"))},
                    note="GPU directly reads its own compute result. No window or desktop capture. Not a standard-load performance or stable compatibility test.")
    response = subprocess.run([str(exe), str(output)], cwd=ROOT, capture_output=True)
    if response.returncode == 0 or any(hashlib.sha256((output / name).read_bytes()).hexdigest() != sha
                                      for name, sha in evidence["output_sha256"].items()):
        raise RuntimeError("Existing composition output was overwritten")
    (output / "evidence.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
    print("Composition proof: " + str(output))
    if args.fixture is not None:
        run_scale(exe, args.fixture, args.repetitions)


if __name__ == "__main__":
    main()
