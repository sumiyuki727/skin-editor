import unittest
from measure_repaint import require_references,permission,repaint_costs
import test_check_main_work as main_tests

class RepaintTests(unittest.TestCase):
    def test_no_product_admission_for_missing_or_failed_strict_reference(self):
        for refs in ([],[dict(passed=True)],[dict(passed=False),dict(passed=True)]):
            with self.assertRaises(ValueError):require_references(refs)
        require_references([dict(passed=True)]*2)

    def fixture(self):
        t,r,b=main_tests.MainWorkTests().fixture();e=r['report']['main_work']['rows'][0][0]
        r['report']['repaints']=dict(schema=1,invalid=0,overflow=0,pending=0,
            requests=[[e+15,3,0],[e+20,3,16_000_000],[e+200,4,0]],
            sources=[dict(file_utf8=list(b'host/src/main.rs'),line=151)],
            passes=[dict(at=e+30,frame=3,causes=[0,0],**{'pass':4})],
            inputs=[[e+15,3,1,2,0,0,0,0,1],[e+40,4,0,0,0,0,0,0,1]])
        return t,r,b

    def test_requests_causes_and_input_counters_stay_distinct_and_in_original_window(self):
        t,r,b=self.fixture();v=repaint_costs(t,r,b)
        self.assertEqual(v['requests'],2);self.assertEqual(v['immediate'],1)
        self.assertEqual(v['previous_pass_cause_counts'][0]['count'],2)
        self.assertEqual(v['pointer_input_calls'],1);self.assertEqual(v['empty_input_calls'],1)
        self.assertEqual(v['input_events']['mouse_moved'],2)

    def test_missing_overflow_or_pending_cannot_be_called_complete(self):
        for key in ('invalid','overflow','pending'):
            t,r,b=self.fixture();r['report']['repaints'][key]=1
            with self.assertRaises(ValueError):repaint_costs(t,r,b)

    def test_duplicate_pass_and_unknown_source_are_rejected(self):
        t,r,b=self.fixture();r['report']['repaints']['passes']*=2
        with self.assertRaises(ValueError):repaint_costs(t,r,b)
        t,r,b=self.fixture();r['report']['repaints']['passes'][0]['causes']=[1]
        with self.assertRaises(ValueError):repaint_costs(t,r,b)

    def test_private_absolute_source_path_is_rejected(self):
        t,r,b=self.fixture();r['report']['repaints']['sources'][0]['file_utf8']=list(b'C:/private/src/main.rs')
        with self.assertRaises(ValueError):repaint_costs(t,r,b)

    def test_permission_rejects_elevated_unowned_or_wrong_session(self):
        import os
        token=dict(elevated=0,type=3);t=dict(observed_runner=True,root_pid=os.getpid(),runner_created=42,child_elevated=0,child_token_type=3)
        reports=[dict(pid=i,runner_pid=i,owner=dict(pid=i,token=token),token=token) for i in (1,2)]
        self.assertTrue(all(permission(t,token,42,reports,2).values()))
        self.assertFalse(all(permission(t,token,43,reports,2).values()))
        reports[0]['owner']['token']=dict(elevated=1,type=2)
        self.assertFalse(all(permission(t,token,42,reports,2).values()))

if __name__=='__main__':unittest.main()
