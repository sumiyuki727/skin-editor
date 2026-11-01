import unittest
from measure_ui_wall import widget_wall, diagnostic_checks
from measure_main_work import MAX_ROWS


def fixture():
    # Query time is exclusive; returning to the prelude must not open a new call.
    kinds = [2,5,15,5,9,10,15,10,11,15,11,12,15,12,13,14,8,7,8,4]
    return dict(schema=3, invalid=0, rows=[[10000+i*10,i*10,k] for i,k in enumerate(kinds)])


class UiWallTests(unittest.TestCase):
    def test_complete_call_has_exact_exclusive_partition(self):
        v = widget_wall(fixture())
        self.assertEqual(v['calls'],1);self.assertEqual(v['total_ns'],150)
        self.assertEqual(sum(s['total_ns'] for s in v['stages'].values()),150)
        self.assertEqual(v['stages']['project_snapshot']['total_ns'],40)
        self.assertEqual(v['stages']['layers_assets']['total_ns'],20)

    def test_multiple_calls_and_no_most_favorable_call_selection(self):
        work=fixture();second=[[a+1000,b+1000,k] for a,b,k in fixture()['rows']]
        work['rows']+=second;v=widget_wall(work,minimum_calls=2)
        self.assertEqual(v['calls'],2);self.assertEqual(v['total_ns'],300)
        with self.assertRaises(ValueError):widget_wall(work,minimum_calls=3)

    def test_incomplete_or_outside_or_host_interleaving_is_rejected(self):
        for change in (lambda w:w['rows'].pop(16),lambda w:w['rows'].pop(1),
                       lambda w:w['rows'].__setitem__(8,[10080,80,4]),
                       lambda w:w.update(rows=w['rows'][:16])):
            work=fixture();change(work)
            with self.assertRaises(ValueError):widget_wall(work)

    def test_capacity_clock_and_unknown_schema_fail_closed(self):
        for change in (lambda w:w.update(invalid=1),lambda w:w.update(schema=2),
                       lambda w:w.update(rows=[[1,1,0]]*(MAX_ROWS+1)),
                       lambda w:w['rows'].__setitem__(3,[10000,30,5]),
                       lambda w:w['rows'].__setitem__(3,[10030,1,5]),
                       lambda w:w['rows'].__setitem__(-1,[200000,190,4])):
            work=fixture();change(work)
            with self.assertRaises(ValueError):widget_wall(work)

    def test_cpu_raw_predicate_is_retained_but_cannot_authorize_acceptance(self):
        checks=dict(cpu_average_le_15_one_core=False,zero_missing_frames=True,clean=True)
        self.assertTrue(all(diagnostic_checks(checks).values()))
        self.assertFalse(checks['cpu_average_le_15_one_core'])
        checks['zero_missing_frames']=False
        self.assertFalse(all(diagnostic_checks(checks).values()))
    def test_new_details_are_exclusive_and_roll_up_without_reinterpreting_old_format(self):
        kinds=[2,5,9,16,17,9,13,18,19,20,21,22,23,24,13,14,8,4]
        w=dict(schema=4,invalid=0,rows=[[10000+i*10,i*10,k] for i,k in enumerate(kinds)])
        v=widget_wall(w)
        self.assertEqual(sum(s['total_ns'] for s in v['stages'].values()),v['total_ns'])
        self.assertEqual(v['groups']['scene_controls']['total_ns'],40)
        self.assertEqual(v['groups']['audio_panel']['total_ns'],90)
        w['schema']=3
        with self.assertRaises(ValueError):widget_wall(w)


if __name__ == '__main__':unittest.main()
