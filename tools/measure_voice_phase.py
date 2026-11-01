"""Exact Voice/render markers and callback-phase diagnostics; never waive 20 ms."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
from collections import Counter
from measure_audio_load import ROOT,observe,distribution,summarize
from measure_combined import source_identity,identity_matches,causal_analysis

STAGES=('unclassified','stream_start','prepare','ready','phase_scan','collect','stop','close')


def underrun_analysis(report):
    """Quiescent totals and first 32 event contexts; never infer a missing row."""
    if report['schema']!=2:raise ValueError('phase report requires stage/context schema 2')
    totals=[report[k] for k in ('callbacks','frames','empty_callbacks','empty_frames','retimed_idle_frames')]
    stages=report['stage_counts']
    unsigned=lambda row:all(type(v) is int and v>=0 for v in row)
    if (not unsigned(totals) or totals[0]==0 or len(stages)!=8 or
            any(len(row)!=5 or not unsigned(row) or row[2]>row[0] or row[3]>row[1] for row in stages) or
            [sum(row[i] for row in stages) for i in range(5)]!=totals or any(stages[0])):
        raise ValueError('missing/inconsistent quiescent stage totals')
    startup=report['startup_observations']
    if len(startup)!=min(32,totals[0]) or any(len(r)!=4 or not unsigned(r) or r[1]>2048 or r[2]>r[0] for r in startup):
        raise ValueError('incomplete startup trace')
    rows=report['underrun_observations'];contexts=report['underrun_contexts']
    if len(rows)!=min(32,totals[2]) or len(contexts)!=len(rows):raise ValueError('missing bounded underrun rows')
    events=[];previous=-1
    for raw,context in zip(rows,contexts):
        if (len(raw)!=6 or not unsigned(raw) or raw[0]<=previous or raw[0]>=totals[0] or
                raw[2]>2048 or not 0<raw[3]<=raw[1] or raw[4]>raw[5] or context is None or len(context)!=4):
            raise ValueError('invalid/partial underrun event')
        previous=raw[0];valid=[]
        for row in context:
            if len(row)!=11 or not unsigned(row):raise ValueError('invalid callback context')
            if not row[0]:
                if valid or any(row):raise ValueError('missing context in middle')
                continue
            if (row[0]>totals[0] or row[2]==0 or row[2]>report['max_callback_frames'] or
                    max(row[3:5])>2048 or row[5]>row[2] or row[6]>row[7] or max(row[9:])>=8 or
                    valid and (row[0]!=valid[-1][0]+1 or row[1]<valid[-1][1])):
                raise ValueError('inconsistent callback context')
            valid.append(row)
        if len(valid)!=min(4,raw[0]+1):raise ValueError('incomplete prior callback window')
        current=valid[-1]
        if [current[0]-1,current[2],current[3],current[5],current[6],current[7]]!=raw:
            raise ValueError('context disagrees with original underrun trace')
        # A pass includes zero-refill attempts. Equal passes mean no completed
        # pump was observed; they do not exclude an in-progress producer.
        drain=[current]
        for row in reversed(valid[:-1]):
            after=drain[0]
            if (row[6]!=row[7] or row[7]!=after[6] or row[4]!=after[3] or row[8] or
                    row[4]!=max(0,row[3]-row[2]) or row[5]!=max(0,row[2]-row[3])):break
            drain.insert(0,row)
        accounting=(current[6]==current[7] and current[8]==0 and
                    current[4]==max(0,current[3]-current[2]) and current[5]==max(0,current[2]-current[3]))
        if not accounting:drain=[]
        events.append(dict(callback_index=raw[0],entry_stage=STAGES[current[9]],exit_stage=STAGES[current[10]],
            stage_crossed=current[9]!=current[10],requested_frames=raw[1],queued_before=raw[2],empty_frames=raw[3],
            retimed_frames=current[8],same_completed_pump_pass=current[6]==current[7],
            queue_accounting_exact=accounting,consecutive_drain_callbacks=len(drain),
            consecutive_drain_queues=[r[3] for r in drain],callback_entry_gaps_ns=[b[1]-a[1] for a,b in zip(drain,drain[1:])],
            context=context))
    if totals[2]<=32 and sum(r[3] for r in rows)!=totals[3]:raise ValueError('underrun frame total mismatch')
    if any(r[1]>report['max_empty_callback_frames'] for r in rows):raise ValueError('empty callback maximum mismatch')
    return dict(stage_names=STAGES,stage_counts=stages,events=events,
        missing_events=max(0,totals[2]-32),complete=totals[2]<=32,
        scope='Entry-stage counts; crossing rows explicit. Equal pump pass is not proof producer was never in progress.')


def exact_analysis(report,count=64):
    if type(count) is not int or not 1<=count<=256:raise ValueError('invalid bounded probe count')
    keys=('exact_voice_intervals_ms','completion_markers','requested_voice_ids',
          'submission_intervals_ms','submission_causal_ms','latency_upper_bound_ms')
    if any(len(report[k])!=2 or any(len(lane)!=count for lane in report[k]) for k in keys):
        raise ValueError('missing bounded exact Voice observations')
    result=[]
    for lane in range(2):
        bounds=report['exact_voice_intervals_ms'][lane];markers=report['completion_markers'][lane]
        voices=report['requested_voice_ids'][lane]
        if any(type(v) is not int or v<=0 for v in voices) or any(b<=a for a,b in zip(voices,voices[1:])):
            raise ValueError('invalid requested Voice identity/order')
        for b,m,v,old,c,claimed in zip(bounds,markers,voices,report['submission_intervals_ms'][lane],report['submission_causal_ms'][lane],report['latency_upper_bound_ms'][lane]):
            if (len(b)!=2 or len(old)!=5 or len(c)!=13 or any(not math.isfinite(x) for x in b) or not 0<=b[0]<=b[1] or
                    len(m)!=7 or any(type(x) is not int or x<0 for x in m) or old[1]!=claimed or not 0<=old[0]<=old[1] or
                    m[0]!=v or m[1]!=c[11] or m[2]!=c[10] or m[4]!=c[8] or m[5]!=c[9] or m[6]!=c[12] or
                    not 0<=m[4]<m[5] or not c[3]<=b[1] or old[0]>b[1]):
                raise ValueError('wrong Voice/epoch/serial/callback or invalid completion bound')
        lower=distribution([b[0] for b in bounds]);upper=distribution([b[1] for b in bounds])
        result.append(dict(lower_ms=lower,upper_ms=upper,
            width_ms=distribution([b[1]-b[0] for b in bounds]),
            callback_entry_to_completion_upper_ms=distribution([b[1]-c[3] for b,c in zip(bounds,report['submission_causal_ms'][lane])]),
            classification='within' if upper['p95']<=20 else 'over' if lower['p95']>20 else 'unresolved',
            individually_confirmed_over=sum(b[0]>20 for b in bounds),individually_straddling=sum(b[0]<=20<b[1] for b in bounds)))
    return result


def phase_analysis(report):
    count=160
    if (report['count_per_lane'],report['requested_bins'],report['repeats'])!=(count,20,8):
        raise ValueError('incomplete phase schedule')
    underruns=underrun_analysis(report)
    exact=exact_analysis(report,count);causal=causal_analysis(report,count=count)
    if len(report['phase_rows'])!=2 or len(report['callback_references'])!=2:
        raise ValueError('missing callback phase references')
    lanes=[]
    for lane in range(2):
        rows=report['phase_rows'][lane];refs=report['callback_references'][lane]
        if len(rows)!=count or len(refs)!=count:raise ValueError('incomplete achieved-phase observations')
        requested=Counter();buckets=[[] for _ in range(10)];unclassified=0
        for index,(r,ref) in enumerate(zip(rows,refs)):
            if (len(r)!=5 or any(not math.isfinite(v) or v<0 for v in r) or int(r[0])!=r[0] or
                    r[0]!=(index*7)%20 or not r[1]<=r[2] or r[4]<=0 or
                    len(ref)!=4 or any(type(v) is not int or v<0 for v in ref) or ref[2]==0):
                raise ValueError('invalid phase/order/reference')
            requested[int(r[0])]+=1
            if r[2]<r[4]:buckets[min(9,int((r[1]+r[2])/2/r[4]*10))].append(index)
            else:unclassified+=1 # in-flight/new callback or delayed stamp: never modulo into a good phase
        if requested!=Counter({n:8 for n in range(20)}):raise ValueError('phase requests missing')
        grouped=[]
        for bucket,indices in enumerate(buckets):
            grouped.append(dict(bin=bucket,count=len(indices),
                old_upper_ms=distribution([report['latency_upper_bound_ms'][lane][i] for i in indices]) if indices else None,
                exact_upper_ms=distribution([report['exact_voice_intervals_ms'][lane][i][1] for i in indices]) if indices else None,
                entry_lower_ms=distribution([report['submission_causal_ms'][lane][i][3] for i in indices]) if indices else None,
                request_to_pump_ms=distribution([report['submission_causal_ms'][lane][i][0] for i in indices]) if indices else None))
        lanes.append(dict(bins=grouped,unclassified=unclassified,
            achieved_age_ms=distribution([(r[1]+r[2])/2 for r in rows]),
            wake_overshoot_ms=distribution([r[3] for r in rows]),
            coverage_qualified=sum(bool(v) for v in buckets)>=8,
            fixed_callback_frames=all(ref[2]==report['period_frames'] for ref in refs)))
    old=[distribution(v) for v in report['latency_upper_bound_ms']]
    checks=dict(schedule_complete=True,exact_voice_identity=True,underrun_trace_complete=underruns['complete'],
        phase_coverage=all(v['coverage_qualified'] and v['fixed_callback_frames'] for v in lanes),
        no_underflow=report['empty_frames']==report['empty_callbacks']==0,
        resources_released=report['pcm_final_bytes']==report['raw_final_bytes']==0,
        managed_limits=report['pcm_peak_bytes']<=32*1024**2 and report['raw_peak_bytes']<=128*1024**2,
        original_submission_p95_le_20_ms=all(v['p95']<=20 for v in old),
        exact_render_p95_le_20_ms=all(v['upper_ms']['p95']<=20 for v in exact))
    return dict(checks=checks,old_latency_ms=old,exact=exact,causal=causal,lanes=lanes,underruns=underruns)


def request_phase_analysis(report,count=64):
    """Actual prior completed callback at each standard request; no modulo
    folding of an old reference or inferred phase from a later callback."""
    exact_analysis(report,count)
    rows=report['request_phase_age_ms'];refs=report['request_callback_refs']
    if len(rows)!=2 or len(refs)!=2 or any(len(v)!=count for v in rows+refs):
        raise ValueError('missing standard request callback references')
    result=[]
    for lane in range(2):
        bins=[[] for _ in range(10)];unknown=[]
        for index,(age,ref,marker) in enumerate(zip(rows[lane],refs[lane],report['completion_markers'][lane])):
            if (len(age)!=2 or any(not math.isfinite(v) for v in age) or not 0<=age[0]<=age[1] or
                    len(ref)!=4 or any(type(v) is not int or v<0 for v in ref) or ref[2]==0 or
                    ref[0]>=marker[6] or ref[1]>=marker[3]):
                raise ValueError('invalid standard phase/callback order')
            period=ref[2]/report['rate']*1000
            if age[1]<period:bins[min(9,int((age[0]+age[1])/2/period*10))].append(index)
            else:unknown.append(index)
        groups=[]
        for i,indices in enumerate(bins):
            groups.append(dict(bin=i,count=len(indices),
                observed_upper_ms=distribution([report['latency_upper_bound_ms'][lane][j] for j in indices]) if indices else None,
                exact_upper_ms=distribution([report['exact_voice_intervals_ms'][lane][j][1] for j in indices]) if indices else None,
                request_to_pump_ms=distribution([report['submission_causal_ms'][lane][j][0] for j in indices]) if indices else None,
                callback_entry_lower_ms=distribution([report['submission_causal_ms'][lane][j][3] for j in indices]) if indices else None))
        result.append(dict(bins=groups,unclassified=len(unknown),unclassified_indices=unknown,
            age_ms=distribution([(a[0]+a[1])/2 for a in rows[lane]])))
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',required=True);parser.add_argument('--output',required=True)
    args=parser.parse_args();binary=Path(args.binary).resolve();out=(ROOT/args.output).resolve()
    if os.name!='nt' or binary!=(ROOT/'desktop/target/release/skin-editor-desktop.exe').resolve() or not binary.is_file():
        raise RuntimeError('requires explicit Windows release desktop binary')
    if out.exists() or ROOT/'artifacts' not in out.parents:raise RuntimeError('new artifacts directory required')
    out.mkdir(parents=True);identity=dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),source_sha256=source_identity())
    (out/'identity.json').write_text(json.dumps(identity,indent=2)+'\n',encoding='utf-8');runs=[]
    for index in range(2):
        print('Starting phase scan %d/2'%(index+1),flush=True)
        try:
            report,samples=observe(binary,out/('run-%02d'%(index+1)),'--verify-voice-phase',timeout_seconds=90)
            analysis=phase_analysis(report)
            analysis['phases']=summarize(samples,('phase_scan','closed'),(('phase_scan',10),('closed',1)))
            runs.append(dict(completed=True,**analysis))
            print(json.dumps(dict(run=index+1,checks=analysis['checks'],old=analysis['old_latency_ms'],exact=analysis['exact'])),flush=True)
        except (OSError,ValueError,RuntimeError,TypeError,KeyError) as e:
            runs.append(dict(completed=False,error=str(e),checks=dict(completed=False)));break
    unchanged=identity_matches(identity['source_sha256']) and hashlib.sha256(binary.read_bytes()).hexdigest()==identity['binary_sha256']
    passed=len(runs)==2 and unchanged and all(all(r['checks'].values()) for r in runs)
    (out/'metrics.json').write_text(json.dumps(dict(schema=1,passed=passed,sources_unchanged=unchanged,platform=platform.platform(),runs=runs,
        scope='current Windows audio backend/device only; supplied callback stamps plus polled exact render-completion, never backend return or acoustics'),indent=2)+'\n',encoding='utf-8')
    if not passed:raise RuntimeError('phase/latency acceptance failed; all results retained')


if __name__=='__main__':main()
