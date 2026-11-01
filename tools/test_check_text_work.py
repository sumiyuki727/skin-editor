import copy
import unittest
from measure_text_work import census

def fixture():
    rows=[[i,4,10,8,9,100,150,20,1024,128,10000,0,0,0,30,1000] for i in range(128)]
    return dict(schema=1,invalid=0,weak_identities=9,rows=rows),dict(schema=1,invalid=0,weak_identities=0,rows=copy.deepcopy(rows))

class TextWorkTests(unittest.TestCase):
    def test_counts_are_weak_identity_reuse_not_cache_miss_or_cpu_estimates(self):
        a,b=fixture();v=census(a,b)
        self.assertEqual(v['voices_32']['reused_percent'],80)
        self.assertEqual(v['voices_32']['row_mesh_vertices'],12800)
        self.assertEqual(v['weak_identities_final'],0)
        self.assertIn('not cache hit count',v['scope'])
    def test_missing_cleanup_or_census_overflow_or_mismatched_cleanup_is_rejected(self):
        for change in [lambda a,b:a.update(invalid=1),lambda a,b:b.update(weak_identities=1),
            lambda a,b:b.update(schema=0),lambda a,b:b.update(rows=[]),lambda a,b:a.update(rows=None)]:
            a,b=fixture();change(a,b)
            with self.assertRaises(ValueError):census(a,b)
    def test_incomplete_stable_phase_and_reversed_pass_or_counts_are_rejected(self):
        for index,value in [(0,-1),(1,18),(2,513),(3,11),(4,11),(8,0),(10,1000001)]:
            a,b=fixture();a['rows'][2][index]=value;b['rows']=copy.deepcopy(a['rows'])
            with self.assertRaises(ValueError):census(a,b)
        a,b=fixture()
        for row in a['rows']:row[1]=0
        b['rows']=copy.deepcopy(a['rows'])
        with self.assertRaises(ValueError):census(a,b)

if __name__=='__main__':unittest.main()
