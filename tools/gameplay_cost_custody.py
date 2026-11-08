"""Bounded non-adversarial custody and read-only derivation of Query-v1 evidence.

This module neither builds nor launches a binary. Hashes bind bytes, not authenticity.
"""
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import stat
import time

import gameplay_cost_query as query
from gameplay_cost_protocol import natural, validate_process_point

MAX_BINARY = 16*1024*1024
MAX_LINE = 32*1024*1024
SPEC = 'docs/K1_COST_CLOSURE_SPEC.md'
COUNTERS = ('user_ns','kernel_ns','working_set_bytes','private_commit_bytes','os_peak_working_set_bytes','os_peak_commit_bytes')


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True).encode('ascii')


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def decode(data):
    def unique(pairs):
        result={}
        for key,value in pairs:
            if key in result: raise ValueError('Duplicate JSON key: '+key)
            result[key]=value
        return result
    def bad(value): raise ValueError('Non-finite JSON: '+value)
    try:
        return json.loads(data,object_pairs_hook=unique,parse_constant=bad)
    except (UnicodeError, json.JSONDecodeError, RecursionError) as error:
        raise ValueError('Invalid bounded JSON') from error


def plain_path(path):
    """Reject links/reparse points including ancestors, without reading their targets."""
    path=Path(os.path.abspath(path))
    for item in reversed((path,*path.parents)):
        if item.exists() or item.is_symlink():
            info=item.lstat()
            if stat.S_ISLNK(info.st_mode) or getattr(info,'st_file_attributes',0)&0x400:
                raise ValueError('Links/reparse points are not evidence paths')
    return path.resolve()


def relative(root, name):
    if not isinstance(name,str) or not re.fullmatch(r'[a-z0-9][a-z0-9._/-]*',name) or any(p in ('','.','..') for p in name.split('/')):
        raise ValueError('Non-canonical evidence path')
    root=plain_path(root)
    path=plain_path(root/name)
    if root not in path.parents: raise ValueError('Evidence path outside root')
    if path.is_file() and path.stat().st_nlink != 1:
        raise ValueError('Evidence hard links cannot prove root confinement')
    return path


def hash_file(path, limit):
    path=plain_path(path)
    info=path.stat()
    if not stat.S_ISREG(info.st_mode) or info.st_size>limit: raise ValueError('Non-file or oversized evidence')
    h=hashlib.sha256(); length=0
    with path.open('rb') as stream:
        while True:
            block=stream.read(min(65536, limit-length+1))
            if not block: break
            length+=len(block)
            if length>limit: raise ValueError('Evidence grew beyond capacity')
            h.update(block)
    if length!=info.st_size or path.stat().st_size!=length: raise ValueError('Evidence changed while hashing')
    return dict(bytes=length,sha256=h.hexdigest())


def read_json(path, limit):
    receipt=hash_file(path,limit)
    with path.open('rb') as stream: data=stream.read(limit+1)
    if len(data)!=receipt['bytes'] or hashlib.sha256(data).hexdigest()!=receipt['sha256']: raise ValueError('Evidence changed while reading')
    return decode(data)


def write_json(path, value):
    plain_path(path)
    with path.open('xb') as stream:
        stream.write(canonical(value)+b'\n'); stream.flush(); os.fsync(stream.fileno())


def names():
    for trace in query.PARAMETERS['traces']:
        for budget in (1,256):
            for repetition in (1,2):
                yield f'{trace}-b{budget}-r{repetition}',trace,budget,repetition


def roles():
    result={'freeze.json':('freeze',1048576),'frozen/probe.exe':('binary',MAX_BINARY),'report.json':('report',16*1024*1024)}
    for name,_,_,_ in names():
        for suffix,role,limit in (('.child.jsonl','child',16*MAX_LINE),('.samples.csv','samples',64*1024*1024),
                                  ('.stderr.log','stderr',1048576),('.process.json','process',65536)):
            result[name+suffix]=(role,limit)
    return result


def validate_freeze(value):
    query.fields(value, ('schema_version','plan_id','validator','parameters','scope','product_verdict','source_sha256',
                         'source_map_sha256','spec_sha256','binary','compiler','build_arguments','target','pointer_bits','diagnostics','host_observer'), 'freeze')
    if type(value['schema_version']) is not int or value['schema_version']!=3 or value['plan_id']!=query.PLAN or value['validator']!=query.VALIDATOR or canonical(value['parameters'])!=canonical(query.PARAMETERS):
        raise ValueError('Unknown frozen plan/validator/parameters')
    if value['scope']!='QueryOnly' or value['product_verdict']!='NotEvaluated': raise ValueError('Frozen scope changed')
    source=value['source_sha256']
    if not isinstance(source,dict) or not 1<=len(source)<=256: raise ValueError('Source map capacity')
    aliases=set()
    for name,sha in source.items():
        if not isinstance(name,str) or not re.fullmatch(r'[A-Za-z0-9_./-]{1,200}',name) or any(p in ('','.','..') for p in name.split('/')) or name.lower() in aliases:
            raise ValueError('Source path alias')
        aliases.add(name.lower())
        if not isinstance(sha,str) or not re.fullmatch('[0-9a-f]{64}',sha): raise ValueError('Source hash')
    required={SPEC,'tools/gameplay-cost/src/query.rs','tools/gameplay_cost_query.py','tools/gameplay_cost_custody.py',
              'tools/gameplay-cost/src/main.rs','tools/gameplay-cost/src/driver.rs','tools/gameplay-cost/src/meter.rs',
              'crates/gameplay/Cargo.toml','crates/gameplay/src/lib.rs','crates/gameplay/src/types.rs',
              'tools/check_gameplay_cost.py','tools/gameplay_cost_protocol.py','tools/gameplay_cost_platform.py',
              'tools/test_check_gameplay_query.py','tools/test_check_gameplay_custody.py',
              'Cargo.toml','Cargo.lock','rust-toolchain.toml','tools/gameplay-cost/Cargo.toml','tools/gameplay-cost/Cargo.lock'}
    if not required<=set(source) or value['source_map_sha256']!=digest(source) or value['spec_sha256']!=source[SPEC]:
        raise ValueError('Incomplete/mismatched frozen source identity')
    query.fields(value['binary'],('path','bytes','sha256'),'frozen binary')
    if value['binary']['path']!='frozen/probe.exe' or not 0<natural(value['binary']['bytes'],'binary bytes',MAX_BINARY) or not re.fullmatch('[0-9a-f]{64}',str(value['binary']['sha256'])):
        raise ValueError('Frozen binary identity')
    if not isinstance(value['compiler'],str) or value['compiler'].split()[:2]!=['rustc','1.95.0'] or len(value['compiler'])>8192:
        raise ValueError('Missing compiler identity')
    if not isinstance(value['build_arguments'],list) or not 7<=len(value['build_arguments'])<=16 or any(not isinstance(s,str) or len(s)>2048 for s in value['build_arguments']):
        raise ValueError('Missing raw build arguments')
    command=value['build_arguments']
    if command[-6:] != ['--manifest-path',command[-5],'--offline','--locked','--release','--message-format=json'] or not command[-5].replace('\\','/').endswith('/tools/gameplay-cost/Cargo.toml') or 'build' not in command:
        raise ValueError('Changed release build arguments')
    prefix=command[:-6]
    if prefix!=[command[0],'build'] and not (len(prefix)==4 and prefix[1]=='--config' and prefix[3]=='build'
            and re.fullmatch(r'target\.x86_64-pc-windows-msvc\.linker="[^"\r\n]+/link\.exe"',prefix[2])):
        raise ValueError('Unknown compiler/linker arguments')
    if value['target'] not in ('x86_64-pc-windows-msvc','i686-pc-windows-msvc','aarch64-pc-windows-msvc') or type(value['pointer_bits']) is not int or value['pointer_bits']!=(32 if value['target'].startswith('i686') else 64) or '\nhost: '+value['target'] not in value['compiler']:
        raise ValueError('Unsupported/mismatched frozen target')
    if canonical(value['diagnostics']) != canonical(dict(always_on_work_counts=True,query_result_guards=True,query_nested_call_timers=False,outer_reference_subtracted=False)):
        raise ValueError('Changed diagnostic contract')
    host=value['host_observer']
    query.fields(host,('platform','processor','logical_cpus','parent_clock','child_clock'),'host observer')
    for key in ('platform','processor'):
        if not isinstance(host[key],str) or not 1<=len(host[key])<=4096: raise ValueError('Missing platform identity')
    if host['logical_cpus'] is not None and natural(host['logical_cpus'],'logical CPUs',65536)==0: raise ValueError('Invalid CPU count')
    query.fields(host['parent_clock'],('implementation','resolution_seconds','monotonic','adjustable'),'parent clock')
    clock=host['parent_clock'];resolution=clock['resolution_seconds']
    if (not isinstance(clock['implementation'],str) or not 1<=len(clock['implementation'])<=4096
            or clock['monotonic'] is not True or clock['adjustable'] is not False
            or type(resolution) not in (int,float) or not math.isfinite(resolution) or resolution<=0):
        raise ValueError('Invalid parent clock metadata')
    if host['child_clock']!='std::time::Instant; observed positive resolution and empty-call p95; no subtraction':
        raise ValueError('Changed child timebase')
    return value


def freeze(executable, compiler, source, expected_hash, output, build_arguments, source_now):
    """Exclusive copy first; publish freeze only after both original and copy match."""
    output=plain_path(output)
    if source_now()!=source: raise ValueError('Source changed after build')
    original=hash_file(executable,MAX_BINARY)
    if original['sha256']!=expected_hash or original['bytes']==0: raise ValueError('Build artifact changed')
    output.mkdir(parents=True,exist_ok=False)
    (output/'frozen').mkdir()
    target=relative(output,'frozen/probe.exe')
    with executable.open('rb') as src, target.open('xb') as dst:
        length=0
        while True:
            block=src.read(min(65536, MAX_BINARY-length+1))
            if not block: break
            length+=len(block)
            if length>MAX_BINARY: raise ValueError('Binary copy capacity')
            dst.write(block)
        dst.flush(); os.fsync(dst.fileno())
    if hash_file(target,MAX_BINARY)!=original or hash_file(executable,MAX_BINARY)!=original or source_now()!=source:
        raise ValueError('Copy/source changed during freeze')
    target_name=next((s[6:] for s in compiler.splitlines() if s.startswith('host: ')), '')
    clock=time.get_clock_info('perf_counter')
    manifest=dict(schema_version=3,plan_id=query.PLAN,validator=query.VALIDATOR,parameters=query.PARAMETERS,
                  scope='QueryOnly',product_verdict='NotEvaluated',source_sha256=source,source_map_sha256=digest(source),
                  spec_sha256=source.get(SPEC),binary=dict(path='frozen/probe.exe',**original),compiler=compiler,
                  build_arguments=build_arguments,target=target_name,pointer_bits=32 if target_name.startswith('i686') else 64,
                  host_observer=dict(platform=platform.platform(),processor=platform.processor() or 'Unknown',logical_cpus=os.cpu_count(),
                                     parent_clock=dict(implementation=clock.implementation,resolution_seconds=clock.resolution,
                                                       monotonic=clock.monotonic,adjustable=clock.adjustable),
                                     child_clock='std::time::Instant; observed positive resolution and empty-call p95; no subtraction'),
                  diagnostics=dict(always_on_work_counts=True,query_result_guards=True,query_nested_call_timers=False,outer_reference_subtracted=False))
    validate_freeze(manifest)
    write_json(output/'freeze.json',manifest)
    return manifest,target


def cpu_reference_valid(user_ns, kernel_ns, action_ns, quantum):
    return action_ns>=2000000000 and quantum is not None and quantum>0 and 0.75<=(user_ns+kernel_ns)/action_ns<=1.25


def process_summary(point,peaks,samples,gap,interval,quantum,reference_valid,previous=None):
    user=None if previous is None else point['user_ns']-previous['user_ns']
    kernel=None if previous is None else point['kernel_ns']-previous['kernel_ns']
    reliable=reference_valid and user is not None and quantum is not None and user+kernel>=20*quantum and interval is not None and interval>0
    return dict(boundary=point,samples=samples,sampled_peaks=peaks,max_sample_gap_ns=gap,parent_interval_ns=interval,
                user_delta_ns=user,kernel_delta_ns=kernel,cpu_status='Valid' if reliable else 'Unknown',
                one_core_percent=100*(user+kernel)/interval if reliable else None,memory_boundary_status='Valid',
                sampled_peak_scope='maximum of recorded samples only',stage_peak_status='Unknown',
                stage_peak_reason='finite sampling may miss transients; no stage peak counter',os_peak_scope='process_lifetime',sampling_target_met=gap<=1000000)


def process_points(path):
    """Stream/reduce at most 120000 CSV samples, including cross-ACK gaps."""
    hash_file(path,64*1024*1024)
    phases=[]; current=None; previous=None; previous_clock=None; total=0
    with path.open('r',encoding='utf-8',newline='') as stream:
        reader=csv.reader(stream)
        if next(reader,None)!=['phase','parent_monotonic_ns',*COUNTERS]: raise ValueError('CSV header')
        for cells in reader:
            if total>=120000: raise ValueError('Process sample capacity')
            total+=1
            if len(cells)!=8 or any(not re.fullmatch('[0-9]{1,20}',s) for s in cells[1:]): raise ValueError('CSV sample fields')
            phase=cells[0]; clock=natural(int(cells[1]),'parent clock')
            point=validate_process_point(dict(zip(COUNTERS,map(int,cells[2:]))),previous)
            if previous_clock is not None and clock<=previous_clock: raise ValueError('Non-monotonic observation clock')
            if current is None or phase!=current['phase']:
                if len(phases)>=len(query.PHASES) or phase!=query.PHASES[len(phases)]: raise ValueError('CSV role/order')
                current=dict(phase=phase,samples=0,peaks=dict.fromkeys(COUNTERS[2:],0),gap=0,quantum=None,previous=previous,previous_clock=previous_clock)
                current['first_clock']=clock
                phases.append(current)
            current['samples']+=1
            if previous_clock is not None: current['gap']=max(current['gap'],clock-previous_clock)
            if previous is not None:
                increment=point['user_ns']+point['kernel_ns']-previous['user_ns']-previous['kernel_ns']
                if increment>0: current['quantum']=min(current['quantum'] or increment,increment)
            for key in current['peaks']: current['peaks'][key]=max(current['peaks'][key],point[key])
            current['point']=point;current['clock']=clock
            previous,previous_clock=point,clock
    if len(phases)!=15 or any(p['samples']<2 for p in phases): raise ValueError('Missing process boundaries')
    if phases[-1]['clock']-phases[0]['clock']>300000000000: raise ValueError('Child total watchdog')
    return phases


def derive_run(root, manifest, name, trace, budget, repetition):
    receipt=read_json(relative(root,name+'.process.json'),65536)
    query.fields(receipt, ('schema_version','plan_id','case_ordinal','pid','exit_code','eof','cleanup_complete','observer_cpu_ns_excluded_from_child',
                           'elapsed_parent_ns','source_before','source_after','binary_before','binary_after'), 'process receipt')
    if type(receipt['schema_version']) is not int or receipt['schema_version']!=3 or receipt['plan_id']!=query.PLAN or natural(receipt['case_ordinal'],'case')!=query.case_ordinal(trace,budget,repetition): raise ValueError('Process identity')
    if type(receipt['exit_code']) is not int or receipt['exit_code']!=0 or receipt['eof'] is not True or receipt['cleanup_complete'] is not True or natural(receipt['pid'],'pid')==0: raise ValueError('Incomplete owned child cleanup')
    natural(receipt['observer_cpu_ns_excluded_from_child'],'observer CPU')
    natural(receipt['elapsed_parent_ns'],'child lifetime',300000000000)
    if any(receipt[k]!=manifest['source_map_sha256'] for k in ('source_before','source_after')) or any(receipt[k]!=manifest['binary']['sha256'] for k in ('binary_before','binary_after')): raise ValueError('Per-child freeze changed')
    observed=process_points(relative(root,name+'.samples.csv'))
    if observed[-1]['clock']-observed[0]['first_clock']>receipt['elapsed_parent_ns']:
        raise ValueError('Observation span exceeds child lifetime receipt')
    raw_path=relative(root,name+'.child.jsonl'); hash_file(raw_path,16*MAX_LINE)
    phases=[]; epsilon=1; quantum=None; reference=False
    with raw_path.open('rb') as stream:
        for i,phase in enumerate(query.PHASES):
            line=stream.readline(MAX_LINE+1)
            if len(line)>MAX_LINE or not line.endswith(b'\n'): raise ValueError('Missing/overfull protocol line')
            row=query.validate_row(decode(line),phase,trace,budget,repetition)
            obs=observed[i]; previous=obs['previous']; point=obs['point']
            interval=None if obs['previous_clock'] is None else obs['clock']-obs['previous_clock']
            if interval is not None and interval>60000000000: raise ValueError('Phase watchdog')
            if interval is not None and row['elapsed_ns']>interval: raise ValueError('Action exceeds observed parent interval')
            if phase=='calibrated':
                epsilon=max(row['calibration']['observed_resolution_ns'],row['metrics'][0]['p95_ns'],1)
                quantum=obs['quantum']
                reference=cpu_reference_valid(point['user_ns']-previous['user_ns'],point['kernel_ns']-previous['kernel_ns'],row['elapsed_ns'],quantum)
                if not reference: raise ValueError('Owned CPU calibration failed')
            result=query.summarize(row,epsilon)
            result['process']=process_summary(point,obs['peaks'],obs['samples'],obs['gap'],interval,quantum,reference,previous)
            if phase=='calibrated': result.update(cpu_reference_valid=True,observed_min_cpu_increment_ns=quantum,call_uncertainty_proxy_ns=epsilon)
            if phases: phases[-1].update(emit_ns=row['previous_emit_ns'],gate_wait_ns=row['previous_gate_ns'])
            phases.append(result)
        line=stream.readline(MAX_LINE+1)
        if len(line)>MAX_LINE or not line.endswith(b'\n'): raise ValueError('Missing bounded end')
        end=query.validate_end(decode(line),trace,budget,repetition)
        if stream.read(1): raise ValueError('Extra protocol output')
        phases[-1].update(emit_ns=end['previous_emit_ns'],gate_wait_ns=end['previous_gate_ns'])
    query.validate_lifecycle(phases)
    target={'x86_64':'x86_64-pc-windows-msvc','x86':'i686-pc-windows-msvc','aarch64':'aarch64-pc-windows-msvc'}
    if phases[0]['target_os']!='windows' or target.get(phases[0]['target_arch'])!=manifest['target'] or phases[0]['pointer_bits']!=manifest['pointer_bits']: raise ValueError('Child/frozen target mismatch')
    return dict(trace=trace,budget=budget,repetition=repetition,objects=10000,status='Valid',phases=phases,
                completion=receipt,raw_file=name+'.child.jsonl',process_samples_file=name+'.samples.csv',
                raw_sha256=hash_file(raw_path,16*MAX_LINE)['sha256'],coverage=query.coverage(phases,epsilon))


def derive_report(root, manifest):
    validate_freeze(manifest)
    binary=hash_file(relative(root,'frozen/probe.exe'),MAX_BINARY)
    if binary!={k:manifest['binary'][k] for k in ('bytes','sha256')}: raise ValueError('Frozen copy mismatch')
    runs=[derive_run(root,manifest,*case) for case in names()]
    return dict(schema_version=3,plan_id=query.PLAN,validator=query.VALIDATOR,status='Valid',coverage='NeedsEvidence',scope='QueryOnly',
                product_status='NotEvaluated',api_version=7,profile_version=1,source_sha256=manifest['source_sha256'],
                source_map_sha256=manifest['source_map_sha256'],executable_sha256=manifest['binary']['sha256'],freeze_sha256=hash_file(root/'freeze.json',1048576)['sha256'],
                host_observer=manifest['host_observer'],
                metric_semantics='empirical resolution only; batch p95/p99=max of 16; CPU/true peaks remain independently Unknown',runs=runs)


def inventory(root):
    return [dict(path=name,role=role,**hash_file(relative(root,name),limit)) for name,(role,limit) in sorted(roles().items())]


def publish(root, manifest):
    report=derive_report(root,manifest)
    write_json(relative(root,'report.json'),report)
    files=inventory(root)
    index=dict(schema_version=3,plan_id=query.PLAN,validator=query.VALIDATOR,files=files)
    write_json(relative(root,'evidence-index.json'),index)
    if inventory(root)!=files: raise ValueError('Evidence changed during final index publication')
    return report


def verify(root):
    """No current workspace/source checks, no mutation, build, execution or sampling."""
    root=plain_path(root)
    index=read_json(relative(root,'evidence-index.json'),65536)
    query.fields(index,('schema_version','plan_id','validator','files'),'index')
    if type(index['schema_version']) is not int or index['schema_version']!=3 or index['plan_id']!=query.PLAN or index['validator']!=query.VALIDATOR: raise ValueError('Unknown custody protocol')
    files=index['files']; expected=roles()
    if not isinstance(files,list) or len(files)!=len(expected): raise ValueError('Incomplete evidence inventory')
    seen=set()
    for item in files:
        query.fields(item,('path','role','bytes','sha256'),'inventory entry')
        name=item['path']
        if not isinstance(name,str) or name not in expected or name in seen or item['role']!=expected[name][0]: raise ValueError('Unknown/duplicate/noncanonical evidence role')
        seen.add(name); relative(root,name)
        natural(item['bytes'],'file size',expected[name][1])
        if not isinstance(item['sha256'],str) or not re.fullmatch('[0-9a-f]{64}',item['sha256']): raise ValueError('Inventory digest')
    actual=inventory(root)
    if files!=actual: raise ValueError('Evidence inventory bytes/hash/order mismatch')
    manifest=validate_freeze(read_json(relative(root,'freeze.json'),1048576))
    recorded=read_json(relative(root,'report.json'),16*1024*1024)
    derived=derive_report(root,manifest)
    if canonical(recorded)!=canonical(derived): raise ValueError('Report differs from raw evidence derivation')
    if inventory(root)!=actual: raise ValueError('Evidence changed during read-only verification')
    return dict(custody_status='Valid',protocol_status='Valid',coverage='NeedsEvidence',product_status='NotEvaluated',
                plan_id=query.PLAN,verified_runs=12,current_workspace_source_checked=False)
