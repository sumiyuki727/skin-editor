import copy
import struct
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from measure_cpu_threads import PhaseIdentities,permission_checks,pe_entry_rva,thread_costs,PLAN,ELEVATED_PLAN
from measure_independent_cpu import BINARY
from test_check_independent_cpu import trace


class CpuThreadChecks(unittest.TestCase):
    def test_fixed_control_brackets_token_comparison_with_limited_plain_observers(self):
        self.assertEqual(PLAN,((True,False),(True,True),(False,True),(False,True),(True,True),(True,False)))
        self.assertEqual(ELEVATED_PLAN,((False,False),(False,True),(False,True),(False,False)))
        self.assertTrue(all(not limited for limited,_ in ELEVATED_PLAN))

    def test_permission_is_verified_by_native_runner_and_product_not_a_requested_label(self):
        for limited,elevated,kind in ((True,0,3),(False,1,2)):
            t=dict(limited_requested=limited,child_elevated=elevated,child_token_type=kind)
            r=dict(token=dict(elevated=elevated,type=kind),metrics={},product_token=dict(elevated=elevated,type=kind))
            self.assertTrue(all(permission_checks(t,r,limited).values()))
            r["product_token"]["elevated"]=1-elevated
            self.assertFalse(all(permission_checks(t,r,limited).values()))
            r.pop("product_token")
            self.assertFalse(all(permission_checks(t,r,limited).values()))

    def test_permission_mismatch_cannot_fallback_to_elevated_as_normal(self):
        t=dict(limited_requested=True,child_elevated=1,child_token_type=2)
        r=dict(token=dict(elevated=1,type=2))
        self.assertFalse(all(permission_checks(t,r,True).values()))
        self.assertFalse(all(permission_checks({},r,False).values()))

    def test_expensive_identity_query_only_on_phase_boundaries(self):
        gate=PhaseIdentities();calls=[];clocks=iter(range(100))
        def identify(p):calls.append(p);return dict(threads=[])
        for phase in ("setup","setup","voices_32","voices_32","closed"):
            self.assertEqual(gate.sample(phase,identify,lambda p:p,lambda:next(clocks)),phase)
        self.assertEqual(calls,["setup","voices_32","closed"])
        self.assertEqual(len(gate.observations),3)

    def test_identity_phase_admission_is_bounded_before_query(self):
        gate=PhaseIdentities();calls=[]
        for i in range(24):gate.sample(str(i),lambda p:calls.append(p) or dict(threads=[]),lambda p:p,lambda:0)
        with self.assertRaises(RuntimeError):gate.sample("overflow",lambda p:calls.append(p),lambda p:p,lambda:0)
        self.assertEqual(len(calls),24)

    def test_identity_failure_does_not_cache_an_incomplete_observation(self):
        gate=PhaseIdentities()
        def fail(p):raise OSError("unavailable")
        with self.assertRaises(OSError):gate.sample("phase",fail,lambda p:p,lambda:0)
        self.assertEqual(gate.observations,[]);self.assertIsNone(gate.phase)

    def report(self):
        return dict(pid=7,frequency=1_000_000_000,metrics={"voices_32":{}},latency=[],original_checks={"raw_cpu":False},
                    samples=[dict(phase="voices_32",qpc_end=q,private_bytes=100,working_set_bytes=100) for q in (10,110)],
                    identities=[dict(phase="voices_32",begin=1,end=2,threads=[dict(key="1:42",created=42,name="",origin=dict(status="mapped-start-only",module=BINARY.name.lower(),offset=123))])],
                    pe_entry_rva=123,observer_close=dict(closed=True,after=0),
                    report=dict(adapter_utf8=list(b'AdapterInfo { backend: Vulkan }'),ui_cost=[[0,0,0]]*18))

    def test_thread_sum_keeps_unknown_threads_and_does_not_repair_original_failure(self):
        t=trace([[10,0,7,1,1],[20,1,7,2,1],[30,0,7,1,0],[40,1,7,2,0]])
        result=thread_costs(t,self.report())
        self.assertEqual(sum(v["cpu_one_core_percent"] for v in result["threads"]),40)
        self.assertTrue(result["threads"][0]["process_entry_start"])
        self.assertEqual(result["threads"][1]["origin"]["status"],"undiscovered")
        self.assertFalse(result["checks"]["raw_cpu"])
        self.assertFalse(result["checks"]["independent_cpu_all_stable_phases_le_15_one_core"])

    def test_v2_reused_tid_cost_is_preserved_without_inheriting_a_name_or_entry(self):
        t=trace([[10,0,7,1,1,1],[20,0,7,1,0,1],[30,0,7,1,1,2],[40,0,7,1,0,2]])
        t.update(schema=2,thread_lifetimes=[[7,1,1],[7,1,2]])
        result=thread_costs(t,self.report())
        self.assertEqual(result['threads'][0]['cpu_one_core_percent'],20)
        self.assertEqual(result['threads'][0]['lifetime_count'],2)
        self.assertEqual(result['threads'][0]['name'],'');self.assertFalse(result['threads'][0]['process_entry_start'])

    def test_module_start_is_not_entry_when_rva_differs(self):
        r=self.report();r["pe_entry_rva"]=456
        result=thread_costs(trace([[10,0,7,1,1],[20,0,7,1,0]]),r)
        self.assertFalse(result["threads"][0]["process_entry_start"])

    def test_owned_baseline_filename_does_not_lose_entry_identity(self):
        r=self.report();r['product_name']='baseline.exe';r['identities'][0]['threads'][0]['origin']['module']='baseline.exe'
        result=thread_costs(trace([[10,0,7,1,1],[20,0,7,1,0]]),r)
        self.assertTrue(result['threads'][0]['process_entry_start'])

    def test_phase_module_identity_is_not_overwritten_by_later_unload(self):
        r=self.report();other=copy.deepcopy(r["identities"][0]);other["phase"]="closed"
        other["threads"][0]["origin"]=dict(status="unmapped",module=None,offset=None);r["identities"].append(other)
        result=thread_costs(trace([[10,0,7,1,1],[20,0,7,1,0]]),r)
        self.assertTrue(result["threads"][0]["process_entry_start"])
        self.assertEqual(result["threads"][0]["identity_phase"],"voices_32")

    def test_reused_tid_with_another_creation_identity_rejects_join(self):
        r=self.report();other=copy.deepcopy(r["identities"][0]);other["threads"][0]["key"]="1:43";r["identities"].append(other)
        with self.assertRaises(ValueError):thread_costs(trace([[10,0,7,1,1],[20,0,7,1,0]]),r)

    def test_open_observer_handles_fail_cleanup_gate(self):
        r=self.report();r["observer_close"]["after"]=1
        self.assertFalse(thread_costs(trace([[10,0,7,1,1],[20,0,7,1,0]]),r)["checks"]["all_identity_handles_closed"])

    def test_pe_entry_uses_bounded_product_header_and_rejects_other_formats(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/"self.exe";data=bytearray(200);data[:2]=b"MZ";struct.pack_into("<I",data,60,80)
            data[80:84]=b"PE\0\0";struct.pack_into("<H",data,104,0x20b);struct.pack_into("<I",data,120,123)
            p.write_bytes(data);self.assertEqual(pe_entry_rva(p),123)
            struct.pack_into("<I",data,60,2**31);p.write_bytes(data)
            with self.assertRaises(ValueError):pe_entry_rva(p)
            p.write_bytes(b"no header")
            with self.assertRaises(ValueError):pe_entry_rva(p)


if __name__=="__main__":unittest.main()
