import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from p0_lifetime_analysis import release_samples, upload_receipts


class LifetimeTests(unittest.TestCase):
    def test_release_alignment_does_not_use_pre_release_or_distant_samples(self):
        rows=[dict(elapsed_seconds=t,private_bytes=None) for t in (1.9,2.05,2.6,5.1)]
        result=release_samples(rows,2,offsets=(0,.1,.5,1,3))
        self.assertAlmostEqual(result[0]["observed_offset_seconds"],.05)
        self.assertIsNone(result[1]["sample"])
        self.assertAlmostEqual(result[2]["observed_offset_seconds"],.6)
        self.assertIsNone(result[3]["sample"])
        self.assertIsNone(result[4]["sample"]["private_bytes"])

    def test_multiple_images_share_receipt_and_payload_is_not_double_counted(self):
        events=[dict(kind="UploadQueued",ticket=1,bytes=4,elapsed_seconds=0),
                dict(kind="UploadQueued",ticket=1,bytes=4,elapsed_seconds=.001),
                dict(kind="UploadComplete",detail=dict(ticket=1,bytes=8,callback_elapsed_seconds=.02))]
        result=upload_receipts(events)
        self.assertEqual(result["peak_payload_bytes"],8)
        self.assertEqual(result["remaining_payload_bytes"],0)
        self.assertEqual(result["queued_to_callback_wall_ms"],[20])
        with self.assertRaises(ValueError):upload_receipts(events+[events[-1]])

    def test_missing_completion_remains_outstanding(self):
        result=upload_receipts([dict(kind="UploadQueued",ticket=3,bytes=8,elapsed_seconds=1)])
        self.assertEqual(result["remaining_payload_bytes"],8)
        self.assertEqual(result["queued_to_callback_wall_ms"],[])


if __name__=="__main__":unittest.main()
