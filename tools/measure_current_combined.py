"""Two fixed 300 s standard graphics/current separated-audio loads; ordinary Windows.

CPU is retained as raw GetProcessTimes, explicitly NOT independently validated.
It is a nonblocking optimization observation; no correction or legacy rewrite.
"""
import argparse
import hashlib
import json
import math
import os
import platform
from pathlib import Path

from measure_audio_load import ROOT, distribution, observe, summarize
from measure_combined import GpuSampler, source_identity
from measure_cpu_threads import token_info
from measure_main_work import ordinary


def finite(values):
    return all(type(v) in (int, float) and math.isfinite(v) and v >= 0 for v in values)


def analyze(report, samples, product=False):
    if (report.get("schema") != 1 or report.get("scope") != "current-separated-streamed-standard-N"
            or report.get("completed") is not True or report.get("seconds") != 300
            or report.get("shape") != [100, 10, 500, 16] or report.get("counts") != [2400, 64, 300, 20]
            or report.get("production_policy", False) is not product
            or report.get("gpu_memory_usage_hint") is not product or report.get("reusable_staging") is not False
            or report.get("rate") not in (44100, 48000) or report.get("channels") not in (1, 2)
            or report.get("controlled_focus") is not True or report.get("normal_auto_controls") is not True
            or report.get("physical_input_tested") is not False):
        raise ValueError("incomplete current route, standard workload or fixed device policy")
    probes = report["probes"]
    if (len(probes) != 2467 or any(len(p) != 10 for p in probes)
            or len({p[0] for p in probes}) != len(probes)
            or len({(p[1], p[2]) for p in probes}) != len(probes)):
        raise ValueError("incomplete/duplicate exact request and Voice observations")
    for p in probes:
        if (not finite(p[:9]) or any(type(p[i]) is not int or p[i] <= 0 for i in (0, 2, 5, 6, 8))
                or p[1] not in (0, 1) or type(p[9]) is not bool or not 0 <= p[7] < p[8]
                or p[3] > p[4]):
            raise ValueError("invalid callback identity, offset or observation clocks")
    lanes = [[p[3] for p in probes if p[9] and p[1] == lane] for lane in (0, 1)]
    if [len(l) for l in lanes] != [2400, 64]:
        raise ValueError("normal scene/audition samples differ from schedule")
    if len(report["edit_activation_ms"]) != 300 or len(report["switch_activation_ms"]) != 20:
        raise ValueError("incomplete material publication observations")
    if not 1000 <= len(report["cpu_frame_ms"]) <= 25000:
        raise ValueError("missing/unbounded framework CPU frame samples")
    if (len(report["hot_continuity"]) != 300 or any(
            len(r) != 3 or r[0] != i or not finite(r) or r[2] < r[1] or r[2] <= 0
            for i, r in enumerate(report["hot_continuity"]))):
        raise ValueError("hot activation lacks continuous background observation")
    if len(report["controls"]) != 6 or any(type(v) is not bool for v in report["controls"]):
        raise ValueError("incomplete commit/pause/cancel/history controls")
    trace = report["preparation_trace"]
    if not 322 <= len(trace) <= 512:
        raise ValueError("missing or unbounded graphics trace")
    for t in trace:
        if (not finite([t["started_ms"], t["decode_ms"], t["upload_ms"]])
                or (t["published_ms"] is not None and (not finite([t["published_ms"]]) or t["published_ms"] < t["started_ms"]))):
            raise ValueError("invalid graphics trace clock")
    resource_peak, resource_final = report["resources_peak"], report["resources_final"]
    if (len(resource_peak) != 5 or len(resource_final) != 5
            or any(type(v) is not int or v < 0 for v in resource_peak + resource_final)):
        raise ValueError("invalid managed resource accounting")
    phases = summarize(samples, ("playing", "closed"), (("playing", 299), ("closed", 1)))
    gpu = {}
    for phase in ("playing", "closed"):
        rows = [s for s in samples if s["phase"] == phase]
        gpu[phase] = {}
        for name in ("dedicated", "shared"):
            values = [s.get("gpu_" + name + "_bytes") for s in rows]
            valid = [v for v in values if type(v) is int and v >= 0]
            gpu[phase][name] = dict(valid_samples=len(valid), missing_samples=len(values)-len(valid),
                                    max_bytes=max(valid) if valid else None)
    latency = [distribution(v) for v in lanes]
    lifecycle_gpu = {}
    for name in ("dedicated", "shared"):
        values = [s.get("gpu_"+name+"_bytes") for s in samples]
        valid = [v for v in values if type(v) is int and v >= 0]
        lifecycle_gpu[name] = dict(valid_samples=len(valid), missing_samples=len(values)-len(valid),
                                   max_bytes=max(valid) if valid else None)
    edits, switches, frames = (distribution(report[k]) for k in ("edit_activation_ms", "switch_activation_ms", "cpu_frame_ms"))
    n, b = report["native"], report["background"]
    checks = dict(
        continuous_background_and_native_300s=report["playing_us"] >= 300_000_000
            and report["native_play_frames"] >= 300*report["rate"]
            and b["end"]-b["start"] >= 300*report["rate"] and b["continuous"] is True,
        zero_missing_including_startup=n["missing_frames"] == 0,
        native_healthy=n["callbacks"] > 0 and n["nonzero_samples"] > 0 and not n["device_fault"]
            and n["render_errors"] == n["observation_overflows"] == 0,
        complete_request_copy_p95_le_20_ms=all(l["p95"] <= 20 for l in latency),
        edit_to_published_gpu_p95_le_50_ms=edits["p95"] <= 50,
        cpu_frame_p95_le_16_7_ms=frames["p95"] <= 16.7,
        all_lifecycle_sampled_memory_le_512_mib=all(
            0 < s[k] <= 512*1024**2 for s in samples for k in ("working_set_bytes", "private_bytes")),
        gpu_pid_budget=all(gpu[p][name]["missing_samples"] == 0 and gpu[p][name]["max_bytes"] is not None
            and gpu[p][name]["max_bytes"] <= limit*1024**2
            for p in ("playing", "closed") for name, limit in (("dedicated", 256), ("shared", 128))),
        all_known_lifecycle_gpu_peaks_budget=all(lifecycle_gpu[name]["max_bytes"] is not None
            and lifecycle_gpu[name]["max_bytes"] <= limit*1024**2 for name,limit in (("dedicated",256),("shared",128))),
        bounded_managed_resources=all(v <= limit for v, limit in zip(resource_peak,
            (32*1024**2, 128*1024**2, 80*1024**2, 8, 80*1024**2))) and 0 < report["max_voices"] <= 32,
        actual_managed_release=resource_final == [0]*5 and report["staging_final_bytes"] == 0
            and b["raw_final_bytes"] == 0 and n["endpoint_returns"] == 1,
        callback_ownership=n["callback_resource_releases"] == n["endpoint_returns_in_render"] == 0
            and n["endpoint_returns_outside_render"] == 1,
        source_bounded_and_not_eof=not b["fault"] and b["end"] < b["frames"] and b["raw_peak_bytes"] == 8192
            and b["read_bytes"] >= 300*report["rate"]*report["channels"]*2,
        unchanged_sources_reused=report["source_allocations"] <= 1 and report["source_uploads"] <= 21,
        current_commit_pause_cancel_history=all(report["controls"]),
        all_material_changes_published=report["publications"] >= 322 and not report["trace_overflow"],
        unchanged_visible_geometry=report["view_valid"] is True and finite(report["geometry"])
            and len(report["geometry"]) == 3 and min(report["geometry"]) > 0,
        no_catchup_burst=report["max_trigger_lateness_us"] <= 125_000,
    )
    return dict(hard_checks=checks, hard_passed=all(checks.values()), phases_raw=phases, gpu=gpu, lifecycle_gpu=lifecycle_gpu,
                latency_ms=latency, edits_ms=edits, switches_ms=switches, cpu_frame_ms=frames,
                cpu_frames_over_16_7_ms=sum(v > 16.7 for v in report["cpu_frame_ms"]),
                all_lifecycle_sampled_memory={k:max(s[k] for s in samples) for k in ("working_set_bytes", "private_bytes")},
                os_lifetime_peaks={k:max(s[k] for s in samples) for k in ("lifetime_peak_working_set_bytes", "lifetime_peak_pagefile_bytes")},
                cpu_meter_independently_validated=False, cpu_goal_one_core_percent=15, cpu_goal_met=None,
                cpu_raw_numeric_le_15=phases["playing"]["cpu_one_logical_core_percent"] <= 15,
                cpu_scope="Uncorrected GetProcessTimes. No independent scheduling validation on this 300s plan; no CPU goal acceptance claim",
                p2_accepted=False, physical_input_tested=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--build-receipt", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--product", action="store_true")
    args = parser.parse_args()
    binary = Path(args.binary).resolve()
    output = (ROOT / args.output).resolve()
    receipt = json.loads(Path(args.build_receipt).read_text(encoding="utf-8"))
    sha = hashlib.sha256(binary.read_bytes()).hexdigest()
    identity = source_identity()
    if (os.name != "nt" or not ordinary(token_info()) or ROOT / "artifacts" not in output.parents or output.exists()
            or ROOT / "artifacts" not in binary.parents or binary.name != "skin-editor-desktop.exe"
            or receipt.get("profile") != "release" or receipt.get("requested_features") != []
            or receipt["source"] != identity or receipt["repeat_sha256"] != [sha, sha]):
        raise ValueError("ordinary frozen default-release artifact build and fresh output required")
    output.mkdir(parents=True)
    proof = dict(schema=1, source=identity, binary_sha256=sha, platform=platform.platform(),
                 production_policy=args.product, plan=["current-standard-N-1", "current-standard-N-2"], products=[], p2_accepted=False,
                 standard_hard_accepted=False, cpu_goal_met=None)
    try:
        for i in (1, 2):
            if source_identity() != identity or hashlib.sha256(binary.read_bytes()).hexdigest() != sha:
                raise ValueError("frozen sources/binary changed")
            print("Starting current-route standard 300s run %d/2" % i, flush=True)
            report, samples = observe(binary, output / ("run-%d" % i), "--verify-product-combined" if args.product else "--verify-streamed-combined", timeout_seconds=450, sampler_factory=GpuSampler)
            result = analyze(report, samples, args.product)
            proof["products"].append(dict(report=report, analysis=result, samples=samples))
            # Both predetermined rounds run even if a numeric hard budget fails.
            (output / "proof.json").write_text(json.dumps(proof, indent=2)+"\n", encoding="utf-8")
            print("Run %d hard checks: %s" % (i, result["hard_passed"]), flush=True)
        proof["standard_hard_accepted"] = all(p["analysis"]["hard_passed"] for p in proof["products"])
    except (OSError, RuntimeError, ValueError, KeyError) as e:
        proof["error"] = str(e)
        print(str(e), flush=True)
    finally:
        proof["unchanged"] = source_identity() == identity and hashlib.sha256(binary.read_bytes()).hexdigest() == sha
        (output / "proof.json").write_text(json.dumps(proof, indent=2)+"\n", encoding="utf-8")
    if not proof["standard_hard_accepted"] or not proof["unchanged"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
