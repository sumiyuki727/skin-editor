"""Opt-in native GPU allocation experiments; never traces another process or reads the desktop."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import uuid

from check_p0 import ROOT, windows_linker
from check_p0_gui import MANIFEST, run_standard


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--performance-only", action="store_true")
    parser.add_argument("--diagnostics-only", action="store_true")
    parser.add_argument("--compact", action="store_true", help="80 MiB GPU policy and full functional validation")
    parser.add_argument("--fixture", type=Path, required=True)
    args = parser.parse_args()
    if args.performance_only and args.diagnostics_only:
        parser.error("Choose one subset")
    if os.name != "nt":
        parser.error("This native process/GPU measurement runner is verified only on Windows")
    environment = dict(os.environ)
    private_bin = ROOT / ".p0-tools/rustup/toolchains/1.95.0-x86_64-pc-windows-msvc/bin"
    environment["PATH"] = str(private_bin) + os.pathsep + environment.get("PATH", "")
    environment["RUSTUP_HOME"] = str(ROOT / ".p0-tools/rustup")
    environment["CARGO_HOME"] = str(ROOT / ".p0-tools/cargo")
    cargo = str(private_bin / "cargo.exe")
    linker = windows_linker(environment)
    common = ["--manifest-path", str(MANIFEST), "--offline", "--locked", "-j", "2"]
    suffix = uuid.uuid4().hex[:8]
    binary_dir = ROOT / "artifacts/p0" / ("gpu-memory-binaries-" + suffix)
    binary_dir.mkdir()
    built = MANIFEST.parent / "target/release/standard-load.exe"
    normal = binary_dir / "normal.exe"
    profiled = binary_dir / "heap-profile.exe"
    subprocess.run([cargo, *linker, "build", *common, "--release"], env=environment, check=True)
    shutil.copyfile(built, normal)
    if not args.performance_only:
        for action, extra in (("clippy", ["--all-targets", "--", "-D", "warnings"]), ("test", ["--release"]), ("build", ["--release"])):
            subprocess.run([cargo, *linker, action, *common, "--features", "heap-profile", *extra], env=environment, check=True)
        shutil.copyfile(built, profiled)
        # Keep the ordinary target path uninstrumented; measurements use the
        # immutable copies, so later builds cannot change a running comparison.
        subprocess.run([cargo, *linker, "build", *common, "--release"], env=environment, check=True)
    configurations = []
    base = ("--memory-usage", "--async-load", "--paced-loading")
    if not args.performance_only:
        profile = (*base, "--profile-memory", "--no-readback")
        if args.compact:
            configurations.extend((
                ("gpu-final-fresh", (*profile, "--backend-vulkan"), profiled),
                ("gpu-compact-vulkan", (*profile, "--backend-vulkan", "--reuse-textures", "--small-texture-cache"), profiled),
                ("gpu-compact-proof", (*base, "--backend-vulkan", "--profile-memory", "--no-edit-readback", "--texture-proof", "--reuse-textures", "--small-texture-cache"), profiled),
            ))
        else:
            configurations.extend((
                ("gpu-fresh-vulkan", (*profile, "--backend-vulkan"), profiled),
                ("gpu-reuse-vulkan", (*profile, "--backend-vulkan", "--reuse-textures"), profiled),
                ("gpu-decode-vulkan", (*profile, "--backend-vulkan", "--no-original-upload"), profiled),
                ("gpu-reuse-dx12", (*profile, "--backend-dx12", "--reuse-textures"), profiled),
                ("gpu-drain-vulkan", (*profile, "--backend-vulkan", "--drain-uploads"), profiled),
                ("gpu-proof-fresh", (*base, "--backend-vulkan", "--profile-memory", "--no-edit-readback", "--texture-proof"), profiled),
                ("gpu-proof-reuse", (*base, "--backend-vulkan", "--profile-memory", "--no-edit-readback", "--texture-proof", "--reuse-textures"), profiled),
            ))
    if not args.diagnostics_only:
        compact = ("--small-texture-cache",) if args.compact else ()
        configurations.extend((
            ("gpu-full-fresh", (*base, "--backend-vulkan", "--no-readback"), normal),
            ("gpu-full-reuse", (*base, "--backend-vulkan", "--no-readback", "--reuse-textures", *compact), normal),
            ("gpu-full-reuse-readback", (*base, "--backend-vulkan", "--reuse-textures", *compact), normal),
        ))
    report = {"schema_version": 1, "cases": [], "note": "Instrumented builds are attribution only. Normal builds separately measure CPU/latency/memory."}
    path = ROOT / "artifacts/p0" / ("gpu-memory-" + suffix + ".json")
    for label, flags, executable in configurations:
        case = run_standard(environment, args.fixture, flags, label, executable=executable)
        case["metrics"].pop("samples")
        report["cases"].append(case)
        path.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print("Batch evidence: " + str(path), flush=True)


if __name__ == "__main__":
    main()
