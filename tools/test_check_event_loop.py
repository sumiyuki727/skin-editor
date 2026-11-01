import copy
import unittest
from measure_event_loop_ui import handler_gaps, handler_wall, delivery_counts, stage_delivery_counts
from measure_gui_base import PLAN
from measure_frame_dispatch import validate_spans


def row(kind, a, b, identity):
    return [identity, kind, a, b, a, b, 4, 4]


class EventLoopTests(unittest.TestCase):
    def test_discrete_stage_counts_cannot_requalify_a_failed_clock_or_missing_record(self):
        rows=[];passes=[]
        for i in range(40):
            for k in (5,19,20,21,22):
                a=100+i*100+k
                rows.append(row(k,a,a+2,len(rows)+2))
            passes.append(dict(row=[100+i,100+i,4,i,i+1,1,0],causes=[]))
        raw=dict(schema=3,invalid=0,active=0,owners=0,spans=len(rows),rows=rows,passes=passes,discard_sources=[])
        # Deliberately invalid UTC agreement: discrete facts remain usable, but
        # the original attribution gate MUST still reject and CPU stays false.
        raw['rows'][0][2]+=100000;raw['rows'][0][3]+=100000
        with self.assertRaises(ValueError):validate_spans(raw)
        r=stage_delivery_counts(raw,40,1.0)
        self.assertFalse(r['cpu_comparison_valid']);self.assertEqual(r['frames_per_second'],40)
        self.assertEqual(r['counts']['handler_user_event'],40)
        for change in [lambda d:d.update(active=1),lambda d:d['rows'].pop(),
                       lambda d:d['rows'][0].__setitem__(0,d['rows'][1][0]),lambda d:d.update(schema=2)]:
            bad=copy.deepcopy(raw);change(bad)
            with self.assertRaises(ValueError):stage_delivery_counts(bad,40,1.0)

    def test_actual_delivery_counts_use_same_window_and_reject_missing_or_partial_data(self):
        values=[]
        for role in PLAN:
            counts=dict(redraw_event=120,handler_user_event=240 if role=='baseline' else 120,handler_new_events=260,handler_about_to_wait=260)
            values.append(dict(role=role,framework_spans=dict(regions={k:dict(count=n) for k,n in counts.items()})))
        result=delivery_counts(values)
        self.assertEqual([r['repaint_events_per_redraw'] for r in result],[2,1,1,2])
        for change in [lambda v:v.pop(), lambda v:v[1].pop('framework_spans'),
                       lambda v:v[1]['framework_spans']['regions']['handler_user_event'].update(count=0)]:
            bad=copy.deepcopy(values);change(bad)
            with self.assertRaises(ValueError):delivery_counts(bad)

    def test_ordinary_wall_excludes_cross_stage_gaps_and_keeps_cpu_unknown(self):
        rows=[]
        for i in range(40):
            for n,k in enumerate((19,21,22,20)):
                a=100+i*100+n*10;rows.append(row(k,a,a+2,len(rows)+2))
        rows[-1][7]=5
        result=handler_wall(rows)
        self.assertIsNone(result['cpu_budget_passed'])
        self.assertFalse(result['cpu_meter_validated'])
        self.assertEqual(result['callback_wall_ms']['handler_about_to_wait']['count'],39)
        self.assertEqual(result['gap_wall_ms']['handler_about_to_wait->handler_new_events']['count'],39)
        with self.assertRaises(ValueError):handler_wall(rows[:4])
        bad=copy.deepcopy(rows);bad[0][5]=bad[1][5]+1
        with self.assertRaises(ValueError):handler_wall(bad)

    def test_gaps_are_same_thread_intersections_not_wait_wall_or_parent_sums(self):
        rows = [row(20, 10, 20, 2), row(19, 100, 110, 3), row(21, 140, 180, 4)]
        running = [(1, 2, 0, 18, 25), (1, 2, 1, 24, 30), (1, 2, 0, 120, 130), (9, 2, 0, 20, 100)]
        result = handler_gaps(rows, 1, 2, 15, 170, 10**9, 0, running, 2)
        self.assertEqual(result['qpc_ticks'], 21)
        g = result['groups']['handler_about_to_wait->handler_new_events']
        self.assertEqual(g['qpc_ticks'], 11)
        self.assertEqual(g['wall_ticks'], 80)
        self.assertGreater(g['boundary_uncertainty_percent_upper'], 0)

    def test_edge_clipping_and_missing_or_reentrant_handlers_fail(self):
        rows = [row(20, 10, 20, 2), row(19, 100, 110, 3)]
        r = handler_gaps(rows, 1, 2, 25, 90, 10**9, 0, [(1, 2, 0, 1, 200)], 2)
        self.assertEqual(r['qpc_ticks'], 65)
        self.assertEqual(next(iter(r['groups'].values()))['clipped'], 1)
        for bad in [[], rows[:1], [rows[0], row(19, 19, 110, 3)]]:
            with self.assertRaises(ValueError):
                handler_gaps(bad, 1, 2, 15, 105, 10**9, 0, [], 2)

    def test_schema_3_handlers_do_not_reinterpret_older_capture_versions(self):
        data = dict(schema=3, invalid=0, active=0, owners=0, spans=1, rows=[row(19, 10, 20, 2)])
        self.assertEqual(validate_spans(data)[1], 0)
        for version in (1, 2):
            d = copy.deepcopy(data); d['schema'] = version
            with self.assertRaises(ValueError):
                validate_spans(d)


if __name__ == '__main__':
    unittest.main()
