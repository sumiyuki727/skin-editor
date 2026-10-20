from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from p0_vulkan_memory import committed_overlap, committed_totals, select_type


class AllocationDiagnosticTests(unittest.TestCase):
    def test_memory_class_requires_allowed_bit_and_exact_properties(self):
        # Device-local alone, host coherent alone, unified, cached host.
        types = [1, 6, 7, 14]
        self.assertEqual(select_type(types, 15, 1, 2), 0)
        self.assertEqual(select_type(types, 14, 6, 1), 1)
        self.assertEqual(select_type(types, 8, 6, 1), 3)
        with self.assertRaises(ValueError):
            select_type(types, 4, 1, 2)
        with self.assertRaises(ValueError):
            select_type(types, 0, 6)

    def test_page_classes_exclude_reserved_and_preserve_unknown_scan(self):
        rows = [dict(base=0, size=4096, state=0x2000, type=0x20000),
                dict(base=4096, size=4096, state=0x1000, type=0x20000),
                dict(base=8192, size=8192, state=0x1000, type=0x40000),
                dict(base=16384, size=4096, state=0x1000, type=0x1000000)]
        snapshot = dict(complete=True, regions=rows)
        self.assertEqual(committed_totals(snapshot), dict(private=4096, mapped=8192, image=4096, other=0))
        self.assertEqual(committed_overlap(snapshot, 6144, 8192), dict(private=2048, mapped=6144, image=0, other=0))
        self.assertEqual(committed_overlap(snapshot, 20480, 4096), dict(private=0, mapped=0, image=0, other=0))
        snapshot["complete"] = False
        self.assertIsNone(committed_totals(snapshot))
        self.assertIsNone(committed_overlap(snapshot, 4096, 4096))


if __name__ == "__main__":
    unittest.main()
