import unittest
from measure_audio_startup import analyze


def fixture():
    report=dict(rounds=[[60,28800,0,1056,0,51200,480,960,0] for _ in range(20)],
                startup_observations=[[[480,1024,0,i+1] for i in range(32)] for _ in range(20)],empty_frames=0)
    samples=[dict(phase=p,time=t,cpu_seconds=t*.01,working_set_bytes=100,private_bytes=100)
             for p in ("idle","closed") for t in range(10)]
    return report,samples


class StartupChecks(unittest.TestCase):
    def test_all_instances_and_windows_are_required(self):
        self.assertTrue(all(analyze(*fixture())["checks"].values()))
        for key in ("rounds","startup_observations"):
            r,s=fixture();r[key].pop()
            with self.assertRaises(ValueError): analyze(r,s)

    def test_underflow_release_capacity_and_memory_fail(self):
        r,s=fixture();r["rounds"][0][2]=192;r["empty_frames"]=192;r["rounds"][0][8]=1;r["rounds"][0][7]=768
        s[0]["private_bytes"]=513*1024**2
        checks=analyze(r,s)["checks"]
        for key in ("no_startup_underflow","resources_released_every_cycle","idle_reserve_covers_two_reported_periods","audio_only_memory_le_512_mib"):
            self.assertFalse(checks[key])

    def test_inconsistent_or_unbounded_trace_data_is_rejected(self):
        r,s=fixture();r["empty_frames"]=1
        with self.assertRaises(ValueError): analyze(r,s)
        r,s=fixture();r["startup_observations"][0][0][1]=2049
        with self.assertRaises(ValueError): analyze(r,s)


if __name__ == "__main__": unittest.main()
