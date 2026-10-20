"""Counterbalanced repetitions; native memory/latency runs remain uninstrumented."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import uuid

from check_p0 import ROOT
from check_p0_gui import MANIFEST, run_standard


ORDERS = (("fresh128", "reuse128", "reuse80"), ("reuse80", "fresh128", "reuse128"),
          ("reuse128", "reuse80", "fresh128"))


def distribution(values):
    valid = [v for v in values if v is not None and math.isfinite(v)]
    return {"count": len(values), "valid": len(valid), "min": min(valid) if valid else None,
            "median": statistics.median(valid) if valid else None, "max": max(valid) if valid else None}


def case_metrics(result, record, timings):
    if result["status"] != "passed" or result["heap_profile_build"] or result["gpu_readbacks_enabled"]:
        raise ValueError("Repetitions require successful uninstrumented runs without readback")
    cycle = record["cycle_statistics"]
    if cycle is None or cycle["documents"] != 200 or len(timings["document_load_wall_ms"]) != 203:
        raise ValueError("Expected 3 initial/winner/cache completions and 200 cycle documents")
    samples = record["samples"]
    def peak(key):
        values = [s[key] for s in samples if s.get(key) is not None]
        return max(values) if values else None
    docs = sorted(timings["document_load_wall_ms"][-200:])
    return {"private_peak_bytes": peak("private_bytes"), "os_peak_commit_bytes": peak("peak_commit_bytes"),
            "working_set_peak_bytes": peak("working_set_bytes"), "gpu_dedicated_peak_bytes": peak("gpu_dedicated_bytes"),
            "cycle_document_p95_ms": docs[(199 * 95) // 100], "cycle_document_max_ms": docs[-1],
            "all_document_p95_ms": result["document_load_wall_ms"]["p95"],
            "loading_frame_p95_ms": result["loading_cpu_frame_ms"]["p95"],
            "cycle_cpu_seconds": cycle["sampled_cpu_seconds"], "cycle_wall_seconds": cycle["phase_wall_seconds"]}


def aggregate(cases):
    groups = {condition: [case for case in cases if case["condition"] == condition] for condition in ORDERS[0]}
    metrics = next(iter(cases))["summary"].keys() if cases else ()
    return {condition: {key: distribution([case["summary"][key] for case in group]) for key in metrics}
            for condition, group in groups.items()}


def fingerprint(paths):
    return {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    args = parser.parse_args()
    if os.name != "nt":
        parser.error("Windows-only native measurements")
    # Run the normal checks first; instrumentation is never built here.
    subprocess.run([sys.executable, str(ROOT / "tools/check_p0_gui.py")], cwd=ROOT, check=True)
    batch = ROOT / "artifacts/p0" / ("gpu-repeats-" + uuid.uuid4().hex[:8])
    batch.mkdir()
    executable = batch / "standard-load.exe"
    shutil.copyfile(MANIFEST.parent / "target/release/standard-load.exe", executable)
    pdb = MANIFEST.parent / "target/release/standard_load.pdb"
    if pdb.exists():
        shutil.copyfile(pdb, batch / pdb.name)
    paths = list((MANIFEST.parent / "src").rglob("*.rs")) + [MANIFEST, MANIFEST.parent / "Cargo.lock"]
    paths += [ROOT / "tools" / name for name in ("check_p0_gui.py", "p0_process_metrics.py", "p0_workload.py", "p0_modules.py", "p0_gpu_repeats.py")]
    hashes = fingerprint(paths)
    for path in paths:
        destination = batch / "sources" / path.relative_to(ROOT)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)
    report = {"schema_version": 1, "complete": False, "orders": ORDERS, "cases": [], "source_sha256": hashes,
              "executable_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
              "note": "Three independent processes per condition; each order position occurs once. No heap/page tracing or GPU readback. Per-trial p95 uses only the 200 cycle documents; repetitions are not pooled into fake independent observations."}
    output = batch / "repetitions.json"
    base = ("--memory-usage", "--async-load", "--paced-loading", "--backend-vulkan", "--no-readback")
    for block, order in enumerate(ORDERS, 1):
        for condition in order:
            if fingerprint(paths) != hashes:
                raise RuntimeError("Measurement sources changed during the batch; preserve existing evidence and start a new batch")
            flags = (*base, *(("--reuse-textures",) if condition != "fresh128" else ()),
                     *(("--small-texture-cache",) if condition == "reuse80" else ()))
            case = run_standard(dict(os.environ), args.fixture, flags, "repeat-" + condition, executable=executable, observe_modules=True)
            if fingerprint(paths) != hashes or case["metrics"]["executable_sha256"] != report["executable_sha256"]:
                raise RuntimeError("Measurement identity changed during execution")
            timing = json.loads((ROOT / case["output"] / "timing-samples.json").read_text(encoding="utf-8"))
            summary = case_metrics(case["result"], case["metrics"], timing)
            case["metrics"].pop("samples")
            case.update(block=block, condition=condition, summary=summary)
            report["cases"].append(case)
            report["aggregate"] = aggregate(report["cases"])
            output.write_text(json.dumps(report, indent=2), encoding="utf-8")
            print("Repetition evidence: " + str(output), flush=True)
    report["complete"] = True
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
