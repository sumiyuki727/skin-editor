"""Fixed owned-window preparation/render faults and failed recreation; no UAC."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from measure_combined import source_identity
from measure_audio_load import ROOT

PLAN=('prepare','paint','rebuild-failure')
INIT_PLAN=('init-consumed','init-prepared','init-audio')
CLEANUP_PLAN=('cleanup-held',)
ENDPOINT_PLAN=('endpoint-held',)
ACTIVE_PLAN=('draft-gpu','draft-audio','draft-both')

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',required=True);p.add_argument('--build-receipt',required=True);p.add_argument('--out',required=True)
    mode=p.add_mutually_exclusive_group()
    mode.add_argument('--initialization',action='store_true',help='Fixed post-consumption Editor/preparation/audio failures and retry')
    mode.add_argument('--cleanup',action='store_true',help='Retained real draw prevents cleanup, then explicit completion and retry')
    mode.add_argument('--endpoint-return',action='store_true',help='Actual stopped audio endpoint held before delivery; cleanup and retry remain gated')
    mode.add_argument('--active-edit',action='store_true',help='Actual egui drag and visual/audio draft interrupted by GPU, audio or joint fault')
    args=p.parse_args();out=Path(args.out).resolve();binary=Path(args.binary).resolve();receipt=Path(args.build_receipt).resolve()
    source=source_identity();sha=hashlib.sha256(binary.read_bytes()).hexdigest();build=json.loads(receipt.read_text())
    if (out.exists() or ROOT/'artifacts' not in out.parents or ROOT/'artifacts' not in binary.parents
            or build.get('profile')!='release' or build.get('requested_features')!=[]
            or build.get('source')!=source or build.get('repeat_sha256')!=[sha,sha]):
        raise ValueError('fresh output and frozen default release required')
    plan_set=ACTIVE_PLAN if args.active_edit else ENDPOINT_PLAN if args.endpoint_return else CLEANUP_PLAN if args.cleanup else INIT_PLAN if args.initialization else PLAN
    out.mkdir();proof=dict(schema=1,source=source,binary_sha256=sha,plan=list(plan_set),runs=[],c4_accepted=False,p2_accepted=False)
    for plan in plan_set:
        directory=out/plan
        with (out/(plan+'.log')).open('x',encoding='utf8') as log:
            result=subprocess.run([sys.executable,str(ROOT/'tools/measure_window_recovery.py'),'--binary',str(binary),
                '--build-receipt',str(receipt),'--out',str(directory),'--fault-plan',plan],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,timeout=180)
        proof['runs'].append(dict(plan=plan,returncode=result.returncode,proof=str(directory.relative_to(ROOT)/'proof.json')))
        # Fixed failures remain; later declared controls do not retry a failed run.
    proof['unchanged']=source_identity()==source and hashlib.sha256(binary.read_bytes()).hexdigest()==sha
    proof['component_passed']=proof['unchanged'] and all(r['returncode']==0 for r in proof['runs'])
    (out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if not proof['component_passed']:raise SystemExit(1)

if __name__=='__main__':main()
