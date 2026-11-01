"""Fixed observer comparison on the same owned GUI binary; never deduct observer CPU."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import time
from measure_audio_load import ROOT, WindowsSampler, observe, summarize, distribution
from measure_combined import source_identity
from measure_background_stream import audit_checks, stream_checks
from measure_gui_candidate import evaluate_gui
from measure_voice_cpu import LIMIT, ThreadSampler, group, thread_cost

# Forward/reverse blocks declared before observing, including every failure.
PLAN = ("process", "enumerate", "cached", "cached", "enumerate", "process") * 2
PHASES = ("idle", "voices_1", "voices_32", "closed")


class OwnedThreadSet:
    """Bounded retained handles: a terminated TID cannot retarget an existing handle."""
    def __init__(self, api, limit=LIMIT):
        if type(limit) is not int or not 0 < limit <= LIMIT:
            raise ValueError("invalid bounded thread handle capacity")
        self.api, self.limit = api, limit
        self.entries = {}
        self.closed = False
        self.last_phase = None
        self.main = None
        self.refreshes = 0

    def refresh(self):
        for tid in self.api.live_ids():
            handle = self.api.open_thread(tid)
            if not handle: continue
            retained = False
            try:
                if self.api.owner(handle) != self.api.pid: continue
                created, seconds = self.api.times(handle)
                key = "%d:%d" % (tid, created)
                if key in self.entries: continue
                if len(self.entries) >= self.limit:
                    raise RuntimeError("retained thread diagnostic limit exceeded")
                name = self.api.description(handle)
                self.entries[key] = dict(handle=handle, key=key, created=created,
                                         name=name, cpu_seconds=seconds)
                retained = True
            finally:
                if not retained: self.api.close_thread(handle)
        if self.main is None and self.entries:
            self.main = min(self.entries.values(), key=lambda e: e["created"])["key"]
        self.refreshes += 1

    def rows(self, phase):
        if self.closed: raise RuntimeError("thread observer already closed")
        if self.last_phase != phase:
            self.refresh()
            self.last_phase = phase
        rows = []
        for entry in self.entries.values():
            created, seconds = self.api.times(entry["handle"])
            if created != entry["created"] or seconds < entry["cpu_seconds"]:
                raise ValueError("retained thread identity/counter changed")
            entry["cpu_seconds"] = seconds
            rows.append(dict(key=entry["key"], created=created, name=entry["name"],
                             cpu_seconds=seconds, group=group(entry["name"], entry["key"] == self.main)))
        return rows

    def close(self):
        if self.closed: return
        self.closed = True
        # Attempt all closes on an error too; do not retain an unbounded failed queue.
        first = None
        for entry in self.entries.values():
            try: self.api.close_thread(entry["handle"])
            except OSError as error:
                if first is None: first = error
        self.entries.clear()
        if first is not None: raise first


class AuditedThreads(ThreadSampler):
    def __init__(self, child):
        super().__init__(child)
        self.inventory_ns = 0
        self.thread_read_ns = 0
        self.refresh_count = 0
        self.owned_live = 0

    def live_ids(self):
        begin = time.perf_counter_ns()
        try:
            live = super().live_ids()
            self.owned_live = len(live)
            self.refresh_count += 1
            return live
        finally: self.inventory_ns += time.perf_counter_ns()-begin

    def thread_rows(self, live):
        begin = time.perf_counter_ns()
        try: return super().thread_rows(live)
        finally: self.thread_read_ns += time.perf_counter_ns()-begin


class CachedThreads(AuditedThreads):
    def __init__(self, child):
        super().__init__(child)
        self.retained = OwnedThreadSet(self)

    def sample(self, phase):
        begin = time.perf_counter_ns()
        rows = self.retained.rows(phase)
        # Includes discovery when phase changes; inventory is also reported separately.
        self.thread_read_ns += time.perf_counter_ns()-begin
        result = WindowsSampler.sample(self, phase)
        result["threads"] = rows
        result["inventory_scope"] = "phase-boundary; transient threads may be undiscovered"
        return result

    def close(self):
        self.retained.close()


class ObserverAudit:
    def __init__(self, child, mode):
        self.backend = {"process": WindowsSampler, "enumerate": AuditedThreads,
                        "cached": CachedThreads}[mode](child)
        self.mode = mode
        self.close_record = None

    def sample(self, phase):
        backend = self.backend
        before = [getattr(backend, name, 0) for name in ("inventory_ns", "thread_read_ns", "refresh_count")]
        begin, cpu = time.perf_counter_ns(), time.process_time_ns()
        result = backend.sample(phase)
        end_cpu, end = time.process_time_ns(), time.perf_counter_ns()
        after = [getattr(backend, name, 0) for name in ("inventory_ns", "thread_read_ns", "refresh_count")]
        result["observer"] = dict(wall_ns=end-begin, runner_cpu_ns=end_cpu-cpu,
                                  inventory_ns=after[0]-before[0], thread_read_ns=after[1]-before[1],
                                  refreshes=after[2]-before[2],
                                  retained_handles=len(backend.retained.entries) if self.mode == "cached" else 0)
        return result

    def close(self):
        before = len(self.backend.retained.entries) if self.mode == "cached" else 0
        if hasattr(self.backend, "close"): self.backend.close()
        self.close_record = dict(before=before,
                                 after=len(self.backend.retained.entries) if self.mode == "cached" else 0)


def observer_cost(samples, phase):
    rows = [s for s in samples if s["phase"] == phase]
    if len(rows) < 2: raise ValueError("missing observer phase")
    span = rows[-1]["time"]-rows[0]["time"]
    if span <= 0: raise ValueError("invalid observer span")
    observations = [s["observer"] for s in rows]
    keys = ("wall_ns", "runner_cpu_ns", "inventory_ns", "thread_read_ns", "refreshes", "retained_handles")
    if any(any(type(s[k]) is not int or s[k] < 0 for k in keys) for s in observations):
        raise ValueError("invalid observer counters")
    # First query precedes the phase's counter interval. Keep it as a raw sample,
    # exclude it only from interval cost sums, never from child CPU or acceptance.
    measured = observations[1:]
    return dict(samples=len(rows), span_seconds=span,
                query_wall_ms=distribution([s["wall_ns"]/1e6 for s in observations]),
                inventory_wall_ms=distribution([s["inventory_ns"]/1e6 for s in observations]),
                sample_gap_ms=distribution([(b["time"]-a["time"])*1000 for a,b in zip(rows, rows[1:])]),
                interval_query_wall_seconds=sum(s["wall_ns"] for s in measured)/1e9,
                interval_inventory_wall_seconds=sum(s["inventory_ns"] for s in measured)/1e9,
                runner_cpu_seconds_during_queries=sum(s["runner_cpu_ns"] for s in measured)/1e9,
                refreshes=sum(s["refreshes"] for s in observations),
                max_retained_handles=max(s["retained_handles"] for s in observations))


def identity(binary):
    return dict(source=source_identity(), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())


def additional_checks(report, samples, close_record):
    checks = audit_checks(report)
    checks.update(stream_checks(report, False))
    checks["all_lifecycle_sampled_memory_le_512_mib"] = bool(samples) and all(
        0 < s[key] <= 512*1024**2 for s in samples for key in ("working_set_bytes", "private_bytes"))
    checks["observer_handles_released"] = close_record is not None and close_record["after"] == 0
    return checks


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    output = (ROOT/args.output).resolve()
    binary = ROOT/"desktop/target/release/skin-editor-desktop.exe"
    if os.name != "nt" or output.exists() or ROOT/"artifacts" not in output.parents:
        raise RuntimeError("Windows and fresh artifacts directory required")
    output.mkdir(parents=True)
    initial = identity(binary)
    rounds = []
    for index, mode in enumerate(PLAN, 1):
        print("Starting %d/%d %s" % (index, len(PLAN), mode), flush=True)
        audit = []
        def factory(child):
            sampler = ObserverAudit(child, mode); audit.append(sampler); return sampler
        try:
            report, samples = observe(binary, output/("run-%02d" % index), "--verify-streamed-gui",
                                      timeout_seconds=40, sampler_factory=factory)
            metrics = summarize(samples, phases=PHASES,
                                minimum_spans=(("idle", .7), ("voices_1", 1.2), ("voices_32", 1.7), ("closed", .7)))
            latency, _, checks = evaluate_gui(report, metrics)
            peaks = {key: max(s[key] for s in samples) for key in ("working_set_bytes", "private_bytes")}
            checks.update(additional_checks(report, samples, audit[0].close_record if len(audit) == 1 else None))
            row = dict(completed=True, mode=mode, report=report, metrics=metrics, latency=latency,
                       all_sampled_memory_max=peaks, checks=checks, observer_close=audit[0].close_record,
                       observer={p: observer_cost(samples, p) for p in PHASES},
                       thread_cost={p: thread_cost(samples, p) for p in PHASES} if mode != "process" else None)
        except (OSError, RuntimeError, ValueError, KeyError, TypeError) as error:
            row = dict(completed=False, mode=mode, error=str(error), checks=dict(completed=False))
        rounds.append(row)
        unchanged = initial == identity(binary)
        proof = dict(schema=1, identity=initial, platform=platform.platform(), plan=PLAN,
                     scope="fixed short GUI; observer process CPU kept separate; no budget subtraction",
                     unchanged=unchanged, rounds=rounds,
                     passed=len(rounds) == len(PLAN) and unchanged and all(all(r["checks"].values()) for r in rounds))
        (output/"proof.json").write_text(json.dumps(proof, indent=2)+"\n", encoding="utf8")
        print(json.dumps(dict(round=index, mode=mode, completed=row["completed"], checks=row["checks"],
                              cpu=row.get("metrics", {}).get("voices_32"))), flush=True)
        if not row["completed"] or not unchanged: break
    if not proof["passed"]: raise RuntimeError("observer comparison has failures; no retry, evidence retained")


if __name__ == "__main__": main()
