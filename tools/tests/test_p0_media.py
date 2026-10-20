import copy
import ctypes
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import wave

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_media import CHECKS, WAV, Reference, check_boundary, make_fixture, validate
from check_p0 import ROOT
from check_p0_transport import sources


def record(fixture):
    reference = Reference(fixture)
    rows = []
    def stats(base, submitted):
        origin = base*48000//1000000
        return {"requested_us": base, "source_origin_frame": origin, "submitted_frames": submitted,
                "packets": submitted//4800, "first": reference.packet(origin, 4800) if submitted else None,
                "last": reference.packet(origin+submitted-4800, 4800) if submitted else None}
    def add(epoch, base, device, phase, submitted, count=1):
        for _ in range(count):
            rows.append({"epoch": epoch, "sequence": len(rows)+1, "phase": phase, "request": 2,
                         "device_us": device, "simulation_us": base+device, "pcm": stats(base, submitted)})
    def frame(position):
        return {"position_us": position, "objects": [{"id": i, "stage": "Past"} for i in range(1, 7)]}
    add(0, 0, 0, "Paused", 0)
    add(0, 0, 600000, "Playing", 33600)
    add(0, 0, 600000, "Paused", 33600, 4)
    for epoch, base in [(1, 1100000), (2, 100000)]:
        add(epoch, base, 0, "Paused", 0)
        add(epoch, base, 500000, "Playing", 28800)
        add(epoch, base, 500000, "Paused", 28800, 4)
    return {"schema_version": 1, "status": "passed", "source": "self_authored_wav_and_v14_circle_fixture",
            "checks": dict.fromkeys(CHECKS, True), "system_capture_used": False, "microphone_or_loopback_used": False,
            "accepted_inputs": 2, "decode_ms": 1.25,
            "decoded": {"rate": 48000, "channels": 2, "frames": 144000, "duration_us": 3000000, "payload_bytes": 1152000, "gain": .04},
            "audio_format": {"rate": 48000, "channels": 2, "bits": 32, "buffer_frames": 4800},
            "event_crossings": {"0": [1, 2, 3], "1": [5], "2": [1, 2, 3]}, "fresh_samples": rows,
            "paused_checks": [{"epoch": e, "position_us": position, "device_us": device, "fresh_samples": 3, "wall_ms": 350, "frame": frame(position)}
                              for e, position, device in [(0, 600000, 600000), (1, 1600000, 500000), (2, 600000, 500000)]],
            "relocations": [{"epoch": e, "position_us": base, "frame": frame(base), "pcm": stats(base, 0)} for e, base in [(1, 1100000), (2, 100000)]],
            "final_frame": frame(600000)}


class MediaTests(unittest.TestCase):
    def test_evidence_inventory_keeps_optional_local_media_package_reproducible(self):
        paths = sources()
        self.assertEqual(len(paths), len(set(paths)))
        media = ROOT/"experiments/p0-media"
        self.assertTrue({media/"Cargo.toml", media/"Cargo.lock", *media.joinpath("src").rglob("*.rs")} <= set(paths))

    def test_boundary_rejects_native_ui_files_clocks_and_unreviewed_decoder_features(self):
        dep = {"name": "symphonia", "req": "=0.6.1", "uses_default_features": False, "features": ["wav", "pcm"]}
        check_boundary([dep], {"audio.rs": "use std::io::Cursor;", "chart.rs": "pub struct Chart;"})
        for code in ["use std::fs;", "use std::time::Instant;", "use eframe::egui;", "use windows::Win32;", "include_str!(\"foreign\");"]:
            with self.subTest(code=code), self.assertRaises(RuntimeError):
                check_boundary([dep], {"audio.rs": code})
        with self.assertRaises(RuntimeError):
            check_boundary([dep], {"chart.rs": "use symphonia::core;"})
        with self.assertRaises(RuntimeError):
            check_boundary([{**dep, "features": ["all"]}], {})
        with self.assertRaises(RuntimeError):
            check_boundary([dep], {"nested/helper.rs": "use std::fs::File;"})
        with self.assertRaises(RuntimeError):
            check_boundary([dep], {"nested/chart.rs": "use symphonia::core;"})

    def test_fixture_has_exact_wave_shape_unicode_reference_and_never_overwrites(self):
        with tempfile.TemporaryDirectory(prefix="p0-media-fixture-test-") as temp:
            path = Path(temp)/"fixture"; manifest = make_fixture(path)
            with wave.open(str(path/WAV), "rb") as f:
                self.assertEqual(f.getparams()[:4], (2, 2, 48000, 144000))
            self.assertIn(WAV, (path/"timeline.osu").read_text(encoding="utf-8"))
            self.assertEqual(manifest["object_times_ms"], [250, 500, 500, 900, 1400, 2100])
            before = (path/WAV).read_bytes()
            with self.assertRaises(FileExistsError):
                make_fixture(path)
            self.assertEqual(before, (path/WAV).read_bytes())

    def test_reference_distinguishes_stereo_gain_source_position_and_eof(self):
        with tempfile.TemporaryDirectory(prefix="p0-media-reference-test-") as temp:
            path = Path(temp)/"fixture"; make_fixture(path); ref = Reference(path)
            self.assertAlmostEqual(ref.packet(900, 1)["probe"][0], ctypes.c_float(.5*ctypes.c_float(.04).value).value)
            self.assertNotEqual(ref.packet(900, 1)["hash"], ref.packet(901, 1)["hash"])
            self.assertEqual(ref.packet(144000, 8)["nonzero_samples"], 0)
            packet = copy.deepcopy(ref.packet(900, 1)); packet["hash"] ^= 1
            with self.assertRaises(RuntimeError):
                ref.check_packet(packet)

    def test_native_evidence_rejects_wrong_clock_content_pause_seek_and_event_replay(self):
        with tempfile.TemporaryDirectory(prefix="p0-media-evidence-test-") as temp:
            path = Path(temp)/"fixture"; make_fixture(path); good = record(path); validate(good, path)
            mutations = [lambda r: r["fresh_samples"][1].update(simulation_us=700000),
                         lambda r: r["fresh_samples"][1]["pcm"]["first"].update(hash=0),
                         lambda r: r["fresh_samples"][1]["pcm"]["last"].update(source_frame=1),
                         lambda r: r["fresh_samples"][3]["pcm"].update(submitted_frames=33601),
                         lambda r: r["event_crossings"].update({"1": [4, 5]}),
                         lambda r: r["relocations"][0]["pcm"].update(submitted_frames=1),
                         lambda r: r["paused_checks"][0].update(fresh_samples=1),
                         lambda r: r["decoded"].update(channels=1),
                         lambda r: r.update(accepted_inputs=3),
                         lambda r: r.update(microphone_or_loopback_used=True)]
            for n, mutate in enumerate(mutations):
                bad = copy.deepcopy(good); mutate(bad)
                with self.subTest(mutation=n), self.assertRaises(RuntimeError):
                    validate(bad, path)


if __name__ == "__main__":
    unittest.main()
