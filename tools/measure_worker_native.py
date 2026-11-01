"""Fixed standalone/worker/worker/standalone native plan, own Windows PIDs only."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
from measure_audio_load import ROOT, observe, summarize, distribution
from measure_combined import source_identity
from measure_separated_native import evaluate as evaluate_standalone

PLAN = (("standalone_before", "--verify-separated-native"), ("worker_1", "--verify-separated-worker"),
        ("worker_2", "--verify-separated-worker"), ("standalone_after", "--verify-separated-native"))


def evaluate_worker(report, metrics):
    if not metrics or report.get("schema") != 1 or report.get("scope") != "bounded-worker-full-request":
        raise ValueError("missing worker scope or process measurements")
    if report.get("rate") not in (44100, 48000) or report.get("channels") not in (1, 2):
        raise ValueError("unsupported report format")
    rows = report["probes"]
    if len(rows) != 128 or any(len(row) != 10 for row in rows):
        raise ValueError("missing full-request probes")
    keys, requests, voices = set(), set(), set()
    latencies, receipts = [[], []], [[], []]
    for round_id, lane, request, voice, receipt, upper, stamp, ordinal, offset, frames in rows:
        if (any(type(v) is not int for v in (round_id, lane, request, voice, stamp, ordinal, offset, frames))
                or not 0 <= round_id < 64 or lane not in (0, 1) or (round_id, lane) in keys
                or request <= 0 or request in requests or voice <= 0 or voice in voices
                or not math.isfinite(receipt) or not math.isfinite(upper) or not 0 <= receipt <= upper
                or stamp <= 0 or ordinal <= 0 or not 0 <= offset < frames):
            raise ValueError("invalid or duplicate request/Voice observation")
        keys.add((round_id, lane)); requests.add(request); voices.add(voice)
        latencies[lane].append(upper); receipts[lane].append(receipt)
    if any(len(lane) != 64 for lane in latencies):
        raise ValueError("missing lane")
    latency = [distribution(v) for v in latencies]
    checks = dict(zero_missing_frames=report["missing_frames"] == 0,
                  full_request_copy_observed_upper_p95_le_20_ms=all(v["p95"] <= 20 for v in latency),
                  native_execution_healthy=not report["device_fault"] and report["render_errors"] == 0 and report["observation_overflows"] == 0 and report["callbacks"] > 0 and report["nonzero_samples"] > 0,
                  thirty_two_and_refuse_33=report["peak_voices"] == 32 and report["refused_33"] is True,
                  endpoint_and_pcm_released=report["endpoint_returns"] == 1 and report["pcm_final_bytes"] == 0 and 0 < report["pcm_peak_bytes"] <= 32*1024**2,
                  cpu_average_le_15_one_core=all(m["cpu_one_logical_core_percent"] <= 15 for m in metrics.values()),
                  process_memory_le_512_mib=all(m["working_set_bytes_max"] <= 512*1024**2 and m["private_bytes_max"] <= 512*1024**2 for m in metrics.values()))
    checks.update({key: report[key] is True for key in ("burst_exact", "hot_continuity", "pause_isolated", "stop_quiet", "resumed", "cancel_clean", "history_clean", "source_not_eof")})
    return latency, [distribution(v) for v in receipts], checks


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
            if kind.startswith("worker"):
                latency, receipts, checks = evaluate_worker(report, metrics)
            else:
                latency, checks = evaluate_standalone(report, metrics); receipts = None
            peaks = {key: max(s[key] for s in samples) for key in ("working_set_bytes", "private_bytes")}
            checks["all_lifecycle_sampled_memory_le_512_mib"] = all(0 < v <= 512*1024**2 for v in peaks.values())
            row = dict(completed=True, kind=kind, report=report, samples=samples, metrics=metrics, all_sampled_memory_max=peaks, latency=latency, receipt_latency=receipts, checks=checks)
        except (OSError, ValueError, RuntimeError, KeyError, TypeError) as error:
            row = dict(completed=False, kind=kind, error=str(error), checks=dict(completed=False))
        rounds.append(row)
        unchanged = identity["source"] == source_identity() and identity["binary_sha256"] == hashlib.sha256(binary.read_bytes()).hexdigest()
        proof = dict(schema=1, identity=identity, plan=PLAN, unchanged=unchanged, rounds=rounds,
                     passed=len(rounds) == 4 and unchanged and all(all(r["checks"].values()) for r in rounds))
        (output/"proof.json").write_text(json.dumps(proof, indent=2), encoding="utf-8")
        print(json.dumps(dict(kind=kind, completed=row["completed"], checks=row["checks"])), flush=True)
        if not row["completed"] or not unchanged: break
    if not proof["passed"]: raise RuntimeError("fixed worker plan failed; no retry, evidence retained")


if __name__ == "__main__":
    main()
