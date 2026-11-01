import copy
import unittest
from measure_mesh_replay import ORDINALS,PLAN,replay

def fixture():
    samples=[];rows=[]
    for n,ordinal in enumerate(ORDINALS):
        samples.append([100+n,ordinal,4,500,500,1000000,0x3f800000,1024,32,4,1])
        for repeat in range(4):
            for position,kind in enumerate(PLAN):
                v,i,p,cb=[(12,18,2,1),(4,6,1,0),(8,12,2,1),(0,0,0,0)][kind]
                rows.append([100+n,n,repeat,position,kind,*([100]*6),v,i,p,cb,v*20+i*4,100])
    data=dict(schema=1,vertex_stride=20,invalid=0,scratch_active=False,error=None,samples=samples,rows=rows)
    return data,copy.deepcopy(data)

class MeshReplayTests(unittest.TestCase):
    def test_fixed_fresh_samples_balanced_cohorts_and_separate_raw_timings(self):
        a,b=fixture();v=replay(a,b)
        self.assertEqual(ORDINALS,(1,33,65,97));self.assertEqual(PLAN,(0,1,2,3,2,1,0))
        self.assertEqual(v['cohorts']['text']['count'],32)
        self.assertEqual(v['cohorts']['empty']['count'],16)
        self.assertIn('not GPU upload',v['scope'])
        self.assertIn('pure_tessellation',v['cohorts']['all']['timing_ms'])
        self.assertNotIn('corrected_cpu',v)
    def test_reordered_or_incomplete_fresh_output_plan_cannot_pass(self):
        for change in [lambda a:a['rows'].pop(),lambda a:a['rows'][0].__setitem__(4,1),
            lambda a:a['samples'][1].__setitem__(1,34),lambda a:a['rows'][28].__setitem__(0,99)]:
            a,b=fixture();change(a);b=copy.deepcopy(a)
            with self.assertRaises(ValueError):replay(a,b)
    def test_failed_exact_checks_or_uncleared_scratch_or_mismatched_shutdown_rejected(self):
        for change in [lambda a:a.update(invalid=1),lambda a:a.update(scratch_active=True),lambda a:a.update(error='mismatch')]:
            a,b=fixture();change(a);b=copy.deepcopy(a)
            with self.assertRaises(ValueError):replay(a,b)
        a,b=fixture();b['rows']=[]
        with self.assertRaises(ValueError):replay(a,b)
    def test_mesh_and_atlas_bounds_empty_and_copy_sizes_checked(self):
        for change in [lambda a:a['rows'][0].__setitem__(11,32769),lambda a:a['rows'][0].__setitem__(15,1),
            lambda a:a['samples'][0].__setitem__(8,8192),lambda a:a['samples'][0].__setitem__(6,0x7f800000),
            lambda a:a['rows'][3].__setitem__(14,1),lambda a:a['rows'][1].__setitem__(14,1)]:
            a,b=fixture();change(a);b=copy.deepcopy(a)
            with self.assertRaises(ValueError):replay(a,b)
    def test_changed_bracket_or_invalid_time_partition_is_not_normalized(self):
        for change in [lambda a:a['rows'][6].__setitem__(13,3),lambda a:a['samples'][0].__setitem__(5,1)]:
            a,b=fixture();change(a);b=copy.deepcopy(a)
            with self.assertRaises(ValueError):replay(a,b)

if __name__=='__main__':unittest.main()
