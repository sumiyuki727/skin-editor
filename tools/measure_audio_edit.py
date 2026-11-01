"""Opt-in hot-edit/warm-WAV observations; owned Windows child, no window capture."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
from measure_audio_load import ROOT, distribution, observe, summarize

PHASES = ("idle", "hot_edits", "warm_prepare", "cleanup", "closed")
COLUMNS = ("activation_observed_ms", "selection_receipt_ms", "queue_wait_ms",
           "probe_allocate_decode_ms", "gain_process_ms", "result_handoff_ms")


def analyze(report, samples):
    phases = summarize(samples, PHASES, (("idle", 2), ("hot_edits", 2), ("warm_prepare", .5), ("cleanup", .5), ("closed", 1)))
    result = {}
    for key, count in (("hot", 300), ("warm", 64)):
        rows = report[key+"_ms"]
        if len(rows) != count or any(len(row) != len(COLUMNS) for row in rows):
            raise ValueError("missing preparation observations: " + key)
        result[key] = {name: distribution([r[i] for r in rows]) for i, name in enumerate(COLUMNS)}
        if any(any(v > r[0] for v in r[1:]) or sum(r[2:]) > r[0]+.01 for r in rows):
            raise ValueError("preparation stages exceed total observation")
        native = report[key+"_native_upper_ms"]
        if len(native) != count:
            raise ValueError("missing new-source native observations")
        if any(n < r[0] for n, r in zip(native, rows)):
            raise ValueError("native observation precedes activation")
        result[key]["edit_to_callback_observed_upper_ms"] = distribution(native)
    reads = report["warm_file_stage_ms"]
    if len(reads) != 64:
        raise ValueError("missing warm file observations")
    result["warm_file_stage_ms"] = distribution(reads)
    result["warm_file_plus_activation_ms"] = distribution([r[0]+f for r, f in zip(report["warm_ms"], reads)])
    gates = dict(hot_300_p95_le_50_ms=result["hot"][COLUMNS[0]]["p95"] <= 50,
                 warm_file_plus_activation_p95_le_250_ms=result["warm_file_plus_activation_ms"]["p95"] <= 250,
                 all_preparations_real_cache_misses=report["queued_hot"] == 300 and report["queued_warm"] == 64,
                 current_source_triggered=report["new_origin_triggers"] == 364,
                 single_draft_commit=report["draft_single_commit"], cancel_history_cleanup=report["cancel_history_cleanup"],
                 settled_old_pcm_released=report["settled_old_pcm_released"],
                 no_startup_underflow=report["startup_empty_frames"] == 0, idle_retiming_exercised=report["retimed_idle_frames"] > 0, no_steady_underflow=report["steady_empty_frames"] == 0,
                 managed_resources_released=report["pcm_final_bytes"] == 0 and report["raw_final_bytes"] == 0 and report["pcm_peak_bytes"] <= 32*1024**2,
                 audio_only_process_memory_le_512_mib=all(m["working_set_bytes_max"] <= 512*1024**2 and m["private_bytes_max"] <= 512*1024**2 for m in phases.values()))
    return dict(report=report, phases=phases, samples=samples, latency=result, checks=gates)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True); parser.add_argument("--output", required=True)
    args = parser.parse_args()
    binary = Path(args.binary).resolve(); output = (ROOT / args.output).resolve()
    if os.name != "nt" or binary != (ROOT / "desktop/target/release/skin-editor-desktop.exe").resolve() or not binary.is_file():
        raise RuntimeError("requires explicit Windows release desktop binary")
    if output.exists() or ROOT / "artifacts" not in output.parents:
        raise RuntimeError("output must be a new directory in artifacts")
    output.mkdir(parents=True)
    fingerprint = hashlib.sha256(binary.read_bytes()).hexdigest()
    runs = [analyze(*observe(binary, output / ("run-%02d" % (i+1)), "--verify-audio-edit")) for i in range(2)]
    if hashlib.sha256(binary.read_bytes()).hexdigest() != fingerprint:
        raise RuntimeError("binary changed during measurements")
    result = dict(schema=1, binary_sha256=fingerprint, builds="release", platform=platform.platform(),
                  logical_processors=os.cpu_count(), sample_interval_ms=100, runs=runs,
                  passed=all(all(r["checks"].values()) for r in runs),
                  scope="audio-only; source raw is resident, gain edits currently decode again; registered warm WAV file stage reported separately; no device consumption/acoustic or full graphics claim")
    (output / "metrics.json").write_text(json.dumps(result, indent=2)+"\n", encoding="utf-8")
    print(json.dumps({"passed": result["passed"], "runs": [{"latency": r["latency"], "phases": r["phases"], "checks": r["checks"]} for r in runs]}, indent=2))
    if not result["passed"]:
        raise RuntimeError("scoped audio edit acceptance failed; evidence retained")


if __name__ == "__main__":
    main()
