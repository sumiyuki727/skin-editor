"""Compare complete reference/streaming validation on one owned P3 fixture."""

import argparse
import cProfile
import hashlib
import json
from pathlib import Path
import platform
import pstats
import sqlite3

from check_p3_storage import checked_output, percentiles, write_json
from p3_storage.metrics import snapshot, difference
from p3_storage.store import Store


def file_hash(path):
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for raw in iter(lambda: stream.read(65536), b""):
            hasher.update(raw)
    return hasher.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--rounds", type=int, default=5)
    args = parser.parse_args()
    if not 3 <= args.rounds <= 10:
        parser.error("rounds must be 3..10")
    source = checked_output(args.input)
    destination = checked_output(args.output, create=True)
    before_hash = file_hash(source)
    with Store.open(source, writable=False) as opened:
        expected = opened.status()
    samples = {"reference": [], "streaming": []}
    for index in range(args.rounds):
        order = ("reference", "streaming") if index % 2 == 0 else ("streaming", "reference")
        for method in order:
            before = snapshot()
            with Store.open(source, writable=False, reference=method == "reference") as opened:
                assert opened.status() == expected
                report = opened.verification_report
            sample = difference(before, snapshot())
            sample["verification"] = report
            samples[method].append(sample)
        print(json.dumps({"round": index + 1, "completed": True}), flush=True)
    diagnostics = {}
    for method in samples:
        # Same fixture already completely validated; trace cost is reported apart.
        with Store.open(source, writable=False, verify=False) as opened:
            opened.verify(reference=method == "reference", trace_sql=True)
            diagnostics[method] = opened.verification_report
        profile = cProfile.Profile()
        profile.enable()
        with Store.open(source, writable=False, reference=method == "reference"):
            pass
        profile.disable()
        profile.dump_stats(str(destination / (method + ".prof")))
        with (destination / (method + "-profile.txt")).open("w", encoding="utf8") as stream:
            pstats.Stats(profile, stream=stream).sort_stats("cumulative").print_stats(25)
    after_hash = file_hash(source)
    assert before_hash == after_hash
    write_json(destination / "result.json", {
        "python": platform.python_version(), "sqlite": sqlite3.sqlite_version,
        "source_sha256_before_after": before_hash, "file_bytes": source.stat().st_size,
        "expected": {key: value.hex() if type(value) is bytes else value for key, value in expected.items()},
        "methods": {key: {"latency": percentiles([s["wall_ms"] for s in values]), "samples": values}
                    for key, values in samples.items()}, "traced_diagnostics": diagnostics,
        "limits": {"both_validate_full_resources_and_indexes": True,
                   "cold_os_cache_controlled": False, "power_loss_tested": False,
                   "diagnostic_trace_and_profile_excluded_from_latency": True,
                   "product_ui_audio_load": False}})


if __name__ == "__main__":
    main()
