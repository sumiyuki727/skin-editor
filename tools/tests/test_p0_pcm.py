import copy
import math
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_pcm import CHECKS, reference_sample, validate


def fixture():
    def packet(start):
        values = [reference_sample(start+n, 48000) for n in range(4800)]
        return {"source_frame": start, "frames": 4800, "peak": max(abs(v) for v in values),
                "nonzero_samples": 2*sum(v != 0 for v in values), "probe": values[:8], "hash": 1}

    def stats(base, submitted):
        origin = base*48000//1000000
        return {"requested_us": base, "source_origin_frame": origin, "submitted_frames": submitted,
                "packets": 1 if submitted else 0, "nonzero_samples": 100 if submitted else 0,
                "first": packet(origin) if submitted else None,
                "last": packet(origin+submitted-4800) if submitted else None}

    rows = []
    def add(epoch, base, request, phase, device, submitted, count=1):
        for _ in range(count):
            rows.append({"epoch": epoch, "base_us": base, "sequence": len(rows)+1, "request": request,
                         "phase": phase, "device_us": device, "simulation_us": base+device, "pcm": stats(base, submitted)})
    add(0, 0, 2, "Paused", 350000, 21600, 3)
    add(0, 0, 3, "Playing", 400000, 24000)
    add(0, 0, 4, "Paused", 700000, 38400, 3)
    for epoch, base in [(1, 1050000), (2, 50000), (3, 400000)]:
        add(epoch, base, 0, "Paused", 0, 0, 3)
        add(epoch, base, 1, "Playing", 350000, 21600)
        if epoch != 2:
            add(epoch, base, 2, "Paused", 350000, 21600, 3)
    return {"schema_version": 1, "status": "passed", "checks": dict.fromkeys(CHECKS, True), "errors": [],
            "accepted_inputs": 2, "nodes": 2, "edit_count": 30, "system_capture_used": False,
            "microphone_or_loopback_used": False, "human_audibility": "unconfirmed", "stale_rejected": 3,
            "frozen_us": 350000, "normal_resume_us": 350000, "recovery_frozen_us": 400000, "final_simulation_us": 750000,
            "audio_format": {"rate": 48000, "channels": 2, "bits": 32, "buffer_frames": 4800}, "fresh_samples": rows,
            "checkpoints": [{"epoch": e, "nodes": n, "accepted_inputs": i, "simulation_us": us, "pcm": stats(base, submitted)}
                            for e, n, i, us, base, submitted in [(0, 1, 1, 350000, 0, 21600), (0, 2, 1, 350000, 0, 21600),
                                                               (1, 2, 2, 1050000, 1050000, 0), (3, 2, 2, 400000, 400000, 0)]],
            "proofs": [{"stage": n, "pixels": 1024*1024, "max_channel_error": 1, "pixels_outside_tolerance": 0,
                        "display_max_channel_error": 1, "display_pixels_outside_tolerance": 0} for n in range(4)]}


class PcmTests(unittest.TestCase):
    def test_independent_pcm_reference_has_known_tone_peak_zero_and_quiet_tail(self):
        self.assertAlmostEqual(reference_sample(900, 48000), .04, places=7)
        self.assertAlmostEqual(reference_sample(600, 48000), 0, places=7)
        for frame in [12000, 48000*6, 48000*10]:
            self.assertEqual(reference_sample(frame, 48000), 0)

    def test_evidence_rejects_wrong_content_clock_seek_and_hearing_claim(self):
        good = fixture()
        validate(good)
        missing=copy.deepcopy(good);del missing["stale_rejected"]
        with self.assertRaises(RuntimeError):
            validate(missing)
        for key, value in [("normal_resume_us", 900000), ("accepted_inputs", 3), ("human_audibility", "confirmed"),
                           ("microphone_or_loopback_used", True), ("stale_rejected", 2)]:
            bad = copy.deepcopy(good); bad[key] = value
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                validate(bad)
        changes = [lambda r: r["fresh_samples"][0].update(simulation_us=450000),
                   lambda r: r["fresh_samples"][1]["pcm"].update(submitted_frames=21800),
                   lambda r: r["fresh_samples"][3]["pcm"]["first"].update(source_frame=1),
                   lambda r: r["fresh_samples"][3]["pcm"]["first"].update(probe=[.1]*8),
                   lambda r: r["fresh_samples"][3]["pcm"]["first"].update(peak=math.nan),
                   lambda r: r["fresh_samples"][7].update(base_us=0),
                   lambda r: r["fresh_samples"][3].update(sequence=1),
                   lambda r: r["checkpoints"][2].update(nodes=3)]
        for n, change in enumerate(changes):
            bad = copy.deepcopy(good); change(bad)
            with self.subTest(change=n), self.assertRaises(RuntimeError):
                validate(bad)


if __name__ == "__main__":
    unittest.main()
