"""Bounded WAV/circle timeline checks; opt-in self-authored native audio proof."""
import argparse
import ctypes
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import uuid
import wave

from check_p0 import ROOT, windows_linker
from check_p0_transport import sources
from p0_process_metrics import environment_info

MANIFEST = ROOT/"experiments/p0-media/Cargo.toml"
WAV = "音频 日本語.wav"
CHECKS = {"device_content_clock", "paused_timeline_and_pcm_frozen", "forward_seek_skips_events", "backward_seek_re_evaluates",
          "relocation_waits_for_ack", "pause_and_held_input_isolation", "out_of_content_seek_rejected"}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_boundary(dependencies, code):
    if (len(dependencies) != 1 or dependencies[0]["name"] != "symphonia" or dependencies[0]["req"] != "=0.6.1"
            or dependencies[0]["uses_default_features"] or set(dependencies[0]["features"]) != {"wav", "pcm"}
            or dependencies[0].get("path")):
        raise RuntimeError("P0 media may use only the pinned WAV/PCM decoder")
    for name, source in code.items():
        body = re.sub(r"//[^\n]*", "", source.split("#[cfg(test)]", 1)[0])
        if re.search(r"\b(?:eframe|egui|wgpu|windows|p0_wasapi)\s*::|\bstd\s*::\s*(?:fs|net|time)|\b(?:File|Instant|SystemTime)\b|\bunsafe\b|include(?:_str|_bytes)?\s*!|#\[path", body):
            raise RuntimeError("Media values must not access UI, native output, files, wall clocks or external sources")
        if Path(name).name == "chart.rs" and re.search(r"\b(?:symphonia|crate::audio)\b", body):
            raise RuntimeError("Timeline must not depend on decoding")


def make_fixture(output):
    output.mkdir(exist_ok=False)
    rate, frames = 48000, 48000*3
    pcm = bytearray()
    for n in range(frames):
        t = n/rate; local = t % .5
        env = min(local/.005, (.2-local)/.005, 1) if local < .2 else 0
        env = max(env, 0)
        pcm.extend(struct.pack("<hh", round(.5*env*math.sin(math.tau*440*t)*32767), round(-.25*env*math.sin(math.tau*660*t)*32767)))
    with wave.open(str(output/WAV), "wb") as f:
        f.setparams((2, 2, rate, frames, "NONE", "not compressed")); f.writeframes(pcm)
    objects = "\n".join(f"{40+i*40},{60+i*20},{t},1,0,0:0:0:0:" for i, t in enumerate([250, 500, 500, 900, 1400, 2100]))
    (output/"timeline.osu").write_text(f"osu file format v14\n[General]\nAudioFilename: {WAV}\nMode: 0\nAudioLeadIn: 0\n[HitObjects]\n{objects}\n", encoding="utf-8")
    manifest = {"schema_version": 1, "generator": "self_authored_pcm16_circle_fixture_v1", "files": {p.name: digest(p) for p in [output/WAV, output/"timeline.osu"]},
                "rate": rate, "channels": 2, "frames": frames, "audio_file": WAV, "object_times_ms": [250, 500, 500, 900, 1400, 2100]}
    (output/"manifest.json").write_text(json.dumps(manifest, indent=2)+"\n", encoding="utf-8")
    return manifest


class Reference:
    def __init__(self, fixture):
        with wave.open(str(fixture/WAV), "rb") as f:
            if f.getparams()[:3] != (2, 2, 48000):
                raise RuntimeError("Unexpected self-authored WAV format")
            self.frames = f.getnframes(); self.data = f.readframes(self.frames)
        self.cache = {}

    def packet(self, start, frames):
        key = (start, frames)
        if key not in self.cache:
            gain = ctypes.c_float(.04).value; values = []; raw = bytearray()
            for i in range(frames):
                for ch in range(2):
                    v = struct.unpack_from("<h", self.data, ((start+i)*2+ch)*2)[0]/32768 if start+i < self.frames else 0
                    v = ctypes.c_float(v*gain).value; values.append(v); raw.extend(struct.pack("<f", v))
            checksum = 0xcbf29ce484222325
            for byte in raw:
                checksum = ((checksum ^ byte)*0x100000001b3) & ((1 << 64)-1)
            probe = values[::2][:8]; probe += [0]*(8-len(probe))
            self.cache[key] = {"source_frame": start, "frames": frames, "hash": checksum, "peak": max(map(abs, values)), "nonzero_samples": sum(v != 0 for v in values), "probe": probe}
        return self.cache[key]

    def check_packet(self, p):
        if (type(p.get("source_frame")) is not int or p["source_frame"] < 0 or type(p.get("frames")) is not int
                or not 0 < p["frames"] <= 1024*1024//8):
            raise RuntimeError("Unbounded file PCM packet")
        expected = self.packet(p["source_frame"], p["frames"])
        if (p["hash"] != expected["hash"] or p["nonzero_samples"] != expected["nonzero_samples"]
                or not math.isfinite(p["peak"]) or abs(p["peak"]-expected["peak"]) > 1e-7 or len(p["probe"]) != 8
                or any(not math.isfinite(v) or abs(v-w) > 1e-7 for v, w in zip(p["probe"], expected["probe"]))):
            raise RuntimeError("Native packet does not match independently decoded source WAV and gain")


def validate(result, fixture):
    if (result.get("schema_version") != 1 or result.get("status") != "passed" or result.get("source") != "self_authored_wav_and_v14_circle_fixture"
            or set(result.get("checks", {})) != CHECKS or any(v is not True for v in result["checks"].values())
            or result.get("system_capture_used") is not False or result.get("microphone_or_loopback_used") is not False
            or result.get("accepted_inputs") != 2 or not math.isfinite(result.get("decode_ms", -1)) or result["decode_ms"] < 0):
        raise RuntimeError("Native media proof has wrong source, missing checks or invalid input/measurement claims")
    ref = Reference(fixture); d = result["decoded"]
    if (d != {"rate": 48000, "channels": 2, "frames": ref.frames, "duration_us": 3000000, "payload_bytes": ref.frames*8, "gain": .04}
            or any(result["audio_format"][k] != v for k, v in {"rate": 48000, "channels": 2, "bits": 32}.items())
            or result["event_crossings"] != {"0": [1, 2, 3], "1": [5], "2": [1, 2, 3]}):
        raise RuntimeError("File format, PCM payload or timeline event crossings differ")
    rows = result["fresh_samples"]
    if not 3 <= len(rows) <= 2048 or any(b["sequence"] <= a["sequence"] for a, b in zip(rows, rows[1:])):
        raise RuntimeError("Missing or stale native media observations")
    submitted = {}; previous_time = {}
    for r in rows:
        epoch = r["epoch"]; pcm = r["pcm"]; base = {0: 0, 1: 1100000, 2: 100000}.get(epoch)
        if (base is None or r["device_us"] < 0 or r["simulation_us"] != base+r["device_us"] or pcm["requested_us"] != base
                or pcm["source_origin_frame"] != base*48000//1000000 or pcm["submitted_frames"] < submitted.get(epoch, 0)
                or r["simulation_us"] < previous_time.get(epoch, base) or pcm["submitted_frames"]+1 < r["device_us"]*48000//1000000):
            raise RuntimeError("Content origin, submission cursor and device consumption are conflated")
        submitted[epoch] = pcm["submitted_frames"]; previous_time[epoch] = r["simulation_us"]
        if pcm["submitted_frames"]:
            if pcm["first"]["source_frame"] != pcm["source_origin_frame"] or pcm["last"]["source_frame"]+pcm["last"]["frames"] != pcm["source_origin_frame"]+pcm["submitted_frames"]:
                raise RuntimeError("File source packets are discontinuous")
            for p in [pcm["first"], pcm["last"]]:
                ref.check_packet(p)
        elif pcm["first"] is not None or pcm["last"] is not None or pcm["packets"] != 0:
            raise RuntimeError("Paused new stream already has PCM submissions")
    pauses = result["paused_checks"]
    if len(pauses) != 3 or [r["epoch"] for r in pauses] != [0, 1, 2]:
        raise RuntimeError("Missing native pause coverage")
    for p in pauses:
        group = [r for r in rows if r["epoch"] == p["epoch"] and r["phase"] == "Paused" and r["simulation_us"] == p["position_us"]]
        if (not math.isfinite(p["wall_ms"]) or p["wall_ms"] < 300 or p["fresh_samples"] < 3 or len(group) < p["fresh_samples"]+1
                or p["frame"]["position_us"] != p["position_us"] or any(r["device_us"] != p["device_us"] for r in group)
                or len({r["pcm"]["submitted_frames"] for r in group}) != 1):
            raise RuntimeError("Pause did not freeze native output, submissions and timeline")
    relocations = result["relocations"]
    if len(relocations) != 2 or [(p["epoch"], p["position_us"]) for p in relocations] != [(1, 1100000), (2, 100000)]:
        raise RuntimeError("Both content directions were not relocated")
    for p in relocations:
        if p["frame"]["position_us"] != p["position_us"] or p["pcm"]["submitted_frames"] != 0 or p["pcm"]["requested_us"] != p["position_us"]:
            raise RuntimeError("Relocation published wrong position or played before explicit resume")
    if result["final_frame"]["position_us"] != rows[-1]["simulation_us"]:
        raise RuntimeError("Final timeline is not driven by the last confirmed audio position")


def check_core(environment, cargo, linker):
    meta = json.loads(subprocess.check_output([cargo, "metadata", "--manifest-path", str(MANIFEST), "--offline", "--locked", "--no-deps", "--format-version", "1"], env=environment, text=True))
    src = MANIFEST.parent / "src"
    check_boundary(meta["packages"][0]["dependencies"], {p.relative_to(src).as_posix(): p.read_text(encoding="utf-8")
                   for p in src.rglob("*.rs") if "bin" not in p.relative_to(src).parts})
    if 'unsafe_code = "forbid"' not in MANIFEST.read_text(encoding="utf-8"):
        raise RuntimeError("P0 media must forbid unsafe")
    common = ["--manifest-path", str(MANIFEST), "--offline", "--locked", "-j", "2"]
    for cmd in [[cargo, "fmt", "--manifest-path", str(MANIFEST), "--", "--check"], [cargo, *linker, "clippy", *common, "--all-targets", "--", "-D", "warnings"], [cargo, *linker, "test", *common, "--release"]]:
        subprocess.run(cmd, env=environment, check=True)


def run_native(exe, fixture):
    output = ROOT/"artifacts/p0"/("media-"+uuid.uuid4().hex[:8])
    paths = sources()+[Path(__file__).resolve(), ROOT/"tools/check_p0.py", ROOT/"tools/tests/test_p0_media.py"]
    identity = {p.relative_to(ROOT).as_posix(): digest(p) for p in paths}; exe_hash = digest(exe)
    inputs = {p.name: digest(p) for p in fixture.iterdir() if p.is_file()}
    log_path = output.with_suffix(".log")
    with log_path.open("wb") as log:
        result = subprocess.run([str(exe), str(output), str(fixture)], cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=25)
    if result.returncode:
        raise RuntimeError("Native media failed: "+str(log_path))
    r = json.loads(output.joinpath("result.json").read_text(encoding="utf-8")); validate(r, fixture)
    if digest(exe) != exe_hash or any(digest(p) != identity[p.relative_to(ROOT).as_posix()] for p in paths) or inputs != {p.name: digest(p) for p in fixture.iterdir() if p.is_file()}:
        raise RuntimeError("Media evidence inputs changed during run")
    for p in paths:
        dest = output/"sources"/p.relative_to(ROOT); dest.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(p, dest)
    shutil.copyfile(log_path, output/"run.log")
    evidence = {"schema_version": 1, "executable_sha256": exe_hash, "source_sha256": identity, "fixture": fixture.relative_to(ROOT).as_posix(), "fixture_sha256": inputs,
                "environment": environment_info(), "note": "Self-authored short WAV and circle fixture, no client files or acoustic/physical latency claim"}
    output.joinpath("evidence.json").write_text(json.dumps(evidence, indent=2)+"\n", encoding="utf-8")
    frozen = {p.name: digest(p) for p in output.iterdir() if p.is_file()}
    escaped = ROOT/("media-escape-"+uuid.uuid4().hex[:8])
    for invalid in [escaped, output]:
        rejection = subprocess.run([str(exe), str(invalid), str(fixture)], cwd=ROOT, capture_output=True, timeout=5)
        if rejection.returncode == 0 or (invalid == escaped and escaped.exists()):
            raise RuntimeError("Media output boundary/overwrite rejection failed")
    if frozen != {p.name: digest(p) for p in output.iterdir() if p.is_file()}:
        raise RuntimeError("Media overwrite rejection changed original evidence")
    print("Native file/timeline evidence: "+str(output), flush=True)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-native", action="store_true", help="Play low-amplitude self-authored file and test native content positions")
    parser.add_argument("--repetitions", type=int, choices=range(1, 4), default=3)
    args = parser.parse_args()
    environment = dict(os.environ); private = ROOT/".p0-tools/rustup/toolchains/1.95.0-x86_64-pc-windows-msvc/bin"
    if os.name == "nt" and private.joinpath("cargo.exe").is_file():
        environment["PATH"] = str(private)+os.pathsep+environment.get("PATH", ""); environment["CARGO_HOME"] = str(ROOT/".p0-tools/cargo"); environment["RUSTUP_HOME"] = str(ROOT/".p0-tools/rustup")
        cargo = str(private/"cargo.exe")
    else:
        cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("Rust 1.95 and native linker required")
    linker = windows_linker(environment)
    check_core(environment, cargo, linker)
    subprocess.run([sys.executable, str(ROOT/"tools/check_p0_gui.py"), "--media"], check=True)
    if args.run_native:
        if os.name != "nt":
            raise RuntimeError("Native file output proof is verified only on Windows")
        fixture = ROOT/"artifacts/p0"/("media-fixture-"+uuid.uuid4().hex[:8]); make_fixture(fixture)
        exe = ROOT/"experiments/p0-egui/target/release/media-proof.exe"
        for _ in range(args.repetitions):
            run_native(exe, fixture)


if __name__ == "__main__":
    main()
