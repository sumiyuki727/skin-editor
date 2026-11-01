"""Fixed worker/GUI/GUI/worker plan; controlled real eframe dispatch, owned PIDs."""
import argparse
import hashlib
import json
import math
import os
import platform
from pathlib import Path
from measure_audio_load import ROOT, observe, summarize, distribution
from measure_combined import source_identity
from measure_worker_native import evaluate_worker

PLAN = (("worker_before", "--verify-separated-worker"), ("gui_1", "--verify-candidate-gui"),
        ("gui_2", "--verify-candidate-gui"), ("worker_after", "--verify-separated-worker"))


def evaluate_gui(report, metrics):
    if (not metrics or report.get("schema") != 2 or report.get("scope") != "gui-action-dispatch-to-native-copy"
            or report.get("controlled_focus") is not True
            or report.get("physical_input") is not False or report.get("gui_passes", 0) < 128
            or report.get("rate") not in (44100, 48000) or report.get("channels") not in (1, 2)):
        raise ValueError("missing controlled GUI scope or measurements")
    rows = report["probes"]
    if len(rows) != 128 or any(len(row) != 10 for row in rows):
        raise ValueError("missing GUI probes")
    keys, requests, voices = set(), set(), set()
    copied, joined = [[], []], [[], []]
    for round_id, lane, request, voice, upper, gui_joined, stamp, ordinal, offset, frames in rows:
        if (any(type(v) is not int for v in (round_id, lane, request, voice, stamp, ordinal, offset, frames))
                or not 0 <= round_id < 64 or lane not in (0, 1) or (round_id, lane) in keys
                or request <= 0 or request in requests or voice <= 0 or voice in voices
                or not math.isfinite(upper) or not math.isfinite(gui_joined) or not 0 <= upper <= gui_joined
                or stamp <= 0 or ordinal <= 0 or not 0 <= offset < frames):
            raise ValueError("invalid GUI request/Voice/time observation")
        keys.add((round_id, lane)); requests.add(request); voices.add(voice)
        copied[lane].append(upper); joined[lane].append(gui_joined)
    latency = [distribution(v) for v in copied]
    lifecycle = report.get("lifecycle", [])
    if len(lifecycle) != 17 or any(len(r) != 9 or r[0] != i+1 for i, r in enumerate(lifecycle)):
        raise ValueError("missing GUI lifecycle observations")
    previous = 0.
    for stage, at, scene, audition, copied_frame, generated, frozen, freezing, pcm in lifecycle:
        if (not math.isfinite(at) or at < previous or any(type(v) is not int or v < 0 for v in (stage, scene, audition, copied_frame, generated, pcm))
                or scene+audition > 32 or type(frozen) is not bool or type(freezing) is not bool or copied_frame > generated or pcm > 32*1024**2):
            raise ValueError("invalid GUI lifecycle observation")
        previous = at
    visual = report.get("visual", [])
    costs = report.get("ui_cost", [])
    if (len(visual) != 17 or any(len(row) != 10 or row[0] != i+1 for i, row in enumerate(visual))
            or len(costs) != 18 or any(len(row) != 3 or any(type(v) is not int or v < 0 for v in row) for row in costs)):
        raise ValueError("missing GUI visual/cost observations")
    checks = dict(zero_missing_frames=report["missing_frames"] == 0,
                  gui_request_copy_observed_upper_p95_le_20_ms=all(v["p95"] <= 20 for v in latency),
                  native_execution_healthy=not report["device_fault"] and report["render_errors"] == 0 and report["observation_overflows"] == 0 and report["callbacks"] > 0 and report["nonzero_samples"] > 0,
                  thirty_two_and_refuse_33=report["peak_voices"] == 32 and report["refused_33"] is True,
                  endpoint_and_pcm_released=report["endpoint_returns"] == 1 and report["pcm_final_bytes"] == 0 and 0 < report["pcm_peak_bytes"] <= 32*1024**2,
                  observed_gui_lifecycle=lifecycle[5][2:4] == [0, 16] and lifecycle[5][6:8] == [True, False] and lifecycle[7][2:4] == [0, 0] and lifecycle[15][2:4] == [0, 0] and lifecycle[16][8] == 0,
                  verified_draft_voice_then_cancel=visual[11][8:10] == [True, True] and 0 <= lifecycle[13][1]-lifecycle[12][1] < 1000,
                  bounded_static_publication=0 <= visual[4][2]-visual[3][2] <= 1 and visual[5][3] >= 2 and 0 < report["max_live_targets"] <= 8,
                  cpu_average_le_15_one_core=all(m["cpu_one_logical_core_percent"] <= 15 for m in metrics.values()),
                  process_memory_le_512_mib=all(m["working_set_bytes_max"] <= 512*1024**2 and m["private_bytes_max"] <= 512*1024**2 for m in metrics.values()))
    checks.update({key: report[key] is True for key in ("burst_exact", "hot_continuity", "pause_isolated", "stop_quiet", "resumed", "cancel_clean", "history_clean", "source_not_eof")})
    return latency, [distribution(v) for v in joined], checks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True); parser.add_argument("--output", required=True)
    args = parser.parse_args()
    binary = Path(args.binary).resolve(); output = (ROOT / args.output).resolve()
    if os.name != "nt" or binary != (ROOT/"desktop/target/release/skin-editor-desktop.exe").resolve() or not binary.is_file():
        raise RuntimeError("explicit Windows release binary required")
    if output.exists() or ROOT/"artifacts" not in output.parents:
        raise RuntimeError("fresh artifacts output required")
    output.mkdir(parents=True)
    identity = dict(source=source_identity(), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), platform=platform.platform())
    (output/"identity.json").write_text(json.dumps(identity, indent=2), encoding="utf-8")
    rounds = []
    for index, (kind, flag) in enumerate(PLAN, 1):
        print("Starting %d/4 %s" % (index, kind), flush=True)
        try:
            report, samples = observe(binary, output/("run-%02d" % index), flag, timeout_seconds=40)
            metrics = summarize(samples, phases=("idle", "voices_1", "voices_32", "closed"), minimum_spans=(("idle", .7), ("voices_1", 1.2), ("voices_32", 1.7), ("closed", .7)))
            latency, secondary, checks = evaluate_gui(report, metrics) if kind.startswith("gui") else evaluate_worker(report, metrics)
            peaks = {key: max(s[key] for s in samples) for key in ("working_set_bytes", "private_bytes")}
            checks["all_lifecycle_sampled_memory_le_512_mib"] = all(0 < v <= 512*1024**2 for v in peaks.values())
            row = dict(completed=True, kind=kind, report=report, samples=samples, metrics=metrics, all_sampled_memory_max=peaks, latency=latency, gui_joined_or_worker_receipt_latency=secondary, checks=checks)
        except (OSError, ValueError, RuntimeError, KeyError, TypeError) as error:
            row = dict(completed=False, kind=kind, error=str(error), checks=dict(completed=False))
        rounds.append(row)
        unchanged = identity["source"] == source_identity() and identity["binary_sha256"] == hashlib.sha256(binary.read_bytes()).hexdigest()
        proof = dict(schema=1, identity=identity, plan=PLAN, unchanged=unchanged, rounds=rounds,
                     passed=len(rounds) == 4 and unchanged and all(all(r["checks"].values()) for r in rounds))
        (output/"proof.json").write_text(json.dumps(proof, indent=2), encoding="utf-8")
        print(json.dumps(dict(kind=kind, completed=row["completed"], checks=row["checks"])), flush=True)
        if not row["completed"] or not unchanged: break
    if not proof["passed"]: raise RuntimeError("fixed GUI plan failed; no retry, evidence retained")


if __name__ == "__main__": main()
