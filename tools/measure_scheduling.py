"""Bounded diagnostic correlation only; never replaces the original gates."""


def parts(r):
    p,s,w,b,e,previous,_,_,_,_=r
    return dict(pass_id=p,previous_finish_ns=previous,finish_ns=e,
        post_pump_ms=(s-previous)/1e6 if s and previous else None,
        sleep_to_wake_ms=(w-s)/1e6 if s and w else None,
        controls_to_pump_ms=(b-w)/1e6 if w else None,pump_ms=(e-b)/1e6)


def analyze_timing(s):
    unsigned=lambda r:all(type(v) is int and v>=0 for v in r)
    a=s['alignment'];head=s['completed_pass']
    if (s['schema']!=1 or len(a)!=3 or not unsigned(a) or a[1]>a[2] or
            type(head) is not int or head<0 or type(s['quiescent']) is not bool or s['rounding_ns'] not in (0,100)):
        raise ValueError('invalid scheduling clock/schema')
    def valid(r,limit):
        if (len(r)!=10 or not unsigned(r) or not 0<r[0]<=limit or not r[5]<=r[3]<=r[4] or
                max(r[6:9])>2048 or r[2]>r[3] or r[1] and not r[5]<=r[1]<=r[2]):
            raise ValueError('invalid supply completion')
    complete=True;recent=s['recent']
    if len(recent)!=4:raise ValueError('missing recent supply slots')
    for i,row in enumerate(recent):
        expected=head-3+i
        if row is not None:
            valid(row,head)
            if row[0]!=expected:raise ValueError('overwritten supply row accepted')
        elif expected>0:complete=False
    slow=s['slow'];events=s['events']
    if (type(s['slow_count']) is not int or type(s['event_count']) is not int or min(s['slow_count'],s['event_count'])<0 or
            len(slow)!=min(32,s['slow_count']) or len(events)!=min(32,s['event_count'])):
        raise ValueError('missing bounded scheduling windows')
    complete &= s['slow_count']<=32 and s['event_count']<=32
    previous=0
    for row in slow:
        if row is None:complete=False;continue
        valid(row,head)
        if row[0]<=previous or row[4]-row[5]<=20_000_000:raise ValueError('invalid slow supply row')
        previous=row[0]
    bounds=lambda stamp:[a[1]+stamp-a[0]-s['rounding_ns'],a[2]+stamp-a[0]+s['rounding_ns']]
    correlated=[];previous=0
    for event in events:
        if event is None:complete=False;continue
        if len(event)!=86 or not unsigned(event) or event[0]>head or not 1<=event[1]<=3:
            raise ValueError('invalid callback scheduling event')
        callbacks=[event[2+i*11:13+i*11] for i in range(4)];cur=callbacks[-1];prior=callbacks[-2]
        if cur[0]<=previous or cur[2]==0 or cur[5]>cur[2]:raise ValueError('invalid event callback identity')
        previous=cur[0];gap=prior[0]>0 and cur[1]-prior[1]>20_000_000
        if bool(event[1]&1)!=gap or bool(event[1]&2)!=(cur[5]>0):raise ValueError('wrong event reason')
        supplies=[]
        for i in range(4):
            row=event[46+i*10:56+i*10];expected=event[0]-3+i
            if row[0]:
                valid(row,event[0])
                if row[0]!=expected:raise ValueError('wrong event supply pass')
                supplies.append(row)
            elif any(row):raise ValueError('partial supply is not unknown')
            elif expected>0:complete=False
        entry=bounds(cur[1]);start=bounds(prior[1]) if prior[0] else None
        overlap=[parts(r) for r in slow if r is not None and start is not None and r[4]>=start[0] and r[5]<=entry[1]]
        correlated.append(dict(callback_index=cur[0]-1,reason=event[1],entry_bounds_ns=entry,
            entry_gap_ms=(cur[1]-prior[1])/1e6 if prior[0] else None,requested_frames=cur[2],queued_before=cur[3],empty_frames=cur[5],
            completed_pass_difference=cur[6]-prior[7] if prior[0] and cur[6]>=prior[7] else None,
            latest_supply_age_at_entry_ms=[(v-supplies[-1][4])/1e6 for v in entry] if supplies else None,
            supplies=[parts(r) for r in supplies],overlapping_slow_passes=overlap))
    return dict(complete=complete,quiescent=s['quiescent'],slow=[parts(r) for r in slow if r is not None],events=correlated,
        scope='Sleep-to-wake includes configured sleep and scheduling; controls include preparation; callback entry is not completion or sound.')


def analyze_host(h):
    rows=h['rows'];count=h['late_count']
    if type(count) is not int or count<0 or len(rows)!=min(64,count):raise ValueError('missing host cadence rows')
    result=[]
    for r in rows:
        if (len(r)!=11 or any(type(v) is not int or v<0 for v in r) or not r[0]<=r[1]<=r[2] or
                r[3]==0 or r[7]-r[3]-r[4]!=r[8] or r[8]<=20_000_000 or
                r[9] and r[7]>r[9] or r[10] and r[9]>r[10]):raise ValueError('invalid host cadence interval')
        result.append(dict(begin_ns=r[7],nominal_wakeup_lateness_ms=r[8]/1e6,
            previous_tick_ms=(r[1]-r[0])/1e6,previous_logic_ms=(r[2]-r[0])/1e6,
            previous_last_ui_ms=(r[6]-r[5])/1e6 if r[5] and r[6] else None,
            previous_app_ended_before_requested_wake=max(r[2],r[6])<=r[3]+r[4],
            current_tick_ms=(r[9]-r[7])/1e6 if r[9] else None))
    if len(h['late_supply'])>8:raise ValueError('unbounded late host snapshots')
    snapshots=[]
    for record in h['late_supply']:
        if record['at_ns'] not in [r[7] for r in rows]:raise ValueError('unmatched host snapshot')
        if not record['at_ns']<=record['sample_at_ns']<=record['sample_end_ns']:raise ValueError('invalid host collection span')
        snapshots.append(dict(at_ns=record['at_ns'],analysis=analyze_timing(record['timing']),
            collection_delay_after_entry_ms=(record['sample_at_ns']-record['at_ns'])/1e6,
            collection_ms=(record['sample_end_ns']-record['sample_at_ns'])/1e6))
    return dict(complete=count<=64,rows=result,supply_snapshots=snapshots,
        scope='Requested repaint is nominal, not an OS guarantee; last UI subpass excludes GPU/presentation and event-loop work.')
