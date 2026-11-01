import unittest
from measure_voice_cpu import thread_cost, group, work_cost


def sample(time, cpu, rows):
    return dict(phase="voices_32", time=time, cpu_seconds=cpu, threads=[dict(key=k, group=g, name=g, cpu_seconds=s) for k,g,s in rows])


class VoiceCpuChecks(unittest.TestCase):
    def test_callback_work_is_wall_time_and_not_subtracted_from_process_cpu(self):
        r=dict(profile_work=True, work_edges=[[0]*6,[200,120000000,1000000,110000000,2000000,96000]], rate=48000,
               lifecycle=[[0,0],[1,0],[2,0],[3,1000],[4,3000]])
        cost=work_cost(r);self.assertEqual(cost["callbacks"],200)
        self.assertEqual(cost["wall_time_fraction_percent"]["total"],6.)
        self.assertEqual(cost["mean_callback_work_us"],600.)

    def test_partial_or_reversed_callback_work_does_not_produce_cost(self):
        for edges in ([],[[0]*6],[[1]*6,[0]*6],[[0]*6,[1,10,4,4,4,480]]):
            with self.subTest(edges=edges),self.assertRaises(ValueError):work_cost(dict(profile_work=True,work_edges=edges))

    def test_unprofiled_rows_cannot_be_relabelled_as_instrumented_work(self):
        with self.assertRaises(ValueError):work_cost(dict(profile_work=False,work_edges=[[0]*6,[0]*6]))

    def test_owned_thread_categories_and_unknowns_not_guessed_as_gpu(self):
        self.assertEqual(group("cpal_wasapi_out", False), "native_output")
        self.assertEqual(group("background-wav-reader", False), "background_read")
        self.assertEqual(group("skin-audio-supply", False), "supply")
        self.assertEqual(group("", True), "gui_main")
        self.assertEqual(group("unrecognised", False), "other")

    def test_cpu_attribution_preserves_process_counter_and_quantisation_gap(self):
        a=sample(1.,1.,[("1:1","gui_main",.5),("2:1","native_output",.2)])
        b=sample(3.,1.4,[("1:1","gui_main",.7),("2:1","native_output",.35)])
        r=thread_cost([a,b]); self.assertTrue(r["complete"])
        self.assertAlmostEqual(r["groups_one_core_percent"]["gui_main"],10.)
        self.assertAlmostEqual(r["groups_one_core_percent"]["native_output"],7.5)
        self.assertAlmostEqual(r["counter_gap_seconds"],.05)

    def test_lifetime_change_is_visible_and_not_invented_as_zero_cost(self):
        a=sample(1.,1.,[("1:1","gui_main",.5),("2:1","other",.2)])
        b=sample(3.,1.4,[("1:1","gui_main",.7),("2:2","other",.1)])
        r=thread_cost([a,b]);self.assertFalse(r["complete"])
        self.assertEqual(r["entered"],["2:2"]);self.assertEqual(r["exited"],["2:1"])

    def test_missing_phase_or_reversed_counters_reject(self):
        with self.assertRaises(ValueError):thread_cost([])
        a=sample(1.,1.,[("1:1","gui_main",.5)])
        b=sample(3.,1.4,[("1:1","gui_main",.4)])
        with self.assertRaises(ValueError):thread_cost([a,b])


if __name__ == "__main__":unittest.main()
