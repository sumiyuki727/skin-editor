import unittest
from types import SimpleNamespace
from unittest.mock import patch
from measure_thread_sampling import OwnedThreadSet, ObserverAudit, observer_cost, additional_checks, PLAN
from measure_voice_cpu import ThreadSampler


class FakeThreads:
    pid = 7

    def __init__(self):
        self.live = [1]
        self.identities = {1: (7, 10, .1, "main")}
        self.handles = {}
        self.opened = []
        self.closed = []
        self.inventories = 0
        self.fail = None

    def live_ids(self):
        self.inventories += 1
        return list(self.live)

    def open_thread(self, tid):
        if tid not in self.identities: return None
        handle = len(self.opened)+1
        self.opened.append(handle)
        self.handles[handle] = self.identities[tid]
        return handle

    def owner(self, handle): return self.handles[handle][0]

    def times(self, handle):
        if self.handles[handle][0] != self.pid: raise AssertionError("foreign counter access")
        if self.fail == "times": raise OSError("counter unavailable")
        _, created, cpu, _ = self.handles[handle]
        return created, cpu

    def description(self, handle):
        if self.fail == "description": raise OSError("description unavailable")
        return self.handles[handle][3]

    def close_thread(self, handle):
        if handle in self.closed: raise AssertionError("double close")
        self.closed.append(handle)


class ThreadSamplingChecks(unittest.TestCase):
    def test_capacity_cannot_disable_the_lifetime_bound(self):
        for limit in (0, -1, 129, None, True):
            with self.subTest(limit=limit), self.assertRaises(ValueError):
                OwnedThreadSet(FakeThreads(), limit=limit)

    def test_reuses_owned_handle_and_refreshes_only_on_phase_transition(self):
        api = FakeThreads(); cache = OwnedThreadSet(api)
        first = cache.rows("voices_32")
        for _ in range(100): self.assertEqual(cache.rows("voices_32"), first)
        self.assertEqual(api.inventories, 1); self.assertEqual(len(api.opened), 1)
        cache.rows("closed")
        self.assertEqual(api.inventories, 2); self.assertEqual(len(api.opened), 2)
        self.assertEqual(len(api.closed), 1)
        cache.close(); cache.close()
        self.assertCountEqual(api.opened, api.closed)
        with self.assertRaises(RuntimeError): cache.rows("closed")

    def test_retains_exited_identity_and_does_not_retarget_reused_tid(self):
        api = FakeThreads(); cache = OwnedThreadSet(api)
        cache.rows("a")
        api.identities[1] = (7, 20, .03, "skin-audio-supply")
        rows = cache.rows("b")
        self.assertEqual([r["key"] for r in rows], ["1:10", "1:20"])
        self.assertEqual([r["cpu_seconds"] for r in rows], [.1, .03])
        self.assertEqual([r["group"] for r in rows], ["gui_main", "supply"])
        api.live = []
        self.assertEqual(cache.rows("c"), rows)
        cache.close(); self.assertCountEqual(api.opened, api.closed)

    def test_foreign_tid_race_and_unopenable_exited_thread_are_not_sampled(self):
        api = FakeThreads(); api.live = [2, 3]
        api.identities[2] = (99, 20, 9., "private")
        cache = OwnedThreadSet(api)
        self.assertEqual(cache.rows("a"), [])
        cache.close(); self.assertCountEqual(api.opened, api.closed)

    def test_limit_refusal_closes_current_and_preserves_bounded_prior_handles(self):
        api = FakeThreads(); api.live = [1, 2]
        api.identities[2] = (7, 20, .2, "other")
        cache = OwnedThreadSet(api, limit=1)
        with self.assertRaises(RuntimeError): cache.rows("a")
        self.assertEqual(len(cache.entries), 1)
        cache.close(); self.assertCountEqual(api.opened, api.closed)

    def test_api_failures_release_unretained_and_prior_handles(self):
        for failure in ("times", "description"):
            with self.subTest(failure=failure):
                api = FakeThreads(); cache = OwnedThreadSet(api)
                cache.rows("a")
                api.live = [2]; api.identities[2] = (7, 20, .2, "other"); api.fail = failure
                with self.assertRaises(OSError): cache.rows("b")
                cache.close(); self.assertCountEqual(api.opened, api.closed)

    def test_corrupted_or_reversed_retained_counter_rejects(self):
        for bad in ((7, 99, .1, "main"), (7, 10, .01, "main")):
            api = FakeThreads(); cache = OwnedThreadSet(api)
            cache.rows("a"); api.handles[1] = bad
            with self.assertRaises(ValueError): cache.rows("a")
            cache.close(); self.assertCountEqual(api.opened, api.closed)

    def test_close_attempts_every_handle_even_after_error(self):
        api = FakeThreads(); api.live = [1, 2]; api.identities[2] = (7, 20, .2, "other")
        cache = OwnedThreadSet(api); cache.rows("a")
        original = api.close_thread
        def close(handle):
            original(handle)
            if handle == 1: raise OSError("close error")
        api.close_thread = close
        with self.assertRaises(OSError): cache.close()
        self.assertCountEqual(api.opened, api.closed); self.assertFalse(cache.entries)
        cache.close()

    def test_enumerating_reference_still_closes_per_query_and_filters_owner(self):
        api = FakeThreads()
        api.identities[2] = (99, 20, 7., "foreign")
        api.live = [1, 2, 3]
        # Invoke the reference collector with the same fake, no Windows/device/timing.
        class Reference:
            pid = api.pid
            handles = {}
            open_thread = staticmethod(api.open_thread)
            owner = staticmethod(api.owner)
            times = staticmethod(api.times)
            close_thread = staticmethod(api.close_thread)
            description = staticmethod(api.description)
        rows = ThreadSampler.thread_rows(Reference(), api.live)
        self.assertEqual([r["key"] for r in rows], ["1:10"])
        self.assertCountEqual(api.opened, api.closed)

    def test_native_close_failure_is_not_reported_as_released(self):
        failing = SimpleNamespace(kernel=SimpleNamespace(CloseHandle=lambda handle: False))
        with patch("measure_voice_cpu.c.get_last_error", return_value=6, create=True):
            with self.assertRaises(OSError): ThreadSampler.close_thread(failing, 1)
        passing = SimpleNamespace(kernel=SimpleNamespace(CloseHandle=lambda handle: True))
        ThreadSampler.close_thread(passing, 1)

    def test_observer_cost_keeps_raw_child_budget_and_excludes_first_query_from_sums(self):
        rows = [dict(phase="p", time=t, cpu_seconds=cpu,
                     observer=dict(wall_ns=2000000, runner_cpu_ns=1000000, inventory_ns=1000000,
                                   thread_read_ns=500000, refreshes=1, retained_handles=3))
                for t, cpu in ((1., 0.), (2., .2), (3., .4))]
        cost = observer_cost(rows, "p")
        self.assertEqual(cost["interval_query_wall_seconds"], .004)
        self.assertEqual(cost["runner_cpu_seconds_during_queries"], .002)
        self.assertEqual(cost["refreshes"], 3)
        self.assertEqual(rows[-1]["cpu_seconds"], .4)
        with self.assertRaises(ValueError): observer_cost(rows, "absent")
        rows[0]["observer"]["wall_ns"] = -1
        with self.assertRaises(ValueError): observer_cost(rows, "p")

    def test_audit_closes_cached_resources_on_early_exit(self):
        class Backend:
            def __init__(self, child):
                self.retained = OwnedThreadSet(FakeThreads())
            def sample(self, phase):
                return dict(threads=self.retained.rows(phase))
            def close(self): self.retained.close()
        with patch("measure_thread_sampling.CachedThreads", Backend):
            audit = ObserverAudit(None, "cached")
            audit.sample("p"); api = audit.backend.retained.api
            audit.close(); self.assertEqual(audit.close_record, dict(before=1, after=0))
            self.assertCountEqual(api.opened, api.closed)

    def test_fixed_counterbalanced_plan_has_no_retry_or_failure_selection(self):
        self.assertEqual(PLAN, ("process", "enumerate", "cached", "cached", "enumerate", "process")*2)

    def test_additional_stream_callback_and_observer_checks_reject_each_leak(self):
        from test_check_background_stream import report
        samples = [dict(working_set_bytes=4096, private_bytes=8192)]
        self.assertTrue(all(additional_checks(report(), samples, dict(after=0)).values()))
        for key, value in (("source_raw_final", 1), ("source_fault", True),
                           ("callback_resource_releases", 1), ("endpoint_returns_in_render", 1)):
            r = report(); r[key] = value
            with self.subTest(key=key):
                self.assertFalse(all(additional_checks(r, samples, dict(after=0)).values()))
        self.assertFalse(all(additional_checks(report(), samples, dict(after=1)).values()))
        self.assertFalse(all(additional_checks(report(), samples, None).values()))
        self.assertFalse(all(additional_checks(report(), [], dict(after=0)).values()))
        samples[0]["private_bytes"] = 512*1024**2+1
        self.assertFalse(all(additional_checks(report(), samples, dict(after=0)).values()))


if __name__ == "__main__": unittest.main()
