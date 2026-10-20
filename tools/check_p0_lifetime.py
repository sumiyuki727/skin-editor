"""One bounded-upload candidate, fixed binary, separate performance and lifetime runs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import uuid

from check_p0 import ROOT
from check_p0_gui import MANIFEST, run_standard
from p0_gpu_repeats import aggregate, case_metrics


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    args = parser.parse_args()
    subprocess.run([sys.executable, str(ROOT / "tools/check_p0_gui.py")], check=True)
    batch = ROOT / "artifacts/p0" / ("upload-lifetime-" + uuid.uuid4().hex[:8])
    batch.mkdir()
    exe = batch / "standard-load.exe"
    shutil.copyfile(MANIFEST.parent / "target/release/standard-load.exe", exe)
    paths = list((MANIFEST.parent / "src").rglob("*.rs")) + [MANIFEST, MANIFEST.parent / "Cargo.lock"]
    paths += [ROOT / "tools" / n for n in ("check_p0_lifetime.py", "check_p0_gui.py", "p0_process_metrics.py", "p0_modules.py", "p0_workload.py", "p0_gpu_repeats.py")]
    hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    for p in paths:
        target = batch / "sources" / p.relative_to(ROOT)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(p, target)
    report = dict(schema_version=1, complete=False, source_sha256=hashes, executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(), cases=[],
                  note="Two independent full processes per condition, opposite order; no statistical confidence claim. GPU receipts bound one submitted upload frame; not driver virtual-page reclamation. Additional short traces/readbacks are diagnostics only.")
    output = batch / "batch.json"
    base = ("--memory-usage", "--async-load", "--paced-loading", "--backend-vulkan", "--reuse-textures", "--small-texture-cache")
    schedules = [("smoke", True, ("--diagnostic", "--no-readback"))]
    for bounded in (False, True, True, False):
        schedules.append(("performance", bounded, ("--no-readback",)))
    for bounded in (False, True):
        schedules.append(("timeline", bounded, ("--profile-memory", "--lifetime-trace", "--no-readback")))
        schedules.append(("pixels", bounded, ("--profile-memory", "--texture-proof", "--no-edit-readback")))
    for kind, bounded, extra in schedules:
        assert all(hashlib.sha256(p.read_bytes()).hexdigest() == hashes[str(p.relative_to(ROOT))] for p in paths), "Frozen measurement source changed"
        flags = (*base, *(("--bounded-uploads",) if bounded else ()), *extra)
        case = run_standard(dict(os.environ), args.fixture, flags, "lifetime-" + kind + ("-bounded" if bounded else "-base"), executable=exe, observe_modules=True)
        case.update(kind=kind, bounded=bounded)
        if kind == "performance":
            timing = json.loads((ROOT / case["output"] / "timing-samples.json").read_text(encoding="utf-8"))
            case["summary"] = case_metrics(case["result"], case["metrics"], timing)
        if bounded:
            result = case["result"]
            assert result["outstanding_upload_peak_bytes"] <= 8 * 1024 * 1024
            assert result["outstanding_upload_last_bytes"] == 0
        case["metrics"].pop("samples")
        report["cases"].append(case)
        output.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print("Lifetime batch: " + str(output), flush=True)
    assert all(hashlib.sha256(p.read_bytes()).hexdigest() == hashes[str(p.relative_to(ROOT))] for p in paths)
    report["complete"] = True
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
