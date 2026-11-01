import copy
import json
import tempfile
import unittest
from pathlib import Path
from measure_painter_cpu import exclusive_cpu, analyze
from measure_main_work import PLAN


def row(kind, a, b):
    return [kind + 2, kind, a, b, a, b, 4, 4]


def fixture():
    return [row(5, 100, 900), row(13, 150, 850), row(15, 500, 800),
            row(1, 510, 530), row(16, 550, 600), row(18, 610, 620),
            row(2, 630, 650), row(3, 660, 700), row(4, 720, 740), row(14, 200, 300)]


def compute(rows=None, intervals=None, begin=50, end=1000):
    return exclusive_cpu(fixture() if rows is None else rows, 1, 2, begin, end, 10**9, 0,
                         [(1, 2, 0, 1, 1000)] if intervals is None else intervals, 2)


class PainterCpuTests(unittest.TestCase):
    def test_incomplete_uncalibrated_or_unowned_capture_is_not_analyzed(self):
        proof = dict(calibrated=True, unchanged=True, plan=list(PLAN), products=[{}] * 4,
                     product_permission=dict(ordinary_runner=True), references=[dict(passed=True)] * 2)
        for change in [lambda p: p.update(calibrated=False), lambda p: p.update(unchanged=False),
                       lambda p: p['products'].pop(), lambda p: p['plan'].__setitem__(0, True),
                       lambda p: p.update(product_permission={}),
                       lambda p: p.update(product_permission=dict(ordinary_runner=False)),
                       lambda p: p.update(references=[dict(passed=True), dict(passed=False)])]:
            p = copy.deepcopy(proof); change(p)
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory); (root / 'proof.json').write_text(json.dumps(p), encoding='utf8')
                with self.assertRaises(ValueError):
                    analyze(root)

    def test_nested_cpu_is_exclusive_not_sum_of_parent_and_child(self):
        r = compute(); regions = r['regions']
        self.assertEqual(r['main_qpc_ticks'], 950)
        self.assertEqual(sum(v['exclusive_qpc_ticks'] for v in regions.values()), 950)
        self.assertEqual(regions['painter_other']['exclusive_qpc_ticks'], 140)
        self.assertEqual(regions['integration_other']['exclusive_qpc_ticks'], 300)
        self.assertEqual(regions['redraw_other']['exclusive_qpc_ticks'], 100)
        self.assertEqual(regions['outside_recorded_scopes']['exclusive_qpc_ticks'], 150)
        self.assertIsNone(regions['texture_upload']['cpu_one_core_percent'])
        self.assertGreater(regions['painter_other']['boundary_uncertainty_percent_upper'], 0)

    def test_clipped_edge_scopes_keep_exact_window_and_wait_is_not_cpu(self):
        r = compute(intervals=[(1, 2, 0, 510, 520)], begin=505, end=750)
        self.assertEqual(r['main_qpc_ticks'], 10)
        self.assertEqual(r['regions']['surface_acquire']['exclusive_qpc_ticks'], 10)
        self.assertEqual(r['regions']['painter_other']['exclusive_qpc_ticks'], 0)
        self.assertGreater(r['regions']['painter_other']['exclusive_wall_ms'], 0)

    def test_wrong_pid_tid_ignored_and_conservative_overlap_never_deducted(self):
        r = compute(intervals=[(1, 2, 0, 510, 525), (1, 2, 1, 520, 530),
                               (9, 2, 0, 1, 1000), (1, 9, 1, 1, 1000)])
        self.assertEqual(r['main_qpc_ticks'], 25)
        self.assertEqual(r['regions']['surface_acquire']['exclusive_qpc_ticks'], 25)

    def test_crossing_same_kind_and_wrong_parent_fail(self):
        for extra in [row(11, 120, 200), row(1, 520, 540), row(17, 810, 820), row(15, 950, 960)]:
            with self.assertRaises(ValueError):
                compute(fixture() + [extra])

    def test_zero_duration_and_reordered_input_do_not_steal_ownership(self):
        r = compute(list(reversed(fixture())) + [row(17, 700, 700)])
        self.assertEqual(r['main_qpc_ticks'], 950)
        self.assertEqual(r['regions']['texture_upload']['calls'], 1)
        self.assertEqual(r['regions']['texture_upload']['exclusive_qpc_ticks'], 0)
        self.assertEqual(r['regions']['painter_other']['exclusive_qpc_ticks'], 140)

    def test_invalid_window_and_unknown_kind_fail(self):
        for kwargs in [dict(begin=1000, end=50), dict(begin=0), dict(rows=[row(25, 100, 200)])]:
            with self.assertRaises(ValueError):
                compute(**kwargs)
        rows = copy.deepcopy(fixture()); rows[0][3] = 1
        with self.assertRaises(ValueError):
            compute(rows)


if __name__ == '__main__':
    unittest.main()
