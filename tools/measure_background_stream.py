"""Fixed finite/streamed worker/GUI/GUI/worker native preparation plan; no retries."""
import argparse
import hashlib
import json
import math
import os
import platform
from pathlib import Path
from measure_audio_load import ROOT, observe, summarize
from measure_combined import source_identity
from measure_worker_native import evaluate_worker
from measure_gui_candidate import evaluate_gui

PLAN = (("finite_before", "--verify-separated-worker"),
        ("stream_worker_1", "--verify-streamed-worker"),
        ("stream_gui_1", "--verify-streamed-gui"),
        ("stream_gui_2", "--verify-streamed-gui"),
        ("stream_worker_2", "--verify-streamed-worker"))


def stream_checks(report, worker):
    if report.get("streamed") is not True or report["source_frames"] != report["rate"]*360:
        raise ValueError("missing streamed source identity")
    checks = dict(source_healthy=report["source_fault"] is False,
                  raw_storage_bounded_and_released=report["source_raw_peak"] == 8192 and report["source_raw_final"] == 0,
                  real_reads=report["source_read_bytes"] > 8192 and report["source_seeks"] > 0)
    if worker:
        seconds, frames = report["stream_seconds"], report["stream_frames"]
        if not math.isfinite(seconds) or not 20 <= seconds < 30 or type(frames) is not int or frames < 0:
            raise ValueError("invalid continuous supply interval")
        checks["continuous_twenty_seconds"] = report["rate"]*19 <= frames <= report["rate"]*22
        checks.update({key: report[key] is True for key in ("located", "seek_isolated", "stream_monotonic", "backwards")})
    return checks


def audit_checks(report):
    return dict(no_managed_release_in_render=report["callback_resource_releases"] == 0,
                endpoint_return_outside_render=report["endpoint_returns_in_render"] == 0 and report["endpoint_returns_outside_render"] == 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True); parser.add_argument("--output", required=True)
    args = parser.parse_args()
    binary = Path(args.binary).resolve(); output = (ROOT/args.output).resolve()
    if os.name != "nt" or binary != (ROOT/"desktop/target/release/skin-editor-desktop.exe").resolve() or not binary.is_file():
        raise RuntimeError("explicit Windows release binary required")
    if output.exists() or ROOT/"artifacts" not in output.parents:
        raise RuntimeError("fresh artifacts output required")
    output.mkdir(parents=True)
    identity = dict(source=source_identity(), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), platform=platform.platform())
    (output/"identity.json").write_text(json.dumps(identity, indent=2), encoding="utf8")
    rounds = []
    for i, (kind, flag) in enumerate(PLAN, 1):
        print("Starting %d/%d %s" % (i, len(PLAN), kind), flush=True)
        try:
            report, samples = observe(binary, output/("run-%02d" % i), flag, timeout_seconds=55)
            is_gui = "gui" in kind
            phases = ("idle", "voices_1", "voices_32", "closed") + (("streaming",) if "stream_worker" in kind else ())
            metrics = summarize(samples, phases=phases, minimum_spans=(("idle", .7), ("voices_1", 1.2), ("voices_32", 1.7), ("closed", .7)) + ((("streaming", 19.),) if "stream_worker" in kind else ()))
            latency, other, checks = evaluate_gui(report, metrics) if is_gui else evaluate_worker(report, metrics)
            checks.update(audit_checks(report))
            if kind != "finite_before": checks.update(stream_checks(report, not is_gui))
            peaks = {key: max(s[key] for s in samples) for key in ("working_set_bytes", "private_bytes")}
            checks["all_lifecycle_sampled_memory_le_512_mib"] = all(0 < v <= 512*1024**2 for v in peaks.values())
            row = dict(completed=True, kind=kind, report=report, metrics=metrics, samples=samples, all_sampled_memory_max=peaks, latency=latency, secondary_latency=other, checks=checks)
        except (OSError, ValueError, RuntimeError, KeyError, TypeError) as error:
            row = dict(completed=False, kind=kind, error=str(error), checks=dict(completed=False))
        rounds.append(row)
        unchanged = identity["source"] == source_identity() and identity["binary_sha256"] == hashlib.sha256(binary.read_bytes()).hexdigest()
        proof = dict(schema=1, identity=identity, plan=PLAN, unchanged=unchanged, rounds=rounds, passed=len(rounds) == len(PLAN) and unchanged and all(all(r["checks"].values()) for r in rounds))
        (output/"proof.json").write_text(json.dumps(proof, indent=2), encoding="utf8")
        print(json.dumps(dict(kind=kind, completed=row["completed"], checks=row["checks"])), flush=True)
        if not row["completed"] or not unchanged: break
    if not proof["passed"]: raise RuntimeError("fixed background plan failed; evidence retained")


if __name__ == "__main__": main()
