import copy
import tempfile
from pathlib import Path
import unittest
from measure_rebuild_policy import PLAN, compare_pools, pool_analysis, pixels
from measure_rebuild_attribution import STAGES


def fixture(enabled):
    growth=4 if enabled else 64
    def allocator(reserved, live=False):
        return dict(supported=True,reserved_bytes=reserved,allocated_bytes=100+(16 if live else 0),
                    allocations=[dict(name_utf8=list(b'own frame readback'))] if live else [])
    rows=[dict(stage=p,allocator=allocator(192),metric=dict(private_bytes=200)) for p in STAGES]
    for b,d,a in ((5,6,9),(16,17,20)):
        rows[d]=dict(rows[d],allocator=allocator(192+growth,True),metric=dict(private_bytes=200+growth))
        rows[a]=dict(rows[a],allocator=allocator(192+growth))
    return dict(error=None,checkpoints=rows,report=dict(schema=5,gpu_memory_usage_hint=enabled,
        device_memory_usage_hints=[enabled,enabled],full_editor=False,pixels_exact=True,
        raw_final=0,rgba_final=0,final_targets=0,worker_closed=True,shutdown_ok=True))


class RebuildPolicy(unittest.TestCase):
    def test_both_incarnations_are_compared_and_all_fixed_controls_required(self):
        rows=[dict(memory_usage=e,pools=pool_analysis(fixture(e),e)) for e in PLAN]
        self.assertTrue(all(compare_pools(rows).values()))
        with self.assertRaises(ValueError):compare_pools(rows[:3])
        rows[1]['pools'][1]['reserve_growth']=64
        self.assertFalse(compare_pools(rows)['smaller_first_and_rebuilt_pool'])

    def test_unavailable_pool_wrong_policy_and_unreleased_owner_are_rejected(self):
        f=fixture(True)
        for change in ('unsupported','wrong_policy','unreleased','old_schema'):
            r=copy.deepcopy(f)
            if change=='unsupported':r['checkpoints'][6]['allocator']=None
            if change=='wrong_policy':r['report']['device_memory_usage_hints']=[True,False]
            if change=='unreleased':r['report']['raw_final']=1
            if change=='old_schema':r['report']['schema']=4
            with self.assertRaises(ValueError):pool_analysis(r,True)

    def test_empty_pool_retention_does_not_mean_readback_object_is_alive(self):
        r=fixture(False);result=pool_analysis(r,False)
        self.assertTrue(result[0]['empty_capacity_retained']);self.assertTrue(result[0]['readback_released'])
        r['checkpoints'][9]['allocator']['allocations']=[dict(name_utf8=list(b'own frame readback'))]
        self.assertFalse(pool_analysis(r,False)[0]['readback_released'])

    def test_pixels_must_be_complete_equal_and_sha_is_content_based(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            for name in ('first.rgba','rebuilt.rgba'):(p/name).write_bytes(bytes(16384))
            self.assertEqual(len(pixels(p)),64)
            (p/'rebuilt.rgba').write_bytes(bytes(16383)+b'1')
            with self.assertRaises(ValueError):pixels(p)
            (p/'first.rgba').write_bytes(b'')
            with self.assertRaises(ValueError):pixels(p)


if __name__=='__main__':unittest.main()
