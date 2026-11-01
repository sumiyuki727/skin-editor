import unittest
from types import SimpleNamespace
from unittest.mock import patch
from measure_cpu_accounting import next_deadline, reference_cost, sampled_tick_model, reference_checks, NativeReference, run_phase, PHASES, MAX_PULSES


class ReferenceChecks(unittest.TestCase):
    def test_periodic_work_skips_missed_pulses_without_catchup(self):
        self.assertEqual(next_deadline(0,5,10),(10,0))
        self.assertEqual(next_deadline(0,10,10),(20,1))
        self.assertEqual(next_deadline(0,35,10),(40,3))
        with self.assertRaises(ValueError):next_deadline(0,1,0)

    def row(self):
        return dict(span_ns=2_000_000_000,busy_wall_ns=40_000_000,awake_wall_ns=41_000_000,pulses=200,missed=0,
                    cycles=12345,iterations=1000,thread_kernel_ticks=0,thread_user_ticks=1_250_000,process_kernel_ticks=0,process_user_ticks=1_250_000)

    def test_awake_wall_cycles_and_charged_cpu_are_separate_not_discounted(self):
        r=reference_cost(self.row())
        self.assertEqual(r["thread_cpu_one_core_percent"],6.25)
        self.assertEqual(r["busy_wall_percent"],2.)
        self.assertAlmostEqual(r["own_awake_wall_percent"],2.05)
        self.assertEqual(r["cycles"],12345)

    def test_invalid_layout_negative_or_unbounded_reference_rejects(self):
        for key,value in (("span_ns",0),("busy_wall_ns",-1),("cycles",-1),("pulses",MAX_PULSES+1),
                          ("awake_wall_ns",1),("awake_wall_ns",3_000_000_000),("thread_user_ticks",True)):
            r=self.row();r[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):reference_cost(r)

    def test_timer_close_is_idempotent_and_closes_even_if_cancel_fails(self):
        closed=[]
        api=NativeReference.__new__(NativeReference);api.timer=42
        api.k=SimpleNamespace(CancelWaitableTimer=lambda h:False,CloseHandle=lambda h:closed.append(h) or True)
        with patch("measure_cpu_accounting.c.get_last_error",return_value=6,create=True),self.assertRaises(OSError):api.close()
        api.close();self.assertEqual(closed,[42]);self.assertIsNone(api.timer)

    def test_timer_uses_relative_one_shot_and_bounded_failure_guard(self):
        calls=[];api=NativeReference.__new__(NativeReference);api.timer=42
        api.k=SimpleNamespace(SetWaitableTimer=lambda h,due,period,callback,arg,resume:calls.append((due._obj.value,period,callback,resume)) or True,
                              WaitForSingleObject=lambda h,timeout:0)
        with patch("measure_cpu_accounting.time.perf_counter_ns",return_value=100):api.wait_until(1100)
        self.assertEqual(calls,[(-10,0,None,False)])
        api.k.WaitForSingleObject=lambda h,timeout:258
        with patch("measure_cpu_accounting.time.perf_counter_ns",return_value=100),self.assertRaises(RuntimeError):api.wait_until(1100)

    def test_idle_fake_has_no_pulses_and_keeps_raw_counter_difference(self):
        readings=iter([dict(cycles=10,thread_kernel_ticks=0,thread_user_ticks=0,process_kernel_ticks=0,process_user_ticks=0),
                       dict(cycles=20,thread_kernel_ticks=0,thread_user_ticks=0,process_kernel_ticks=0,process_user_ticks=0)])
        at=[0];api=SimpleNamespace(counters=lambda:next(readings),wait_until=lambda deadline:at.__setitem__(0,deadline))
        r=run_phase(api,("idle",2100,0),clock=lambda:at[0])
        self.assertEqual(r["pulses"],0);self.assertEqual(r["cycles"],10);self.assertEqual(r["span_ns"],2_100_000_000)

    def test_fixed_reference_has_short_and_long_windows_without_product_acceptance(self):
        self.assertEqual([r[0] for r in PHASES],["idle_before","pulse_0","pulse_2","pulse_long","dense_control","pulse_5","idle_after"])
        self.assertEqual(PHASES[3][1],12600)

    def test_periodic_fake_uses_bounded_measured_work_without_real_sleep(self):
        now=[0]
        def clock():
            value=now[0];now[0]+=100000;return value
        def counters():
            return dict(cycles=0,thread_kernel_ticks=0,thread_user_ticks=0,process_kernel_ticks=0,process_user_ticks=0)
        api=SimpleNamespace(counters=counters,wait_until=lambda deadline:now.__setitem__(0,max(now[0],deadline)))
        row=run_phase(api,("pulse_fake",30,0),clock=clock)
        self.assertEqual(row["pulses"],3);self.assertEqual(row["missed"],0)
        self.assertGreater(row["busy_wall_ns"],0)
        self.assertGreater(row["checksum"],0)
        self.assertGreater(row["iterations"],0)

    def test_dense_fake_counts_executed_work_not_waits(self):
        at=[0]
        def clock():value=at[0];at[0]+=100000;return value
        def counters():return dict(cycles=0,thread_kernel_ticks=0,thread_user_ticks=0,process_kernel_ticks=0,process_user_ticks=0)
        def wait(deadline):raise AssertionError("dense work must not wait")
        row=run_phase(SimpleNamespace(counters=counters,wait_until=wait),("dense_control",1,0),clock)
        self.assertEqual(row["pulses"],1);self.assertGreater(row["iterations"],0)
        self.assertGreater(row["metrics"]["busy_wall_percent"],50)

    def test_periodic_sampling_model_can_overcount_or_miss_identical_work(self):
        a=sampled_tick_model(2_100_000_000,0)
        b=sampled_tick_model(2_100_000_000,200000)
        self.assertEqual(a["work_wall_ns"],b["work_wall_ns"])
        self.assertEqual(a["work_wall_percent"],2.)
        self.assertEqual(b["charged_percent"],0.)
        self.assertGreater(a["charged_ns"]-a["work_wall_ns"],a["quantum_ns"])

    def test_longer_model_window_does_not_remove_phase_bias(self):
        a=sampled_tick_model(12_600_000_000,0)
        b=sampled_tick_model(12_600_000_000,200000)
        self.assertEqual(b["charged_percent"],0.)
        self.assertGreater(a["charged_percent"],6.)
        self.assertEqual(a["work_wall_percent"],2.)

    def test_model_capacity_and_layout_are_bounded(self):
        for args in ((0,0),(1,0,0),(10,10,10),(1_000_001,0,1,1,1),(10,0,1,2,1)):
            with self.subTest(args=args),self.assertRaises(ValueError):sampled_tick_model(*args)

    def test_report_flags_cannot_replace_work_or_owner_evidence(self):
        row=self.row();row.update(phase="pulse",requested_ms=2000,offset_ns=0)
        phases=(("pulse",2000,0),)
        report=dict(schema=2,pid=7,timer_closed=True,rows=[row])
        self.assertTrue(all(reference_checks(report,7,phases).values()))
        self.assertFalse(all(reference_checks(report,8,phases).values()))
        row["iterations"]=0
        self.assertFalse(all(reference_checks(report,7,phases).values()))
        row["phase"]="wrong"
        with self.assertRaises(ValueError):reference_checks(report,7,phases)


if __name__=="__main__":unittest.main()
