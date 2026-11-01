import copy
import unittest
from measure_voice_phase import exact_analysis,phase_analysis,request_phase_analysis,underrun_analysis


def fixture(count=64):
    r=dict(rate=48000,channels=2,period_frames=480,empty_frames=0,empty_callbacks=0,
           pcm_peak_bytes=100,pcm_final_bytes=0,raw_peak_bytes=100,raw_final_bytes=0,
           schema=2,callbacks=400,frames=192000,retimed_idle_frames=0,max_callback_frames=480,max_empty_callback_frames=0,
           stage_counts=[[0]*5 for _ in range(8)],startup_observations=[[480,1024,0,i+1] for i in range(32)],
           underrun_observations=[],underrun_contexts=[])
    r['stage_counts'][4]=[400,192000,0,0,0]
    for key in ('exact_voice_intervals_ms','completion_markers','requested_voice_ids','submission_intervals_ms','submission_causal_ms','latency_upper_bound_ms','request_phase_age_ms','request_callback_refs'):
        r[key]=[[],[]]
    for lane in range(2):
        for i in range(count):
            voice=i+1;serial=100+i
            r['exact_voice_intervals_ms'][lane].append([11.1,12.1])
            r['completion_markers'][lane].append([voice,2,serial,10_000_000,33,480,i+2])
            r['requested_voice_ids'][lane].append(voice)
            r['submission_intervals_ms'][lane].append([11,12,2,2,0])
            r['submission_causal_ms'][lane].append([2,2.1,2.11,10,10.001,768,128,1,33,480,serial,2,i+2])
            r['latency_upper_bound_ms'][lane].append(12)
            r['request_phase_age_ms'][lane].append([i%10+.2,i%10+.201])
            r['request_callback_refs'][lane].append([i,1_000_000+i*60,480,832])
    if count==160:
        r.update(count_per_lane=160,requested_bins=20,repeats=8,
                 phase_rows=[[[i*7%20,(i*7%20)*.5+.05,(i*7%20)*.5+.051,.05,10] for i in range(160)] for _ in range(2)],
                 callback_references=[[[i,1_000_000+i*60,480,832] for i in range(160)] for _ in range(2)])
    return r


def with_underrun(r):
    r.update(empty_frames=416,empty_callbacks=1,max_empty_callback_frames=480,
        underrun_observations=[[2,480,64,416,1,1]],
        underrun_contexts=[[[0]*11,[1,100,480,1024,544,0,1,1,0,4,4],
                            [2,200,480,544,64,0,1,1,0,4,4],[3,300,480,64,0,416,1,1,0,4,4]]])
    r['stage_counts'][4][2:4]=[1,416]
    return r


class VoicePhaseChecks(unittest.TestCase):
    def test_standard_phase_uses_prior_callback_and_does_not_fold_old_reference(self):
        r=fixture();a=request_phase_analysis(r)
        self.assertEqual(sum(b['count'] for b in a[0]['bins']),64)
        r['request_phase_age_ms'][0][0]=[11,11.1]
        self.assertEqual(request_phase_analysis(r)[0]['unclassified'],1)
        r['request_callback_refs'][0][0][0]=r['completion_markers'][0][0][6]
        with self.assertRaises(ValueError):request_phase_analysis(r)
    def test_exact_voice_marker_requires_same_epoch_serial_callback_and_requested_identity(self):
        r=fixture();self.assertEqual(exact_analysis(r)[0]['classification'],'within')
        for index in (0,1,2,4,5,6):
            bad=copy.deepcopy(r);bad['completion_markers'][0][3][index]+=1
            with self.subTest(field=index),self.assertRaises(ValueError):exact_analysis(bad)
        for key in ('exact_voice_intervals_ms','requested_voice_ids'):
            bad=copy.deepcopy(r);bad[key][1].pop()
            with self.assertRaises(ValueError):exact_analysis(bad)
        with self.assertRaises(ValueError):exact_analysis(r,257)

    def test_completion_is_an_interval_and_entry_stamp_cannot_replace_its_upper_bound(self):
        r=fixture();r['exact_voice_intervals_ms']=[[[18,21] for _ in range(64)] for _ in range(2)]
        a=exact_analysis(r);self.assertEqual(a[0]['classification'],'unresolved')
        self.assertEqual(a[0]['individually_straddling'],64)
        r['exact_voice_intervals_ms']=[[[21,22] for _ in range(64)] for _ in range(2)]
        self.assertEqual(exact_analysis(r)[0]['classification'],'over')
        r['exact_voice_intervals_ms'][0][0]=[8,9]
        with self.assertRaises(ValueError):exact_analysis(r) # before callback entry

    def test_balanced_requests_do_not_prove_achieved_phase_coverage(self):
        r=fixture(160);a=phase_analysis(r);self.assertTrue(all(a['checks'].values()))
        for lane in r['phase_rows']:
            for row in lane:row[1:3]=[1,1.01]
        self.assertFalse(phase_analysis(r)['checks']['phase_coverage'])
        for lane in r['phase_rows']:
            for row in lane:row[1:3]=[11,11.01]
        a=phase_analysis(r);self.assertEqual(a['lanes'][0]['unclassified'],160)
        self.assertFalse(a['checks']['phase_coverage'])

    def test_exact_marker_does_not_waive_old_gate_or_resource_failures(self):
        r=fixture(160)
        for lane in range(2):
            r['latency_upper_bound_ms'][lane]=[21]*160
            r['submission_intervals_ms'][lane]=[[19,21,2,2,0] for _ in range(160)]
            r['exact_voice_intervals_ms'][lane]=[[19,19.9] for _ in range(160)]
        a=phase_analysis(r)
        self.assertTrue(a['checks']['exact_render_p95_le_20_ms'])
        self.assertFalse(a['checks']['original_submission_p95_le_20_ms'])
        with_underrun(r);r.update(pcm_final_bytes=4,pcm_peak_bytes=33*1024**2)
        a=phase_analysis(r)
        for key in ('no_underflow','resources_released','managed_limits'):self.assertFalse(a['checks'][key])

    def test_missing_order_reference_and_variable_callback_are_visible(self):
        r=fixture(160);r['phase_rows'][0][0][0]=1
        with self.assertRaises(ValueError):phase_analysis(r)
        r=fixture(160);r['callback_references'][0][0][2]=960
        self.assertFalse(phase_analysis(r)['checks']['phase_coverage'])

    def test_event_context_distinguishes_consecutive_drain_from_one_large_callback(self):
        r=with_underrun(fixture(160));a=underrun_analysis(r)['events'][0]
        self.assertEqual(a['consecutive_drain_callbacks'],3)
        self.assertEqual(a['consecutive_drain_queues'],[1024,544,64])
        self.assertEqual(a['callback_entry_gaps_ns'],[100,100])
        self.assertFalse(phase_analysis(r)['checks']['no_underflow'])
        r['underrun_observations'][0]=[2,1056,640,416,9,9]
        r['underrun_contexts'][0][3]=[3,300,1056,640,0,416,9,9,0,2,6]
        r['max_callback_frames']=r['max_empty_callback_frames']=1056
        a=underrun_analysis(r)['events'][0]
        self.assertEqual(a['consecutive_drain_callbacks'],1)
        self.assertEqual((a['entry_stage'],a['exit_stage']),('prepare','stop'))
        self.assertTrue(a['stage_crossed'])

    def test_partial_context_and_missing_inconsistent_stage_counts_are_rejected(self):
        r=with_underrun(fixture(160))
        for mutate in (lambda b:b['stage_counts'].pop(),lambda b:b['stage_counts'][4].__setitem__(3,0),
                       lambda b:b['underrun_contexts'].__setitem__(0,None),
                       lambda b:b['underrun_contexts'][0][3].__setitem__(5,415),
                       lambda b:b['underrun_contexts'][0][2].__setitem__(0,0)):
            bad=copy.deepcopy(r);mutate(bad)
            with self.assertRaises(ValueError):underrun_analysis(bad)
        r['schema']=1
        with self.assertRaises(ValueError):underrun_analysis(r)

    def test_in_progress_supply_or_retiming_cannot_be_labelled_uninterrupted_drain(self):
        r=with_underrun(fixture(160))
        r['underrun_observations'][0][5]=2;r['underrun_contexts'][0][3][7]=2
        a=underrun_analysis(r)['events'][0]
        self.assertFalse(a['queue_accounting_exact']);self.assertEqual(a['consecutive_drain_callbacks'],0)
        r=with_underrun(fixture(160));r['underrun_contexts'][0][3][8]=32
        self.assertFalse(underrun_analysis(r)['events'][0]['queue_accounting_exact'])

    def test_trace_overflow_retains_totals_and_fails_completeness(self):
        r=fixture(160);r.update(empty_callbacks=40,empty_frames=19200,max_empty_callback_frames=480)
        r['stage_counts'][4][2:4]=[40,19200]
        r['underrun_observations']=[[i,480,0,480,0,0] for i in range(32)]
        r['underrun_contexts']=[]
        for i in range(32):
            rows=[[j+1,100+j,480,0,0,480,0,0,0,4,4] for j in range(max(0,i-3),i+1)]
            r['underrun_contexts'].append([[0]*11]*(4-len(rows))+rows)
        a=phase_analysis(r)
        self.assertEqual(a['underruns']['missing_events'],8)
        self.assertFalse(a['checks']['underrun_trace_complete'])


if __name__=='__main__':unittest.main()
