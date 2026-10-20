import json
from pathlib import Path
import sys
import tempfile
import unittest
import wave

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import p0_evidence as evidence


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        evidence.ARTIFACTS.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=evidence.ARTIFACTS)
        self.root = Path(self.temporary.name)
        self.assertIn(evidence.ARTIFACTS.resolve(), self.root.resolve().parents)

    def tearDown(self):
        self.assertIn(evidence.ARTIFACTS.resolve(), self.root.resolve().parents)
        self.temporary.cleanup()

    def client(self):
        root = self.root / "client"
        (root / "Logs").mkdir(parents=True)
        (root / "osu!.exe").write_bytes(b"synthetic executable identity")
        (root / "osu!.cfg").write_text("_ReleaseStream=Stable40\nPassword=PRIVATE\n")
        (root / "osu!.user.cfg").write_text("Password=PRIVATE_USER\n")
        (root / "Logs/osu!auth.log").write_text("Version: b20261018 PRIVATE_AUTH\n")
        (root / "Logs/session.log").write_text("Version: b20240101.1\nSome private unrelated text\n")
        return root

    def test_inspection_is_read_only_and_excludes_sensitive_sources(self):
        client = self.client()
        before = {str(p): p.read_bytes() for p in client.rglob("*") if p.is_file()}
        result = evidence.inspect_client(client)
        serialized = json.dumps(result)
        self.assertNotIn("PRIVATE", serialized)
        self.assertNotIn("unrelated text", serialized)
        self.assertNotIn("b20261018", serialized)
        self.assertEqual(before, {str(p): p.read_bytes() for p in client.rglob("*") if p.is_file()})

    def test_historical_log_tokens_are_not_current_release_proof(self):
        result = evidence.inspect_client(self.client())
        self.assertIsNone(result["actual_release"])
        self.assertEqual(result["compatibility_cases_executed"], 0)
        self.assertEqual(result["logs"][0]["unverified_release_tokens"], ["b20240101.1"])
        self.assertFalse(result["logs"][0]["tokens_are_current_release_proof"])

    def test_fixtures_are_deterministic_self_contained_and_unverified(self):
        first = evidence.make_fixture(self.root / "first")
        second = evidence.make_fixture(self.root / "second")
        self.assertEqual(first, second)
        self.assertFalse(first["client_execution_verified"])
        for item in first["files"]:
            self.assertEqual(evidence.sha((self.root / "first" / item["path"]).read_bytes()), item["sha256"])
        with wave.open(str(self.root / "first/beatmap/silence.wav")) as audio:
            self.assertEqual(audio.getnframes(), 220500)
        map_text = (self.root / "first/beatmap/P0 Circles.osu").read_text()
        self.assertIn("AudioFilename: silence.wav", map_text)
        self.assertEqual(map_text.split("[HitObjects]\n")[1].strip().count("\n") + 1, 6)

    def test_registration_preserves_image_and_cannot_pass_a_case(self):
        report = self.root / "client.json"
        evidence.write_json(report, evidence.inspect_client(self.client()))
        image = self.root / "explicit.png"
        original = evidence.png(4, 3, lambda x, y: (x, y, 50, 255))
        image.write_bytes(original)
        record = evidence.register_screenshot(image, report, self.root / "case", "C01")
        self.assertEqual((self.root / "case/original.png").read_bytes(), original)
        self.assertEqual(record["image_metadata"]["width"], 4)
        self.assertEqual(record["review_status"], "awaiting_review")
        self.assertIsNone(record["compatibility_passed"])
        self.assertIsNone(record["declared_client_release"])

    def test_existing_outputs_and_escape_paths_are_rejected(self):
        destination = self.root / "existing"
        destination.mkdir()
        marker = destination / "keep.txt"
        marker.write_text("preserve")
        with self.assertRaises(ValueError):
            evidence.make_fixture(destination)
        with self.assertRaises(ValueError):
            evidence.output_path(evidence.ROOT / "outside-p0.json")
        self.assertEqual(marker.read_text(), "preserve")

    def test_invalid_or_corrupted_images_are_rejected(self):
        with self.assertRaises(ValueError):
            evidence.image_metadata(b"not a screenshot")
        data = bytearray(evidence.png(2, 2, lambda x, y: (1, 2, 3, 4)))
        data[20] ^= 1
        with self.assertRaises(ValueError):
            evidence.image_metadata(bytes(data))


if __name__ == "__main__":
    unittest.main()
