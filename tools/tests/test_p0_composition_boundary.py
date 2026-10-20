import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_p0_composition import check_cpu_boundary


class CompositionBoundaryTests(unittest.TestCase):
    def test_accepts_pure_math_and_detects_reverse_or_platform_imports(self):
        check_cpu_boundary("use std::array; fn pixel() {} // wgpu::Queue is an adapter concern")
        for source in ("use eframe::egui;", "fn draw() { wgpu::Device::new(); }",
                       "use std::fs;", "use std::{collections::BTreeMap, net::TcpStream};",
                       "std::thread::spawn(worker);"):
            with self.subTest(source=source), self.assertRaises(RuntimeError):
                check_cpu_boundary(source)


if __name__ == "__main__":
    unittest.main()
