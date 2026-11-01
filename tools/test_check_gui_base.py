import copy
import unittest
from measure_gui_base import PLAN,candidate_passed,comparison_checks

class GuiBaseTests(unittest.TestCase):
    def rows(self):return [dict(role=role,checks=dict(cpu=role=='candidate',audio=True,
        independent_cpu_all_stable_phases_le_15_one_core=True,zero_missing_frames=True,
        gui_request_copy_observed_upper_p95_le_20_ms=True)) for role in PLAN]
    def test_fixed_abba_retains_failed_baselines_without_using_them_as_candidate_acceptance(self):
        rows=self.rows();self.assertTrue(candidate_passed(rows));self.assertFalse(rows[0]['checks']['cpu'])
    def test_candidate_failure_or_missing_result_cannot_pass(self):
        rows=self.rows();rows[2]['checks']['audio']=False;self.assertFalse(candidate_passed(rows))
        self.assertFalse(candidate_passed(self.rows()[:-1]))
    def test_reordered_or_extra_results_are_not_cherry_picked(self):
        rows=self.rows();rows[0],rows[1]=rows[1],rows[0];self.assertFalse(candidate_passed(rows))
        self.assertFalse(candidate_passed(self.rows()+[copy.deepcopy(self.rows()[1])]))
    def test_missing_predicates_do_not_authorize_candidate(self):
        rows=self.rows();rows[1]['checks']={};self.assertFalse(candidate_passed(rows))
    def test_rejected_boundary_clock_remains_a_failed_comparison_without_partial_selection(self):
        v,r=self.plan();v[0].pop('main_categories');v[0]['attribution_error']='clock disagreement'
        self.assertEqual(comparison_checks(v,r),dict(observations_complete_for_comparison=False))
        self.assertIn('attribution_error',v[0])
    def plan(self):
        values=[dict(v,adapter='fixture',main_categories=dict(ui_calls_per_second=60,present_edges_per_second=60),
            repaint_sources=dict(immediate=0,input_calls=120,input_events=dict(pointer=0,focused_calls=0))) for v in self.rows()]
        reports=[dict(report=dict(gui_control=None,visual=[[5,0,0,0,0,1,679,741,False,False]]*5,
            target_format_utf8=[1],rate=48000,channels=2,period_frames=480,source_frames=17280000)) for _ in PLAN]
        return values,reports
    def test_ordinary_comparison_rejects_geometry_device_rate_input_and_focus_mismatch(self):
        v,r=self.plan();self.assertTrue(all(comparison_checks(v,r).values()))
        for change in [lambda v,r:r[1]['report'].update(rate=44100),lambda v,r:v[1].update(adapter='different'),
            lambda v,r:r[1]['report']['visual'][4].__setitem__(6,680),
            lambda v,r:v[1]['main_categories'].update(ui_calls_per_second=64),
            lambda v,r:v[1]['repaint_sources'].update(immediate=1),
            lambda v,r:v[1]['repaint_sources']['input_events'].update(pointer=1),
            lambda v,r:v[1]['repaint_sources']['input_events'].update(focused_calls=120)]:
            v,r=self.plan();change(v,r);self.assertFalse(all(comparison_checks(v,r).values()))
        v,r=self.plan();r[1]['report']['gui_control']={}
        with self.assertRaises(ValueError):comparison_checks(v,r)
    def test_incomplete_or_reordered_comparison_is_rejected(self):
        v,r=self.plan()
        with self.assertRaises(ValueError):comparison_checks(v[:-1],r[:-1])
        v[0],v[1]=v[1],v[0]
        with self.assertRaises(ValueError):comparison_checks(v,r)
        for change in [lambda r:r[1]['report'].pop('gui_control'),
            lambda r:r[1]['report']['visual'].__setitem__(4,[])]:
            v,r=self.plan();change(r)
            with self.assertRaises(ValueError):comparison_checks(v,r)

if __name__=='__main__':unittest.main()
