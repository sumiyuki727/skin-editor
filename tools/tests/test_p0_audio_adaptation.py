import copy
import json
import math
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import check_p0_audio_adaptation as a


class AdaptationTests(unittest.TestCase):
    def test_phase_costs_do_not_mix_active_and_paused_or_closed_samples(self):
        def sample(t,cpu,phase):return dict(monotonic_seconds=t,cpu_seconds=cpu,phase=phase,working_set_bytes=100,private_bytes=200,gpu_dedicated_bytes=None,gpu_shared_bytes=None)
        rows=a.summarize([sample(0,0,'Playing'),sample(10,1,'Playing'),sample(11,2,'PausedIdle'),sample(76,2,'PausedIdle'),sample(80,3,'ClosedIdle'),sample(145,3.01,'ClosedIdle')])
        self.assertEqual([r['phase'] for r in rows],['Playing','PausedIdle','ClosedIdle'])
        self.assertEqual(rows[0]['cpu_one_logical_core_percent'],10)
        self.assertEqual(rows[1]['cpu_one_logical_core_percent'],0)
        self.assertEqual(rows[2]['sample_span_seconds'],65)
        self.assertAlmostEqual(rows[2]['cpu_one_logical_core_percent'],.01/65*100)

    def test_mono_reference_keeps_source_rate_shape_gain_and_native_hash(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'reference';p.write_bytes(struct.pack('<fff',.25,-.5,0))
            ref=a.Reference(p,44100,1,3)
            raw=b''.join(struct.pack('<f',v*struct.unpack('<f',struct.pack('<f',.002))[0])for v in [.25,-.5,0])
            values=struct.unpack('<fff',raw);h=0xcbf29ce484222325
            for b in raw:h=((h^b)*0x100000001b3)&((1<<64)-1)
            packet=dict(source_frame=0,frames=3,hash=h,nonzero_samples=2,peak=max(map(abs,values)),probe=[*values,0,0,0,0,0])
            ref.check(packet)
            for bad in [dict(packet,source_frame=True),dict(packet,frames=4),dict(packet,hash=0),dict(packet,peak=math.nan),dict(packet,probe=[0]*8)]:
                with self.assertRaises(RuntimeError):ref.check(bad)
            with self.assertRaises(RuntimeError):a.Reference(p,44100,2,3)

    def test_full_comparison_has_no_trimmed_tail_and_checks_finite_mono_samples(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'a';q=Path(tmp)/'b';p.write_bytes(struct.pack('<fff',0,.25,-.5));q.write_bytes(p.read_bytes())
            self.assertEqual(a.compare(p,q,1)['frames'],3)
            q.write_bytes(p.read_bytes()[:8])
            with self.assertRaises(RuntimeError):a.compare(p,q,1)
            q.write_bytes(struct.pack('<fff',0,.25,math.nan))
            with self.assertRaises(RuntimeError):a.compare(p,q,1)

    def test_fixture_preparation_refuses_escape_or_existing_without_encoder(self):
        with tempfile.TemporaryDirectory() as tmp,patch.object(a,'ROOT',Path(tmp)):
            base=Path(tmp)/'artifacts/p0';base.mkdir(parents=True);existing=base/'present';existing.mkdir();(existing/'keep').write_text('original')
            with self.assertRaises(RuntimeError):a.prepare(Path(tmp)/'escape')
            with self.assertRaises(FileExistsError):a.prepare(existing)
            self.assertEqual((existing/'keep').read_text(),'original')

    def test_fixture_requires_current_decoder_complete_references_and_finite_proof(self):
        with tempfile.TemporaryDirectory() as tmp,patch.object(a,'ROOT',Path(tmp)),patch.object(a,'identity_paths',return_value=[]),patch.object(a,'Reference'):
            base=Path(tmp)/'artifacts/p0';base.mkdir(parents=True);fixture=base/'fixture';fixture.mkdir();rows=[]
            for name in dict.fromkeys(n for n,_ in a.CASES):
                rate,channels,_=name.split('-');rate=int(rate);channels=int(channels)
                source=fixture/(name+'.bin');source.write_bytes(b'owned')
                ref=fixture/(name+'.f32');ref.write_bytes(b'pcm')
                proof=fixture/(name+'.json');proof.write_text(json.dumps(dict(rate=rate,channels=channels,frames=rate*120,audio_device_opened=False)))
                rows.append(dict(name=name,rate=rate,channels=channels,frames=rate*120,source=source.name,source_sha256=a.digest(source),
                    independent_reference=ref.name,independent_reference_sha256=a.digest(ref),reference=ref.relative_to(Path(tmp)).as_posix(),reference_sha256=a.digest(ref),
                    pull_result=proof.relative_to(Path(tmp)).as_posix(),pull_result_sha256=a.digest(proof),full_comparison=dict(frames=rate*120,samples=rate*120*channels,max_error=0)))
            manifest=dict(schema_version=1,generator='self_authored_audio_adaptation_v1',decoder_source_sha256={},entries=rows)
            path=fixture/'manifest.json';path.write_text(json.dumps(manifest));self.assertEqual(len(a.load_fixture(fixture)),7)
            for kind in ('decoder','short','nan','escape'):
                bad=copy.deepcopy(manifest)
                if kind=='decoder':bad['decoder_source_sha256']={'old.rs':'old'}
                if kind=='short':bad['entries'][0]['full_comparison']['frames']-=1
                if kind=='nan':bad['entries'][0]['full_comparison']['max_error']=math.nan
                if kind=='escape':bad['entries'][0]['source']='../escape.bin';(base/'escape.bin').write_bytes(b'owned')
                path.write_text(json.dumps(bad))
                with self.assertRaises(RuntimeError):a.load_fixture(fixture)

    def proof(self):
        stats=dict(active=False,in_flight=False,queued_blocks=0,free_blocks=5,underruns=0,decode_failures=0,trace_overflow=False,stale_blocks=1)
        names=['continuous','pause_with_old_inflight','old_block_rejected','seek_paused','fault_frozen','recovered_paused','final_pause']
        r=dict(schema_version=1,status='passed',accepted_inputs=2,system_capture_used=False,recording_used=False,engine_output_samples_verified=False,
            source_format=dict(rate=44100,channels=1,frames=44100*120,gain=.002),final_stats=stats,conversion_requested=True,
            checkpoints=[dict(name=n,simulation_us=8_000_000 if n=='continuous' else 1_234_567)for n in names],
            observations=[dict(format=dict(rate=44100,channels=1,conversion_enabled=True))])
        def e(kind,g,first=0,frames=0):return dict(kind=kind,generation=g,first_frame=first,frames=frames,native_discarded_frames=0)
        trace=[e('Activate',1)]
        for n in range(300):trace.append(dict(e('Submit',1,n,1),packet=dict(source_frame=n,frames=1)))
        trace.extend([e('Pause',2,300),e('Stale',1),e('Pause',3,300),e('Pause',4,300),e('Retire',5),e('Activate',6),e('Retire',7),e('Activate',8),e('Retire',9)])
        reference=SimpleNamespace(rate=44100,channels=1,frames=44100*120,check=lambda p:None)
        return r,trace,reference

    def test_trace_requires_generation_order_continuity_pause_tail_and_old_rejection(self):
        r,t,ref=self.proof();self.assertEqual(a.validate(r,t,ref)['accepted_packets'],300)
        for at,field,value in [(100,'generation',0),(100,'first_frame',500),(301,'native_discarded_frames',10),(302,'generation',2)]:
            bad=copy.deepcopy(t);bad[at][field]=value
            with self.assertRaises(RuntimeError):a.validate(r,bad,ref)
        with self.assertRaises(RuntimeError):a.validate(r,t[:301],ref)

    def test_proof_rejects_unfrozen_recovery_wrong_format_or_missing_pool(self):
        r,t,ref=self.proof()
        for kind in ('recovery','pool','format','observations','engine_claim'):
            bad=copy.deepcopy(r)
            if kind=='recovery':bad['checkpoints'][5]['simulation_us']+=1
            if kind=='pool':bad['final_stats']['free_blocks']=4
            if kind=='format':bad['observations'][0]['format']['rate']=48000
            if kind=='observations':bad['observations']=[]
            if kind=='engine_claim':bad['engine_output_samples_verified']=True
            with self.assertRaises(RuntimeError):a.validate(bad,t,ref)


if __name__=='__main__':unittest.main()
