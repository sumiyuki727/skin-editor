"""Two explicit Windows native-startup rounds, 20 empty-output instances each."""
import argparse
import hashlib
import json
import os
from pathlib import Path
from measure_audio_load import ROOT, observe, summarize


def analyze(report, samples):
    phases = summarize(samples, ("idle", "closed"), (("idle", 8), ("closed", 1)))
    rounds, traces = report["rounds"], report["startup_observations"]
    if len(rounds) != 20 or len(traces) != 20 or any(len(r) != 9 or any(not isinstance(v, int) or v < 0 for v in r) for r in rounds):
        raise ValueError("missing/invalid native starts")
    for rows in traces:
        if len(rows) != 32 or any(len(r) != 4 or any(not isinstance(v, int) or v < 0 for v in r) or r[1] > 2048 or r[2] > r[0] for r in rows):
            raise ValueError("missing/invalid bounded startup window")
    if report["empty_frames"] != sum(r[2] for r in rounds):
        raise ValueError("inconsistent underflow total")
    checks = dict(twenty_starts=all(r[0] >= 32 for r in rounds), no_startup_underflow=report["empty_frames"] == 0,
                  resources_released_every_cycle=all(r[8] == 0 and r[5] <= 32*1024**2 for r in rounds),
                  idle_reserve_covers_two_reported_periods=all(r[6] > 0 and r[7] >= 2*r[6] for r in rounds),
                  audio_only_memory_le_512_mib=all(m["working_set_bytes_max"] <= 512*1024**2 and m["private_bytes_max"] <= 512*1024**2 for m in phases.values()))
    return dict(report=report, phases=phases, samples=samples, checks=checks)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary",required=True); parser.add_argument("--output",required=True)
    args=parser.parse_args(); binary=Path(args.binary).resolve(); output=(ROOT/args.output).resolve()
    if os.name != "nt" or binary != (ROOT/"desktop/target/release/skin-editor-desktop.exe").resolve() or not binary.is_file():
        raise RuntimeError("explicit Windows release binary required")
    if output.exists() or ROOT/"artifacts" not in output.parents:
        raise RuntimeError("output must be a new directory in artifacts")
    output.mkdir(parents=True); fingerprint=hashlib.sha256(binary.read_bytes()).hexdigest()
    runs=[analyze(*observe(binary,output/("run-%02d"%(i+1)),"--verify-audio-startup")) for i in range(2)]
    if hashlib.sha256(binary.read_bytes()).hexdigest() != fingerprint: raise RuntimeError("binary changed")
    result=dict(schema=1,binary_sha256=fingerprint,runs=runs,passed=all(all(r["checks"].values()) for r in runs),
                scope="20 empty native-output instances per run; not full editor or graphics lifecycle; no acoustic or device-consumption claim")
    (output/"metrics.json").write_text(json.dumps(result,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(dict(passed=result["passed"],runs=[dict(phases=r["phases"],checks=r["checks"]) for r in runs]),indent=2))
    if not result["passed"]: raise RuntimeError("native startup acceptance failed; evidence retained")


if __name__ == "__main__": main()
