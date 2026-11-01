import unittest
from measure_rebuild_attribution import STAGES, PLAN, validate_checkpoint, region_delta, region_summary, timeline_interval


class RebuildAttribution(unittest.TestCase):
    def test_fixed_plan_cannot_select_just_low_result(self):
        self.assertEqual(PLAN,('default','no_implicit','no_implicit','default'))
        self.assertEqual(len(STAGES),26)

    def test_checkpoint_rejects_late_duplicate_and_wrong_stage(self):
        row=dict(checkpoint=1,stage=STAGES[0],allocator=None)
        validate_checkpoint(row,0)
        for value,count in [(row,1),(dict(row,checkpoint=2),0),(dict(row,stage='device'),0),(dict(row,checkpoint=True),0)]:
            with self.assertRaises(ValueError):validate_checkpoint(value,count)

    def test_unavailable_allocator_does_not_become_zero(self):
        row=dict(checkpoint=3,stage='device',allocator={'supported':False})
        validate_checkpoint(row,2)
        for value in ({},{'supported':True,'allocated_bytes':None,'reserved_bytes':0},{'supported':True,'allocated_bytes':0,'reserved_bytes':-1}):
            with self.assertRaises(ValueError):validate_checkpoint(dict(row,allocator=value),2)

    def test_retention_requires_exact_region_identity_not_just_same_address(self):
        a=dict(base=100,allocation=100,bytes=64,protect=0x404)
        b=dict(base=200,allocation=200,bytes=32,protect=4)
        after=[a,dict(b,bytes=16)]
        self.assertEqual(region_delta([a,b],after),dict(retained_bytes=64,new_or_changed_bytes=16,gone_or_changed_bytes=32))
        self.assertEqual(region_summary(after),{'0x404':64,'0x4':16})

    def test_timeline_phase_change_during_query_keeps_both_owners_and_clock_bracket(self):
        rows=[]; phases=iter(('between:device->model','held:model')); times=iter((10.,10.002))
        timeline_interval(rows,lambda:next(phases),lambda _:dict(private_bytes=200,working_set_bytes=100),lambda:next(times))
        self.assertEqual((rows[0]['start'],rows[0]['end']),(10.,10.002))
        self.assertNotEqual(rows[0]['before'],rows[0]['after'])

    def test_timeline_unknown_memory_reverse_clock_and_overflow_cannot_be_accepted(self):
        for metric,times in [(dict(private_bytes=None,working_set_bytes=1),(1.,2.)),(dict(private_bytes=1,working_set_bytes=1),(2.,1.))]:
            values=iter(times)
            with self.assertRaises(ValueError):timeline_interval([],lambda:'held:device',lambda _:metric,lambda:next(values))
        with self.assertRaises(ValueError):timeline_interval([None]*65536,lambda:'unused',lambda _:None,lambda:0)


if __name__=='__main__':unittest.main()
