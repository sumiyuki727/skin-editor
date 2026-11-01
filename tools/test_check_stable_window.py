import copy
import unittest
from pathlib import Path
import tempfile
from unittest.mock import patch, Mock
from measure_stable_window import phases_for, report_checks, analyze
from measure_main_work import product_argument,launch_owned,product,run_child


def row(phase):
    return dict(phase=phase,span_us=65_000_000,draws=0,inputs=0,frozen=True,view_valid=True,geometry=[1180.,800.,1.],before=[0]*5,after=[0]*5,shape=[0]*3)


class StableWindowTests(unittest.TestCase):
    def empty(self):
        return dict(schema=1,scope="full-editor-stable-windows",completed=True,resources_case=False,shape=[0]*3,rows=[row("empty")])
    def test_full_empty_contract_and_modes(self):
        self.assertTrue(all(report_checks(self.empty(),"stable-empty").values()))
        self.assertEqual(phases_for("stable-resources"),("paused","closed"))
        self.assertEqual(product_argument(False,"stable-empty"),"--verify-stable-empty")
        self.assertEqual(product_argument(False,"stable-resources"),"--verify-stable-resources")
        with self.assertRaises(ValueError):product_argument(True,"stable-empty")
    def test_product_routes_use_ordinary_assembly_without_enabling_profiling(self):
        self.assertEqual(product_argument(False,"product-empty"),"--verify-product-empty")
        self.assertEqual(product_argument(False,"product-stable"),"--verify-product-stable")
        for mode in ("product-empty","product-stable"):
            with self.assertRaises(ValueError):product_argument(True,mode)
    def test_product_child_keeps_stable_protocol_and_requests_product_policy(self):
        for launch,mode in (("product-empty","stable-empty"),("product-stable","stable-resources")):
            with patch('measure_stable_window.observe_product',return_value={'sentinel':True}) as observe:
                self.assertEqual(product(Path('unused'),False,Path('frozen.exe'),launch),{'sentinel':True})
                observe.assert_called_once_with(Path('unused'),Path('frozen.exe'),mode,product=True)
    def test_product_policy_requires_both_explicit_assembly_markers(self):
        valid=dict(self.empty(),production_policy=True,gpu_memory_usage_hint=True)
        self.assertTrue(all(report_checks(valid,"stable-empty",product=True).values()))
        for name in ('production_policy','gpu_memory_usage_hint'):
            for value in (False,None,1,'true'):
                bad=dict(valid);bad[name]=value
                self.assertFalse(all(report_checks(bad,"stable-empty",product=True).values()))
        self.assertFalse(all(report_checks(self.empty(),"stable-empty",product=True).values()))
    def test_legacy_control_is_retained_but_cannot_prove_product_policy(self):
        legacy=dict(self.empty(),production_policy=False,gpu_memory_usage_hint=False)
        self.assertTrue(all(report_checks(legacy,"stable-empty",product=False).values()))
        self.assertFalse(all(report_checks(legacy,"stable-empty",product=True).values()))
    def test_observer_uses_product_entry_and_retains_actual_policy(self):
        from measure_stable_window import observe_product
        report=dict(self.empty(),production_policy=True,gpu_memory_usage_hint=True)
        def fake_observe(binary,output,argument,**kw):
            kw['sampler_factory'](Mock(pid=19))
            return report,[]
        def fake_sampler(sampler,child):sampler.handle=42
        with patch('measure_stable_window.observe',side_effect=fake_observe) as observe,patch(
                'measure_stable_window.stamp_api',return_value=(Mock(return_value=1),1000)),patch(
                'measure_stable_window.summarize',return_value={}),patch(
                'measure_stable_window.GpuSampler.__init__',fake_sampler),patch(
                'measure_stable_window.token_info',return_value=dict(elevated=0,type=3)):
            record=observe_product(Path('unused'),Path('frozen.exe'),'stable-empty',product=True)
            self.assertEqual(observe.call_args.args[2],'--verify-product-empty')
            self.assertEqual(observe.call_args.kwargs['timeout_seconds'],90)
            self.assertEqual(record['pid'],19)
            self.assertTrue(all(record['original_checks'].values()))
    def test_product_child_preserves_bounded_long_window_timeouts(self):
        for mode,timeout in (('product-empty',95),('product-stable',220)):
            with tempfile.TemporaryDirectory() as directory,patch('measure_main_work.c.WinDLL',return_value=Mock()),patch(
                    'measure_main_work.launch_owned',return_value={}) as launch:
                run_child(Path(directory)/'owned','product',binary=Path('frozen.exe'),ui_mode=mode)
                self.assertEqual(launch.call_args.kwargs['timeout'],timeout)
                args=launch.call_args.args[3]
                self.assertEqual(args[args.index('--ui-mode')+1],mode)
                self.assertNotIn('--profile',args)
    def test_short_tail_input_extra_draw_and_moving_clock_remain_failures(self):
        for change in (dict(span_us=64_999_999),dict(draws=3),dict(inputs=1),dict(frozen=False),dict(view_valid=False)):
            r=self.empty();r["rows"][0].update(change)
            self.assertFalse(all(report_checks(r,"stable-empty").values()))
    def test_sample_or_retained_resource_prevents_empty_acceptance(self):
        for change in (dict(before=[1,0,0,0,0]),dict(after=[0,0,0,1,0]),dict(shape=[1,1,1])):
            r=self.empty();r["rows"][0].update(change)
            self.assertFalse(report_checks(r,"stable-empty")["empty_or_closed_resources_released"])
    def test_missing_or_extra_window_is_rejected(self):
        for rows in ([],[row("closed")],[row("empty"),row("empty")]):
            r=self.empty();r["rows"]=rows
            with self.assertRaises(ValueError):report_checks(r,"stable-empty")
    def test_retained_pause_and_real_closed_native_audit(self):
        r=dict(self.empty(),resources_case=True,shape=[100,10,500],source_raw_final=0,
               rows=[dict(row("paused"),shape=[100,10,500],before=[10]*5,after=[10]*5),row("closed")],
               native=dict(callbacks=100,missing_frames=0,render_errors=0,observation_overflows=0,device_fault=False,
                           callback_resource_releases=0,endpoint_returns_in_render=0,endpoint_returns_outside_render=1))
        self.assertTrue(all(report_checks(r,"stable-resources").values()))
        for field,value in (("missing_frames",1),("device_fault",True),("callback_resource_releases",1),("endpoint_returns_outside_render",0)):
            bad=copy.deepcopy(r);bad["native"][field]=value
            self.assertFalse(all(report_checks(bad,"stable-resources").values()))
    def test_stable_child_timeout_cannot_be_unbounded(self):
        for value in (0,221,True,None):
            with self.assertRaises(ValueError):launch_owned(None,None,None,[],None,timeout=value)
    def test_independent_two_percent_gate_calibration_and_missing_gpu_fail_closed(self):
        report=self.empty()
        record=dict(pid=7,frequency=1_000_000,mode="stable-empty",report=report,metrics={},
                    original_checks=report_checks(report,"stable-empty"),samples=[
                        dict(phase="empty",qpc_end=q,working_set_bytes=100*1024**2,private_bytes=120*1024**2,
                             gpu_dedicated_bytes=50*1024**2,gpu_shared_bytes=20*1024**2)
                        for q in (1,65_000_001)])
        trace=dict(schema=1,status=0,stop_status=0,trace_status=0,child_status=0,invalid=0,events_lost=0,buffers_lost=0,
                   frequency=1_000_000,root_pid=7,buffer_count=1,buffers_allocated=32,thread_count=1,
                   events=[[1,0,7,1,1],[1_300_001,0,7,1,0]],callback_wall_qpc=1,callback_count=2)
        self.assertTrue(analyze(trace,record,True)["passed"])
        legacy=copy.deepcopy(record);legacy['report'].update(production_policy=False,gpu_memory_usage_hint=False)
        self.assertFalse(analyze(trace,legacy,True,product=True)['passed'])
        current=copy.deepcopy(record);current['report'].update(production_policy=True,gpu_memory_usage_hint=True)
        self.assertTrue(analyze(trace,current,True,product=True)['passed'])
        self.assertFalse(analyze(trace,record,False)["passed"])
        over=copy.deepcopy(trace);over["events"][1][0]+=1
        self.assertFalse(analyze(over,record,True)["checks"]["empty_independent_cpu_le_2_percent"])
        missing=copy.deepcopy(record);missing["samples"][0]["gpu_shared_bytes"]=None
        self.assertFalse(analyze(trace,missing,True)["checks"]["empty_gpu_shared_budget"])


if __name__=="__main__":unittest.main()
