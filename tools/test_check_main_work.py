import copy
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch,Mock
from measure_main_work import (PLAN, ordinary, permission_checks, clock_mapping, main_costs, launch_owned, MAX_ROWS,
                               reference_workload_valid,evaluate_reports,row_limit)

class MainWorkTests(unittest.TestCase):
    def test_fixed_plan_brackets_timing_instrumentation(self):
        self.assertEqual(PLAN,(False,True,True,False))

    def test_missed_reference_pulse_blocks_later_admission(self):
        r=dict(pid=1,phase_plan=[],rows=[dict(phase="pulse_long",missed=1,busy_wall_ns=20,span_ns=1000)])
        with patch('measure_main_work.reference_checks',return_value=dict(valid=True)):
            self.assertFalse(reference_workload_valid(r));r["rows"][0]["missed"]=0
            self.assertTrue(reference_workload_valid(r))

    def test_failed_calibration_retains_all_diagnostic_rows_and_cannot_pass(self):
        with patch('measure_main_work.calibration',side_effect=[dict(passed=False),dict(passed=True)]),patch(
                'measure_main_work.thread_costs',side_effect=lambda t,r:dict(checks=dict(raw_cpu=True))),patch(
                'measure_main_work.main_costs',return_value=dict(total=20)):
            result=evaluate_reports({},[dict(pid=1),dict(pid=2)],[{}]*4)
        self.assertEqual(len(result["products"]),4);self.assertFalse(result["passed"])
        self.assertTrue(all(v["diagnostic_only"] and not v["checks"]["reference_calibration_passed"] for v in result["products"]))

    def test_incomplete_or_extra_report_plan_is_rejected(self):
        for refs,products in (([{}], [{}]*4),([{}]*2,[{}]*3),([{}]*2,[{}]*5)):
            with self.assertRaises(ValueError):evaluate_reports({},refs,products)

    def test_failed_attribution_does_not_drop_remaining_captured_rows(self):
        with patch('measure_main_work.calibration',return_value=dict(passed=True)),patch(
                'measure_main_work.thread_costs',side_effect=lambda t,r:dict(checks=dict(raw_cpu=True))),patch(
                'measure_main_work.main_costs',side_effect=ValueError('invalid clock')):
            result=evaluate_reports({},[dict(pid=1),dict(pid=2)],[{}]*4)
        self.assertEqual(len(result["products"]),4);self.assertFalse(result["passed"])
        self.assertTrue(result["calibrated"]);self.assertIn('attribution_error',result['products'][2])

    def test_both_real_ordinary_token_types_are_accepted_not_elevated(self):
        for t in (1,3):self.assertTrue(ordinary(dict(elevated=0,type=t)))
        for value in (None,{},dict(elevated=1,type=2),dict(elevated=0,type=2)):self.assertFalse(ordinary(value))

    def permission_fixture(self):
        trace=dict(observed_runner=True,launch_api="ordinary-runner-owned-children",root_pid=os.getpid(),runner_created=42,child_elevated=0,child_token_type=3)
        token=dict(elevated=0,type=3)
        reports=[dict(pid=i+20,token=token,runner_pid=i,owner=dict(pid=i,token=token),product_token=token) for i in range(1,7)]
        return trace,token,reports

    def test_permission_requires_native_and_parent_and_each_actual_target(self):
        t,token,r=self.permission_fixture();self.assertTrue(all(permission_checks(t,token,r,42).values()))
        for field,value in (("observed_runner",False),("root_pid",0),("runner_created",43),("child_elevated",1)):
            bad=dict(t);bad[field]=value;self.assertFalse(all(permission_checks(bad,token,r,42).values()))
        r[0]["owner"]["token"]=dict(elevated=1,type=2)
        self.assertFalse(all(permission_checks(t,token,r,42).values()))

    def test_missing_or_pid_mismatched_target_is_not_permission_success(self):
        t,token,r=self.permission_fixture();self.assertFalse(all(permission_checks(t,token,r[:-1],42).values()))
        r[1]["runner_pid"]=999;self.assertFalse(all(permission_checks(t,token,r,42).values()))

    def test_reused_process_pid_is_not_silently_merged(self):
        t,token,r=self.permission_fixture();r[1]["pid"]=r[0]["pid"]
        self.assertFalse(all(permission_checks(t,token,r,42).values()))

    def test_same_tid_in_different_owned_processes_has_separate_intervals(self):
        from test_check_independent_cpu import trace
        from measure_independent_cpu import scheduler_intervals
        t=trace([[10,0,7,9,1],[20,0,7,9,0],[30,0,8,9,1],[40,0,8,9,0]])
        self.assertEqual(scheduler_intervals(t),[(7,9,0,10,20),(8,9,0,30,40)])

    def fixture(self):
        e=1_700_000_000_000_000_000
        report=dict(pid=7,clock_pairs=[[1,1,e+1],[121,121,e+121]],report=dict(main_work=dict(schema=1,invalid=0,
            rows=[[e,0,0],[e+20,20,1],[e+40,40,2],[e+60,60,3],[e+80,80,4],[e+120,120,0]])))
        base=dict(threads=[dict(tid=9,process_entry_start=True,cpu_one_core_percent=20)],costs=dict(voices_32=dict(qpc_begin=10,qpc_end=110)))
        return dict(frequency=1_000_000_000),report,base

    def costs(self,t,r,b):
        with patch('measure_main_work.scheduler_intervals',return_value=[(7,9,0,10,30)]):return main_costs(t,r,b)

    def test_exact_partition_uses_cpu_intersections_not_wall_subtraction(self):
        t,r,b=self.fixture();v=self.costs(t,r,b)
        self.assertEqual(v["cpu_one_core_percent"]["application_logic"],10)
        self.assertEqual(v["cpu_one_core_percent"]["framework_ui_and_tessellation"],10)
        self.assertEqual(v["total"],20)

    def detailed_fixture(self):
        t,r,b=self.fixture();e=r["report"]["main_work"]["rows"][0][0]
        # One UI entry; widget, host dispatch and preview request are disjoint.
        kinds=(0,1,6,1,2,5,8,7,8,3,4,0,0)
        r["report"]["main_work"]=dict(schema=2,invalid=0,rows=[[e+i*10,i*10,k] for i,k in enumerate(kinds)])
        return t,r,b

    def test_detailed_cpu_partition_rolls_up_without_double_counting(self):
        t,r,b=self.detailed_fixture();b['threads'][0]['cpu_one_core_percent']=100
        with patch('measure_main_work.scheduler_intervals',return_value=[(7,9,0,10,110)]):v=main_costs(t,r,b)
        self.assertEqual(v['cpu_one_core_percent']['editor_widgets'],10)
        self.assertEqual(v['cpu_one_core_percent']['graphics_poll'],10)
        self.assertEqual(v['rollup_cpu_one_core_percent']['application_logic'],30)
        self.assertEqual(v['rollup_cpu_one_core_percent']['application_ui'],50)
        self.assertEqual(sum(v['rollup_cpu_one_core_percent'].values()),100)
        self.assertEqual(v['boundary_counts']['application_ui'],1)
        self.assertEqual(v['boundary_counts']['host_ui_dispatch'],2)

    def test_poll_wait_is_not_counted_as_cpu_and_parent_windows_stay_intact(self):
        t,r,b=self.detailed_fixture()
        with patch('measure_main_work.scheduler_intervals',return_value=[(7,9,0,10,20),(7,9,0,50,60)]):v=main_costs(t,r,b)
        self.assertEqual(v['cpu_one_core_percent']['graphics_poll'],0)
        self.assertEqual(v['cpu_one_core_percent']['editor_widgets'],10)
        self.assertEqual(v['total'],20)

    def test_schema_extension_does_not_reinterpret_legacy_records(self):
        t,r,b=self.fixture();v=self.costs(t,r,b)
        self.assertEqual(v['rollup_cpu_one_core_percent'],v['cpu_one_core_percent'])
        t,r,b=self.detailed_fixture();r['report']['main_work']['schema']=1
        with self.assertRaises(ValueError):self.costs(t,r,b)
        for schema in (0,5):
            r['report']['main_work']['schema']=schema
            with self.assertRaises(ValueError):self.costs(t,r,b)

    def test_panel_boundaries_roll_up_to_ui_with_no_new_cpu_allowance(self):
        t,r,b=self.detailed_fixture();e=r['report']['main_work']['rows'][0][0]
        kinds=(0,1,6,2,5,15,9,10,11,12,13,14,8,7,8,4,0)
        r['report']['main_work']=dict(schema=3,invalid=0,rows=[[e+i*10,i*10,k] for i,k in enumerate(kinds)])
        b['costs']['voices_32']['qpc_end']=160;b['threads'][0]['cpu_one_core_percent']=100
        with patch('measure_main_work.scheduler_intervals',return_value=[(7,9,0,10,160)]):v=main_costs(t,r,b)
        self.assertAlmostEqual(v['total'],100)
        self.assertAlmostEqual(v['rollup_cpu_one_core_percent']['application_ui'],80)
        self.assertAlmostEqual(sum(v['rollup_cpu_one_core_percent'].values()),100)
        self.assertEqual(v['boundary_counts']['application_ui'],1)
        r['report']['main_work']['schema']=2
        with self.assertRaises(ValueError):self.costs(t,r,b)

    def test_boundary_unknown_or_capacity_or_clock_reversal_fails(self):
        for change in (lambda w:w.update(invalid=1),lambda w:w.update(rows=[]),
                       lambda w:w["rows"].append([1,1,8]),lambda w:w.update(rows=[[1,1,0]]*(MAX_ROWS+1))):
            t,r,b=self.fixture();change(r["report"]["main_work"])
            with self.assertRaises(ValueError):self.costs(t,r,b)
    def test_text_detail_schema_has_explicit_capacity_and_conserves_ui_cpu(self):
        self.assertEqual(row_limit(3),16384);self.assertEqual(row_limit(4),32768)
        t,r,b=self.fixture();e=r['report']['main_work']['rows'][0][0]
        kinds=(0,1,2,5,9,16,17,13,18,19,20,21,22,23,24,14,8,4,0)
        r['report']['main_work']=dict(schema=4,invalid=0,rows=[[e+i*5,i*5,k] for i,k in enumerate(kinds)])
        b['costs']['voices_32'].update(qpc_begin=10,qpc_end=85);b['threads'][0]['cpu_one_core_percent']=100
        with patch('measure_main_work.scheduler_intervals',return_value=[(7,9,0,10,85)]):v=main_costs(t,r,b)
        self.assertAlmostEqual(sum(v['rollup_cpu_one_core_percent'].values()),100)
        self.assertAlmostEqual(v['cpu_one_core_percent']['audio_readiness'],100/15)
        r['report']['main_work']['schema']=3
        with self.assertRaises(ValueError):main_costs(t,r,b)

    def test_clock_pair_delay_and_time_adjustment_are_not_cpu_corrections(self):
        for pairs in ([[1,60001,1000000],[100001,100001,1100000]],[[1,1,1000000],[100001,100001,1200000]]):
            with self.assertRaises(ValueError):clock_mapping(pairs,1_000_000_000)

    def test_large_epoch_mapping_keeps_integer_precision(self):
        t,r,b=self.fixture();offset,clock=clock_mapping(r["clock_pairs"],t["frequency"])
        self.assertEqual((r["report"]["main_work"]["rows"][0][0]*t["frequency"]-offset)//1_000_000_000,0)

    def test_all_clock_brackets_constrain_offset_including_preempted_pair(self):
        offset,clock=clock_mapping([[10,12,1011],[1,40001,1100]],1_000_000_000)
        self.assertLess(clock["feasible_offset_width_ns"],1000)
        self.assertEqual(clock["max_pair_ns"],40000)
        with self.assertRaises(ValueError):clock_mapping([[10,10,1010],[20,20,2020]],1_000_000_000)

    def test_monotonic_disagreement_rejects_system_clock_fallback(self):
        t,r,b=self.fixture();r["report"]["main_work"]["rows"][-1][1]+=100000
        with self.assertRaises(ValueError):self.costs(t,r,b)

    def test_incomplete_window_and_ambiguous_main_cannot_be_attributed(self):
        t,r,b=self.fixture();b["costs"]["voices_32"]["qpc_end"]=130
        with self.assertRaises(ValueError):self.costs(t,r,b)
        t,r,b=self.fixture();b["threads"].append(copy.deepcopy(b["threads"][0]))
        with self.assertRaises(ValueError):self.costs(t,r,b)

    def test_partition_mismatch_does_not_hide_unaccounted_main_cost(self):
        t,r,b=self.fixture();b["threads"][0]["cpu_one_core_percent"]=30
        with self.assertRaises(ValueError):self.costs(t,r,b)

    def test_failed_admission_does_not_release_target_and_kills_owned_child(self):
        with tempfile.TemporaryDirectory() as d:
            child=Mock(pid=7,_handle=123);child.poll.return_value=None
            k=Mock();k.AssignProcessToJobObject.return_value=False
            with patch('measure_main_work.subprocess.Popen',return_value=child),patch('measure_main_work.token_info',return_value=dict(elevated=0,type=1)),patch('measure_main_work.c.get_last_error',return_value=5,create=True):
                with self.assertRaises(OSError):launch_owned(k,1,2,[],Path(d))
            k.SetEvent.assert_not_called();child.kill.assert_called_once();child.wait.assert_called_once_with(timeout=5)

    def test_timeout_cleans_owned_target_without_releasing_another(self):
        with tempfile.TemporaryDirectory() as d:
            import subprocess
            child=Mock(pid=7,_handle=123);child.poll.return_value=None
            child.wait.side_effect=[subprocess.TimeoutExpired('owned',55),0]
            k=Mock()
            with patch('measure_main_work.subprocess.Popen',return_value=child),patch('measure_main_work.token_info',return_value=dict(elevated=0,type=3)):
                with self.assertRaises(subprocess.TimeoutExpired):launch_owned(k,1,2,[],Path(d))
            k.SetEvent.assert_called_once_with(1);child.kill.assert_called_once()

if __name__=='__main__':unittest.main()
