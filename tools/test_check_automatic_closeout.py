import unittest
from measure_automatic_closeout import gpu_budget, process_boundary


class CompleteGpuBudget(unittest.TestCase):
    def test_unknown_missing_and_all_lifecycle_samples_are_not_waived(self):
        good=dict(gpu_dedicated_bytes=256*1024**2,gpu_shared_bytes=128*1024**2)
        self.assertTrue(all(v['passed'] for v in gpu_budget([good]).values()))
        for bad in ({},dict(good,gpu_dedicated_bytes=None),dict(good,gpu_shared_bytes=128*1024**2+1)):
            self.assertFalse(all(v['passed'] for v in gpu_budget([good,bad]).values()))
        self.assertFalse(all(v['passed'] for v in gpu_budget([]).values()))

    def test_exit_boundary_is_recorded_unknown_and_never_excuses_live_unknown_or_known_overrun(self):
        good=process_boundary(dict(phase='closed',gpu_dedicated_bytes=100,gpu_shared_bytes=100),True,True)
        exit_sample=process_boundary(dict(phase='closed'),True,False)
        a=gpu_budget([good,exit_sample])
        self.assertTrue(all(v['passed'] for v in a.values()))
        self.assertEqual(a['dedicated']['missing'],1)
        self.assertEqual(a['dedicated']['unknown_exit_boundary'],1)
        self.assertEqual(a['dedicated']['max_bytes'],100)
        self.assertFalse(all(v['passed'] for v in gpu_budget([exit_sample]).values()))
        live_unknown=process_boundary(dict(phase='closed'),True,True)
        self.assertFalse(all(v['passed'] for v in gpu_budget([good,live_unknown]).values()))
        overrun=process_boundary(dict(gpu_dedicated_bytes=256*1024**2+1,gpu_shared_bytes=100),True,False)
        self.assertFalse(all(v['passed'] for v in gpu_budget([good,overrun]).values()))

    def test_only_a_contiguous_initial_loading_prefix_can_be_reported_separately(self):
        loading=process_boundary(dict(phase='loading'),True,True)
        known=process_boundary(dict(phase='idle',gpu_dedicated_bytes=100,gpu_shared_bytes=100),True,True)
        a=gpu_budget([loading,known])
        self.assertTrue(all(v['passed'] for v in a.values()))
        self.assertEqual(a['dedicated']['missing'],1)
        self.assertEqual(a['dedicated']['unknown_initial_loading'],1)
        self.assertFalse(all(v['passed'] for v in gpu_budget([loading]).values()))
        self.assertFalse(all(v['passed'] for v in gpu_budget([known,loading]).values()))
        for phase in ('idle','voices_1','voices_32','closed','finished'):
            missing=process_boundary(dict(phase=phase),True,True)
            self.assertFalse(all(v['passed'] for v in gpu_budget([missing,known]).values()))


if __name__=='__main__':unittest.main()
