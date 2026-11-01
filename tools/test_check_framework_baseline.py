import unittest
from measure_framework_baseline import PLAN, control_checks, comparison_checks, editor_passed
from measure_main_work import product_argument


def fixture(role):
    stages=[[0]*4 for _ in range(18)];stages[0]=[10,10,9,0]
    stages[4]=[120,120 if role=='editor' else 0,0 if role=='framework' else 120,0 if role=='editor' else 120]
    return dict(gui_control=dict(schema=3,inactive_on_create=True,mouse_passthrough=True,mode=role,geometry_changed=False,stages=stages,
        geometry=dict(window=[1180,800],pixels_per_point=1,preview=[260,40,930,790],size=[670,750],generation=1),
        audio_policy='unchanged-streamed-runtime',nominal_playing_repaint_ms=16),ui_cost=[[0]]*4+[[120]],gui_passes=130,
        target_format_utf8=[1],rate=48000,channels=2,period_frames=480,source_frames=17280000)


def plan():
    values=[dict(role=r,adapter={'name':'fixture'},checks={
        'independent_cpu_all_stable_phases_le_15_one_core':True,'zero_missing_frames':True,
        'gui_request_copy_observed_upper_p95_le_20_ms':True},
        main_categories=dict(ui_calls_per_second=60,present_edges_per_second=60),
        repaint_sources=dict(immediate=0,input_calls=120,input_events=dict(pointer=0,focused_calls=0))) for r in PLAN]
    return values,[dict(report=fixture(r)) for r in PLAN]


class FrameworkBaselineTests(unittest.TestCase):
    def test_fixed_symmetric_plan_and_explicit_flags_leave_existing_defaults(self):
        self.assertEqual(PLAN,('framework','presentation','editor','editor','presentation','framework'))
        self.assertEqual(product_argument(False),'--verify-streamed-gui')
        self.assertEqual(product_argument(True),'--verify-main-work-gui')
        for role in set(PLAN):self.assertIn(role,product_argument(True,role))
        for profile,mode in [(False,'framework'),(True,'unknown')]:
            with self.assertRaises(ValueError):product_argument(profile,mode)

    def test_all_modes_keep_bootstrap_and_exercise_only_intended_measured_draw(self):
        for role in set(PLAN):self.assertTrue(all(control_checks(fixture(role),role).values()))
        for role in ('framework','presentation'):
            r=fixture(role);r['gui_control']['stages'][4][1]=1;r['gui_control']['stages'][4][3]=119
            with self.assertRaises(ValueError):control_checks(r,role)
        r=fixture('presentation');r['gui_control']['stages'][4][2]=0
        with self.assertRaises(ValueError):control_checks(r,'presentation')

    def test_geometry_generation_count_and_policy_errors_are_not_normalized(self):
        for change in (lambda r:r['gui_control'].update(geometry_changed=True),
                       lambda r:r['gui_control'].update(schema=1),
                       lambda r:r['gui_control'].update(inactive_on_create=False),
                       lambda r:r['gui_control'].update(mouse_passthrough=False),
                       lambda r:r['gui_control']['geometry'].update(pixels_per_point=float('nan')),
                       lambda r:r['gui_control']['geometry'].update(size=[0,750]),
                       lambda r:r['gui_control']['geometry'].update(generation=0),
                       lambda r:r.update(gui_passes=129),lambda r:r['gui_control'].update(audio_policy='changed')):
            r=fixture('framework');change(r)
            with self.assertRaises(ValueError):control_checks(r,'framework')

    def test_comparison_checks_device_geometry_audio_rate_focus_and_input(self):
        v,r=plan();self.assertTrue(all(comparison_checks(v,r).values()))
        changes=[lambda v,r:v[1]['adapter'].update(name='different'),
                 lambda v,r:r[1]['report'].update(rate=44100),
                 lambda v,r:r[1]['report']['gui_control']['geometry'].update(pixels_per_point=2),
                 lambda v,r:v[1]['main_categories'].update(ui_calls_per_second=64),
                 lambda v,r:v[1]['main_categories'].update(present_edges_per_second=64),
                 lambda v,r:v[1]['repaint_sources'].update(immediate=1),
                 lambda v,r:v[1]['repaint_sources']['input_events'].update(pointer=1),
                 lambda v,r:v[1]['repaint_sources']['input_events'].update(focused_calls=120)]
        for change in changes:
            v,r=plan();change(v,r);self.assertFalse(all(comparison_checks(v,r).values()))

    def test_control_success_cannot_cover_editor_failure_or_incomplete_plan(self):
        v,r=plan();self.assertTrue(editor_passed(v))
        v[2]['checks']['independent_cpu_all_stable_phases_le_15_one_core']=False;self.assertFalse(editor_passed(v))
        v,r=plan();v[3]['checks']['gui_request_copy_observed_upper_p95_le_20_ms']=False;self.assertFalse(editor_passed(v))
        self.assertFalse(editor_passed(v[:-1]))
        with self.assertRaises(ValueError):comparison_checks(v[:-1],r[:-1])
        with self.assertRaises(ValueError):comparison_checks([v[1],v[0],*v[2:]],r)

    def test_missing_metadata_or_missing_acceptance_predicates_cannot_pass(self):
        for change in [lambda r:r.update(gui_control=None),lambda r:r['gui_control'].update(stages=None),
                       lambda r:r['gui_control']['geometry'].update(preview=None)]:
            r=fixture('editor');change(r)
            with self.assertRaises(ValueError):control_checks(r,'editor')
        v,r=plan();v[2]['checks']={};self.assertFalse(editor_passed(v))


if __name__=='__main__':unittest.main()
