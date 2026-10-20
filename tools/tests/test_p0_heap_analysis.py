import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from p0_heap_analysis import summarize_heap_lines


class HeapAnalysisTests(unittest.TestCase):
    def test_moving_realloc_completion_keeps_internal_allocation_stack(self):
        rows = ["HeapAlloc, 1, Unknown, 4, h, 0x10, 0x20, X\n",
                "HeapAlloc, 2, Unknown, 4, h, 0x11, 0x30, X\n",
                "Stack, -200, 4, 1, 0x2001, Unknown\n",
                "HeapFree, 3, Unknown, 4, h, 0x10, 0, X\n",
                "HeapRealloc, 3, Unknown, 4, h, 0x11, 0x10, 0x30, 0x20, X\n"]
        result = summarize_heap_lines(rows, [{"name": "nvoglv64.dll", "base": 0x2000, "size": 100}])
        self.assertEqual(result["observed_requested_last_bytes"], 48)
        self.assertEqual(result["observed_requested_peak_bytes"], 80)
        self.assertEqual(result["observed_last_by_origin"]["nvidia-driver"], 48)
        self.assertEqual(result["unmatched_free_or_realloc"], 0)
        self.assertEqual(result["address_collisions"], 0)
        self.assertEqual(result["coalesced_moving_realloc_completions"], 1)

    def test_reallocation_free_and_unknown_pretrace_blocks(self):
        rows = ["HeapFree, 1, Unknown, 4, h, 0x99, 0, X\n",
                "HeapAlloc, 2, Unknown, 4, h, 0x10, 0x20, X\n",
                "HeapRealloc, 3, Unknown, 4, h, 0x11, 0x10, 0x30, 0x20, X\n",
                "HeapFree, 4, Unknown, 4, h, 0x11, 0, X\n"]
        result = summarize_heap_lines(rows)
        self.assertEqual(result["observed_requested_peak_bytes"], 48)
        self.assertEqual(result["observed_requested_last_bytes"], 0)
        self.assertEqual(result["unmatched_free_or_realloc"], 1)

    def test_module_origin_skips_system_and_rejects_other_thread_stack(self):
        modules = [{"name": "ntdll.dll", "base": 0x1000, "size": 100},
                   {"name": "nvoglv64.dll", "base": 0x2000, "size": 100}]
        rows = ["HeapAlloc, 2, Unknown, 4, h, 0x10, 0x20, X\n",
                "Stack, -200, 5, 1, 0x2001, Unknown\n",
                "Stack, -200, 4, 1, 0x1001, Unknown\n",
                "Stack, -200, 4, 2, 0x2001, Unknown\n"]
        result = summarize_heap_lines(rows, modules)
        self.assertEqual(result["observed_last_by_origin"], {"nvidia-driver": 32})

    def test_failed_realloc_preserves_old_block_and_destroy_releases_it(self):
        rows = ["HeapAlloc, 2, Unknown, 4, h, 0x10, 0x20, X\n",
                "HeapRealloc, 3, Unknown, 4, h, 0x0, 0x10, 0x30, 0x20, X\n",
                "HeapDestroy, 4, Unknown, 4, h\n"]
        result = summarize_heap_lines(rows)
        self.assertEqual(result["failed_allocations"], 1)
        self.assertEqual(result["observed_requested_peak_bytes"], 32)
        self.assertEqual(result["observed_requested_last_bytes"], 0)

    def test_nvidia_overlay_is_not_classified_as_driver(self):
        rows = ["HeapAlloc, 2, Unknown, 4, h, 0x10, 0x20, X\n",
                "Stack, -200, 4, 1, 0x2001, Unknown\n"]
        result = summarize_heap_lines(rows, [{"name": "nvspcap64.dll", "base": 0x2000, "size": 100}])
        self.assertEqual(result["observed_last_by_origin"], {"native:nvspcap64.dll": 32})

    def test_reported_segments_are_separate_from_requested_allocations(self):
        rows = ["HeapExtend, 1, Unknown, 4, h, 0x50, 4096, 1024, 8192, 16384, 1, 128\n",
                "HeapAlloc, 2, Unknown, 4, h, 0x10, 0x20, X\n"]
        result = summarize_heap_lines(rows)
        segment = result["samples"][0]["reported_heap_segments"]["h"]
        self.assertEqual(segment["committed_bytes"], 8192)
        self.assertEqual(segment["allocated_counter_bytes"], 128)
        self.assertEqual(result["observed_requested_peak_bytes"], 32)


if __name__ == "__main__":
    unittest.main()
