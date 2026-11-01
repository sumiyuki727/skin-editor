import copy
import unittest
from unittest.mock import patch
from measure_independent_cpu import scheduler_intervals,scheduled_cost,calibration,product_cost,overlap_diagnostic,MAX_EVENTS
from measure_cpu_accounting import PHASES,reference_cost


def trace(events):
    return dict(schema=1,status=0,stop_status=0,trace_status=0,child_status=0,invalid=0,
                events_lost=0,buffers_lost=0,frequency=1_000_000_000,root_pid=7,buffer_count=1,
                buffers_allocated=32,thread_count=2,events=events,callback_wall_qpc=1000,callback_count=10)


class IndependentCpuTests(unittest.TestCase):
    def test_v2_reused_tid_has_independent_switch_pairs(self):
        t=trace([[10,0,7,1,1,1],[20,0,7,1,0,1],[30,0,7,1,1,2],[40,0,7,1,0,2]])
        t.update(schema=2,thread_lifetimes=[[7,1,1],[7,1,2]])
        self.assertEqual(scheduler_intervals(t),[(7,1,0,10,20),(7,1,0,30,40)])

    def test_v2_lifetime_cannot_close_another_lifetime_or_change_identity(self):
        for events in ([[10,0,7,1,1,1],[20,0,7,1,0,2]],[[10,0,7,1,1,1],[20,0,7,2,0,1]]):
            t=trace(events);t.update(schema=2,thread_lifetimes=[[7,1,1],[7,1,2]])
            with self.assertRaises(ValueError):scheduler_intervals(t)

    def test_v2_missing_or_duplicate_metadata_is_rejected(self):
        for metadata in ([],[[7,1,1],[7,2,1]],[[7,1,1],[7,2,2]]):
            t=trace([[10,0,7,1,1,3],[20,0,7,1,0,3]]);t.update(schema=2,thread_lifetimes=metadata)
            with self.assertRaises(ValueError):scheduler_intervals(t)

    def test_exact_switches_clip_to_requested_window(self):
        intervals=scheduler_intervals(trace([[10,0,7,1,1],[30,0,7,1,0],[40,0,7,1,1],[70,0,7,1,0]]))
        cost=scheduled_cost(intervals,7,20,60,10)
        self.assertEqual(cost["scheduled_seconds"],3)
        self.assertEqual(cost["cpu_one_core_percent"],75)

    def test_parallel_threads_sum_without_dividing_by_machine_core_count(self):
        intervals=scheduler_intervals(trace([[10,0,7,1,1],[10,1,7,2,1],[30,0,7,1,0],[30,1,7,2,0]]))
        self.assertEqual(scheduled_cost(intervals,7,10,30,1)["cpu_one_core_percent"],200)

    def test_pid_filter_does_not_add_another_owned_process(self):
        intervals=scheduler_intervals(trace([[10,0,8,1,1],[30,0,8,1,0]]))
        self.assertEqual(scheduled_cost(intervals,7,10,30,1)["cpu_one_core_percent"],0)

    def test_equal_timestamp_migration_is_order_independent_across_cpus(self):
        intervals=scheduler_intervals(trace([[10,0,7,1,1],[20,1,7,1,1],[20,0,7,1,0],[40,1,7,1,0]]))
        self.assertEqual(scheduled_cost(intervals,7,10,40,1)["cpu_one_core_percent"],100)

    def test_small_cross_cpu_switch_overlap_is_preserved_not_discounted(self):
        intervals=scheduler_intervals(trace([[10,0,7,1,1],[19,1,7,1,1],[20,0,7,1,0],[40,1,7,1,0]]))
        result=overlap_diagnostic(intervals,1_000_000_000)
        self.assertEqual(result["max_overlap_ns"],1)
        self.assertTrue(result["valid"])
        self.assertGreater(scheduled_cost(intervals,7,10,40,1)["cpu_one_core_percent"],100)
        self.assertFalse(overlap_diagnostic([(7,1,0,1,300000),(7,1,1,2,300000)],1_000_000_000)["valid"])

    def test_unmatched_or_unclosed_switches_fail_closed(self):
        for events in ([[[10,0,7,1,0]],[[10,0,7,1,1]],[[10,0,7,1,1],[20,0,7,2,1]]]):
            with self.subTest(events=events),self.assertRaises(ValueError):scheduler_intervals(trace(events))

    def test_switch_out_must_match_cpu_and_identity_without_duplicate_same_cpu_entry(self):
        for row in ([20,0,7,1,1],[20,1,7,1,0],[20,0,8,1,0]):
            with self.subTest(row=row),self.assertRaises(ValueError):scheduler_intervals(trace([[10,0,7,1,1],row]))

    def test_same_cpu_reversal_is_rejected_not_sorted_away(self):
        with self.assertRaises(ValueError):scheduler_intervals(trace([[20,0,7,1,1],[10,0,7,1,0]]))

    def test_malformed_or_unbounded_input_rejected(self):
        for row in (None,[True,0,7,1,1],[0,0,7,1,1],[10,64,7,1,1],[10,0,7,1,2],[10,0,7,1]):
            with self.subTest(row=row),self.assertRaises(ValueError):scheduler_intervals(trace([row]))
        with patch("measure_independent_cpu.MAX_EVENTS",1),self.assertRaises(ValueError):
            scheduler_intervals(trace([[10,0,7,1,1],[20,0,7,1,0]]))
        self.assertEqual(MAX_EVENTS,1_000_000)

    def test_all_loss_timeout_permission_and_capacity_errors_invalidate_meter(self):
        valid=trace([[10,0,7,1,1],[20,0,7,1,0]])
        for key,value in (("status",5),("stop_status",5),("trace_status",1),("child_status",1),("invalid",1),
                          ("events_lost",1),("buffers_lost",1),("buffers_allocated",129),("thread_count",513),("frequency",0)):
            bad=copy.deepcopy(valid);bad[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):scheduler_intervals(bad)

    def test_invalid_window_cannot_be_used_as_zero_cpu(self):
        for args in ((7,20,10,1),(0,10,20,1),(7,10,20,0),(7,True,20,1)):
            with self.assertRaises(ValueError):scheduled_cost([], *args)

    def reference(self):
        rows=[];events=[];at=1_000_000_000
        for name,millis,offset in PHASES:
            span=millis*1_000_000;idle=name.startswith("idle");dense=name=="dense_control"
            pulses=0 if idle else 1 if dense else millis//10
            work=0 if idle else span if dense else pulses*200_000
            row=dict(phase=name,requested_ms=millis,offset_ns=offset,span_ns=span,pulses=pulses,
                     busy_wall_ns=work,awake_wall_ns=work,missed=0,iterations=0 if idle else 123,
                     cycles=0 if idle else 456,thread_kernel_ticks=0,thread_user_ticks=0,
                     process_kernel_ticks=0,process_user_ticks=0,qpc_begin=at,qpc_end=at+span)
            row["metrics"]=reference_cost(row);rows.append(row)
            if work:events.extend([[at,0,7,1,1],[at+work,0,7,1,0]])
            at+=span
        return trace(events),dict(schema=2,pid=7,frequency=1_000_000_000,phase_plan=PHASES,rows=rows,timer_closed=True)

    def test_continuous_periodic_and_phase_reference_accept_independent_time_without_repairing_raw_zero(self):
        t,r=self.reference();value=calibration(t,r)
        self.assertTrue(value["passed"])
        pulse=next(v for v in value["costs"] if v["phase"]=="pulse_0")
        self.assertEqual(pulse["scheduled"]["cpu_one_core_percent"],2)
        self.assertEqual(pulse["raw"]["process_cpu_one_core_percent"],0)

    def test_aliased_periodic_measurement_cannot_pass_reference(self):
        t,r=self.reference();t["events"]=t["events"][2:]
        self.assertFalse(calibration(t,r)["passed"])

    def test_phase_identity_timer_and_clock_are_checked(self):
        t,r=self.reference();r["frequency"]=123
        with self.assertRaises(ValueError):calibration(t,r)
        t,r=self.reference();r["timer_closed"]=False
        self.assertFalse(calibration(t,r)["passed"])
        t,r=self.reference();r["phase_plan"]=[("other",1,0)]
        with self.assertRaises(ValueError):calibration(t,r)

    def test_missing_or_mismatched_pid_is_not_a_zero_cost_pass(self):
        t,r=self.reference();r["pid"]=8
        with self.assertRaises(ValueError):calibration(t,r)
        t=trace([[10,0,8,1,1],[30,0,8,1,0]])
        with self.assertRaises(ValueError):product_cost(t,dict(pid=7,frequency=t["frequency"]))

    def test_product_retains_original_failure_and_independent_15_percent_gate(self):
        t=trace([[10,0,7,1,1],[30,0,7,1,0]])
        r=dict(pid=7,frequency=t["frequency"],metrics={"voices_32":{}},latency=[],original_checks={"raw_cpu":False,"zero_missing":True},
               samples=[dict(phase="voices_32",qpc_end=q,private_bytes=100,working_set_bytes=100) for q in (10,110)])
        result=product_cost(t,r)
        self.assertFalse(result["checks"]["raw_cpu"])
        self.assertFalse(result["checks"]["independent_cpu_all_stable_phases_le_15_one_core"])
        self.assertEqual(result["costs"]["voices_32"]["cpu_one_core_percent"],20)
        self.assertEqual(r["original_checks"],{"raw_cpu":False,"zero_missing":True})


if __name__=="__main__":unittest.main()
