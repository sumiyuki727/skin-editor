import copy
import unittest
from measure_scheduling import analyze_timing,analyze_host,parts


def timing():
    rows=[[i,(i-1)*1_000_000+100,(i-1)*1_000_000+200,(i-1)*1_000_000+300,i*1_000_000,(i-1)*1_000_000,832,832,0,2] for i in range(1,5)]
    rows[3]=[4,3_000_100,53_000_000,53_010_000,53_020_000,3_000_000,832,832,0,2]
    context=[[0]*11,[1,1_000_000,480,1024,544,0,1,1,0,4,4],
             [2,10_000_000,480,800,320,0,3,3,0,4,4],[3,112_900_000,1056,832,0,224,4,4,0,4,4]]
    return dict(schema=1,rounding_ns=100,alignment=[200_000_000,200_000_000,200_000_100],quiescent=True,completed_pass=4,
        recent=rows,slow_count=1,slow=[rows[3]],event_count=1,events=[[4,3]+sum(context,[])+sum(rows,[])])


class SchedulingChecks(unittest.TestCase):
    def test_same_event_links_supply_sleep_work_and_callback_clock_bracket(self):
        a=analyze_timing(timing());e=a['events'][0]
        self.assertTrue(a['complete']);self.assertEqual(e['callback_index'],2)
        self.assertEqual(e['entry_gap_ms'],102.9);self.assertEqual(e['empty_frames'],224)
        self.assertEqual(e['entry_bounds_ns'],[112899900,112900200])
        self.assertEqual(e['overlapping_slow_passes'][0]['pass_id'],4)
        self.assertGreater(e['overlapping_slow_passes'][0]['sleep_to_wake_ms'],49)
        self.assertEqual(e['overlapping_slow_passes'][0]['pump_ms'],.01)

    def test_controls_and_pump_cost_are_not_mislabelled_sleep(self):
        r=[1,10,20,30_000_020,60_000_020,1,0,0,0,0]
        a=parts(r);self.assertEqual(a['controls_to_pump_ms'],30)
        self.assertEqual(a['pump_ms'],30);self.assertEqual(a['sleep_to_wake_ms'],.00001)

    def test_partial_overwritten_or_truncated_data_never_proves_completeness(self):
        s=timing();s['events'][0][76:86]=[0]*10
        self.assertFalse(analyze_timing(s)['complete'])
        s=timing();s['recent'][3]=None;self.assertFalse(analyze_timing(s)['complete'])
        s=timing();s['events'][0][76]=3
        with self.assertRaises(ValueError):analyze_timing(s)
        s=timing();s['slow_count']=33;s['slow']*=32
        with self.assertRaises(ValueError):analyze_timing(s)
        s=timing();s['event_count']=33;s['events']=[None]*32
        self.assertFalse(analyze_timing(s)['complete'])

    def test_host_wakeup_lateness_is_distinct_from_previous_app_work(self):
        h=dict(late_count=1,rows=[[1,2,4,3,125_000_000,5,6,545_500_003,420_500_000,546_000_003,546_000_004]],late_supply=[])
        a=analyze_host(h)['rows'][0]
        self.assertEqual(a['nominal_wakeup_lateness_ms'],420.5)
        self.assertTrue(a['previous_app_ended_before_requested_wake'])
        self.assertLess(a['previous_logic_ms'],.001)
        h['rows'][0][6]=500_000_000
        self.assertFalse(analyze_host(h)['rows'][0]['previous_app_ended_before_requested_wake'])
        h['rows'][0][8]+=1
        with self.assertRaises(ValueError):analyze_host(h)

    def test_missing_clock_or_wrong_host_snapshot_is_rejected(self):
        s=timing();s['alignment'][2]=1
        with self.assertRaises(ValueError):analyze_timing(s)
        h=dict(late_count=0,rows=[],late_supply=[dict(at_ns=1,timing=timing())])
        with self.assertRaises(ValueError):analyze_host(h)


if __name__=='__main__':unittest.main()
