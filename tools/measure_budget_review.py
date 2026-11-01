"""Budget v2: fixed two default-release GUI short runs, no retries or CPU deductions.

Normal standard 300 s load and overall P2 acceptance remain explicitly open.
The existing scripts and their legacy predicates are not rewritten.
"""
import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess

from measure_audio_load import ROOT
from measure_cpu_threads import token_info
from measure_independent_cpu import fingerprint, build_collector, calibration, product_cost
from measure_main_work import Collector, run_child, ordinary, reference_workload_valid
from measure_repaint import close_session, require_references

POLICY = "resource-budget-v2"
PHASES = ("idle", "voices_1", "voices_32", "closed")
LEGACY_CPU = "cpu_average_le_15_one_core"
REQUIRED_HARD = frozenset((
    "zero_missing_frames", "gui_request_copy_observed_upper_p95_le_20_ms",
    "native_execution_healthy", "thirty_two_and_refuse_33", "endpoint_and_pcm_released",
    "observed_gui_lifecycle", "verified_draft_voice_then_cancel", "bounded_static_publication",
    "process_memory_le_512_mib", "burst_exact", "hot_continuity", "pause_isolated",
    "stop_quiet", "resumed", "cancel_clean", "history_clean", "source_not_eof",
    "source_healthy", "raw_storage_bounded_and_released", "real_reads",
    "no_managed_release_in_render", "endpoint_return_outside_render",
))


def review(original, costs, meter_valid, peaks):
    """Only the explicitly approved playback-CPU predicate is nonblocking."""
    if (not isinstance(original, dict) or not (REQUIRED_HARD | {LEGACY_CPU}) <= original.keys()
            or any(type(v) is not bool for v in original.values())
            or type(meter_valid) is not bool or set(costs) != set(PHASES)):
        raise ValueError("incomplete checks, meter or workload phases")
    for phase in PHASES:
        value = costs[phase].get("cpu_one_core_percent")
        if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
            raise ValueError("invalid independent CPU")
    if (set(peaks) != {"working_set_bytes", "private_bytes"}
            or any(type(v) is not int or v <= 0 for v in peaks.values())):
        raise ValueError("missing lifecycle memory observation")
    hard = {k: v for k, v in original.items() if k != LEGACY_CPU}
    hard["all_lifecycle_sampled_memory_le_512_mib"] = all(v <= 512*1024**2 for v in peaks.values())
    return dict(policy=POLICY, legacy_checks=copy.deepcopy(original), hard_checks=hard,
                bounded_hard_checks_passed=all(hard.values()), cpu_meter_valid=meter_valid,
                cpu_target_one_core_percent=15,
                cpu_target_met={p: costs[p]["cpu_one_core_percent"] <= 15 if meter_valid else None for p in PHASES},
                phases=dict(idle="short-idle-not-B05", voices_1="light-proxy-not-standard-N",
                            voices_32="stress-S-short", closed="short-closed-not-B05"),
                standard_workload_accepted=False, p2_accepted=False,
                note="B06 goal is nonblocking; no new ceiling, CPU correction or old acceptance rewrite")


def release_receipt(receipt, binary):
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    if (receipt.get("schema") != 1 or receipt.get("profile") != "release"
            or receipt.get("requested_features") != [] or receipt.get("host_resolved_features") != []
            or receipt.get("repeat_binary_sha256") != [digest, digest]
            or receipt.get("source") != fingerprint()["source"]):
        raise ValueError("two identical frozen default-release builds and matching source required")
    return digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--build-receipt", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    output = (ROOT / args.output).resolve()
    binary = Path(args.binary).resolve()
    receipt_path = Path(args.build_receipt).resolve()
    if (os.name != "nt" or not ordinary(token_info()) or output.exists()
            or ROOT / "artifacts" not in output.parents or ROOT / "artifacts" not in binary.parents
            or binary.name != "skin-editor-desktop.exe" or ROOT / "artifacts" not in receipt_path.parents):
        raise ValueError("ordinary Windows runner, frozen artifacts binary and fresh output required")
    receipt = json.loads(receipt_path.read_text(encoding="utf8"))
    digest = release_receipt(receipt, binary)
    output.mkdir()
    initial = fingerprint()
    collector_binary = build_collector(output)
    runner = token_info()
    collector = None
    refs, reports = [], []
    proof = dict(schema=1, policy=POLICY, identity=initial, binary_sha256=digest,
                 build_receipt_sha256=hashlib.sha256(receipt_path.read_bytes()).hexdigest(),
                 plan=["default-release-Q-1", "default-release-Q-2"], references=[], products=[],
                 calibrated=False, bounded_review_completed=False, p2_accepted=False,
                 standard_workload_accepted=False)
    try:
        check = json.loads(subprocess.check_output([str(collector_binary), "--self-test"], text=True))
        proof["collector_self_test"] = check
        if check.get("lifetime_self_test") is not True:
            raise ValueError("collector lifetime test failed")
        refpath = output / "references"
        refpath.mkdir()
        print("UAC 1/2: strict reference collector; products stay ordinary", flush=True)
        collector = Collector(collector_binary, refpath)
        for i, reverse in enumerate((False, True), 1):
            print("strict reference %d/2" % i, flush=True)
            refs.append(run_child(refpath / ("reference-%d" % i), "reference", reverse=reverse))
            if not reference_workload_valid(refs[-1]):
                raise ValueError("strict reference failed; no product admission or retry")
        trace, permission = close_session(collector, refpath, runner, refs, 2)
        collector = None
        proof["reference_permission"] = permission
        proof["references"] = [calibration(dict(trace, root_pid=r["pid"]), r) for r in refs]
        require_references(proof["references"])
        proof["calibrated"] = True
        prodpath = output / "products"
        prodpath.mkdir()
        print("UAC 2/2: two default-release Q runs, no framework tracing", flush=True)
        collector = Collector(collector_binary, prodpath)
        for i in (1, 2):
            if fingerprint() != initial or release_receipt(receipt, binary) != digest:
                raise ValueError("frozen source or binary changed")
            print("default release GUI %d/2" % i, flush=True)
            reports.append(run_child(prodpath / ("product-%d" % i), "product", binary=binary))
        trace, permission = close_session(collector, prodpath, runner, reports, 2)
        collector = None
        proof["product_permission"] = permission
        for r in reports:
            value = product_cost(trace, r)
            valid = value["checks"]["cross_cpu_overlap_le_100_us"] and proof["calibrated"]
            value["review"] = review(r["original_checks"], value["costs"], valid, value["all_sampled_memory_max"])
            value["product_token"] = r["product_token"]
            value["physical_input_tested"] = False
            proof["products"].append(value)
        proof["bounded_review_completed"] = len(proof["products"]) == 2 and all(
            v["review"]["bounded_hard_checks_passed"] and v["review"]["cpu_meter_valid"]
            for v in proof["products"])
    except (OSError, RuntimeError, ValueError, KeyError, subprocess.SubprocessError) as error:
        proof["error"] = str(error)
        print(str(error), flush=True)
    finally:
        if collector:
            try:
                collector.close()
            except (OSError, RuntimeError) as error:
                proof["cleanup_error"] = str(error)
                proof["bounded_review_completed"] = False
        proof["reference_report_paths"] = ["references/reference-%d/report.json" % (i+1) for i in range(len(refs))]
        proof["product_report_paths"] = ["products/product-%d/report.json" % (i+1) for i in range(len(reports))]
        proof["unchanged"] = initial == fingerprint() and digest == hashlib.sha256(binary.read_bytes()).hexdigest()
        proof["collector_sha256"] = hashlib.sha256(collector_binary.read_bytes()).hexdigest()
        (output / "proof.json").write_text(json.dumps(proof, indent=2) + "\n", encoding="utf8")
    if not proof["bounded_review_completed"] or not proof["unchanged"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
