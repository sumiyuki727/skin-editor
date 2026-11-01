"""Offline desktop format, dependency, Clippy and headless behavior checks; GPU opt-in."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from check_core import ROOT, tool_environment, linker_arguments
from check_desktop_architecture import validate_metadata, validate_sources

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolchain-dir")
    parser.add_argument("--architecture-only", action="store_true")
    parser.add_argument("--run-native", action="store_true", help="Own Vulkan/egui readback; no game or OS capture")
    parser.add_argument("--run-audio-ui", action="store_true", help="Own GPU editor readback with explicit native WAV audition")
    parser.add_argument("--run-audio", action="store_true", help="Explicit low-volume self-authored WAV native-output diagnostic")
    parser.add_argument("--run-audio-supply", action="store_true", help="Independent audio supply and finite background-reference diagnostic")
    parser.add_argument("--run-audio-load", action="store_true", help="Release native latency/32-voice load and owned Windows process metrics, two runs")
    parser.add_argument("--run-audio-edit", action="store_true", help="Release 300 edits/warm WAV preparation and owned Windows process metrics, two runs")
    parser.add_argument("--run-audio-startup", action="store_true", help="Two release rounds of 20 native empty-output starts; Windows owned process metrics")
    parser.add_argument("--run-combined", action="store_true", help="Two ordinary 300 s graphics/native-audio release loads with owned PID CPU/memory/GPU metrics")
    parser.add_argument("--memory-usage", action="store_true", help="MemoryUsage for explicit native GPU or combined diagnostics; production unchanged")
    parser.add_argument("--output", default="artifacts/p2/native-proof")
    args = parser.parse_args()
    if args.memory_usage and (not (args.run_native or args.run_combined) or args.run_audio_ui):
        parser.error('--memory-usage requires --run-native or --run-combined and cannot select --run-audio-ui')
    destination = (ROOT / args.output).resolve()
    if (args.run_native or args.run_audio_ui or args.run_audio or args.run_audio_supply or args.run_audio_load or args.run_audio_edit or args.run_audio_startup or args.run_combined) and (destination.exists() or ROOT / "artifacts" not in destination.parents):
        raise RuntimeError("Native output must be a new directory inside artifacts")
    cargo, env = tool_environment(args.toolchain_dir)
    common = ["--manifest-path", str(ROOT / "desktop/Cargo.toml"), "--offline", "--locked"]
    metadata = json.loads(subprocess.check_output([cargo, "metadata", *common, "--format-version", "1", "--all-features"], env=env, cwd=ROOT, text=True, encoding="utf-8"))
    errors = validate_metadata(metadata, ROOT) + validate_sources(ROOT)
    if errors:
        raise RuntimeError("\n".join(errors))
    print("PASS: explicit desktop edges; core/UI/simulator contain no native GPU writer", flush=True)
    if args.architecture_only:
        return
    linker = linker_arguments(env)
    for command in [
        [cargo, "fmt", "--manifest-path", str(ROOT / "desktop/Cargo.toml"), "--all", "--", "--check"],
        [cargo, *linker, "clippy", *common, "--workspace", "--all-targets", "-j", "2", "--", "-D", "warnings"],
        [cargo, *linker, "test", *common, "--workspace", "-j", "2"],
        [cargo, *linker, "clippy", *common, "--workspace", "--all-targets", "--all-features", "-j", "2", "--", "-D", "warnings"],
        [cargo, *linker, "test", *common, "--workspace", "--all-features", "-j", "2"],
        [cargo, *linker, "build", *common, "-p", "skin-editor-desktop", "-j", "2"],
    ]:
        subprocess.run(command, cwd=ROOT, env=env, check=True)
    if args.run_native or args.run_audio_ui:
        binary = ROOT / "desktop/target/debug" / ("skin-editor-desktop.exe" if os.name == "nt" else "skin-editor-desktop")
        subprocess.run([str(binary), "--verify-audio-ui" if args.run_audio_ui else "--verify-gpu-memory" if args.memory_usage else "--verify-gpu", str(destination)], cwd=ROOT, env=env, check=True, timeout=120)
        report = json.loads((destination / "report.json").read_text(encoding="utf-8"))
        if report["max_channel_error"] > 2 or report["unchanged_uploads"] != 0 or report.get('gpu_memory_usage_hint') is not args.memory_usage:
            raise RuntimeError("GPU pixel/upload check failed")
        if args.run_audio_ui and (not report["audio_ui_native_audition"] or report["audio_ui_submitted_nonzero_samples"] == 0 or not report["audio_ui_library_readonly"] or report["audio_ui_library_nonzero_samples"] == 0 or report["monitor_volume_permille"] != 500): raise RuntimeError("audio UI native audition failed")
    if args.run_audio:
        binary = ROOT / "desktop/target/debug" / ("skin-editor-desktop.exe" if os.name == "nt" else "skin-editor-desktop")
        audio_destination = destination / "audio" if args.run_native or args.run_audio_ui else destination
        subprocess.run([str(binary), "--verify-audio", str(audio_destination)], cwd=ROOT, env=env, check=True, timeout=30)
        report = json.loads((audio_destination / "report.json").read_text(encoding="utf-8"))
        if report["pcm_final_bytes"] != 0 or report["nonzero_samples"] == 0:
            raise RuntimeError("native audio output/release check failed")
    if args.run_audio_supply:
        binary = ROOT / "desktop/target/debug" / ("skin-editor-desktop.exe" if os.name == "nt" else "skin-editor-desktop")
        supply_destination = destination / "supply" if args.run_native or args.run_audio_ui or args.run_audio else destination
        subprocess.run([str(binary), "--verify-audio-supply", str(supply_destination)], cwd=ROOT, env=env, check=True, timeout=30)
        report = json.loads((supply_destination / "report.json").read_text(encoding="utf-8"))
        if report["pcm_final_bytes"] != 0 or report["background_position_gaps"] != 0 or report["background_submitted_frames"] == 0 or report["steady_empty_frames"] != 0:
            raise RuntimeError("independent audio supply/continuity/release check failed")
    if args.run_audio_load:
        subprocess.run([cargo, *linker, "build", *common, "-p", "skin-editor-desktop", "--release", "-j", "2"], cwd=ROOT, env=env, check=True)
        binary = ROOT / "desktop/target/release/skin-editor-desktop.exe"
        load_destination = destination / "load" if args.run_native or args.run_audio_ui or args.run_audio or args.run_audio_supply else destination
        subprocess.run([sys.executable, str(ROOT / "tools/measure_audio_load.py"), "--binary", str(binary), "--output", str(load_destination)], cwd=ROOT, env=env, check=True, timeout=180)
    if args.run_audio_edit:
        subprocess.run([cargo, *linker, "build", *common, "-p", "skin-editor-desktop", "--release", "-j", "2"], cwd=ROOT, env=env, check=True)
        binary = ROOT / "desktop/target/release/skin-editor-desktop.exe"
        edit_destination = destination / "edit" if args.run_native or args.run_audio_ui or args.run_audio or args.run_audio_supply or args.run_audio_load else destination
        subprocess.run([sys.executable, str(ROOT / "tools/measure_audio_edit.py"), "--binary", str(binary), "--output", str(edit_destination)], cwd=ROOT, env=env, check=True, timeout=180)
    if args.run_audio_startup:
        subprocess.run([cargo, *linker, "build", *common, "-p", "skin-editor-desktop", "--release", "-j", "2"], cwd=ROOT, env=env, check=True)
        binary = ROOT / "desktop/target/release/skin-editor-desktop.exe"
        startup_destination = destination / "startup" if args.run_native or args.run_audio_ui or args.run_audio or args.run_audio_supply or args.run_audio_load or args.run_audio_edit else destination
        subprocess.run([sys.executable, str(ROOT / "tools/measure_audio_startup.py"), "--binary", str(binary), "--output", str(startup_destination)], cwd=ROOT, env=env, check=True, timeout=180)
    if args.run_combined:
        subprocess.run([cargo, *linker, "build", *common, "-p", "skin-editor-desktop", "--release", "-j", "2"], cwd=ROOT, env=env, check=True)
        binary = ROOT / "desktop/target/release/skin-editor-desktop.exe"
        combined_destination = destination / "combined" if any((args.run_native,args.run_audio_ui,args.run_audio,args.run_audio_supply,args.run_audio_load,args.run_audio_edit,args.run_audio_startup)) else destination
        subprocess.run([sys.executable,str(ROOT/"tools/measure_combined.py"),"--binary",str(binary),"--output",str(combined_destination),*(["--memory-usage"] if args.memory_usage else [])],cwd=ROOT,env=env,check=True,timeout=850)
    print("PASS: desktop checks completed" + ("; native proof in " + str(destination) if args.run_native or args.run_audio_ui or args.run_combined else "; GPU not opened"), flush=True)

if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print("Desktop checks failed: " + str(error), file=sys.stderr)
        sys.exit(1)
