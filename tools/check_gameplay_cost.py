"""Check the independent gameplay diagnostic, or explicitly run a new cost suite.

Measurement never launches the desktop, opens devices, or reads client media.
"""
import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import threading
import time

from check_core import ROOT, linker_arguments, tool_environment
from gameplay_cost_platform import OwnedProcess
from gameplay_cost_protocol import tool_graph_errors, tool_source_errors, validate_process_point
from gameplay_cost_query import (PHASES, SCHEMA, PLAN, PARAMETERS, case_ordinal, summarize,
                                 validate_end, validate_lifecycle, validate_row, validate_durations, validate_canary)
import gameplay_cost_custody as custody
import gameplay_cost_query as query
import gameplay_cost_closure as closure
import gameplay_cost_pc128 as pc128

MAX_LINE = 32*1024*1024
SAMPLE_LIMIT = 120000
PHASE_TIMEOUT = 60


def safe_output(value):
    output = custody.plain_path(value)
    base = custody.plain_path(ROOT/'artifacts')
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
    frozen=identity()
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
    canary = subprocess.check_output([cargo, *linker, 'test', *common, '--all-features',
        'tests::query_wire_canary_without_clocks', '--', '--exact', '--nocapture'], cwd=ROOT, env=env, text=True)
    cases=validate_canary(canary,custody.decode)
    print(f'Query wire canary: {cases} real Rust cases, no timing/native observation',flush=True)
    command=[cargo,*linker,'test',*common,'--all-features','tests::closure_wire_canary_without_clocks','--','--ignored','--exact','--nocapture']
    cases=check_full_canary(command,env)
    print(f'Complete wire canary: {cases} real Rust cases, no timing/native observation',flush=True)
    build_arguments = [cargo, *linker, 'build', *common, '--release', '--message-format=json']
    output = subprocess.check_output(build_arguments, cwd=ROOT, env=env, text=True)
    executable = built_executable(output)
    compiler = subprocess.check_output([env['RUSTC'], '-vV'], env=env, text=True)
    if identity()!=frozen:
        raise ValueError('Source changed while checking/building the diagnostic; do not measure this binary')
    return executable, compiler, frozen, custody.hash_file(executable,custody.MAX_BINARY)['sha256'], build_arguments


def check_full_canary(command, env):
    with subprocess.Popen(command,cwd=ROOT,env=env,stdout=subprocess.PIPE) as child:
        largest=0
        def lines():
            nonlocal largest
            while True:
                line=child.stdout.readline(1024*1024+1)
                if not line: return
                if len(line)>1024*1024: raise ValueError('Full canary line capacity')
                if line.startswith(b'PC_CANARY\t'):largest=max(largest,len(line))
                yield line.decode('utf-8')
        try:
            cases=pc128.validate_canary(lines(),custody.decode)
            if child.wait(timeout=30)!=0: raise ValueError('Full canary child failed')
        except BaseException:
            child.terminate()
            try: child.wait(timeout=5)
            except subprocess.TimeoutExpired: child.kill(); child.wait(timeout=5)
            raise
        print(f'PC128 supplemental wire maximum: {largest} bytes including prefix/newline; limit=1048576',flush=True)
    return cases


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
                               'tools/gameplay_cost_query.py', 'tools/gameplay_cost_custody.py', 'tools/gameplay_cost_closure.py',
                               'tools/test_check_gameplay_closure.py', 'tools/gameplay_cost_pc128.py', 'tools/test_check_gameplay_pc128.py',
                               'tools/test_check_gameplay_query.py', 'tools/test_check_gameplay_custody.py',
                               'docs/K1_COST_CLOSURE_SPEC.md',
                               'tools/test_check_gameplay_cost.py', 'tools/check_core_architecture.py')]
    return {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(source)}


def cpu_reference_valid(user_ns, kernel_ns, action_ns, quantum):
    return (action_ns >= 2_000_000_000 and quantum is not None and quantum > 0
            and 0.75 <= (user_ns+kernel_ns)/action_ns <= 1.25)


def decode_line(line):
    if len(line)>MAX_LINE or not line.endswith(b'\n'):
        raise ValueError('Unbounded/incomplete child line')
    row=custody.decode(line)
    if not isinstance(row,dict):raise ValueError('Protocol row must be an object')
    return row


def completion_timeout(child_begin):
    remaining=300-(time.monotonic_ns()-child_begin)/1e9
    if remaining<=0: raise ValueError('Total child watchdog exceeded before completion')
    return min(5,remaining)


def process_summary(point,peaks,samples,gap,interval,quantum,reference_valid,previous=None):
    user=None if previous is None else point['user_ns']-previous['user_ns']
    kernel=None if previous is None else point['kernel_ns']-previous['kernel_ns']
    reliable=(reference_valid and user is not None and quantum is not None and user+kernel>=20*quantum
              and interval is not None and interval>0)
    return dict(boundary=point,samples=samples,sampled_peaks=peaks,max_sample_gap_ns=gap,parent_interval_ns=interval,
                user_delta_ns=user,kernel_delta_ns=kernel,cpu_status='Valid' if reliable else 'Unknown',
                one_core_percent=100*(user+kernel)/interval if reliable else None,
                memory_boundary_status='Valid',sampled_peak_scope='maximum of recorded samples only',
                stage_peak_status='Unknown',stage_peak_reason='finite sampling may miss transients; no stage peak counter',
                os_peak_scope='process_lifetime',sampling_target_met=gap<=1_000_000)


def run_probe(executable, trace, budget, repetition, output, plan=query):
    validator=plan.Stream() if plan in (closure,pc128) else None
    name = f'{trace}-b{budget}-r{repetition}'
    record = dict(trace=trace, budget=budget, repetition=repetition, objects=10000, status='Invalid', phases=[])
    messages = queue.Queue(maxsize=2)
    stop_reader = threading.Event()
    observer_begin = time.process_time_ns()
    child_begin = time.monotonic_ns()
    total_samples = 0
    eof = False
    exit_code = None
    with (output/(name+'.stderr.log')).open('x', encoding='utf-8') as stderr, \
            (output/(name+'.child.jsonl')).open('xb') as raw, \
            (output/(name+'.samples.csv')).open('x', encoding='utf-8', newline='') as samples:
        process = subprocess.Popen([str(executable), trace, str(budget), str(case_ordinal(trace,budget,repetition))], cwd=ROOT,
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
        writer = csv.writer(samples)
        keys = ('user_ns', 'kernel_ns', 'working_set_bytes', 'private_commit_bytes', 'os_peak_working_set_bytes', 'os_peak_commit_bytes')
        previous = None
        uncertainty = 1
        cpu_quantum = None
        reference_valid = False
        previous_boundary_clock = None
        try:
            reader.start()
            writer.writerow(('phase', 'parent_monotonic_ns', *keys))
            monitor = OwnedProcess(process.pid)
            for phase in plan.PHASES:
                deadline = time.monotonic()+PHASE_TIMEOUT
                count = 0
                max_gap = 0
                last_clock = previous_boundary_clock
                maxima = {key: 0 for key in keys[2:]}
                previous_sample = previous
                min_positive = None
                while True:
                    if total_samples>=SAMPLE_LIMIT or time.monotonic()>deadline or time.monotonic_ns()-child_begin>300_000_000_000:
                        raise ValueError('Bounded process observation timed out or exceeded total sample capacity')
                    clock = time.perf_counter_ns()
                    point = validate_process_point(monitor.sample(),previous_sample)
                    writer.writerow((phase, clock, *(point[key] for key in keys)))
                    total_samples += 1
                    count += 1
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
                    row = plan.validate_row(decode_line(line), phase, trace, budget, repetition)
                    if validator is not None: row=validator.consume(row)
                    if total_samples>=SAMPLE_LIMIT or time.monotonic()>deadline or time.monotonic_ns()-child_begin>300_000_000_000:
                        raise ValueError('Validation exceeded total observation capacity/deadline')
                    clock = time.perf_counter_ns()
                    point = validate_process_point(monitor.sample(),previous_sample)
                    total_samples += 1
                    delta=point['user_ns']+point['kernel_ns']-previous_sample['user_ns']-previous_sample['kernel_ns']
                    if delta>0:min_positive=delta if min_positive is None else min(min_positive,delta)
                    if last_clock is not None:
                        max_gap=max(max_gap,clock-last_clock)
                    if time.monotonic()>deadline:
                        raise ValueError('Protocol validation exceeded the phase observation deadline')
                    writer.writerow((phase, clock, *(point[key] for key in keys)))
                    for key in maxima:
                        maxima[key] = max(maxima[key], point[key])
                    delta_user = None if previous is None else point['user_ns']-previous['user_ns']
                    delta_kernel = None if previous is None else point['kernel_ns']-previous['kernel_ns']
                    if delta_user is not None and (delta_user < 0 or delta_kernel < 0):
                        raise ValueError('CPU boundary counter regressed')
                    interval = None if previous_boundary_clock is None else clock-previous_boundary_clock
                    if interval is not None and row['elapsed_ns']>interval:
                        raise ValueError('Action exceeds observed parent interval')
                    if phase == 'calibrated':
                        c = row['calibration']
                        uncertainty = max(c['observed_resolution_ns'], row['metrics'][0]['p95_ns'], 1)
                        cpu_quantum = min_positive
                        reference_valid = cpu_reference_valid(delta_user, delta_kernel, row['elapsed_ns'], cpu_quantum)
                        if not reference_valid:
                            raise ValueError('Owned continuous representative CPU reference failed')
                    result = plan.summarize(row, uncertainty)
                    result['process'] = process_summary(point,maxima,count+1,max_gap,interval,cpu_quantum,reference_valid,previous)
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
            line = messages.get(timeout=completion_timeout(child_begin))
            if line is None or len(line) > MAX_LINE:
                raise ValueError('Missing bounded end receipt')
            raw.write(line)
            raw.flush()
            end = plan.validate_end(decode_line(line),trace,budget,repetition)
            record['phases'][-1]['emit_ns'] = end['previous_emit_ns']
            record['phases'][-1]['gate_wait_ns'] = end['previous_gate_ns']
            process.stdin.close()
            exit_code = process.wait(timeout=completion_timeout(child_begin))
            if exit_code != 0:
                raise ValueError('Child reported failure')
            trailing = messages.get(timeout=completion_timeout(child_begin))
            if trailing is not None:
                raw.write(trailing); raw.flush()
                raise ValueError('Unexpected trailing protocol output')
            eof = True
            if validator is not None: validator.finish()
            plan.validate_lifecycle(record['phases'])
            record['status'] = 'Valid'
        except (OSError, ValueError, KeyError, TypeError, queue.Empty, subprocess.SubprocessError) as error:
            record['reason'] = str(error)
        finally:
            stop_reader.set()
            cleanup=[]
            try:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=3)
            except (OSError,subprocess.SubprocessError) as error:
                cleanup.append('Child cleanup: '+str(error))
            if reader.ident is not None:
                reader.join(timeout=3)
            reader_alive=reader.is_alive()
            if reader_alive:
                cleanup.append('Observer reader failed to stop; stdout close deferred')
            # Buffered stdout.close can wait on a reader's lock forever when
            # termination failed. Do not enter it until that reader has exited.
            for pipe in ((process.stdin,) if reader_alive else (process.stdout,process.stdin)):
                try:
                    if not pipe.closed:pipe.close()
                except OSError as error:cleanup.append('Pipe cleanup: '+str(error))
            if monitor:
                try:monitor.close()
                except OSError as error:cleanup.append('Monitor cleanup: '+str(error))
            if cleanup:
                record.update(status='Invalid',reason='; '.join(([record['reason']] if 'reason' in record else [])+cleanup))
    record['observer_cpu_ns_excluded_from_child'] = time.process_time_ns()-observer_begin
    elapsed_parent = time.monotonic_ns()-child_begin
    if elapsed_parent > 300_000_000_000:
        record.update(status='Invalid',reason='Total child watchdog exceeded')
    if record['status']=='Valid':
        try: plan.validate_durations(record['phases'],elapsed_parent)
        except ValueError as error: record.update(status='Invalid',reason=str(error))
    record['completion'] = dict(schema_version=SCHEMA,plan_id=plan.PLAN,case_ordinal=case_ordinal(trace,budget,repetition),pid=process.pid,
                                exit_code=exit_code,eof=eof,cleanup_complete=not cleanup,
                                observer_cpu_ns_excluded_from_child=record['observer_cpu_ns_excluded_from_child'],elapsed_parent_ns=elapsed_parent)
    record['raw_file'] = name+'.child.jsonl'
    record['raw_sha256'] = custody.hash_file(output/record['raw_file'],(len(plan.PHASES)+2)*MAX_LINE)['sha256']
    record['process_samples_file'] = name+'.samples.csv'
    return record


def supports_observation():
    return os.name == 'nt'


def measure(executable, compiler, output, repetitions, frozen, binary_hash, build_arguments, plan=query):
    output = custody.plain_path(output)
    if repetitions != 2:
        raise ValueError(plan.PLAN+' requires exactly two processes per case')
    if output.exists():
        raise ValueError('Measurement output must be new; existing evidence is never overwritten')
    report = dict(schema_version=SCHEMA,plan_id=plan.PLAN,status='Invalid',coverage='NeedsEvidence',
                  product_status='NotEvaluated',provisional=True,source_sha256=frozen,runs=[])
    try:
        if identity()!=frozen:
            raise ValueError('Source changed after build')
        if not supports_observation():
            report.update(status='NotRun',observation_status='Unknown',reason='Owned process observation supports Windows only')
            output.mkdir(parents=True,exist_ok=False)
            partial(output,report)
            return report
        manifest, executable = custody.freeze(executable,compiler,frozen,binary_hash,output,build_arguments,identity,plan)
        freeze_hash=custody.hash_file(output/'freeze.json',1048576)
        for name,trace,budget,repetition in custody.names():
            before_source=custody.digest(identity())
            before_binary=custody.hash_file(executable,custody.MAX_BINARY)['sha256']
            if before_source!=manifest['source_map_sha256'] or before_binary!=binary_hash or custody.hash_file(output/'freeze.json',1048576)!=freeze_hash:
                raise ValueError('Frozen source, binary or manifest changed before child')
            run=run_probe(executable,trace,budget,repetition,output) if plan is query else run_probe(executable,trace,budget,repetition,output,plan)
            after_source=custody.digest(identity())
            after_binary=custody.hash_file(executable,custody.MAX_BINARY)['sha256']
            freeze_changed=custody.hash_file(output/'freeze.json',1048576)!=freeze_hash
            if freeze_changed: run.update(status='Invalid',reason='Freeze manifest changed during child')
            receipt=dict(run['completion'],source_before=before_source,source_after=after_source,
                         binary_before=before_binary,binary_after=after_binary)
            if run['status']!='Valid': receipt.update(failure=run.get('reason','Invalid child'))
            custody.write_json(output/(name+'.process.json'),receipt)
            report['runs'].append(run)
            partial(output,report)
            if run['status']!='Valid' or after_source!=before_source or after_binary!=before_binary:
                raise ValueError('Child failed or source/frozen copy changed after child')
        if (identity()!=frozen or custody.hash_file(executable,custody.MAX_BINARY)['sha256']!=binary_hash
                or custody.hash_file(output/'freeze.json',1048576)!=freeze_hash):
            raise ValueError('Frozen evidence changed at final boundary')
        return custody.publish(output,manifest)
    except (OSError,ValueError,KeyError,TypeError,csv.Error,subprocess.SubprocessError) as error:
        report['reason']=str(error)
        if output.is_dir(): partial(output,report)
        return report


def partial(output, report):
    path=custody.relative(output,'report.partial')
    with path.open('w',encoding='utf-8') as file:
        json.dump(report,file,sort_keys=True)
        file.flush()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument('--check', action='store_true')
    action.add_argument('--measure', action='store_true')
    action.add_argument('--verify', metavar='EVIDENCE_DIRECTORY', help='read-only Query-v1 / CostClosure-v1 / CostClosure-PC128-v1 evidence verification; no build or execution')
    parser.add_argument('--toolchain-dir')
    parser.add_argument('--output')
    parser.add_argument('--repetitions', type=int, choices=(2,), default=2)
    args = parser.parse_args()
    if args.verify:
        if args.output or args.toolchain_dir:
            parser.error('--verify cannot build or write an output directory')
        print(json.dumps(custody.verify(Path(args.verify)),sort_keys=True))
        return
    output = safe_output(args.output) if args.measure and args.output else None
    if args.measure and output is None:
        parser.error('--measure requires a new --output below artifacts/')
    if args.check and args.output:
        parser.error('--output requires --measure')
    executable, compiler, frozen, binary_hash, build_arguments = check_tool(args.toolchain_dir)
    if args.check:
        print('PASS: independent gameplay cost tool format/Clippy/tests/release and dependency/source rules')
        return
    report = measure(executable, compiler, output, args.repetitions, frozen, binary_hash, build_arguments, pc128)
    print('Measurement protocol status:', report['status'], '; coverage:', report['coverage'], '; product verdict: NotEvaluated;', output)
    if report['status'] != 'Valid':
        raise RuntimeError('Incomplete/invalid/unknown measurement; original evidence retained')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, RuntimeError, csv.Error, subprocess.SubprocessError) as error:
        print('Gameplay cost check failed: '+str(error), file=sys.stderr)
        sys.exit(1)
