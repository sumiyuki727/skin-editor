"""Opt-in shared-engine format conversion and default-device lifecycle evidence."""
from array import array
import argparse
import ctypes
import hashlib
import json
import math
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import time
import uuid
import wave
from check_p0 import ROOT
from check_p0_compressed import identity_paths
from check_p0_switch import phase_summary
from p0_process_metrics import WindowsProcessMetrics, environment_info

CASES = [('48000-2-wav', False), ('48000-2-wav', True), ('44100-1-wav', True),
         ('32000-2-wav', True), ('96000-2-wav', True), ('48000-1-wav', True),
         ('44100-1-mp3', True), ('44100-1-ogg', True)]


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda: f.read(1048576), b''): h.update(b)
    return h.hexdigest()


def owned(path, exists=True):
    path = path.resolve(strict=exists)
    if (ROOT / 'artifacts/p0').resolve() not in path.parents: raise RuntimeError('Use an independent artifacts/p0 path')
    return path


def compare(actual, reference, channels):
    size = actual.stat().st_size
    if size == 0 or size != reference.stat().st_size or size % (channels * 4): raise RuntimeError('Unmatched full reference length')
    maximum = 0.; count = 0
    with actual.open('rb') as a, reference.open('rb') as b:
        while True:
            x = a.read(65536); y = b.read(65536)
            if not x: break
            av = array('f'); av.frombytes(x); bv = array('f'); bv.frombytes(y)
            if sys.byteorder != 'little': av.byteswap(); bv.byteswap()
            for v, w in zip(av, bv):
                if not math.isfinite(v) or not math.isfinite(w) or abs(v-w)>1e-5: raise RuntimeError('Full PCM differs from independent reference')
                maximum = max(maximum, abs(v-w)); count += 1
    return {'frames': size // (channels*4), 'samples': count, 'max_error': maximum, 'tolerance': 1e-5}


def pull(source, reference, rate, channels):
    out = ROOT / 'artifacts/p0' / ('adaptation-pull-' + uuid.uuid4().hex[:8])
    exe = ROOT / 'experiments/p0-media/target/release/compressed-proof.exe'
    with out.with_suffix('.log').open('wb') as log, subprocess.Popen([str(exe), str(source), str(out)], stdout=log, stderr=subprocess.STDOUT) as p:
        deadline = time.monotonic()+15
        while not (out/'ready').exists():
            if p.poll() is not None or time.monotonic()>deadline: raise RuntimeError('Pull handshake failed')
            time.sleep(.01)
        (out/'go').write_text('1'); code=p.wait(timeout=45)
    if code: raise RuntimeError('Full decode failed: '+str(out))
    r=json.loads((out/'result.json').read_text())
    if (r['rate'],r['channels'],r['frames']) != (rate,channels,rate*120): raise RuntimeError('Wrong decoded source shape')
    proof=compare(out/'decoded.f32',reference,channels)
    for n, row in enumerate(r['seeks']):
        expected=min(257,r['frames']-row['target'])
        with (out/'decoded.f32').open('rb') as f: f.seek(row['target']*channels*4); raw=f.read(expected*channels*4)
        if row['frames']!=expected or (out/('seek-%d.f32'%n)).read_bytes()!=raw: raise RuntimeError('Nonmatching source-format seek')
    return out, proof


def prepare(target):
    target=owned(target,False)
    if target.exists(): raise FileExistsError(str(target))
    encoder=ROOT/'.p0-tools/encoder/imageio_ffmpeg/binaries/ffmpeg-win-x86_64-v7.1.exe'
    if not encoder.is_file(): raise RuntimeError('Existing private offline FFmpeg required')
    target.mkdir(); rows=[]
    for rate, channels in [(48000,2),(44100,1),(32000,2),(96000,2),(48000,1)]:
        source=target/('%d-%d.wav'%(rate,channels)); reference=target/(source.stem+'.reference.f32')
        second=bytearray()
        for n in range(rate):
            t=n/rate;env=min(1,t/.02,(1-t)/.02)
            for c in range(channels): second.extend(struct.pack('<h',round(.3*env*math.sin(math.tau*(440+220*c)*t)*32767)))
        with wave.open(str(source),'wb') as f:
            f.setparams((channels,2,rate,rate*120,'NONE','not compressed'))
            for _ in range(120): f.writeframesraw(second)
        with reference.open('xb') as f:
            floats=array('f',(v[0]/32768 for v in struct.iter_unpack('<h',second)))
            if sys.byteorder!='little':floats.byteswap()
            for _ in range(120):f.write(floats.tobytes())
        rows.append((str(rate)+'-'+str(channels)+'-wav',source,reference,rate,channels))
    for codec in ('mp3','ogg'):
        source=target/('44100-1.'+codec);reference=target/(source.name+'.reference.f32')
        options=['-c:a','libmp3lame','-b:a','192k','-write_xing','1','-id3v2_version','0'] if codec=='mp3' else ['-c:a','libvorbis','-q:a','4']
        subprocess.run([str(encoder),'-nostdin','-hide_banner','-loglevel','error','-i',str(target/'44100-1.wav'),'-map_metadata','-1',*options,'-n',str(source)],check=True)
        subprocess.run([str(encoder),'-nostdin','-hide_banner','-loglevel','error','-i',str(source),'-f','f32le','-acodec','pcm_f32le','-n',str(reference)],check=True)
        rows.append(('44100-1-'+codec,source,reference,44100,1))
    manifest={'schema_version':1,'generator':'self_authored_audio_adaptation_v1','encoder_sha256':digest(encoder),'entries':[]}
    for name,source,reference,rate,channels in rows:
        out,proof=pull(source,reference,rate,channels)
        manifest['entries'].append({'name':name,'source':source.name,'source_sha256':digest(source),'rate':rate,'channels':channels,'frames':rate*120,
            'reference':(out/'decoded.f32').relative_to(ROOT).as_posix(),'reference_sha256':digest(out/'decoded.f32'),
            'independent_reference':reference.name,'independent_reference_sha256':digest(reference),'full_comparison':proof,
            'pull_result':(out/'result.json').relative_to(ROOT).as_posix(),'pull_result_sha256':digest(out/'result.json')})
    manifest['decoder_source_sha256']={p.relative_to(ROOT).as_posix():digest(p) for p in identity_paths() if p.relative_to(ROOT).as_posix().startswith('experiments/p0-media/')}
    (target/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');return target


def load_fixture(path):
    path=owned(path);m=json.loads((path/'manifest.json').read_text())
    decoder={p.relative_to(ROOT).as_posix():digest(p) for p in identity_paths() if p.relative_to(ROOT).as_posix().startswith('experiments/p0-media/')}
    if m.get('schema_version')!=1 or m.get('generator')!='self_authored_audio_adaptation_v1' or m.get('decoder_source_sha256')!=decoder:raise RuntimeError('Stale fixture/decoder')
    rows={r['name']:r for r in m['entries']}
    if len(rows)!=7 or set(rows)!={c for c,_ in CASES}:raise RuntimeError('Incomplete source matrix')
    for row in rows.values():
        rate,channels,codec=row['name'].split('-');rate=int(rate);channels=int(channels)
        if (row.get('rate'),row.get('channels'),row.get('frames'))!=(rate,channels,rate*120):raise RuntimeError('Source matrix shape changed')
        for key in ('source','independent_reference'):
            p=owned(path/row[key])
            if path not in p.parents or digest(p)!=row[key+'_sha256']:raise RuntimeError('Source/reference identity')
        for key in ('reference','pull_result'):
            if digest(owned(ROOT/row[key]))!=row[key+'_sha256']:raise RuntimeError('Decoder proof identity')
        c=row['full_comparison']
        if c['frames']!=row['frames'] or c['samples']!=row['frames']*row['channels'] or not math.isfinite(c['max_error']) or not 0<=c['max_error']<=1e-5:raise RuntimeError('Incomplete independent comparison')
        decoded=json.loads((ROOT/row['pull_result']).read_text())
        if (decoded.get('rate'),decoded.get('channels'),decoded.get('frames'),decoded.get('audio_device_opened'))!=(rate,channels,row['frames'],False):raise RuntimeError('Wrong full decode proof')
        Reference(ROOT/row['reference'],rate,channels,row['frames'])
    return rows


class Reference:
    def __init__(self,path,rate,channels,frames):
        self.path=path;self.rate=rate;self.channels=channels;self.frames=frames
        if channels not in (1,2) or not 8000<=rate<=192000 or frames<=0 or path.stat().st_size!=frames*channels*4:raise RuntimeError('Reference shape')
    def check(self,p):
        first=p.get('source_frame');frames=p.get('frames')
        if type(first)is not int or type(frames)is not int or first<0 or not 0<frames<=16384 or first+frames>self.frames:raise RuntimeError('Packet range')
        with self.path.open('rb') as f:f.seek(first*self.channels*4);raw=f.read(frames*self.channels*4)
        values=array('f');values.frombytes(raw)
        if sys.byteorder!='little':values.byteswap()
        gain=ctypes.c_float(.002).value;values=array('f',(v*gain for v in values));encoded=values.tobytes()
        if sys.byteorder!='little':values.byteswap();encoded=values.tobytes()
        h=0xcbf29ce484222325
        for b in encoded:h=((h^b)*0x100000001b3)&((1<<64)-1)
        probe=list(values[::self.channels][:8]);probe += [0.]*(8-len(probe))
        if (p.get('hash')!=h or p.get('nonzero_samples')!=sum(v!=0 for v in values) or not math.isfinite(p.get('peak',math.nan))
                or abs(p['peak']-max(map(abs,values)))>1e-7 or len(p.get('probe',[]))!=8
                or any(not math.isfinite(a) or abs(a-b)>1e-7 for a,b in zip(p['probe'],probe))):raise RuntimeError('Native packet differs from source format/gain')


def validate(r, trace, reference):
    if r.get('schema_version')!=1 or r.get('status')!='passed' or r.get('accepted_inputs')!=2 or any(r.get(k)is not False for k in ('system_capture_used','recording_used','engine_output_samples_verified')):raise RuntimeError('Incomplete native proof')
    cfg=r['source_format']
    if (cfg['rate'],cfg['channels'],cfg['frames'],cfg['gain'])!=(reference.rate,reference.channels,reference.frames,.002):raise RuntimeError('Wrong source metadata')
    final=r['final_stats']
    if any(final[k] for k in ('active','in_flight','queued_blocks','underruns','decode_failures','trace_overflow')) or final['free_blocks']!=5 or final['stale_blocks']<1:raise RuntimeError('Queue/fault/old block lifecycle')
    names=[c['name'] for c in r['checkpoints']]
    if names!=['continuous','pause_with_old_inflight','old_block_rejected','seek_paused','fault_frozen','recovered_paused','final_pause']:raise RuntimeError('Checkpoint coverage')
    checks={c['name']:c for c in r['checkpoints']}
    if checks['continuous']['simulation_us']<8_000_000 or checks['seek_paused']['simulation_us']!=1_234_567 or checks['fault_frozen']['simulation_us']!=checks['recovered_paused']['simulation_us']:raise RuntimeError('Frozen content/seek anchor')
    gen=0;cursor=0;active=False;packets=stale=retired=pauses=0
    for e in trace:
        kind=e['kind']
        if kind in ('Activate','Pause','Retire'):
            if type(e['generation'])is not int or e['generation']<=gen:raise RuntimeError('Nonmonotonic PCM generation')
            if kind=='Pause':
                if not active or abs(cursor-e['first_frame']-e['native_discarded_frames'])>1:raise RuntimeError('Unaccounted native pause tail')
                pauses+=1
            retired+=kind=='Retire';gen=e['generation'];cursor=e['first_frame'];active=kind!='Retire'
        elif kind=='Submit':
            if not active or e['generation']!=gen or e['first_frame']!=cursor or e['packet']['source_frame']!=cursor or e['frames']!=e['packet']['frames']:raise RuntimeError('Old/discontinuous block submitted')
            reference.check(e['packet']);cursor+=e['frames'];packets+=1
        elif kind=='Stale':
            if e['generation']>=gen:raise RuntimeError('Invalid stale completion')
            stale+=1
        else:raise RuntimeError('Unexpected PCM event')
    if packets<300 or stale<1 or retired<3 or pauses<3:raise RuntimeError('Missing PCM transition coverage')
    formats=[s['format'] for s in r['observations'] if s['format']]
    if not formats or any((f['rate'],f['channels'])!=(reference.rate,reference.channels) or f['conversion_enabled']is not r['conversion_requested'] for f in formats):raise RuntimeError('Client format reinterpreted')
    return {'accepted_packets':packets,'stale_events':stale,'retirements':retired,'pauses':pauses}


def summarize(samples):
    result=[]
    for stage in dict.fromkeys(s['phase'] for s in samples):
        selected=[{**s,'phase':'Idle'} for s in samples if s['phase']==stage]
        row=next(r for r in phase_summary(selected) if r['phase']=='Idle')
        row['phase']=stage;result.append(row)
    return result


def run(source,row,convert,idle=3,manual=False):
    exe=ROOT/'experiments/p0-egui/target/release/format-device-proof.exe'
    frozen={p.relative_to(ROOT).as_posix():digest(p) for p in identity_paths()};binary=digest(exe)
    ref_path=owned(ROOT/row['reference'])
    if digest(source)!=row['source_sha256'] or digest(ref_path)!=row['reference_sha256']:raise RuntimeError('Source/reference changed before launch')
    out=ROOT/'artifacts/p0'/('audio-adaptation-'+uuid.uuid4().hex[:8]);samples=[];phase=None
    with out.with_suffix('.log').open('wb') as log,subprocess.Popen([str(exe),str(out),str(source),'convert' if convert else 'strict',str(idle),'manual' if manual else 'auto'],stdout=log,stderr=subprocess.STDOUT) as p:
        metrics=WindowsProcessMetrics(p);deadline=time.monotonic()+(700 if manual else 190)
        try:
            handshake=False
            while p.poll()is None:
                if time.monotonic()>deadline:p.kill();p.wait();raise RuntimeError('Native protocol timeout')
                if not handshake and (out/'ready').exists():(out/'go').write_text('1');handshake=True
                try:before=json.loads((out/'phase.json').read_text()).get('stage')
                except (FileNotFoundError,json.JSONDecodeError):before=None
                sample=metrics.sample()
                try:after=json.loads((out/'phase.json').read_text()).get('stage')
                except (FileNotFoundError,json.JSONDecodeError):after=None
                if sample and before==after and before:
                    if before!=phase:phase=before;start=sample['monotonic_seconds']
                    if sample['monotonic_seconds']-start>=1:sample['phase']=before;samples.append(sample)
                time.sleep(.1)
        finally:metrics.close()
    if p.returncode:raise RuntimeError('Native proof failed: '+str(out))
    r=json.loads((out/'result.json').read_text());trace=[json.loads(s) for s in (out/'packets.jsonl').read_text().splitlines()]
    if r.get('conversion_requested')is not convert or r.get('manual_device_change')is not manual:raise RuntimeError('Native mode differs from request')
    observed=r.get('device_change_observed')
    if type(observed)is not bool or (observed and (not manual or r.get('fault',{}).get('operation')=='explicit stream invalidation')):raise RuntimeError('Controlled fault cannot prove physical change')
    reference=Reference(owned(ROOT/row['reference']),row['rate'],row['channels'],row['frames']);checks=validate(r,trace,reference)
    if frozen!={p.relative_to(ROOT).as_posix():digest(p) for p in identity_paths()} or digest(exe)!=binary or digest(source)!=row['source_sha256'] or digest(ref_path)!=row['reference_sha256']:raise RuntimeError('Measured identity changed')
    phases=summarize(samples)
    budgets=None
    if idle==65:
        lookup={p['phase']:p for p in phases}
        budgets={name:(lookup.get(name,{}).get('sample_span_seconds',0)>=60 and lookup[name]['cpu_one_logical_core_percent']<=2) for name in ('PausedIdle','ClosedIdle')}
        budgets['sampled_working_set_le_512_mib']=max(s['working_set_bytes'] for s in samples)<=512*1048576
    evidence={'schema_version':1,'status':'validated','source_sha256':frozen,'executable_sha256':binary,'source':source.relative_to(ROOT).as_posix(),
        'audio_sha256':row['source_sha256'],'reference_sha256':row['reference_sha256'],'checks':checks,'samples':samples,'phases':phases,'budgets':budgets,'environment':environment_info()}
    (out/'evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
    for name,sha in frozen.items():
        dst=out/'sources'/name;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dst)
    print('Validated '+str(out),flush=True);return out,evidence


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--prepare',type=Path);p.add_argument('--fixture',type=Path)
    p.add_argument('--skip-checks',action='store_true');p.add_argument('--run-native',action='store_true');p.add_argument('--manual',action='store_true');p.add_argument('--smoke',action='store_true')
    p.add_argument('--repetitions',type=int,choices=range(1,4),default=2);args=p.parse_args()
    if not args.skip_checks:subprocess.run([sys.executable,str(ROOT/'tools/check_p0_compressed.py')],check=True)
    if args.prepare:prepare(args.prepare)
    if not args.run_native:return
    if sys.platform!='win32' or args.fixture is None:raise RuntimeError('Windows and explicit verified fixture required')
    fixture=owned(args.fixture);rows=load_fixture(fixture)
    frozen={p.relative_to(ROOT).as_posix():digest(p) for p in identity_paths()}
    binary=digest(ROOT/'experiments/p0-egui/target/release/format-device-proof.exe');manifest_sha=digest(fixture/'manifest.json')
    summary={'schema_version':1,'status':'running','source_sha256':frozen,'executable_sha256':binary,'fixture_manifest_sha256':manifest_sha,'runs':[]}
    output=ROOT/'artifacts/p0'/('audio-adaptation-summary-'+uuid.uuid4().hex[:8]+'.json')
    try:
        cases=[('44100-1-wav',True)] if args.manual or args.smoke else CASES
        for repeat in range(1 if args.manual or args.smoke else args.repetitions):
            for name,convert in cases:
                idle=65 if not(args.manual or args.smoke) and repeat==0 and (name,convert) in [CASES[0],CASES[2]] else 3
                out,e=run(fixture/rows[name]['source'],rows[name],convert,idle,args.manual)
                if e['source_sha256']!=frozen or e['executable_sha256']!=binary or digest(fixture/'manifest.json')!=manifest_sha:raise RuntimeError('Identity changed within matrix')
                summary['runs'].append({'repeat':repeat,'case':name,'convert':convert,'directory':out.relative_to(ROOT).as_posix(),'checks':e['checks'],'budgets':e['budgets']})
                output.write_text(json.dumps(summary,indent=2)+'\n')
        summary['status']='passed_scoped_protocol' if all(not r['budgets']or all(v is True for v in r['budgets'].values()) for r in summary['runs']) else 'failed_budget'
    except Exception as exc:summary.update(status='rejected',error=str(exc));raise
    finally:output.write_text(json.dumps(summary,indent=2)+'\n');print('Summary: '+str(output),flush=True)
    if summary['status']!='passed_scoped_protocol':raise RuntimeError('Budget failed')


if __name__=='__main__':main()
