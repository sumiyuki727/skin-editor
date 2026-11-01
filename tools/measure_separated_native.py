"""Two fixed short Windows F32 candidate runs; no GUI, GPU, personal media or retries."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
from measure_audio_load import ROOT, observe, summarize, distribution
from measure_combined import source_identity


def evaluate(report, metrics):
    if not metrics:
        raise ValueError("missing process measurements")
    if report.get("schema") != 1 or report.get("rate") not in (44100, 48000) or report.get("channels") not in (1, 2):
        raise ValueError("unsupported native report format")
    rows = report["probes"]
    if len(rows) != 128 or any(len(row) != 8 for row in rows):
        raise ValueError("missing exact first-copy probes")
    keys, voices = set(), set()
    lanes = [[], []]
    for round_id, lane, voice, upper, stamp, ordinal, offset, frames in rows:
        if (not isinstance(round_id, int) or not 0 <= round_id < 64 or lane not in (0, 1)
                or (round_id, lane) in keys or voice in voices or voice <= 0
                or not math.isfinite(upper) or upper < 0
                or any(type(v) is not int for v in (voice, stamp, ordinal, offset, frames))
                or stamp <= 0 or ordinal <= 0 or not 0 <= offset < frames):
            raise ValueError("invalid, duplicate or foreign probe")
        keys.add((round_id, lane)); voices.add(voice); lanes[lane].append(upper)
    if any(len(lane) != 64 for lane in lanes):
        raise ValueError("missing lane observations")
    latency = [distribution(lane) for lane in lanes]
    flags = ("burst_exact", "hot_continuity", "audition_survives_pause", "pause_stable", "resumed", "seek_exact", "seeking_resumed", "source_not_eof")
    checks = {
        "zero_missing_frames": report["missing_frames"] == 0,
        "native_first_copy_observed_upper_p95_le_20_ms": all(row["p95"] <= 20 for row in latency),
        "native_execution_healthy": not report["device_fault"] and report["render_errors"] == 0 and report["observation_overflows"] == 0 and report["callbacks"] > 0 and report["nonzero_samples"] > 0,
        "thirty_two_voices_refuse_33_exact_observations": report["peak_voices"] == 32 and report["refused_33"] is True,
        "endpoint_and_pcm_released": report["endpoint_returns"] == 1 and report["pcm_final_bytes"] == 0 and 0 < report["pcm_peak_bytes"] <= 32 * 1024**2,
        "cpu_average_le_15_one_core": all(row["cpu_one_logical_core_percent"] <= 15 for row in metrics.values()),
        "process_memory_le_512_mib": all(row["working_set_bytes_max"] <= 512 * 1024**2 and row["private_bytes_max"] <= 512 * 1024**2 for row in metrics.values()),
    }
    checks.update({name: report[name] is True for name in flags})
    return latency, checks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    binary = Path(args.binary).resolve(); output = (ROOT / args.output).resolve()
    if os.name != "nt" or binary != (ROOT / "desktop/target/release/skin-editor-desktop.exe").resolve() or not binary.is_file():
        raise RuntimeError("requires the explicit Windows release desktop binary")
    if output.exists() or ROOT / "artifacts" not in output.parents:
        raise RuntimeError("output must be a fresh directory in artifacts")
    output.mkdir(parents=True)
    source = source_identity()
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    identity = dict(schema=1, platform=platform.platform(), source=source, binary_sha256=digest)
    (output / "identity.json").write_text(json.dumps(identity, indent=2), encoding="utf-8")
    rounds = []
    for number in range(1, 3):
        try:
            report, samples = observe(binary, output / ("round-%02d" % number), "--verify-separated-native", timeout_seconds=35)
            phases = ("idle", "voices_1", "voices_32", "closed")
            metrics = summarize(samples, phases=phases, minimum_spans=(("idle", .7), ("voices_1", 1.2), ("voices_32", 1.7), ("closed", .7)))
            latency, checks = evaluate(report, metrics)
            rounds.append(dict(completed=True, report=report, samples=samples, metrics=metrics, latency=latency, checks=checks))
        except (OSError, ValueError, RuntimeError, KeyError, TypeError) as failure:
            rounds.append(dict(completed=False, error=str(failure), checks=dict(completed=False)))
        (output / ("evaluated-%02d.json" % number)).write_text(json.dumps(rounds[-1], indent=2), encoding="utf-8")
        if not rounds[-1]["completed"]:
            break
    unchanged = source_identity() == source and hashlib.sha256(binary.read_bytes()).hexdigest() == digest
    proof = dict(schema=1, platform=platform.platform(), source=source, binary_sha256=digest, unchanged=unchanged, rounds=rounds)
    (output / "proof.json").write_text(json.dumps(proof, indent=2), encoding="utf-8")
    print(json.dumps(dict(unchanged=unchanged, results=[r["checks"] for r in rounds]), indent=2))
    if len(rounds) != 2 or not unchanged or any(not all(row["checks"].values()) for row in rounds):
        raise RuntimeError("candidate short plan did not satisfy every unchanged gate; preserve all rows")


if __name__ == "__main__":
    main()
