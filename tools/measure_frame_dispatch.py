"""Fixed main-boundaries off / on / on / off; all official spans/repaint sources.

All four use the same explicit feature binary and record actual API/event spans.
No CPU deduction, source patch, claim of default-product passing or sound change.
"""
import argparse,hashlib,json,os,subprocess
from collections import Counter
from pathlib import Path
from measure_audio_load import ROOT,distribution
from measure_audio_load import observe,summarize
from measure_gui_candidate import evaluate_gui
from measure_ui_wall import diagnostic_checks
from measure_background_stream import stream_checks,audit_checks
from measure_cpu_threads import token_info,thread_costs
from measure_independent_cpu import fingerprint,build_collector,calibration,scheduler_intervals
from measure_main_work import PLAN,Collector,run_child,ordinary,reference_workload_valid,main_costs,clock_mapping,MAX_CLOCK_NS
from measure_repaint import close_session,require_references,repaint_costs

NAMES=('ignored','surface_acquire','encoder_finish','rendered_queue_submit','rendered_present','redraw_event',
       'pointer_event','keyboard_ime_event','focus_event','geometry_event','other_window_event','repaint_user_event',
       'device_event','paint_integration','app_logic','painter_total','buffer_upload','texture_upload','draw_encoding',
       'handler_new_events','handler_about_to_wait','handler_window_event','handler_user_event','handler_device_event','handler_lifecycle')

def validate_spans(data):
    if (not isinstance(data,dict) or data.get('schema') not in (1,2,3) or any(data.get(k)!=0 for k in ('invalid','active','owners'))
            or type(data.get('spans')) is not int or not 0<data['spans']<=32768):
        raise ValueError('incomplete bounded framework spans or owner cleanup')
    rows=data.get('rows')
    if not isinstance(rows,list) or len(rows)!=data['spans']:raise ValueError('unclosed/unentered span')
    ids=set();offsets=[]
    for r in rows:
        if (not isinstance(r,list) or len(r)!=8 or any(type(v) is not int or v<0 for v in r)
                or not 2<=r[0]<=32769 or r[0] in ids or not 1<=r[1]<({1:15,2:19,3:25}[data['schema']])
                or not 0<r[2]<=r[3] or r[4]>r[5] or max(r[6:])>17):
            raise ValueError('invalid/duplicate/reversed framework span')
        ids.add(r[0]);offsets.extend((r[2]-r[4],r[3]-r[5]))
    spread=max(offsets)-min(offsets)
    if spread>MAX_CLOCK_NS:raise ValueError('span UTC/monotonic disagreement exceeds original bound')
    return rows,spread

def cost_regions(rows,pid,tid,a,b,frequency,offset,intervals,bound,extended=False,handlers=False):
    def qpc(epoch):return (epoch*frequency-offset)//1_000_000_000
    running=[(x,y) for p,t,c,x,y in intervals if p==pid and t==tid]
    result={}
    for k,name in enumerate(NAMES[1:(25 if handlers else 19)] if extended else NAMES[1:15],1):
        spans=sorted((qpc(r[2]),qpc(r[3])) for r in rows if r[1]==k)
        previous=0
        for start,end in spans:
            if start<previous:raise ValueError('same-kind overlapping framework spans')
            previous=end
        selected=[(max(start,a),min(end,b)) for start,end in spans if start<b and end>a]
        count=len(selected)
        ticks=sum(max(0,min(end,y)-max(start,x)) for start,end in selected for x,y in running)
        result[name]=dict(count=count,clipped=sum(start<a or end>b for start,end in spans if start<b and end>a),
            cpu_one_core_percent=100*ticks/(b-a),
            wall_ms=distribution([(end-start)*1000/frequency for start,end in selected]) if selected else None,
            boundary_uncertainty_percent_upper=2*bound*count*frequency/1e9*100/(b-a))
    required=(1,2,3,4,5,11,13,14)+((15,16,18) if extended else ())+((19,20,21,22) if handlers else ())
    if any(result[NAMES[k]]['count']<32 for k in required):
        raise ValueError('real normal-rendered API/event/logic spans missing')
    return result

def span_costs(trace,report,base,raw,intervals):
    rows,spread=validate_spans(raw)
    offset,clock=clock_mapping(report['clock_pairs'],trace['frequency'])
    main=[t for t in base['threads'] if t.get('process_entry_start')]
    if len(main)!=1:raise ValueError('main thread identity missing')
    w=base['costs']['voices_32'];a,b=w['qpc_begin'],w['qpc_end']
    regions=cost_regions(rows,report['pid'],main[0]['tid'],a,b,trace['frequency'],offset,intervals,clock['bound_ns'],raw['schema']>=2,raw['schema']==3)
    return dict(clock=clock,monotonic_disagreement_ns=spread,regions=regions,layout=layout_passes(raw,report['report']['ui_cost'][4][0]),
        scope='actual rendered API / nested event CPU wall and scheduled main-thread intersections; parents/children must not be added; includes unseparated ISR/DPC, no deduction')

def painter_partition(rows):
    """Whole stage-4 painter geometry, monotonic ns, exclusive child intervals.

    All enclosing windows/children are validated BEFORE subtracting any interval;
    this is not a difference of medians or an attribution of remainder to drivers.
    """
    child_kinds=(1,2,3,4,16,17,18)
    parents=sorted((r[4],r[5]) for r in rows if r[1]==15 and r[6:]==[4,4])
    children=sorted((r[4],r[5],r[1]) for r in rows if r[1] in child_kinds)
    if not 32<=len(parents)<=4096:raise ValueError('whole painter observations missing')
    groups={NAMES[k]:[] for k in child_kinds};groups['painter_other']=[];whole=[];last=0
    for a,b in parents:
        if a<last or b<=a:raise ValueError('overlapping/reversed painters')
        last=b;selected=[(x,y,k) for x,y,k in children if x<b and y>a]
        cursor=a;other=0;counts=Counter()
        for x,y,k in selected:
            if x<a or y>b or x<cursor or y<=x:raise ValueError('child not disjoint/contained in painter')
            other+=x-cursor;cursor=y;groups[NAMES[k]].append((y-x)/1e6);counts[k]+=1
        if any(counts[k]!=1 for k in (1,2,3,4,16,18)):
            raise ValueError('normal painter API/upload/encoding incomplete')
        other+=b-cursor;groups['painter_other'].append(other/1e6);whole.append((b-a)/1e6)
    return dict(painter_wall_ms=distribution(whole),exclusive_wall_ms={k:distribution(v) if v else None for k,v in groups.items()},
        scope='whole stage-4 painter, exclusive monotonic wall; parent/child containment checked; other includes locks, setup, notify, frees and observation, not driver attribution')

def run_without_uac(output,feature,initial,feature_hash):
    """No collector construction/build and no elevation path in this mode."""
    proof=dict(schema=1,identity=initial,trace_binary_sha256=feature_hash,rounds=[],uac_requested=False,
        cpu_meter_validated=False,cpu_budget_passed=None,p2_a3_accepted=False,diagnostic_complete=False)
    for i in (1,2):
        row=dict(round=i);path=output/('run-%d'%i)
        try:
            print('ordinary no-UAC painter diagnostic %d/2'%i,flush=True)
            if fingerprint()!=initial or hashlib.sha256(feature.read_bytes()).hexdigest()!=feature_hash:
                raise ValueError('frozen source/binary changed')
            report,samples=observe(feature,path,'--verify-dispatch-gui',timeout_seconds=45)
            metrics=summarize(samples,phases=('idle','voices_1','voices_32','closed'),minimum_spans=(('idle',.7),('voices_1',1.2),('voices_32',1.7),('closed',.7)))
            latency,joined,checks=evaluate_gui(report,metrics);checks.update(stream_checks(report,False));checks.update(audit_checks(report))
            raw=json.loads((path/'framework_trace.json').read_text(encoding='utf8'));rows,spread=validate_spans(raw)
            if raw['schema'] not in (2,3):raise ValueError('explicit schema 2/3 painter build required')
            row.update(latency=latency,joined_latency=joined,raw_metrics=metrics,original_checks=checks,diagnostic_checks=diagnostic_checks(checks),
                layout=layout_passes(raw,report['ui_cost'][4][0]),painter=painter_partition(rows),span_monotonic_disagreement_ns=spread,
                audio_health={k:report[k] for k in ('missing_frames','pcm_peak_bytes','pcm_final_bytes','source_raw_peak','source_raw_final',
                    'callback_resource_releases','endpoint_returns_in_render','endpoint_returns_outside_render','max_live_targets')},
                memory_peak={k:max(s[k] for s in samples) for k in ('working_set_bytes','private_bytes')})
        except (OSError,RuntimeError,ValueError,KeyError) as e:row['error']=str(e)
        proof['rounds'].append(row)
    proof['unchanged']=initial==fingerprint() and hashlib.sha256(feature.read_bytes()).hexdigest()==feature_hash
    proof['diagnostic_complete']=proof['unchanged'] and all('error' not in r and all(r['diagnostic_checks'].values()) for r in proof['rounds'])
    (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if not proof['diagnostic_complete']:raise SystemExit(1)

def layout_passes(raw,ui_calls):
    passes=raw.get('passes');sources=raw.get('discard_sources')
    if not isinstance(passes,list) or not 1<=len(passes)<=4096 or not isinstance(sources,list) or len(sources)>128:
        raise ValueError('missing layout/pass observation')
    for s in sources:
        if not isinstance(s,dict) or type(s.get('line')) is not int or not 0<s['line'] or not isinstance(s.get('file_utf8'),list):
            raise ValueError('invalid discard source')
        file=bytes(s['file_utf8']).decode('utf8')
        if not file or len(file)>160 or file.startswith(('/', '\\')) or ':' in file:raise ValueError('non-relative discard source')
    previous=None;frames=Counter();causes=Counter();discarded=0;count=0
    for p in passes:
        if not isinstance(p,dict) or not isinstance(p.get('row'),list) or len(p['row'])!=7 or any(type(v) is not int or v<0 for v in p['row']):
            raise ValueError('invalid output-pass row')
        r=p['row'];ids=p.get('causes')
        if (not r[0] or r[2]>17 or r[5]>2 or r[6]>1 or not isinstance(ids,list) or len(ids)>8
                or any(type(i) is not int or not 0<=i<len(sources) for i in ids)
                or previous and (r[0]<previous[0] or r[1]<previous[1] or r[3]<previous[3] or r[4]<=previous[4])):
            raise ValueError('unordered pass/frame or missing discard source')
        previous=r
        if r[2]==4:
            count+=1;frames[r[3]]+=1;discarded+=r[6];causes.update(ids)
    if type(ui_calls) is not int or count!=ui_calls or any(n>2 for n in frames.values()):
        raise ValueError('UI calls do not match fresh output passes')
    return dict(whole_stage_ui_calls=count,whole_stage_frame_ids=len(frames),extra_ui_passes=count-len(frames),
        will_discard=discarded,passes_per_frame=dict(sorted(Counter(frames.values()).items())),
        discard_sources=[dict(**sources[i],count=n) for i,n in causes.most_common()],
        scope='whole stage output-pass/frame identities, not inner CPU window refresh rate')

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',required=True);p.add_argument('--trace-binary',required=True)
    p.add_argument('--no-uac',action='store_true',help='Only two ordinary wall/health rounds; never build/start collector, CPU acceptance unavailable');args=p.parse_args()
    output=(ROOT/args.output).resolve();feature=Path(args.trace_binary).resolve()
    if (os.name!='nt' or ROOT/'artifacts' not in output.parents or output.exists() or not ordinary(token_info())
            or ROOT/'artifacts' not in feature.parents or not feature.is_file()):
        raise ValueError('ordinary runner, owned feature binary and fresh artifacts required')
    output.mkdir();initial=fingerprint();feature_hash=hashlib.sha256(feature.read_bytes()).hexdigest()
    if args.no_uac:return run_without_uac(output,feature,initial,feature_hash)
    binary=build_collector(output);runner=token_info();collector=None;refs=[];reports=[]
    self_test=json.loads(subprocess.check_output([str(binary),'--self-test'],text=True))
    if self_test.get('lifetime_self_test') is not True:raise ValueError('collector self-test failed')
    refpath=output/'references';refpath.mkdir()
    proof=dict(schema=1,identity=initial,trace_binary_sha256=feature_hash,plan=PLAN,runner_token=runner,collector_self_test=self_test,
        references=[],products=[],calibrated=False,products_started=False,passed=False,p2_a3_accepted=False)
    try:
        print('UAC 1/2: strict reference collector',flush=True);collector=Collector(binary,refpath)
        for i,reverse in enumerate((False,True),1):
            print('strict reference %d/2'%i,flush=True);r=run_child(refpath/('reference-%d'%i),'reference',reverse=reverse);refs.append(r)
            if not reference_workload_valid(r):raise ValueError('reference workload failed; products not admitted')
        trace,checks=close_session(collector,refpath,runner,refs,2);collector=None
        proof['reference_permission']=checks;proof['references']=[calibration(dict(trace,root_pid=r['pid']),r) for r in refs]
        require_references(proof['references']);proof['calibrated']=True
        path=output/'products';path.mkdir();print('strict references passed; UAC 2/2: products',flush=True);collector=Collector(binary,path)
        for i,profile in enumerate(PLAN,1):
            print('GUI %d/4, all repaint sources/spans; main boundaries=%s'%(i,profile),flush=True)
            proof['products_started']=True
            reports.append(run_child(path/('product-%d'%i),'product',profile=profile,
                binary=feature,ui_mode='dispatch' if profile else 'dispatch-repaint'))
        trace,checks=close_session(collector,path,runner,reports,4);collector=None;proof['product_permission']=checks
        intervals=scheduler_intervals(trace)
        for i,(profile,r) in enumerate(zip(PLAN,reports),1):
            v=thread_costs(trace,r);v['profile']=profile
            try:
                v['repaint_sources']=repaint_costs(trace,r,v)
                v['checks']['repaint_observation_complete']=True
                if profile:
                    v['main_categories']=main_costs(trace,r,v)
                raw=json.loads((path/('product-%d/gui/framework_trace.json'%i)).read_text(encoding='utf8'))
                v['framework_spans']=span_costs(trace,r,v,raw,intervals)
                v['checks']['real_framework_spans_complete']=True
            except (ValueError,KeyError) as e:
                v['attribution_error']=str(e);v['checks']['new_attribution_complete']=False
            v['checks']['strict_reference_calibration_passed']=True;proof['products'].append(v)
        proof['passed']=all(all(v['checks'].values()) for v in proof['products'])
    except (OSError,RuntimeError,ValueError,KeyError,subprocess.SubprocessError) as e:proof['error']=str(e);print(str(e),flush=True)
    finally:
        if collector:
            try:collector.close()
            except (OSError,RuntimeError) as e:proof['cleanup_error']=str(e)
        proof['unchanged']=initial==fingerprint() and hashlib.sha256(feature.read_bytes()).hexdigest()==feature_hash
        (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    if not proof['passed'] or not proof['unchanged']:raise SystemExit(1)

if __name__=='__main__':main()
