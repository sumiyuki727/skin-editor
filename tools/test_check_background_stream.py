import unittest
from measure_background_stream import stream_checks, audit_checks, PLAN


def report():
    return dict(streamed=True, rate=48000, source_frames=48000*360, source_fault=False,
                source_raw_peak=8192, source_raw_final=0, source_read_bytes=200000, source_seeks=1,
                stream_seconds=20.01, stream_frames=960000, located=True, seek_isolated=True,
                stream_monotonic=True, backwards=True, callback_resource_releases=0,
                endpoint_returns_in_render=0, endpoint_returns_outside_render=1)


class BackgroundStreamChecks(unittest.TestCase):
    def test_fixed_plan_and_long_source_not_whole_pcm(self):
        self.assertEqual(len(PLAN), 5)
        self.assertEqual([v[0] for v in PLAN], ["finite_before", "stream_worker_1", "stream_gui_1", "stream_gui_2", "stream_worker_2"])
        self.assertTrue(all(stream_checks(report(), True).values()))
        self.assertTrue(all(audit_checks(report()).values()))

    def test_fault_unreleased_raw_and_failed_seek_cannot_pass(self):
        for key, value in (("source_fault", True), ("source_raw_final", 1), ("source_raw_peak", 8193), ("located", False), ("backwards", False), ("seek_isolated", False), ("stream_monotonic", False), ("stream_frames", 1)):
            r = report(); r[key] = value
            with self.subTest(key=key): self.assertFalse(all(stream_checks(r, True).values()))

    def test_missing_stream_identity_or_invalid_duration_is_rejected(self):
        for key, value in (("streamed", False), ("source_frames", 48000*10), ("stream_seconds", float("nan")), ("stream_seconds", 19.), ("stream_seconds", 30.)):
            r = report(); r[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError): stream_checks(r, True)

    def test_each_callback_ownership_violation_fails(self):
        for key, value in (("callback_resource_releases", 1), ("endpoint_returns_in_render", 1), ("endpoint_returns_outside_render", 0)):
            r = report(); r[key] = value
            with self.subTest(key=key): self.assertFalse(all(audit_checks(r).values()))


if __name__ == "__main__": unittest.main()
