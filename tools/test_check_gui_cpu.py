import ctypes as c
import os
from types import SimpleNamespace
import unittest
from unittest.mock import patch
from measure_gui_cpu import origin, cycle_cost, gui_cost, read_cycles, adapter_info, backend_override, AttributedThreads, MODULE_LIMIT, PLAN, BACKEND_PLAN


def report():
    return dict(profile_work=True, ui_cost=[[100,100000000,2000000] for _ in range(18)],
                gui_work=dict(scope="previous-framework-frame-wall-excluding-reported-vsync", invalid=0,
                              logic=[[100,10000000,200000] for _ in range(18)],
                              frame_wall=[[100,400000000,8000000] for _ in range(18)]))


def sample(at, cycles, threads):
    return dict(phase="voices_32", highres_time_ns=at, process_cycles=cycles,
                threads=[dict(key=key, cycles=n, kernel_seconds=k, user_seconds=u, group=g, name="",
                              origin=dict(status="unmapped-or-ambiguous",module=None,offset=None))
                         for key,n,k,u,g in threads])


class GuiCpuChecks(unittest.TestCase):
    def test_start_mapping_uses_half_open_ranges_and_does_not_guess_ambiguous_origins(self):
        modules=[dict(name="a.dll",base=100,bytes=10)]
        self.assertEqual(origin(100,modules,"ok"),dict(status="mapped-start-only",module="a.dll",offset=0))
        self.assertEqual(origin(109,modules,"ok")["offset"],9)
        self.assertIsNone(origin(110,modules,"ok")["module"])
        self.assertIsNone(origin(100,modules+modules,"ok")["module"])
        self.assertEqual(origin(100,modules,"start-query-unavailable")["status"],"start-query-unavailable")

    def test_cycles_stay_cycles_and_counter_gap_is_not_deducted_or_clamped(self):
        a=sample(1000000000,100,[("1:1",70,.1,.2,"gui_main"),("2:1",20,0.,0.,"other")])
        b=sample(3000000000,200,[("1:1",190,.2,.3,"gui_main"),("2:2",5,0.,0.,"other")])
        cost=cycle_cost([a,b]); self.assertEqual(cost["process_cycles"],100)
        self.assertEqual(cost["cycle_counter_gap"],-20)
        self.assertEqual(cost["highres_span_seconds"],2.)
        self.assertEqual(cost["entered"],["2:2"]);self.assertEqual(cost["exited"],["2:1"])
        self.assertFalse(any("percent" in key for key in cost))

    def test_reversed_cycles_times_or_group_changes_reject(self):
        a=sample(1,100,[("1:1",70,.1,.2,"gui_main")])
        cases=[sample(2,99,[("1:1",80,.1,.2,"gui_main")]),
               sample(1,110,[("1:1",80,.1,.2,"gui_main")]),
               sample(2,110,[("1:1",69,.1,.2,"gui_main")]),
               sample(2,110,[("1:1",80,.1,.2,"other")])]
        for b in cases:
            with self.subTest(b=b),self.assertRaises(ValueError):cycle_cost([a,b])
        with self.assertRaises(ValueError):cycle_cost([])

    def test_gui_framework_and_app_wall_remain_separate_from_cpu(self):
        result=gui_cost(report())
        self.assertEqual(result["logic"]["mean_wall_ms"],.1)
        self.assertEqual(result["ui_body"]["mean_wall_ms"],1.)
        self.assertEqual(result["framework_previous_frame"]["mean_wall_ms"],4.)

    def test_gui_missing_scope_invalid_or_insufficient_samples_reject(self):
        for field,value in (("invalid",1),("invalid",False),("scope","cpu")):
            r=report();r["gui_work"][field]=value
            with self.subTest(field=field,value=value),self.assertRaises(ValueError):gui_cost(r)
        for key in ("logic","frame_wall","ui_cost"):
            r=report();rows=r[key] if key=="ui_cost" else r["gui_work"][key]
            rows[4]=[0,0,0]
            with self.subTest(key=key),self.assertRaises(ValueError):gui_cost(r)
        r=report();r["gui_work"]=None
        with self.assertRaises(ValueError):gui_cost(r)

    def test_cycle_api_failure_does_not_become_zero_measurement(self):
        with patch("measure_gui_cpu.c.get_last_error",return_value=6,create=True):
            with self.assertRaises(OSError):read_cycles(lambda h,p:False,1)
        def fill(handle, ptr): ptr._obj.value=123;return True
        self.assertEqual(read_cycles(fill,1),123)

    def module_api(self, rows, error=18):
        index=[-1];closed=[]
        def fill(handle, ptr):
            index[0]+=1
            if index[0]>=len(rows):return False
            for key,value in rows[index[0]].items():setattr(ptr._obj,key,value)
            return True
        kernel=SimpleNamespace(CreateToolhelp32Snapshot=lambda flags,pid:99,
                               Module32FirstW=fill, Module32NextW=fill,
                               CloseHandle=lambda handle:closed.append(handle) or True)
        api=AttributedThreads.__new__(AttributedThreads);api.kernel=kernel;api.pid=7
        return api,closed

    def test_module_snapshot_only_records_owned_basenames_and_closes(self):
        api,closed=self.module_api([dict(pid=7,name="Driver.DLL",path="private-path",base=100,bytes=10)])
        with patch("measure_gui_cpu.c.get_last_error",return_value=18,create=True),patch("measure_gui_cpu.c.set_last_error",create=True):
            rows,status=api.module_snapshot()
        self.assertEqual(rows,[dict(name="driver.dll",base=100,bytes=10)])
        self.assertEqual(status,"ok");self.assertEqual(closed,[99])

    def test_module_pid_race_limit_and_enumeration_error_do_not_claim_mapping(self):
        for rows in ([dict(pid=99)], [dict(pid=7)]*(MODULE_LIMIT+1)):
            api,closed=self.module_api(rows)
            with patch("measure_gui_cpu.c.set_last_error",create=True),self.assertRaises(RuntimeError):api.module_snapshot()
            self.assertEqual(closed,[99])
        api,closed=self.module_api([])
        with patch("measure_gui_cpu.c.get_last_error",return_value=5,create=True),patch("measure_gui_cpu.c.set_last_error",create=True):
            rows,status=api.module_snapshot()
        self.assertEqual(rows,[]);self.assertEqual(status,"module-enumeration-error-5");self.assertEqual(closed,[99])

    def test_optional_start_api_is_unknown_when_absent_not_product_failure(self):
        api=AttributedThreads.__new__(AttributedThreads)
        buffer=c.create_unicode_buffer("owned")
        def description(handle, ptr):ptr._obj.value=c.addressof(buffer);return 0
        def times(handle, created, exited, kernel, user):created._obj.dwLowDateTime=10;return True
        api.kernel=SimpleNamespace(GetThreadDescription=description,LocalFree=lambda ptr:None,GetThreadTimes=times)
        api.tids={1:12};api.starts={};api.query_start=None
        self.assertEqual(api.description(1),"owned")
        self.assertEqual(api.starts,{"12:10":(0,"start-query-unavailable")})

    def test_fixed_same_profiled_binary_control_plan_does_not_retry(self):
        self.assertEqual(PLAN,("process","attributed","attributed","process"))
        self.assertEqual(BACKEND_PLAN,("default","primary","primary","default"))

    def test_backend_override_restores_local_environment_even_after_failure(self):
        for previous in (None,"gl"):
            with patch.dict(os.environ):
                if previous is None:os.environ.pop("WGPU_BACKEND",None)
                else:os.environ["WGPU_BACKEND"]=previous
                with self.assertRaises(RuntimeError):
                    with backend_override("vulkan"):
                        self.assertEqual(os.environ["WGPU_BACKEND"],"vulkan")
                        raise RuntimeError("owned child failure")
                self.assertEqual(os.environ.get("WGPU_BACKEND"),previous)
                with backend_override(None):self.assertEqual(os.environ.get("WGPU_BACKEND"),previous)

    def test_actual_adapter_is_required_instead_of_assuming_requested_backend(self):
        raw=list(b'AdapterInfo { name: "own", backend: Vulkan }')
        self.assertEqual(adapter_info(dict(adapter_utf8=raw))["backend"],"Vulkan")
        for value in (None,[],[True],[256],list(b"no actual backend"),[65]*4097):
            with self.subTest(value=str(value)[:30]),self.assertRaises(ValueError):adapter_info(dict(adapter_utf8=value))


if __name__=="__main__":unittest.main()
