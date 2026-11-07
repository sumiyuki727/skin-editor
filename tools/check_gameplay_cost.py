"""Check the independent gameplay diagnostic, or explicitly run a new cost suite.

Measurement never launches the desktop, opens devices, or reads client media.
"""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import platform
import queue
import subprocess
import sys
import threading
import time

from check_core import ROOT, linker_arguments, tool_environment
from gameplay_cost_platform import OwnedProcess
from gameplay_cost_protocol import PHASES, SCHEMA, summarize, tool_graph_errors, tool_source_errors, validate_row

MAX_LINE = 32*1024*1024
SAMPLE_LIMIT = 120000
PHASE_TIMEOUT = 60


def safe_output(value):
    output = Path(value).resolve()
    base = (ROOT/'artifacts').resolve()
    if output == base or base not in output.parents or output.exists():
        raise ValueError('Output must be a new directory below artifacts/')
    return output


def built_executable(output):
    files = [m['executable'] for m in map(json.loads, output.splitlines())
             if m.get('reason') == 'compiler-artifact' and m.get('target', {}).get('name') == 'skin-editor-gameplay-cost' and m.get('executable')]
    if len(files) != 1:
        raise ValueError('Exactly one current Cargo executable is required')
    return Path(files[0])


def check_tool(directory=None):
    cargo, env = tool_environment(directory)
    validate_build_environment(env)
    manifest = ROOT/'tools/gameplay-cost/Cargo.toml'
    common = ['--manifest-path', str(manifest), '--offline', '--locked']
    metadata = json.loads(subprocess.check_output([cargo, 'metadata', *common, '--format-version', '1', '--all-features'], cwd=ROOT, env=env, text=True))
    errors = tool_graph_errors(metadata, ROOT)+tool_source_errors(ROOT)
    if errors:
        raise ValueError('\n'.join(errors))
    linker = linker_arguments(env)
    commands = [[cargo, 'fmt', '--manifest-path', str(manifest), '--', '--check'],
                [cargo, *linker, 'clippy', *common, '--all-targets', '--all-features', '--', '-D', 'warnings'],
                [cargo, *linker, 'test', *common, '--all-features']]
    for command in commands:
        subprocess.run(command, cwd=ROOT, env=env, check=True)
    output = subprocess.check_output([cargo, *linker, 'build', *common, '--release', '--message-format=json'], cwd=ROOT, env=env, text=True)
    executable = built_executable(output)
    compiler = subprocess.check_output([env['RUSTC'], '-vV'], env=env, text=True)
    return executable, compiler


def validate_build_environment(environment):
    if any(value and (name.startswith('CARGO_PROFILE_') or name in ('CARGO_BUILD_TARGET','CARGO_INCREMENTAL','RUSTC_BOOTSTRAP')
                      or (name.startswith('CARGO_TARGET_') and name.endswith('_RUSTFLAGS')))
           for name,value in environment.items()):
        raise ValueError('Unset Cargo profile/target/compiler overrides before the frozen native cost build')


def identity():
    source = list((ROOT/'crates/gameplay').rglob('*.rs'))+[ROOT/'crates/gameplay/Cargo.toml']
    source += list((ROOT/'tools/gameplay-cost/src').glob('*.rs'))
    source += [ROOT/p for p in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'tools/gameplay-cost/Cargo.toml',
                               'tools/gameplay-cost/Cargo.lock', 'tools/check_core.py', 'tools/check_gameplay_cost.py',
                               'tools/gameplay_cost_protocol.py', 'tools/gameplay_cost_platform.py',
                               'tools/test_check_gameplay_cost.py', 'tools/check_core_architecture.py')]
    return {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(source)}


def cpu_reference_valid(user_ns, kernel_ns, action_ns, quantum):
    return (action_ns >= 2_000_000_000 and quantum is not None and quantum > 0
            and 0.75 <= (user_ns+kernel_ns)/action_ns <= 1.25)


def run_probe(executable, trace, budget, repetition, output):
    name = f'{trace}-b{budget}-r{repetition}'
    record = dict(trace=trace, budget=budget, repetition=repetition, objects=10000, status='Invalid', phases=[])
    messages = queue.Queue(maxsize=2)
    stop_reader = threading.Event()
    observer_begin = time.process_time_ns()
    with (output/(name+'.stderr.log')).open('x', encoding='utf-8') as stderr, \
            (output/(name+'.child.jsonl')).open('xb') as raw, \
            (output/(name+'.samples.csv')).open('x', encoding='utf-8', newline='') as samples:
        process = subprocess.Popen([str(executable), trace, str(budget), '10000'], cwd=ROOT,
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr)
        record['pid']=process.pid
        monitor = None

        def read_lines():
            try:
                while not stop_reader.is_set():
                    line = process.stdout.readline(MAX_LINE+1)
                    if not line:
                        break
                    try:
                        messages.put(line,timeout=1)
                    except queue.Full:
                        return
                    if len(line) > MAX_LINE:
                        break
            except (OSError,ValueError):
                pass
            finally:
                try:
                    messages.put(None,timeout=1)
                except queue.Full:
                    pass

        reader = threading.Thread(target=read_lines, daemon=True)
        reader.start()
        writer = csv.writer(samples)
        keys = ('user_ns', 'kernel_ns', 'working_set_bytes', 'private_commit_bytes', 'os_peak_working_set_bytes', 'os_peak_commit_bytes')
        writer.writerow(('phase', 'parent_monotonic_ns', *keys))
        previous = None
        uncertainty = 1
        cpu_quantum = None
        reference_valid = False
        previous_boundary_clock = None
        try:
            monitor = OwnedProcess(process.pid)
            for phase in PHASES:
                deadline = time.monotonic()+PHASE_TIMEOUT
                count = 0
                max_gap = 0
                last_clock = None
                maxima = {key: 0 for key in keys[2:]}
                previous_sample = None
                min_positive = None
                while True:
                    clock = time.perf_counter_ns()
                    point = monitor.sample()
                    writer.writerow((phase, clock, *(point[key] for key in keys)))
                    count += 1
                    if count > SAMPLE_LIMIT or time.monotonic() > deadline:
                        raise ValueError('Bounded process observation timed out or exceeded sample capacity')
                    if last_clock is not None:
                        max_gap = max(max_gap, clock-last_clock)
                    last_clock = clock
                    if previous_sample is not None:
                        delta = point['user_ns']+point['kernel_ns']-previous_sample['user_ns']-previous_sample['kernel_ns']
                        if delta < 0:
                            raise ValueError('CPU counter regressed')
                        if delta > 0:
                            min_positive = delta if min_positive is None else min(min_positive, delta)
                    previous_sample = point
                    for key in maxima:
                        maxima[key] = max(maxima[key], point[key])
                    try:
                        line = messages.get(timeout=0.001)
                    except queue.Empty:
                        continue
                    if line is None:
                        raise ValueError('Child exited before completing '+phase)
                    raw.write(line)
                    raw.flush()
                    if len(line) > MAX_LINE or not line.endswith(b'\n'):
                        raise ValueError('Unbounded/malformed child line')
                    row = validate_row(json.loads(line), phase, trace, budget)
                    clock = time.perf_counter_ns()
                    point = monitor.sample()
                    if last_clock is not None:
                        max_gap=max(max_gap,clock-last_clock)
                    writer.writerow((phase, clock, *(point[key] for key in keys)))
                    for key in maxima:
                        maxima[key] = max(maxima[key], point[key])
                    delta_user = None if previous is None else point['user_ns']-previous['user_ns']
                    delta_kernel = None if previous is None else point['kernel_ns']-previous['kernel_ns']
                    if delta_user is not None and (delta_user < 0 or delta_kernel < 0):
                        raise ValueError('CPU boundary counter regressed')
                    interval = None if previous_boundary_clock is None else clock-previous_boundary_clock
                    if phase == 'calibrated':
                        c = row['calibration']
                        uncertainty = max(c['observed_resolution_ns'], row['metrics'][0]['p95_ns'], 1)
                        cpu_quantum = min_positive
                        reference_valid = cpu_reference_valid(delta_user, delta_kernel, row['elapsed_ns'], cpu_quantum)
                        if not reference_valid:
                            raise ValueError('Owned continuous representative CPU reference failed')
                    result = summarize(row, uncertainty)
                    reliable_cpu = (reference_valid and delta_user is not None and cpu_quantum is not None
                                    and delta_user+delta_kernel >= 20*cpu_quantum and interval > 0)
                    result['process'] = dict(boundary=point, samples=count+1, sampled_peaks=maxima,
                                             max_sample_gap_ns=max_gap, parent_interval_ns=interval,
                                             user_delta_ns=delta_user, kernel_delta_ns=delta_kernel,
                                             cpu_status='Valid' if reliable_cpu else 'Unknown',
                                             one_core_percent=None if not reliable_cpu else 100*(delta_user+delta_kernel)/interval)
                    if phase == 'calibrated':
                        result['cpu_reference_valid'] = reference_valid
                        result['observed_min_cpu_increment_ns'] = cpu_quantum
                        result['call_uncertainty_proxy_ns'] = uncertainty
                    if record['phases']:
                        record['phases'][-1]['emit_ns'] = row['previous_emit_ns']
                        record['phases'][-1]['gate_wait_ns'] = row['previous_gate_ns']
                    record['phases'].append(result)
                    previous, previous_boundary_clock = point, clock
                    process.stdin.write(b'continue\n')
                    process.stdin.flush()
                    print(name, phase, 'work_ms=%.3f' % (row['elapsed_ns']/1e6), flush=True)
                    break
            line = messages.get(timeout=5)
            if line is None or len(line) > MAX_LINE:
                raise ValueError('Missing bounded end receipt')
            raw.write(line)
            end = json.loads(line)
            if type(end.get('schema_version')) is not int or end.get('schema_version') != SCHEMA or end.get('phase') != 'end':
                raise ValueError('Missing versioned end receipt')
            record['phases'][-1]['emit_ns'] = end['previous_emit_ns']
            record['phases'][-1]['gate_wait_ns'] = end['previous_gate_ns']
            process.stdin.close()
            if process.wait(timeout=5) != 0:
                raise ValueError('Child reported failure')
            if messages.get(timeout=5) is not None:
                raise ValueError('Unexpected trailing protocol output')
            inspected = record['phases'][5]['managed']
            dropped = record['phases'][7]['managed']
            if any(inspected[key] != dropped[key] for key in ('source_reserved', 'runtime_reserved', 'source_payload', 'runtime_payload', 'source_allocations', 'runtime_allocations')):
                raise ValueError('Cancelled task did not return to the active inspection baseline')
            record['status'] = 'Valid'
        except (OSError, ValueError, KeyError, TypeError, queue.Empty, json.JSONDecodeError) as error:
            record['reason'] = str(error)
        finally:
            stop_reader.set()
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=3)
            process.stdout.close()
            if not process.stdin.closed:
                process.stdin.close()
            if monitor:
                monitor.close()
            reader.join(timeout=3)
            if reader.is_alive():
                record.update(status='Invalid',reason='Observer reader failed to stop')
    record['observer_cpu_ns_excluded_from_child'] = time.process_time_ns()-observer_begin
    record['raw_file'] = name+'.child.jsonl'
    record['raw_sha256'] = hashlib.sha256((output/record['raw_file']).read_bytes()).hexdigest()
    record['process_samples_file'] = name+'.samples.csv'
    return record


def measure(executable, compiler, output, repetitions):
    frozen = identity()
    binary_hash = hashlib.sha256(executable.read_bytes()).hexdigest()
    clock = time.get_clock_info('perf_counter')
    report = dict(schema_version=SCHEMA, status='NotRun', product_status='NotEvaluated', api_version=7, profile_version=1,
                  status_semantics='Valid validates frozen protocol/workload/references/receipts; timing and CPU confidence remain per metric',
                  head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  head_is_complete_source_identity=False, source_sha256=frozen, executable_sha256=binary_hash,
                  compiler=compiler, build='release --offline --locked; no features; always-on local WorkCounts',
                  platform=platform.platform(), cpu=platform.processor() or os.environ.get('PROCESSOR_IDENTIFIER', 'Unknown'),
                  logical_cpus=os.cpu_count(), pointer_bits=64 if sys.maxsize > 2**32 else 32,
                  parent_clock=dict(implementation=clock.implementation, resolution_seconds=clock.resolution, monotonic=clock.monotonic),
                  child_clock='std::time::Instant; observed resolution and empty-call reference; no overhead subtraction',
                  protocol=dict(objects=10000, traces=['g0', 'g1', 'a1'], budgets=[1, 256], repetitions=repetitions,
                                phases=PHASES, input_batch_objects=32, query_batch_pages=4096, cancel_probe_budget=1,
                                control_batch_cycles=256, sampling_target_ms=1, phase_timeout_seconds=PHASE_TIMEOUT,
                                cpu_reference_minimum_seconds=2, cpu_reference_ratio_range=[0.75, 1.25],
                                cpu_reference_budget=256, small_reference_objects=32, large_reference_objects=2048,
                                reference_repetitions=4,
                                call_resolution_proxy_multiplier=20, stage_resolution_proxy_multiplier=100),
                  scope='owned synthetic child; diagnostic samples/validation included, parent observer excluded; no GUI/devices/media',
                  limitations=['single-call raw percentiles below the resolution proxy are Unknown; batch averages are not call percentiles',
                               'CPU intervals include protocol emission/waits; short intervals are Unknown, never zero-cost passes',
                               'continuous busy reference is not validation for periodic real-time playback or product CPU limits',
                               'observed minimum CPU increment is not a worst-case quantization bound',
                               'sampled memory may miss transients; OS peaks are cumulative; allocator/control-header/full-stack costs not isolated',
                               'released has no core observer: managed/work fields null; explicit Drop and OS boundary are separate evidence'], runs=[])
    output.mkdir(parents=True, exist_ok=False)
    with (output/'identity.json').open('x', encoding='utf-8') as file:
        json.dump(report, file, indent=2)
    if os.name != 'nt':
        report.update(status='Unknown', reason='Process measurement v1 supports Windows only')
    else:
        for trace in ('g0', 'g1', 'a1'):
            for budget in (1, 256):
                for repetition in range(1, repetitions+1):
                    if identity() != frozen or hashlib.sha256(executable.read_bytes()).hexdigest() != binary_hash:
                        report.update(status='Invalid', reason='Frozen source or binary changed during measurement')
                        break
                    try:
                        run = run_probe(executable, trace, budget, repetition, output)
                    except (OSError,ValueError,subprocess.SubprocessError) as error:
                        run = dict(trace=trace,budget=budget,repetition=repetition,objects=10000,status='Invalid',reason=str(error),phases=[])
                    report['runs'].append(run)
                    report['status'] = 'Invalid' if run['status'] != 'Valid' else 'NotRun'
                    with (output/'report.json').open('w', encoding='utf-8') as file:
                        json.dump(report, file, indent=2)
                    if run['status'] != 'Valid':
                        break
                if report['status'] == 'Invalid':
                    break
            if report['status'] == 'Invalid':
                break
        if report['status'] != 'Invalid':
            report['status'] = 'Valid' if (len(report['runs']) == 6*repetitions and identity() == frozen
                                          and hashlib.sha256(executable.read_bytes()).hexdigest() == binary_hash) else 'Invalid'
            if report['status']=='Invalid':
                report['reason']='Incomplete run set or frozen source/binary changed at the final boundary'
    with (output/'report.json').open('w', encoding='utf-8') as file:
        json.dump(report, file, indent=2)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument('--check', action='store_true')
    action.add_argument('--measure', action='store_true')
    parser.add_argument('--toolchain-dir')
    parser.add_argument('--output')
    parser.add_argument('--repetitions', type=int, choices=(2, 3), default=2)
    args = parser.parse_args()
    output = safe_output(args.output) if args.measure and args.output else None
    if args.measure and output is None:
        parser.error('--measure requires a new --output below artifacts/')
    if args.check and args.output:
        parser.error('--output requires --measure')
    executable, compiler = check_tool(args.toolchain_dir)
    if args.check:
        print('PASS: independent gameplay cost tool format/Clippy/tests/release and dependency/source rules')
        return
    report = measure(executable, compiler, output, args.repetitions)
    print('Measurement protocol status:', report['status'], '; product verdict: NotEvaluated;', output/'report.json')
    if report['status'] != 'Valid':
        raise RuntimeError('Incomplete/invalid/unknown measurement; original evidence retained')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print('Gameplay cost check failed: '+str(error), file=sys.stderr)
        sys.exit(1)
