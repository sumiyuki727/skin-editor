"""Two ordinary 300 s eframe/native-audio runs; own PID metrics, no OS capture."""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import subprocess
from measure_audio_load import ROOT, WindowsSampler, observe, distribution, summarize


def source_identity():
    paths = set((ROOT/"crates").glob("*/src/**/*.rs")) | set((ROOT/"desktop").glob("*/src/**/*.rs"))
    paths |= set((ROOT/"crates").glob("*/Cargo.toml")) | set((ROOT/"desktop").glob("*/Cargo.toml"))
    paths |= set((ROOT/"tools").glob("check_*.py")) | set((ROOT/"tools").glob("test_check_*.py")) | set((ROOT/"tools").glob("measure_*.py"))
    paths |= {ROOT/p for p in ("AGENTS.md","Cargo.toml","Cargo.lock","desktop/Cargo.toml","desktop/Cargo.lock","rust-toolchain.toml")}
    return {p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}


def identity_matches(snapshot):
    return all((ROOT/p).is_file() and hashlib.sha256((ROOT/p).read_bytes()).hexdigest()==digest for p,digest in snapshot.items())


class Value(c.Union):
    _fields_ = [("integer", c.c_longlong), ("floating", c.c_double)]


class Formatted(c.Structure):
    _fields_ = [("status", w.DWORD), ("value", Value)]


class Item(c.Structure):
    _fields_ = [("name", w.LPWSTR), ("formatted", Formatted)]


def pid_total(rows, pid):
    """No matching/valid counter is unknown, never a fabricated zero."""
    values = [value for name, status, value in rows
              if name.startswith("pid_%d_" % pid) and status in (0, 1) and value >= 0]
    return sum(values) if values else None


class GpuSampler(WindowsSampler):
    def __init__(self, child):
        super().__init__(child)
        self.pid = child.pid
        self.pdh = c.WinDLL("pdh")
        self.pdh.PdhOpenQueryW.argtypes = [w.LPCWSTR, c.c_size_t, c.POINTER(w.HANDLE)]
        self.pdh.PdhAddEnglishCounterW.argtypes = [w.HANDLE, w.LPCWSTR, c.c_size_t, c.POINTER(w.HANDLE)]
        self.pdh.PdhCollectQueryData.argtypes = [w.HANDLE]
        self.pdh.PdhGetFormattedCounterArrayW.argtypes = [w.HANDLE, w.DWORD, c.POINTER(w.DWORD), c.POINTER(w.DWORD), c.c_void_p]
        self.pdh.PdhCloseQuery.argtypes = [w.HANDLE]
        self.query = w.HANDLE()
        self.counters = {}
        self.errors = []
        status = self.pdh.PdhOpenQueryW(None, 0, c.byref(self.query))
        if status:
            self.errors.append(["open", status & 0xffffffff])
            return
        for name, counter in (("dedicated", "Dedicated Usage"), ("shared", "Shared Usage")):
            handle = w.HANDLE()
            status = self.pdh.PdhAddEnglishCounterW(self.query, "\\GPU Process Memory(pid_%d_*)\\%s" % (self.pid, counter), 0, c.byref(handle))
            if status:
                self.errors.append([name, status & 0xffffffff])
            else:
                self.counters[name] = handle

    def sample(self, phase):
        result = super().sample(phase)
        result.update(gpu_dedicated_bytes=None, gpu_shared_bytes=None, gpu_counter_errors=self.errors)
        if self.query and self.pdh.PdhCollectQueryData(self.query) == 0:
            for name, handle in self.counters.items():
                size, count = w.DWORD(), w.DWORD()
                self.pdh.PdhGetFormattedCounterArrayW(handle, 0x400, c.byref(size), c.byref(count), None)
                if not 0 < size.value <= 65536:
                    continue
                buffer = c.create_string_buffer(size.value)
                status = self.pdh.PdhGetFormattedCounterArrayW(handle, 0x400, c.byref(size), c.byref(count), buffer)
                if status == 0 and count.value <= 256:
                    items = c.cast(buffer, c.POINTER(Item))
                    rows = [(items[i].name, items[i].formatted.status, items[i].formatted.value.integer) for i in range(count.value)]
                    result["gpu_" + name + "_bytes"] = pid_total(rows, self.pid)
                    result["gpu_" + name + "_instances"] = [dict(name=n,status=s,bytes=v) for n,s,v in rows if n.startswith("pid_%d_" % self.pid)]
        return result

    def close(self):
        if self.query:
            self.pdh.PdhCloseQuery(self.query)
            self.query = w.HANDLE()


def submission_analysis(report):
    """Per-probe brackets, never subtract one global poll gap from a percentile."""
    lanes = report["submission_intervals_ms"]
    if len(lanes) != 2 or any(len(lane) != 64 for lane in lanes):
        raise ValueError("missing bounded submission intervals")
    result = []
    for index, lane in enumerate(lanes):
        for row, upper in zip(lane, report["latency_upper_bound_ms"][index]):
            if (len(row) != 5 or any(not math.isfinite(v) or v < 0 for v in row)
                    or not 0 <= row[0] <= row[1] == upper or not row[2] <= row[3] <= row[1]):
                raise ValueError("invalid submission observation bracket")
        lower = distribution([r[0] for r in lane])
        upper = distribution([r[1] for r in lane])
        result.append(dict(lower_ms=lower,upper_ms=upper,
                           uncertainty_ms=distribution([r[1]-r[0] for r in lane]),
                           request_to_receipt_ms=distribution([r[2] for r in lane]),
                           after_receipt_lower_ms=distribution([max(0,r[0]-r[2]) for r in lane]),
                           p95_classification="confirmed_within" if upper["p95"] <= 20 else "confirmed_over" if lower["p95"] > 20 else "unresolved",
                           individually_confirmed_over_20=sum(r[0]>20 for r in lane),
                           individually_straddling_20=sum(r[0]<=20<r[1] for r in lane)))
    return result


def causal_analysis(report, *, count=64):
    """Callback-entry clock is a lower bound, never callback completion/acoustics.

    The residual includes backend dispatch, callback work and observer polling.
    It cannot all be relabelled as either real queue time or measurement error.
    """
    lanes = report['submission_causal_ms']
    if len(lanes)!=2 or any(len(lane)!=count for lane in lanes):
        raise ValueError('missing causal Voice observations')
    result=[]
    for index,lane in enumerate(lanes):
        intervals=report['submission_intervals_ms'][index]
        if len(intervals)!=count: raise ValueError('causal/observation count differs')
        for r,b in zip(lane,intervals):
            if (len(r)!=13 or any(not math.isfinite(v) for v in r) or
                    not 0<=r[0]<=r[1]<=r[2] or not r[3]<=r[4] or r[3]>b[1] or r[1]>b[1] or
                    not 0<=r[5]<=2048 or not 0<r[6]<=2048 or not 0<=r[7]<r[6] or
                    not 0<=r[8]<r[9] or any(v<0 or int(v)!=v for v in r[5:])):
                raise ValueError('invalid causal stages/queue facts')
        result.append(dict(
            request_to_pump_ms=distribution([r[0] for r in lane]),
            request_to_commit_before_ms=distribution([r[1] for r in lane]),
            commit_bracket_ms=distribution([r[2]-r[1] for r in lane]),
            callback_entry_lower_ms=distribution([r[3] for r in lane]),
            clock_calibration_width_ms=distribution([r[4]-r[3] for r in lane]),
            committed_wait_until_callback_entry_lower_ms=distribution([max(0,r[3]-r[2]) for r in lane]),
            callback_entry_to_positive_upper_ms=distribution([b[1]-r[3] for r,b in zip(lane,intervals)]),
            queued_before_frames=distribution([r[5] for r in lane]),
            first_audible_callback_offset_frames=distribution([r[8] for r in lane]),
            callback_entered_before_commit=sum(r[3]<r[2] for r in lane)))
    return result


def preparation_analysis(report):
    """Paired per-request wall stages, not a sum of independent percentiles."""
    groups={}
    for reused,name in ((False,'cold'),(True,'warm')):
        rows=[r for r in report['preparation_trace'] if r['reused'] is reused and r['published_ms'] is not None]
        if not rows: raise ValueError('missing preparation group')
        values={key:[] for key in ('total_ms','queue_ms','worker_ms','receive_gap_ms','gpu_notify_to_publish_ms',
                                   'decode_ms','pack_ms','upload_ms','write_ms','flush_ms','create_ms','encode_submit_ms',
                                   'source_lock_ms','submission_lock_ms','staging_wait_ms','staging_alloc_ms','staging_copy_ms')}
        for r in rows:
            if (not r['started_ms']<=r['worker_started_ms']<=r['worker_finished_ms']<=r['received_ms']<=r['published_ms'] or
                    r['gpu_done_ms'] is None or not r['worker_started_ms']<=r['gpu_done_ms']<=r['published_ms']):
                raise ValueError('invalid worker/GPU/publication lifecycle')
            for key in values:
                v=({'total_ms':r['published_ms']-r['started_ms'],
                    'worker_ms':r['worker_finished_ms']-r['worker_started_ms'],
                    'receive_gap_ms':r['received_ms']-r['worker_finished_ms'],
                    'gpu_notify_to_publish_ms':r['published_ms']-r['gpu_done_ms']}.get(key,r.get(key)))
                if v is None or not math.isfinite(v) or v<0: raise ValueError('invalid preparation segment')
                values[key].append(v)
        groups[name]={key:distribution(v) for key,v in values.items()}
    return groups


def allocator_analysis(snapshots):
    """Device allocator accounting, deliberately not subtracted from PDH totals.

    Blocks span memory types. Swapchain/driver allocations are outside this
    report; reserved bytes cannot be relabelled as dedicated resident bytes.
    """
    if not 1 <= len(snapshots) <= 16:
        raise ValueError("missing or unbounded allocator snapshots")
    result = []
    for s in snapshots:
        if not s['supported']:
            result.append(dict(stage=s['stage'],supported=False))
            continue
        allocations, blocks = s['allocations'], s['blocks']
        if (len(allocations)>512 or len(blocks)>512 or
                sum(a['bytes'] for a in allocations)!=s['allocated_bytes'] or
                sum(b['bytes'] for b in blocks)!=s['reserved_bytes'] or
                s['allocated_bytes']>s['reserved_bytes']):
            raise ValueError("inconsistent allocator report")
        categories = {}
        covered = 0
        for b in blocks:
            if not covered == b['begin'] <= b['end'] <= len(allocations):
                raise ValueError("invalid allocator block range")
            for a in allocations[b['begin']:b['end']]:
                if not 0 <= a['offset'] <= a['offset']+a['bytes'] <= b['bytes']:
                    raise ValueError("allocation exceeds block")
            covered = b['end']
        if covered != len(allocations): raise ValueError("unaccounted allocations")
        for a in allocations:
            label=bytes(a['name_utf8']).decode('utf-8')
            categories[label]=categories.get(label,0)+a['bytes']
        result.append(dict(stage=s['stage'],supported=True,allocated_bytes=s['allocated_bytes'],reserved_bytes=s['reserved_bytes'],
                           unused_capacity_bytes=s['reserved_bytes']-s['allocated_bytes'],categories=categories,capture_ms=s['capture_ms']))
    return result


def compare_pixel_reports(reports, traces, *, staging=False):
    """Seven bounded GPU readbacks must agree exactly across device policies."""
    sizes = [[256,256]]*6 + [[257,193]]
    flags = ('paused_draft_refresh','cancel_exact_restore','branch_restore','stale_viewport_rejected',
             'shutdown_held_frame_tracked','source_buffer_queue_order_verified')
    if len(reports)!=2 or len(traces)!=2:
        raise ValueError('two pixel policy controls required')
    for index, r in enumerate(reports):
        if (r.get('gpu_memory_usage_hint') is not bool(index) or r.get('reusable_staging') is not staging or r.get('pixel_trace_sizes')!=sizes or
                not 0 <= r['max_channel_error'] <= 2 or r['unchanged_uploads']!=0 or r['source_uploads']!=1 or
                not all(r.get(key) is True for key in flags)):
            raise ValueError('incomplete pixel control or wrong device policy')
    if reports[0]['adapter']!=reports[1]['adapter']:
        raise ValueError('pixel controls used different adapters')
    expected=sum(w*h*4 for w,h in sizes)
    if any(len(t)!=expected for t in traces):
        raise ValueError('incomplete pixel trace')
    if traces[0]!=traces[1]:
        raise ValueError('GPU pixels differ across memory policies')
    return dict(frames=len(sizes),bytes=expected,exact_match=True,sha256=hashlib.sha256(traces[0]).hexdigest(),
                max_cpu_reference_error=max(r['max_channel_error'] for r in reports))


def staging_analysis(report, requested):
    if report.get('reusable_staging') is not requested:
        raise ValueError('wrong upload strategy')
    fields=('staging_peak_bytes','staging_final_bytes','staging_allocations','staging_copies','staging_remaps')
    if any(type(report.get(k)) is not int or report[k]<0 for k in fields):
        raise ValueError('missing staging accounting')
    if requested:
        return dict(staging_bounded=0<report['staging_peak_bytes']<=80*1024**2,
                    staging_reused=report['staging_allocations']==1 and report['staging_copies']==report['source_uploads'] and report['staging_remaps']==report['source_uploads']-1,
                    staging_released=report['staging_final_bytes']==0)
    if any(report[k] for k in fields):raise ValueError('reference unexpectedly holds reusable staging')
    return {}


def analyze(report, samples, *, memory_usage=False, staging=False):
    if report.get('gpu_memory_usage_hint') is not memory_usage:
        raise ValueError('combined device policy differs from requested policy')
    if (report["original_pngs"], report["documents"], report["layers"], report["seconds"], report["triggers"], report["edits"], report["switches"]) != (100, 10, 500, 300, 2400, 300, 20):
        raise ValueError("incomplete standard fixture/schedule")
    if len(report["latency_upper_bound_ms"]) != 2 or any(len(v)!=64 for v in report["latency_upper_bound_ms"]) or len(report["edit_activation_ms"]) != 300 or not 1000 <= len(report["cpu_frame_ms"]) <= 25000:
        raise ValueError("missing or unbounded latency/frame trace")
    trace = report["preparation_trace"]
    if report["trace_overflow"] or not 321 <= len(trace) <= 512 or len(report["slow_ui_ms"]) > 256:
        raise ValueError("missing or overflowing preparation trace")
    for item in trace:
        for key in ("started_ms", "plan_ms", "decode_ms", "pack_ms", "upload_ms", "encode_submit_ms"):
            if not math.isfinite(item[key]) or item[key] < 0:
                raise ValueError("invalid preparation duration")
        if item["published_ms"] is not None and (not math.isfinite(item["published_ms"]) or item["published_ms"] < item["started_ms"]):
            raise ValueError("invalid publication observation")
    graphics = [v["published_ms"]-v["started_ms"] for v in trace if v["published_ms"] is not None]
    if len(graphics) < 321:
        raise ValueError("incomplete graphics publication trace")
    phases = summarize(samples, ("playing", "closed"), (("playing", 299), ("closed", 1)))
    gpu = {}
    for phase in phases:
        selected = [s for s in samples if s["phase"] == phase]
        gpu[phase] = {}
        for name in ("dedicated", "shared"):
            values = [s.get("gpu_"+name+"_bytes") for s in selected]
            valid = [v for v in values if v is not None and v >= 0]
            gpu[phase][name] = {"valid_samples": len(valid), "missing_samples": len(values)-len(valid), "max_bytes": max(valid) if valid else None}
    latency = [distribution(v) for v in report["latency_upper_bound_ms"]]
    edits = distribution(report["edit_activation_ms"])
    frames = distribution(report["cpu_frame_ms"])
    playing = phases["playing"]
    playing_gpu = gpu["playing"]
    checks = dict(
        continuous_native_submission_300s=report["native_play_frames"] >= 300*report["rate"],
        no_underflow_including_startup=report["empty_frames"] == report["empty_callbacks"] == 0 and not report["underrun_observations"],
        ready_both_lanes_under_graphics_p95_le_20_ms=all(v["p95"] <= 20 for v in latency),
        edit_activation_p95_le_50_ms=edits["p95"] <= 50,
        cpu_playing_one_core_le_15_percent=playing["cpu_one_logical_core_percent"] <= 15,
        cpu_frame_p95_le_16_7_ms=frames["p95"] <= 16.7,
        cpu_frame_max_le_16_7_ms=frames["max"] <= 16.7,
        working_set_private_le_512_mib=all(s["working_set_bytes"] <= 512*1024**2 and s["private_bytes"] <= 512*1024**2 for s in samples),
        gpu_pid_budget=all(playing_gpu[n]["missing_samples"] == 0 and playing_gpu[n]["max_bytes"] is not None and playing_gpu[n]["max_bytes"] <= limit*1024**2 for n, limit in (("dedicated",256),("shared",128))),
        bounded_managed_resources=report["pcm_peak_bytes"] <= 32*1024**2 and report["raw_peak_bytes"] <= 128*1024**2 and report["rgba_peak_bytes"] <= 80*1024**2 and report["max_targets"] <= 8 and report["peak_voices"] <= 32,
        old_resources_released=report["resources_released"] and report["pcm_final_bytes"] == report["raw_final_bytes"] == report["rgba_final_bytes"] == report["final_targets"] == report["source_cache_final_bytes"] == 0,
        unchanged_sources_reused=report["source_uploads"] <= 21,
        source_allocation_reused=report["source_allocations"] <= 1,
        dual_channel_isolation=report["dual_channel_isolation"],
        draft_history_cleanup=report["draft_history_cleanup"],
        all_graphics_changes_published=report["publications"] >= 321,
    )
    checks.update(staging_analysis(report,staging))
    return dict(phases=phases, gpu=gpu, allocator=allocator_analysis(report['gpu_allocator_snapshots']), latency_ms=latency, submission=submission_analysis(report), edit_ms=edits, cpu_frame_ms=frames,
                graphics_request_to_publication_ms=distribution(graphics),
                cpu_frames_over_16_7_ms=sum(v>16.7 for v in report["cpu_frame_ms"]),
                lifetime_peak_working_set_bytes=max(s["lifetime_peak_working_set_bytes"] for s in samples),
                lifetime_peak_pagefile_bytes=max(s["lifetime_peak_pagefile_bytes"] for s in samples), checks=checks)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--output", required=True)
    modes=parser.add_mutually_exclusive_group()
    modes.add_argument("--gpu-controls", action="store_true", help="Separate two-size empty-window and standard 3s controls; not acceptance")
    modes.add_argument("--pixel-controls", action="store_true", help="Separate exact seven-frame GPU readback across both device policies")
    modes.add_argument("--cold-controls", action="store_true", help="14s same-fixture cold preparations in Performance/MemoryUsage/MemoryUsage/Performance order; not acceptance")
    modes.add_argument("--staging-controls", action="store_true", help="14s same-MemoryUsage upload controls in reference/reuse/reuse/reference order; not acceptance")
    modes.add_argument("--memory-usage", action="store_true", help="Two full standard runs with the MemoryUsage device hint")
    parser.add_argument('--reusable-staging',action='store_true',help='Explicit bounded single-slot upload candidate, with --memory-usage or --pixel-controls')
    args = parser.parse_args()
    if args.reusable_staging and not (args.memory_usage or args.pixel_controls):
        parser.error('--reusable-staging requires --memory-usage or --pixel-controls')
    binary = Path(args.binary).resolve(); output = (ROOT/args.output).resolve()
    if os.name != "nt" or binary != (ROOT/"desktop/target/release/skin-editor-desktop.exe").resolve() or not binary.is_file():
        raise RuntimeError("requires explicit Windows release desktop binary")
    if output.exists() or ROOT/"artifacts" not in output.parents:
        raise RuntimeError("output must be new, inside artifacts")
    output.mkdir(parents=True)
    fingerprint = hashlib.sha256(binary.read_bytes()).hexdigest()
    identity = dict(binary_sha256=fingerprint,source_sha256=source_identity(),build="release",mode="ordinary-static-editor-deadline-driven",reusable_staging=args.reusable_staging,gpu_memory_usage_hint=None if args.gpu_controls or args.pixel_controls or args.cold_controls or args.staging_controls else args.memory_usage)
    (output/"identity.json").write_text(json.dumps(identity,indent=2)+"\n",encoding="utf-8")
    runs = []
    if args.staging_controls:
        for index,reuse in enumerate((False,True,True,False)):
            argument='--verify-cold-staging-memory' if reuse else '--verify-cold-memory'
            print('Starting staging control %d/4: %s'%(index+1,argument),flush=True)
            report,samples=observe(binary,output/('run-%02d'%(index+1)),argument,timeout_seconds=90,sampler_factory=GpuSampler)
            if report['seconds']!=14 or report['switches']!=20 or report['gpu_memory_usage_hint'] is not True:
                raise ValueError('wrong staging control workload/policy')
            checks=staging_analysis(report,reuse)
            runs.append(dict(report=report,samples=samples,checks=checks,preparation=preparation_analysis(report)))
        unchanged=hashlib.sha256(binary.read_bytes()).hexdigest()==fingerprint and identity_matches(identity['source_sha256'])
        (output/'metrics.json').write_text(json.dumps(dict(schema=1,mode='interleaved-staging-controls-not-acceptance',sources_unchanged=unchanged,runs=runs),indent=2)+'\n',encoding='utf-8')
        if not unchanged or not all(all(r['checks'].values()) for r in runs):raise RuntimeError('staging control identity/accounting failed')
        return
    if args.cold_controls:
        for index,memory in enumerate((False,True,True,False)):
            argument='--verify-cold-memory' if memory else '--verify-cold'
            print('Starting cold control %d/4: %s' % (index+1,argument),flush=True)
            report,samples=observe(binary,output/('run-%02d'%(index+1)),argument,timeout_seconds=90,sampler_factory=GpuSampler)
            if report['switches']!=20 or report['seconds']!=14 or report['gpu_memory_usage_hint'] is not memory:
                raise ValueError('wrong cold control workload/policy')
            runs.append(dict(report=report,samples=samples,preparation=preparation_analysis(report)))
        unchanged=hashlib.sha256(binary.read_bytes()).hexdigest()==fingerprint and identity_matches(identity['source_sha256'])
        (output/'metrics.json').write_text(json.dumps(dict(schema=1,mode='interleaved-cold-controls-not-acceptance',sources_unchanged=unchanged,runs=runs),indent=2)+'\n',encoding='utf-8')
        if not unchanged: raise RuntimeError('cold control identity changed')
        return
    if args.pixel_controls:
        reports=[];traces=[]
        for name, argument in (('original','--verify-gpu-staging' if args.reusable_staging else '--verify-gpu'),('memory','--verify-gpu-staging-memory' if args.reusable_staging else '--verify-gpu-memory')):
            with (output/(name+'.log')).open('x',encoding='utf-8') as log:
                subprocess.run([str(binary),argument,str(output/name)],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=120)
            reports.append(json.loads((output/name/'report.json').read_text(encoding='utf-8')))
            traces.append((output/name/'pixel_trace.rgba').read_bytes())
        comparison=compare_pixel_reports(reports,traces,staging=args.reusable_staging)
        unchanged=hashlib.sha256(binary.read_bytes()).hexdigest()==fingerprint and identity_matches(identity['source_sha256'])
        (output/'metrics.json').write_text(json.dumps(dict(schema=1,mode='pixel-policy-controls-not-performance',reports=reports,comparison=comparison,sources_unchanged=unchanged),indent=2)+'\n',encoding='utf-8')
        if not unchanged: raise RuntimeError('pixel control identity changed')
        return
    if args.gpu_controls:
        for argument in ("--verify-gpu-cost", "--verify-gpu-cost-large", "--verify-combined-control", "--verify-gpu-cost-memory", "--verify-combined-control-memory"):
            for repeat in range(2):
                name = argument.removeprefix("--verify-") + "-%02d" % (repeat+1)
                print("Starting GPU attribution control " + name, flush=True)
                report, samples = observe(binary, output/name, argument, timeout_seconds=60, sampler_factory=GpuSampler)
                runs.append(dict(argument=argument,report=report,samples=samples,allocator=allocator_analysis(report.get('snapshots',report.get('gpu_allocator_snapshots',[])))))
        unchanged = hashlib.sha256(binary.read_bytes()).hexdigest() == fingerprint and identity_matches(identity["source_sha256"])
        (output/"metrics.json").write_text(json.dumps(dict(schema=1,mode="attribution-controls-not-acceptance",sources_unchanged=unchanged,runs=runs),indent=2)+"\n",encoding="utf-8")
        if not unchanged: raise RuntimeError("control identity changed")
        return
    for index in range(2):
        print("Starting ordinary combined run %d/2 (300 s)" % (index+1), flush=True)
        try:
            argument="--verify-combined-staging-memory" if args.reusable_staging else "--verify-combined-memory" if args.memory_usage else "--verify-combined"
            report, samples = observe(binary, output/("run-%02d" % (index+1)), argument, timeout_seconds=420, sampler_factory=GpuSampler)
            runs.append(dict(completed=True,report=report, samples=samples,causal=causal_analysis(report),preparation=preparation_analysis(report), **analyze(report, samples,memory_usage=args.memory_usage,staging=args.reusable_staging)))
        except (OSError,ValueError,RuntimeError,KeyError,TypeError) as failure:
            runs.append(dict(completed=False,error=str(failure),checks={"completed":False}))
            print(json.dumps(runs[-1]),flush=True)
            break
        print(json.dumps({"run": index+1, "checks": runs[-1]["checks"], "latency_ms": runs[-1]["latency_ms"], "phases":runs[-1]["phases"], "gpu": runs[-1]["gpu"]}), flush=True)
    unchanged = hashlib.sha256(binary.read_bytes()).hexdigest() == fingerprint and identity_matches(identity["source_sha256"])
    passed = len(runs)==2 and unchanged and all(all(r["checks"].values()) for r in runs)
    result = dict(schema=1,binary_sha256=fingerprint,sources_unchanged=unchanged,gpu_memory_usage_hint=args.memory_usage,reusable_staging=args.reusable_staging,platform=platform.platform(),logical_processors=os.cpu_count(),sample_interval_ms=100,runs=runs,passed=passed,
                  scope="ordinary eframe presentation, own PID across adapters; callback submission is not device consumption/acoustic evidence; no song stream; system audio engine and driver attribution unknown")
    (output/"metrics.json").write_text(json.dumps(result,indent=2)+"\n",encoding="utf-8")
    if not passed: raise RuntimeError("combined acceptance failed; all observations retained")


if __name__ == "__main__": main()
