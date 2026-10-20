"""Build/check the isolated egui candidate; optional internal render-target test."""

import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import time
import uuid

from check_p0 import ROOT, windows_linker
from p0_process_metrics import WindowsProcessMetrics, environment_info, summarize, cycle_statistics
from p0_workload import make_workload


MANIFEST = ROOT / "experiments/p0-egui/Cargo.toml"


def check_native_ui_features(dependency):
    expected = {"default_fonts", "system_fonts", "wgpu", "x11", "wayland", "clipboard"}
    if dependency["uses_default_features"] or set(dependency["features"]) != expected:
        raise RuntimeError("Review native UI features; system clipboard must be enabled for external copy/paste")


def check_audio_port_boundary(sources):
    for source in sources:
        code = re.sub(r"//[^\n]*", "", source.split("#[cfg(test)]", 1)[0])
        if re.search(r"\b(?:eframe|egui|wgpu|windows|p0_wasapi)\s*::|\bstd\s*::|include(?:_str|_bytes)?\s*!|#\[path", code):
            raise RuntimeError("Audio port/transport must use values, without native, UI, I/O or host clocks")


def check_audio_adapter_boundary(dependencies, sources):
    if (len(dependencies) != 1 or dependencies[0]["name"] != "windows"
            or dependencies[0]["req"] != "=0.62.2" or dependencies[0]["target"] != "cfg(windows)"
            or dependencies[0].get("path")):
        raise RuntimeError("Audio experiment may depend only on the pinned Windows platform bindings")
    if set(dependencies[0].get("features", [])) != {"Win32_Foundation", "Win32_Media_Audio", "Win32_System_Com", "Win32_System_Com_StructuredStorage", "Win32_System_Variant"}:
        raise RuntimeError("Review changes to the adapter's Windows API surface")
    for name, source in sources.items():
        code = re.sub(r"//[^\n]*", "", source)
        if re.search(r"\b(?:eframe|egui|image|serde_json)\s*::|\bstd\s*::\s*(?:fs|net)\b|use\s+std\s*::\s*\{[^;]*\b(?:fs|net)\b|include(?:_str|_bytes)?\s*!|#\[path", code, re.DOTALL):
            raise RuntimeError("Audio adapter must not depend on UI, projects or external source files")
        if name != "platform.rs" and re.search(r"\bunsafe\b", code):
            raise RuntimeError("Native pointer operations belong only in the isolated platform adapter")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-native", action="store_true")
    parser.add_argument("--run-standard", action="store_true")
    parser.add_argument("--run-comparison", action="store_true")
    parser.add_argument("--run-loading-comparison", action="store_true")
    parser.add_argument("--run-loader-tuning", action="store_true")
    parser.add_argument("--device-audio", action="store_true", help="Check/build optional Windows output adapter")
    parser.add_argument("--media", action="store_true", help="Check/build optional short-file decoding and timeline proof")
    parser.add_argument("--compressed", action="store_true", help="Check/build optional MP3 and Vorbis pull/output experiments")
    arguments = parser.parse_args()
    if arguments.compressed:
        arguments.media = True
    check_audio_port_boundary([(MANIFEST.parent / "src" / name).read_text(encoding="utf-8")
                               for name in ("audio_output.rs", "audio_transport.rs")])
    environment = dict(os.environ)
    private_bin = ROOT / ".p0-tools/rustup/toolchains/1.95.0-x86_64-pc-windows-msvc/bin"
    if os.name == "nt" and (private_bin / "cargo.exe").is_file():
        environment["PATH"] = str(private_bin) + os.pathsep + environment.get("PATH", "")
        environment["RUSTUP_HOME"] = str(ROOT / ".p0-tools/rustup")
        environment["CARGO_HOME"] = str(ROOT / ".p0-tools/cargo")
        cargo = str(private_bin / "cargo.exe")
    else:
        cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("Rust 1.95 with rustfmt/clippy and a native linker is required")
    metadata = json.loads(subprocess.check_output(
        [cargo, "metadata", "--manifest-path", str(MANIFEST), "--no-deps",
         "--offline", "--locked", "--format-version", "1"],
        cwd=ROOT, env=environment, text=True,
    ))
    dependencies = metadata["packages"][0]["dependencies"]
    check_native_ui_features(next(item for item in dependencies if item["name"] == "eframe"))
    if {item["name"] for item in dependencies} != {"eframe", "image", "p0-preview-contract", "p0-wasapi", "p0-media", "serde_json", "stats_alloc", "pollster"}:
        raise RuntimeError("Review changes to the candidate dependency boundary")
    heap_dependency = next(item for item in dependencies if item["name"] == "stats_alloc")
    if not heap_dependency["optional"] or metadata["packages"][0]["features"].get("default"):
        raise RuntimeError("Heap instrumentation must stay optional and disabled by default")
    local = [item for item in dependencies if item.get("path")]
    if {Path(item["path"]).resolve() for item in local} != {(ROOT / "experiments/p0-preview-contract").resolve(), (ROOT / "experiments/p0-wasapi").resolve(), (ROOT / "experiments/p0-media").resolve()}:
        raise RuntimeError("Only the independent P0 contract, media and optional Windows output adapter may be local dependencies")
    media_dependency = next(item for item in dependencies if item["name"] == "p0-media")
    if not media_dependency["optional"] or metadata["packages"][0]["features"].get("media-proof") != ["device-audio", "dep:p0-media"]:
        raise RuntimeError("File decoding must remain an explicit optional P0 feature")
    if metadata["packages"][0]["features"].get("compressed-proof") != ["media-proof", "p0-media/compressed"]:
        raise RuntimeError("Compressed decoding must remain a separate optional P0 feature")
    audio_dependency = next(item for item in dependencies if item["name"] == "p0-wasapi")
    if not audio_dependency["optional"] or audio_dependency["target"] != "cfg(windows)":
        raise RuntimeError("Device output adapter must remain optional and Windows-only")
    if 'unsafe_code = "forbid"' not in MANIFEST.read_text(encoding="utf-8"):
        raise RuntimeError("UI and pure preview experiment must keep unsafe forbidden")
    linker = windows_linker(environment)
    common = ["--manifest-path", str(MANIFEST), "--offline", "--locked", "-j", "2"]
    if arguments.device_audio or arguments.media:
        common += ["--features", "compressed-proof" if arguments.compressed else "media-proof" if arguments.media else "device-audio"]
        adapter = ROOT / "experiments/p0-wasapi"
        adapter_metadata = json.loads(subprocess.check_output([cargo, "metadata", "--manifest-path", str(adapter / "Cargo.toml"), "--no-deps", "--offline", "--locked", "--format-version", "1"], cwd=ROOT, env=environment, text=True))
        check_audio_adapter_boundary(adapter_metadata["packages"][0]["dependencies"], {p.name: p.read_text(encoding="utf-8") for p in (adapter / "src").glob("*.rs")})
        adapter_common = ["--manifest-path", str(adapter / "Cargo.toml"), "--offline", "--locked", "-j", "2"]
        for command in ([cargo, "fmt", "--manifest-path", str(adapter / "Cargo.toml"), "--", "--check"],
                        [cargo, *linker, "clippy", *adapter_common, "--all-targets", "--", "-D", "warnings"],
                        [cargo, *linker, "test", *adapter_common, "--release"]):
            subprocess.run(command, cwd=ROOT, env=environment, check=True)
    commands = [
        [cargo, "fmt", "--manifest-path", str(MANIFEST), "--", "--check"],
        [cargo, *linker, "clippy", *common, "--all-targets", "--", "-D", "warnings"],
        [cargo, *linker, "test", *common, "--release"],
        [cargo, *linker, "build", *common, "--release"],
    ]
    for command in commands:
        subprocess.run(command, cwd=ROOT, env=environment, check=True)
    if arguments.run_comparison:
        run_comparison(environment)
    if arguments.run_loading_comparison:
        run_comparison(environment, loading=True)
    if arguments.run_loader_tuning:
        run_loader_tuning(environment)
    if not arguments.run_native and not arguments.run_standard:
        return
    if arguments.run_standard:
        run_standard(environment)
    if not arguments.run_native:
        return
    output = ROOT / "artifacts/p0" / ("egui-" + uuid.uuid4().hex[:8])
    executable = MANIFEST.parent / "target/release" / ("p0-egui.exe" if os.name == "nt" else "p0-egui")
    rejected = ROOT / ("p0-output-escape-" + uuid.uuid4().hex[:8])
    rejection = subprocess.run([str(executable), "--self-test", str(rejected)], cwd=ROOT, env=environment, capture_output=True)
    if rejection.returncode == 0 or rejected.exists():
        raise RuntimeError("Candidate output boundary did not reject a path outside artifacts/p0")
    executable_sha256 = hashlib.sha256(executable.read_bytes()).hexdigest()
    samples = []
    with subprocess.Popen([str(executable), "--self-test", str(output)], cwd=ROOT, env=environment) as process:
        deadline = time.monotonic() + 40
        shell = shutil.which("pwsh") or shutil.which("powershell") if os.name == "nt" else None
        while process.poll() is None:
            if time.monotonic() > deadline:
                # Only the process this runner created is stopped; no other apps are targeted.
                process.kill()
                process.wait()
                raise RuntimeError("Candidate self-test exceeded its 40-second limit")
            if shell:
                command = f"Get-Process -Id {process.pid} -ErrorAction SilentlyContinue | Select-Object WorkingSet64,PrivateMemorySize64,CPU | ConvertTo-Json -Compress"
                result = subprocess.run([shell, "-NoProfile", "-Command", command], capture_output=True, text=True)
                if result.stdout.strip():
                    samples.append(json.loads(result.stdout))
            time.sleep(0.5)
        if process.returncode:
            raise RuntimeError(f"Native candidate exited with {process.returncode}")
    result = json.loads((output / "result.json").read_text(encoding="utf-8"))
    (output / "process-samples.json").write_text(json.dumps({
        "schema_version": 1, "samples": samples,
        "executable_sha256": executable_sha256,
        "note": "Sparse process working-set/private-byte samples, excludes GPU memory; not the full project workload",
    }, indent=2), encoding="utf-8")
    print("Native evidence: " + str(output), flush=True)
    if result["status"] != "passed":
        raise RuntimeError("Native render contract failed: " + str(result["error"]))


def evaluate_budgets(result, summary):
    idle = next((s for s in summary if s["phase"] == "Idle"), {})
    closed = next((s for s in summary if s["phase"] == "Closed"), {})
    standard_rows = [s for s in summary if s["phase"] in ("InitialLoadStart", "Loading", "Warm", "Edit", "Before", "After", "IdleSettle", "Idle", "RapidSwitch", "Cycle", "RoundEnd")]
    ws_peak = max((s["working_set_bytes_max"] or 0 for s in standard_rows), default=0)
    empty = next((s for s in summary if s["phase"] == "Empty"), {})
    frame_stats = result["eframe_cpu_frame_ms_during_edits"] or {}
    edit_stats = result["edit_to_gpu_readback_ms"] or {}
    closed_phase = next((p for p in result["phases"] if p["phase"] == "Closed"), {})
    final = result["phases"][-1] if result["phases"] else {}
    checks = {
        "empty_working_set_le_200_mib": 0 < (empty.get("working_set_bytes_max") or 0) <= 200 * 1024 * 1024,
        "standard_working_set_le_512_mib": 0 < ws_peak <= 512 * 1024 * 1024,
        "paused_cpu_le_2_percent_one_core": idle.get("cpu_one_logical_core_percent", 100) <= 2,
        "paused_observation_at_least_60_seconds": idle.get("duration_seconds", 0) >= 60,
        "paused_no_continuous_redraw": result["idle_ui_frames"] <= 1,
        "frame_cpu_p95_le_16_7_ms": frame_stats.get("p95", 100) <= 16.7,
        "edit_readback_p95_le_50_ms": edit_stats.get("p95", 100) <= 50 if result["per_edit_gpu_readback"] else None,
        "at_least_300_edit_samples": result["edit_count"] >= 300,
        "at_least_300_readback_samples": edit_stats.get("count", 0) >= 300 if result["per_edit_gpu_readback"] else None,
        "pause_png_refresh_verified": result["pause_png_refresh_verified"],
        "twenty_resource_rounds": result["cycles_completed"] == 20,
        "only_active_originals_initially_decoded": result["first_document_original_decode_count"] == 10,
        "closed_observation_at_least_60_seconds": closed.get("duration_seconds", 0) >= 60,
        "closed_cpu_le_2_percent_one_core": closed.get("cpu_one_logical_core_percent", 100) <= 2,
        "closed_no_continuous_redraw": bool(closed_phase) and final.get("ui_frames", 100) - closed_phase.get("ui_frames", 0) <= 1,
        "original_decoded_images_released": final.get("original_decoded_images_live") == 0,
        "owned_textures_released_from_renderer": final.get("owned_textures_in_renderer") == 0,
    }
    if result.get("async_png_loading"):
        loader = result.get("loader") or {}
        checks.update({
            "loading_frame_cpu_p95_le_16_7_ms": (result.get("loading_cpu_frame_ms") or {}).get("p95", 100) <= 16.7,
            "per_frame_upload_bytes_bounded": 0 < result.get("max_upload_bytes_in_one_frame", 0) <= 8 * 1024 * 1024,
            "per_frame_upload_count_bounded": 0 < result.get("max_upload_count_in_one_frame", 0) <= 2,
            "obsolete_decode_result_rejected": loader.get("stale_results_rejected", 0) >= 1,
            "rapid_switch_winner_verified": result.get("rapid_switch_winner_verified"),
            "worker_idle_at_finish": loader.get("idle_at_finish") is True,
        })
    return checks, ws_peak, idle


def run_standard(environment, fixture=None, flags=(), label="standard", executable=None, observe_modules=False):
    if os.name != "nt":
        raise RuntimeError("Standard process/GPU metrics runner is currently verified only on Windows")
    suffix = uuid.uuid4().hex[:8]
    if fixture is None:
        fixture = ROOT / "artifacts/p0" / ("standard-fixture-" + suffix)
        print("Generating 100 original PNGs: " + str(fixture), flush=True)
        inventory = make_workload(fixture)
    else:
        fixture = Path(fixture).resolve()
        if (ROOT / "artifacts/p0").resolve() not in fixture.parents:
            raise RuntimeError("Comparison fixture must be inside artifacts/p0")
        inventory = json.loads((fixture / "inventory.json").read_text(encoding="utf-8"))
    output = ROOT / "artifacts/p0" / (label + "-" + suffix)
    executable = executable or MANIFEST.parent / "target/release" / ("standard-load.exe" if os.name == "nt" else "standard-load")
    executable_hash = hashlib.sha256(executable.read_bytes()).hexdigest()
    source_hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                     for p in sorted((MANIFEST.parent / "src").rglob("*.rs"))}
    harness_hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                      for p in (Path(__file__).resolve(), ROOT / "tools/p0_process_metrics.py", ROOT / "tools/p0_workload.py")}
    if observe_modules:
        from p0_modules import module_snapshot
        harness_hashes["tools/p0_modules.py"] = hashlib.sha256((ROOT / "tools/p0_modules.py").read_bytes()).hexdigest()
    rejected = ROOT / ("p0-output-escape-" + suffix)
    rejection = subprocess.run([str(executable), str(fixture), str(rejected)], cwd=ROOT, env=environment, capture_output=True)
    if rejection.returncode == 0 or rejected.exists():
        raise RuntimeError("Standard benchmark output boundary failed")
    samples = []
    print("Running " + label + ": " + " ".join(flags), flush=True)
    vm_samples = []
    observed_modules = None
    profile_memory = "--profile-memory" in flags
    with subprocess.Popen([str(executable), str(fixture), str(output), *flags], cwd=ROOT, env=environment) as process:
        started = time.monotonic()
        runner_unix_anchor = time.time()
        metrics = WindowsProcessMetrics(process)
        next_vm = started
        try:
            while process.poll() is None:
                if time.monotonic() - started > 260:
                    process.kill()
                    process.wait()
                    raise RuntimeError("Runner stopped its own benchmark after 260 seconds")
                sample = metrics.sample()
                if sample:
                    sample["runner_elapsed_seconds"] = sample.pop("monotonic_seconds") - started
                    samples.append(sample)
                if observe_modules and observed_modules is None and time.monotonic() - started >= 2:
                    observed_modules = module_snapshot(process)
                    observed_modules["runner_elapsed_seconds"] = time.monotonic() - started
                if profile_memory and time.monotonic() >= next_vm:
                    snapshot = metrics.virtual_memory_snapshot()
                    snapshot["runner_elapsed_seconds"] = snapshot.pop("monotonic_seconds") - started
                    vm_samples.append(snapshot)
                    next_vm = time.monotonic() + 0.5
                time.sleep(0.1)
        finally:
            metrics.close()
        if process.returncode:
            raise RuntimeError(f"Standard benchmark exited with {process.returncode}")
    result = json.loads((output / "result.json").read_text(encoding="utf-8"))
    for entry in inventory["files"]:
        if hashlib.sha256((fixture / entry["path"]).read_bytes()).hexdigest() != entry["sha256"]:
            raise RuntimeError("Read-only benchmark changed its input: " + entry["path"])
    # Synchronize runner monotonic timestamps to the application's own phase clock.
    # Anchor both monotonic clocks once using UTC; sample intervals use monotonic time.
    # UTC adjustment during startup would invalidate phase labeling and requires rerun.
    anchor = json.loads((output / "clock-anchor.json").read_text(encoding="utf-8"))
    offset = anchor["unix_seconds"] - runner_unix_anchor
    for sample in samples:
        sample["elapsed_seconds"] = sample["runner_elapsed_seconds"] - offset
    for snapshot in vm_samples:
        snapshot["elapsed_seconds"] = snapshot["runner_elapsed_seconds"] - offset
    if profile_memory:
        (output / "virtual-memory.json").write_text(json.dumps({"schema_version": 1, "snapshots": vm_samples,
            "note": "VirtualQueryEx metadata for this runner-owned PID only; non-atomic scans; no page contents read. Region types do not identify allocation owners or equal PrivateUsage."}, indent=2), encoding="utf-8")
    summary = summarize(samples, result["phases"])
    checks, ws_peak, idle = evaluate_budgets(result, summary)
    record = {"schema_version": 4, "environment": environment_info(), "process_id": process.pid, "arguments": list(flags),
              "module_observation": observed_modules,
              "child_graphics_environment": {key: environment.get(key) for key in ("VK_LOADER_LAYERS_DISABLE", "VK_LOADER_LAYERS_ENABLE", "VK_INSTANCE_LAYERS")},
              "executable_sha256": executable_hash, "fixture_inventory_sha256": hashlib.sha256((fixture / "inventory.json").read_bytes()).hexdigest(),
              "fixture": str(fixture.relative_to(ROOT)), "fixture_file_count": len(inventory["files"]),
              "sample_interval_seconds": 0.1, "samples": samples, "phase_summary": summary, "budget_checks": checks,
              "cycle_statistics": cycle_statistics(samples, result["phases"]),
              "gpu_counter_errors": metrics.gpu_errors,
              "cargo_lock_sha256": hashlib.sha256((MANIFEST.parent / "Cargo.lock").read_bytes()).hexdigest(),
              "source_sha256": source_hashes,
              "harness_sha256": harness_hashes,
              "measurement": "GetProcessMemoryInfo/GetProcessTimes for runner-owned PID; PDH GPU Process Memory per PID across adapters; no window capture. Working set and private bytes are different metrics. GPU counters are OS accounting, not allocation payload.",
              "limits": ["Visible native test window, vsync disabled; script targets 60 Hz. Edit-to-readback includes queue, GPU execution, copy and callback; not physical input/display latency", "GPU execution timestamps unavailable", "Single run on one Windows machine; no cross-platform or full-product budget conclusion"]}
    (output / "process-samples.json").write_text(json.dumps(record, indent=2), encoding="utf-8")
    print("Standard evidence: " + str(output), flush=True)
    print(json.dumps({"functional_status": result["status"], "budget_checks": checks,
                      "standard_working_set_peak_bytes": ws_peak,
                      "idle": idle, "process_sample_count": len(samples)}, indent=2), flush=True)
    if result["status"] != "passed":
        raise RuntimeError("Standard workload failed: " + str(result["error"]))
    return {"label": label, "output": str(output.relative_to(ROOT)), "result": result, "metrics": record}


def run_comparison(environment, loading=False):
    suffix = uuid.uuid4().hex[:8]
    fixture = ROOT / "artifacts/p0" / ("comparison-fixture-" + suffix)
    make_workload(fixture)
    cases = []
    configurations = (
        ("compare-performance-none", ("--no-readback",)),
        ("compare-memory-none", ("--memory-usage", "--no-readback")),
        ("compare-memory-readback", ("--memory-usage",)),
    ) if not loading else (
        ("load-sync-none", ("--memory-usage", "--no-readback")),
        ("load-async-none", ("--memory-usage", "--no-readback", "--async-load")),
        ("load-async-readback", ("--memory-usage", "--async-load")),
    )
    for label, flags in configurations:
        cases.append(run_standard(environment, fixture=fixture, flags=flags, label=label))
    path = ROOT / "artifacts/p0" / (("loading-comparison-" if loading else "comparison-") + suffix + ".json")
    # Per-sample data stays in each case; a compact comparison retains identities and summaries.
    for case in cases:
        case["metrics"].pop("samples")
    with path.open("x", encoding="utf-8") as stream:
        json.dump({"schema_version": 1, "fixture": str(fixture.relative_to(ROOT)), "cases": cases}, stream, indent=2)
    print("Comparison evidence: " + str(path), flush=True)


def run_loader_tuning(environment):
    suffix = uuid.uuid4().hex[:8]
    fixture = ROOT / "artifacts/p0" / ("tuning-fixture-" + suffix)
    make_workload(fixture)
    common = ("--memory-usage", "--async-load")
    cases = []
    for label, flags in (
        ("tuning-immediate", (*common, "--no-readback")),
        ("tuning-paced", (*common, "--no-readback", "--paced-loading")),
        ("tuning-paced-readback", (*common, "--paced-loading")),
        ("memory-upload-draw", (*common, "--no-readback", "--paced-loading", "--profile-memory")),
        ("memory-upload-only", (*common, "--no-readback", "--paced-loading", "--profile-memory", "--no-original-draw")),
        ("memory-decode-only", (*common, "--no-readback", "--paced-loading", "--profile-memory", "--no-original-upload")),
    ):
        case = run_standard(environment, fixture=fixture, flags=flags, label=label)
        case["metrics"].pop("samples")
        cases.append(case)
    path = ROOT / "artifacts/p0" / ("loader-tuning-" + suffix + ".json")
    with path.open("x", encoding="utf-8") as stream:
        json.dump({"schema_version": 1, "fixture": str(fixture.relative_to(ROOT)), "cases": cases}, stream, indent=2)
    print("Loader tuning evidence: " + str(path), flush=True)


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print("P0 GUI checks failed: " + str(error), file=sys.stderr)
        sys.exit(1)
