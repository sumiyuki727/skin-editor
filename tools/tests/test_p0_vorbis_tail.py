from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from p0_vorbis_tail import checked_paths, inspect_ogg, ogg_crc


class VorbisTailTests(unittest.TestCase):
    def test_original_fixture_crc_packet_identity_and_end_granule(self):
        data = (Path(__file__).resolve().parents[2] / 'experiments/p0-media/tests/fixtures/audio-0_37.ogg').read_bytes()
        info = inspect_ogg(data)
        self.assertEqual(info['final_granule'], 17760)
        self.assertEqual(len(info['pages']), 3)
        self.assertGreater(info['audio_packets'], 2)
        self.assertEqual(info, inspect_ogg(bytes(data)))

    def test_corrupt_crc_truncated_page_and_partial_stream_are_rejected(self):
        data = (Path(__file__).resolve().parents[2] / 'experiments/p0-media/tests/fixtures/audio-0_37.ogg').read_bytes()
        broken = bytearray(data); broken[-1] ^= 1
        for bad in [bytes(broken), data[:-1], data[:58], b'x' * (1024 * 1024 + 1)]:
            with self.subTest(size=len(bad)), self.assertRaises(ValueError): inspect_ogg(bad)

    def test_recomputed_crc_does_not_hide_serial_sequence_or_continuation_errors(self):
        data = bytearray((Path(__file__).resolve().parents[2] / 'experiments/p0-media/tests/fixtures/audio-0_37.ogg').read_bytes())
        for offset, value in [(18, 7), (5, 3)]:
            bad = bytearray(data); bad[offset] = value
            end = 27 + bad[26] + sum(bad[27:27 + bad[26]])
            bad[22:26] = b'\0' * 4; struct.pack_into('<I', bad, 22, ogg_crc(bad[:end]))
            with self.assertRaises(ValueError): inspect_ogg(bytes(bad))

    def test_output_escape_and_existing_output_are_rejected_before_oracle_load(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); allowed = root / 'artifacts/p0'; allowed.mkdir(parents=True)
            fixture = allowed / 'fixture'; fixture.mkdir(); existing = allowed / 'existing'; existing.mkdir()
            with patch('p0_vorbis_tail.ROOT', root):
                self.assertEqual(checked_paths(fixture, allowed / 'new'), (fixture.resolve(), (allowed / 'new').resolve()))
                for output in [root / 'escape', existing]:
                    with self.assertRaises(ValueError): checked_paths(fixture, output)
                with self.assertRaises(ValueError): checked_paths(root, allowed / 'new')


if __name__ == '__main__': unittest.main()
