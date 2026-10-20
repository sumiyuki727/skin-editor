from array import array
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_compressed import compare_float_files, check_compressed_features
from p0_compressed_fixture import make_fixture
from p0_stable_batch import prepare


class CompressedEvidenceTests(unittest.TestCase):
    def test_independent_tail_disagreement_is_not_silently_trimmed(self):
        with tempfile.TemporaryDirectory() as tmp:
            a = Path(tmp) / 'a'; b = Path(tmp) / 'b'
            a.write_bytes(array('f', [0., .1, .2, .3]).tobytes()); b.write_bytes(array('f', [0., .1]).tobytes())
            r = compare_float_files(a, b)
            self.assertFalse(r['frames_match']); self.assertEqual(r['uncompared_tail_frames'], 1); self.assertEqual(r['compared_samples'], 2)

    def test_nonfinite_and_corrupt_float_data_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            a = Path(tmp) / 'a'; b = Path(tmp) / 'b'; b.write_bytes(array('f', [0., .1]).tobytes())
            a.write_bytes(array('f', [float('nan'), .1]).tobytes())
            with self.assertRaises(RuntimeError): compare_float_files(a, b)
            a.write_bytes(b'bad')
            with self.assertRaises(RuntimeError): compare_float_files(a, b)

    def test_only_reviewed_optional_codec_features_are_allowed(self):
        f = {'compressed': ['symphonia/mp3', 'symphonia/ogg', 'symphonia/vorbis']}; check_compressed_features(f)
        with self.assertRaises(RuntimeError): check_compressed_features({**f, 'default': ['compressed']})
        with self.assertRaises(RuntimeError): check_compressed_features({'compressed': ['symphonia/all']})

    def test_both_codec_references_have_distinct_files_and_generation_never_overwrites(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); encoder = root / 'fake-encoder'; encoder.write_bytes(b'test')
            def run(command, stdout=None, **kwargs):
                if stdout is None: Path(command[-1]).write_bytes(b'independent fixture placeholder')
                else: stdout.write(array('f', [0., .1]).tobytes())
            with patch('p0_compressed_fixture.subprocess.run', side_effect=run), patch('p0_compressed_fixture.subprocess.check_output', return_value='FFmpeg test stub\n'):
                m = make_fixture(root / 'fixture', (.01,), encoder)
                refs = [x['reference'] for x in m['items']]
                self.assertEqual(len(refs), len(set(refs))); self.assertEqual(len(refs), 2)
                with self.assertRaises(FileExistsError): make_fixture(root / 'fixture', (.01,), encoder)


class StableBatchTests(unittest.TestCase):
    def context(self, root):
        allowed = root / 'artifacts/p0'; allowed.mkdir(parents=True)
        return allowed

    def test_packages_have_matching_baselines_and_remain_unexecuted(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); allowed = self.context(root)
            with patch('p0_stable_batch.ROOT', root), patch('p0_evidence.ARTIFACTS', allowed):
                m = prepare(allowed / 'batch')
            self.assertEqual(m['compatibility_cases_executed'], 0); self.assertFalse(m['client_installation_modified'])
            out = allowed / 'batch'; skins = out / 'skins'
            for name in ('hitcircle.png', 'cursor.png', 'default-1.png'):
                self.assertEqual((skins / 'baseline' / name).read_bytes(), (skins / 'transparent' / name).read_bytes())
            self.assertFalse((skins / 'missing/hitcircleoverlay.png').exists())
            self.assertNotEqual((skins / 'baseline/hitcircleoverlay.png').read_bytes(), (skins / 'transparent/hitcircleoverlay.png').read_bytes())
            packages = list((out / 'packages').iterdir()); self.assertEqual(len(packages), 4)
            for package in packages:
                with zipfile.ZipFile(package) as z:
                    self.assertTrue(all('/' not in n and '..' not in n for n in z.namelist()))
            osu = (out / 'beatmap/P0 First Batch.osu').read_text()
            self.assertIn('23000,1,0', osu); self.assertIn('ApproachRate:0', osu)
            observation = json.loads((out / 'observation-template.json').read_text()); self.assertIsNone(observation['compatibility_passed'])

    def test_output_escape_overwrite_and_failed_preparation_preserve_original(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); allowed = self.context(root); target = allowed / 'existing'; target.mkdir(); (target / 'keep').write_text('original')
            with patch('p0_stable_batch.ROOT', root), patch('p0_evidence.ARTIFACTS', allowed):
                with self.assertRaises(ValueError): prepare(root / 'escape')
                with self.assertRaises(ValueError): prepare(target)
                with patch('p0_stable_batch.zipfile.ZipFile', side_effect=OSError('injected failure')):
                    with self.assertRaises(OSError): prepare(allowed / 'failed')
            self.assertEqual((target / 'keep').read_text(), 'original'); self.assertFalse((allowed / 'failed').exists())


if __name__ == '__main__': unittest.main()
