from array import array
import copy
import io
import json
from pathlib import Path
from types import SimpleNamespace
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_compressed import NativeReference
from check_p0_combined_soak import validate_decoder_mode
from check_p0_compressed_soak import CODECS, REQUIRED_BUDGETS, digest, load_audio, prepare_audio, result_status


class CompressedSoakTests(unittest.TestCase):
    def matrix(self):
        return [dict(repeat=n, codec=c, status='validated', gpu_samples_available=True, budgets={k: True for k in REQUIRED_BUDGETS})
                for n in range(2) for c in CODECS]

    def test_matrix_requires_all_distinct_repetitions_and_all_budget_fields(self):
        rows = self.matrix()
        self.assertEqual(result_status(rows, 2, False), 'passed_scoped_combined_budgets')
        for bad in [rows[:-1], rows + [rows[0]], [rows[0]] * 6]:
            with self.assertRaises(RuntimeError): result_status(bad, 2, False)
        for change in [False, 'true', None]:
            bad = copy.deepcopy(rows); bad[0]['budgets']['playing_working_set_le_512_mib'] = change
            self.assertEqual(result_status(bad, 2, False), 'failed_budget')
        rows[0]['budgets'].pop('closed_observed_at_least_60_seconds')
        self.assertEqual(result_status(rows, 2, False), 'failed_budget')
        rows = self.matrix(); rows[0]['gpu_samples_available'] = False
        self.assertEqual(result_status(rows, 2, False), 'incomplete_gpu_measurement')

    def test_smoke_has_no_budget_claim_and_feature_mode_cannot_be_a_wav_stub(self):
        rows = self.matrix()
        for row in rows: row['budgets'] = None
        self.assertEqual(result_status(rows, 2, True), 'passed_smoke_not_budgeted')
        self.assertEqual(result_status(rows, 2, False), 'failed_budget')
        rows[0]['budgets'] = {'passed': True}
        with self.assertRaises(RuntimeError): result_status(rows, 2, True)
        validate_decoder_mode({'compressed_feature_enabled': True}, True)
        validate_decoder_mode({'compressed_feature_enabled': False}, False)
        for record in [{}, {'compressed_feature_enabled': False}, {'compressed_feature_enabled': 1}]:
            with self.assertRaises(RuntimeError): validate_decoder_mode(record, True)

    def test_reference_can_check_late_frames_without_retaining_the_full_file(self):
        # Sparse read-only source model: only the requested zero PCM is produced.
        class Zeros(io.BytesIO):
            def seek(self, pos): self.position = pos
            def read(self, count): return bytes(count)
        class Sparse:
            def stat(self): return SimpleNamespace(st_size=48000 * 600 * 8)
            def open(self, mode): return Zeros()
        reference = NativeReference(Sparse(), 48000 * 600)
        packet = dict(source_frame=48000 * 299, frames=2, hash=0x88201fb960ff6465,
                      nonzero_samples=0, peak=0., probe=[0.] * 8)
        reference.check(packet)
        for start in [-1, 48000 * 600, True]:
            with self.assertRaises(RuntimeError): reference.check({**packet, 'source_frame': start})
        with self.assertRaises(RuntimeError): reference.check({**packet, 'hash': 1})

    def test_reference_rejects_wrong_length_and_unbounded_duration(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'pcm'; path.write_bytes(array('f', [0., 0.]).tobytes())
            NativeReference(path, 1)
            for frames in [0, True, 48000 * 600 + 1, 2]:
                with self.assertRaises(RuntimeError): NativeReference(path, frames)

    def fixture(self, root):
        audio = root / 'artifacts/p0/audio'; audio.mkdir(parents=True)
        (audio / 'manifest.json').write_text('{"source":"independent fixture"}')
        decoder = {'experiments/p0-media/src/compressed.rs': 'decoder-identity'}
        rows = []
        for codec in CODECS:
            source = audio / ('source.' + codec); source.write_bytes(codec.encode())
            directory = root / 'artifacts/p0' / ('pull-' + codec); directory.mkdir()
            reference = directory / 'decoded.f32'; reference.write_bytes(bytes(480 * 8))
            proof_path = directory / 'evidence.json'
            proof = {'status':'validated', 'mode':'pull', 'codec':codec, 'source_file_sha256':digest(source),
                     'source_sha256':decoder, 'comparison':{'frames_match':True,'samples_outside_tolerance':0}}
            proof_path.write_text(json.dumps(proof))
            rows.append({'codec':codec, 'source':source.name, 'source_sha256':digest(source),
                         'linear_reference':reference.relative_to(root).as_posix(), 'linear_reference_sha256':digest(reference),
                         'pull_evidence':proof_path.relative_to(root).as_posix(), 'pull_evidence_sha256':digest(proof_path)})
        manifest = {'schema_version':1, 'generator':'verified_compressed_soak_audio_v1', 'seconds':.01,
                    'rate':48000, 'channels':2, 'frames':480, 'source_manifest_sha256':digest(audio / 'manifest.json'),
                    'decoder_source_sha256':decoder, 'entries':rows}
        (audio / 'soak.json').write_text(json.dumps(manifest))
        return audio, manifest, decoder

    def test_sources_and_independent_full_pcm_proofs_are_required_before_device_work(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); audio, manifest, decoder = self.fixture(root)
            with patch('check_p0_compressed_soak.ROOT', root), patch('check_p0_compressed_soak.decoder_identities', return_value=decoder):
                self.assertEqual(set(load_audio(audio, .01)), set(CODECS))
                with self.assertRaises(RuntimeError): load_audio(audio)  # Native soak requires 600 s.
                for field, value in [('generator','unverified'),('frames',481),('decoder_source_sha256',{})]:
                    bad = copy.deepcopy(manifest); bad[field] = value
                    (audio / 'soak.json').write_text(json.dumps(bad))
                    with self.assertRaises(RuntimeError): load_audio(audio, .01)
                (audio / 'soak.json').write_text(json.dumps(manifest))
                (audio / 'source.mp3').write_bytes(b'changed')
                with self.assertRaises(RuntimeError): load_audio(audio, .01)

    def test_path_escape_mismatched_decoder_and_trimmed_reference_cannot_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); audio, manifest, decoder = self.fixture(root)
            with patch('check_p0_compressed_soak.ROOT', root), patch('check_p0_compressed_soak.decoder_identities', return_value=decoder):
                for field, value in [('source','../pull-wav/decoded.f32'),('linear_reference',str(root / 'outside'))]:
                    (root / 'outside').write_bytes(b'private placeholder')
                    bad = copy.deepcopy(manifest); bad['entries'][0][field] = value
                    (audio / 'soak.json').write_text(json.dumps(bad))
                    with self.assertRaises(RuntimeError): load_audio(audio, .01)
                (audio / 'soak.json').write_text(json.dumps(manifest))
                row = manifest['entries'][0]; proof_path = root / row['pull_evidence']; proof = json.loads(proof_path.read_text())
                for field, value in [('status','rejected'),('source_sha256',{}),('comparison',{'frames_match':False,'samples_outside_tolerance':0})]:
                    bad = copy.deepcopy(proof); bad[field] = value; proof_path.write_text(json.dumps(bad))
                    row['pull_evidence_sha256'] = digest(proof_path); (audio / 'soak.json').write_text(json.dumps(manifest))
                    with self.assertRaises(RuntimeError): load_audio(audio, .01)

    def test_preparation_refuses_escape_and_existing_directory_without_starting_encoder(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); (root / 'artifacts/p0').mkdir(parents=True)
            with patch('check_p0_compressed_soak.ROOT', root), patch('check_p0_compressed_soak.make_fixture') as make:
                with self.assertRaises(RuntimeError): prepare_audio(root / 'outside')
                existing = root / 'artifacts/p0/existing'; existing.mkdir(); (existing / 'keep').write_text('original')
                with self.assertRaises(FileExistsError): prepare_audio(existing)
                self.assertEqual((existing / 'keep').read_text(), 'original')
                make.assert_not_called()


if __name__ == '__main__': unittest.main()
