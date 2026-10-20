import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from p0_evidence import ARTIFACTS
from p0_workload import make_workload, png
from p0_process_metrics import summarize


class WorkloadTests(unittest.TestCase):
    def test_original_png_rgba_dimensions_and_determinism(self):
        data = png(16, 2)
        self.assertEqual(data, png(16, 2))
        self.assertNotEqual(data, png(16, 3))
        self.assertEqual(struct.unpack(">II", data[16:24]), (16, 16))
        offset, compressed = 8, b""
        while offset < len(data):
            length = struct.unpack(">I", data[offset:offset+4])[0]
            kind, payload = data[offset+4:offset+8], data[offset+8:offset+8+length]
            self.assertEqual(zlib.crc32(kind + payload), struct.unpack(">I", data[offset+8+length:offset+12+length])[0])
            if kind == b"IDAT":
                compressed += payload
            offset += 12 + length
        raw = zlib.decompress(compressed)
        self.assertEqual(len(raw), 16 * (16 * 4 + 1))
        self.assertEqual(set(raw[4:65:4]), {0, 64, 160, 255})

    def test_fixture_references_inventory_and_preservation(self):
        ARTIFACTS.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ARTIFACTS) as temporary:
            root = Path(temporary)
            self.assertIn(ARTIFACTS.resolve(), root.resolve().parents)
            first = make_workload(root / "one", asset_count=4, document_count=2, side=16)
            second = make_workload(root / "two", asset_count=4, document_count=2, side=16)
            self.assertEqual(first, second)
            manifest = json.loads((root / "one/workload.json").read_text(encoding="utf-8"))
            self.assertEqual(len(manifest["documents"]), 2)
            self.assertEqual(first["layer_count"], 20)
            for item in first["files"]:
                data = (root / "one" / item["path"]).read_bytes()
                self.assertEqual(hashlib.sha256(data).hexdigest(), item["sha256"])
            with self.assertRaises(ValueError):
                make_workload(root / "one")
            with self.assertRaises(ValueError):
                make_workload(ARTIFACTS.parent / "outside")

    def test_cpu_summary_never_spans_phase_boundaries(self):
        samples = [dict(elapsed_seconds=t, cpu_seconds=cpu, working_set_bytes=10,
                        private_bytes=20, gpu_dedicated_bytes=None, gpu_shared_bytes=None)
                   for t, cpu in ((0, 0), (1, 1), (2, 9), (3, 9.01), (62, 9.02))]
        rows = summarize(samples, [{"phase": "Load", "elapsed_seconds": 0},
                                   {"phase": "Idle", "elapsed_seconds": 2},
                                   {"phase": "Done", "elapsed_seconds": 63}])
        self.assertEqual(rows[0]["cpu_one_logical_core_percent"], 100)
        self.assertAlmostEqual(rows[1]["cpu_one_logical_core_percent"], 0.02 / 60 * 100)
        self.assertIsNone(rows[1]["gpu_dedicated_bytes_max"])


if __name__ == "__main__":
    unittest.main()
