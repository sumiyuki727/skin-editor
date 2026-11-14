import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from check_core_architecture import PACKAGES, save_control_issues, source_issues, validate_metadata, validate_sources


ROOT = Path(__file__).resolve().parents[1]


def metadata_fixture():
    packages = []
    for name, (directory, allowed) in PACKAGES.items():
        dependencies = [{"name": dependency, "path": str(ROOT / PACKAGES[dependency][0]),
                         "source": None, "kind": None, "target": None, "optional": False, "rename": None}
                        for dependency in sorted(allowed)]
        packages.append({"name": name, "id": name, "manifest_path": str(ROOT / directory / "Cargo.toml"),
                         "source": None, "edition": "2024", "rust_version": "1.95", "publish": [],
                         "features": {}, "targets": [{"kind": ["lib"]}], "dependencies": dependencies})
    return {"workspace_root": str(ROOT), "packages": packages, "workspace_members": [p["id"] for p in packages]}


class ArchitectureChecks(unittest.TestCase):
    def test_candidate_is_a_narrow_payload_island_not_a_control_container_exception(self):
        path = "src/save_coordination/candidate.rs"
        valid = "Vec::new(); value.try_reserve_exact(count); String::from_utf8(bytes); Arc::new(metadata);"
        self.assertEqual(save_control_issues(valid, path), [])
        for relative in ("src/save_coordination/nested/candidate.rs", "src/save_coordination/protocol.rs"):
            self.assertTrue(save_control_issues(valid, relative))
        for forbidden in ("Vec::with_capacity(n);", "String::new();", "BTreeMap::new();", "Box::new(0);",
                          "value.reserve(n);", "value.to_vec();", "vec![0];", "value.collect();",
                          "Arc::new(x); Arc::new(y);", "a.try_reserve_exact(n); b.try_reserve_exact(n);"):
            self.assertTrue(save_control_issues(forbidden, path), forbidden)

    def test_query_allocations_require_explicit_fixed_control_sites(self):
        path = "src/save_coordination/protocol.rs"
        self.assertEqual(save_control_issues("Arc::new(a); Arc::new(b); CaptureCancellation::default();", path), [])
        self.assertTrue(save_control_issues("Arc::new(a); Arc::new(b); Arc::new(c);", path))
        self.assertTrue(save_control_issues("CaptureCancellation::default(); CaptureCancellation::default();", path))
        self.assertTrue(save_control_issues("Vec::new();", path))

    def test_save_control_cannot_hide_variable_buffers_or_unbudgeted_allocations(self):
        for text in ("Vec::with_capacity(100);", "String::new();", "let q = vec![1];",
                     "let q = BTreeMap::new();", 'let q = format!("{}");'):
            self.assertTrue(save_control_issues(text, "src/save_coordination.rs"), text)
        self.assertEqual(save_control_issues("let x: [u64; 4] = [0; 4];", "src/save_coordination/values.rs"), [])
        self.assertTrue(save_control_issues("Arc::new(0);", "src/save_coordination/hidden.rs"))
        self.assertTrue(save_control_issues("Box::new(0); Box::new(0);", "src/save_coordination.rs"))
        self.assertTrue(save_control_issues("Arc::new_cyclic(f);", "src/save_coordination.rs"))
        self.assertEqual(save_control_issues("Box::new(0); Arc::new(0); Arc::new(0);", "src/save_coordination.rs"), [])

    def test_nested_save_control_modules_get_the_fixed_storage_gate(self):
        parent = ROOT / "artifacts/p1"
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="save-control-canary-", dir=parent) as directory:
            base = Path(directory)
            for relative, _ in PACKAGES.values():
                entry = base / relative / "src/lib.rs"
                entry.parent.mkdir(parents=True)
                entry.write_text("#![forbid(unsafe_code)]\n", encoding="utf-8")
            source = base / "crates/application/src/save_coordination/nested/buffer.rs"
            source.parent.mkdir(parents=True)
            source.write_text("let q = Vec::new();", encoding="utf-8")
            self.assertTrue(any("save control storage" in e for e in validate_sources(base)))
            source.write_text("let q = Box::new(0);", encoding="utf-8")
            self.assertTrue(any("capacity/ownership" in e for e in validate_sources(base)))
            source.write_text("let q = [0; 4];", encoding="utf-8")
            self.assertEqual(validate_sources(base), [])

    def test_gameplay_variable_buffers_cannot_bypass_the_reviewed_allocation_module(self):
        parent = ROOT / "artifacts/p1"
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="storage-canary-", dir=parent) as directory:
            base = Path(directory)
            for relative, _ in PACKAGES.values():
                entry = base / relative / "src/lib.rs"
                entry.parent.mkdir(parents=True)
                entry.write_text("#![forbid(unsafe_code)]\n", encoding="utf-8")
            source = base / "crates/gameplay/src/map/preparation.rs"
            source.parent.mkdir()
            for content in ("let data = Vec::new();", "let data = vec![0; 10000];", "data.resize(10000, 0);", "data.sort_unstable();"):
                source.write_text(content, encoding="utf-8")
                self.assertTrue(any("reviewed storage" in error for error in validate_sources(base)))
            source.write_text("let data = Buffer::new(10000);", encoding="utf-8")
            storage = base / "crates/gameplay/src/storage.rs"
            storage.write_text("let data = Vec::new();", encoding="utf-8")
            self.assertEqual(validate_sources(base), [])

    def test_nested_gameplay_frame_module_uses_the_same_source_boundary_checks(self):
        parent = ROOT / "artifacts/p1"
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="frame-source-canary-", dir=parent) as directory:
            base = Path(directory)
            for relative, _ in PACKAGES.values():
                entry = base / relative / "src/lib.rs"
                entry.parent.mkdir(parents=True)
                entry.write_text("#![forbid(unsafe_code)]\n", encoding="utf-8")
            frame = base / "crates/gameplay/src/run/frame.rs"
            frame.parent.mkdir()
            for source in ("use std::time::Instant;", "use skin_editor_application::ProjectSession;"):
                frame.write_text(source, encoding="utf-8")
                self.assertTrue(any("crates/gameplay/src/run/frame.rs" in error for error in validate_sources(base)))
            frame.write_text("use std::rc::Rc; use core::num::NonZeroU64;", encoding="utf-8")
            self.assertEqual(validate_sources(base), [])

    def test_pure_core_rejects_standard_io_platform_and_opaque_clock_imports(self):
        for source in ["use std::io;", "std::io::stdin();", "use std::os::windows::fs::MetadataExt;",
                       "use std::time as clock;", "use std::time;", "use std::time::{Instant as Clock};",
                       "use std::time::*;"]:
            self.assertTrue(source_issues(source, "skin-editor-gameplay"), source)
        self.assertEqual(source_issues("use std::time::Duration;", "skin-editor-gameplay"), [])

    def test_gameplay_is_independent_and_cannot_be_imported_by_project_packages(self):
        for edge in ("skin-editor-domain", "skin-editor-application", "skin-editor-preview", "wgpu"):
            metadata = metadata_fixture()
            gameplay = next(p for p in metadata["packages"] if p["name"] == "skin-editor-gameplay")
            gameplay["dependencies"].append({"name": edge})
            self.assertTrue(any("forbidden dependency" in e for e in validate_metadata(metadata, ROOT)), edge)
        for name in ("skin-editor-domain", "skin-editor-application"):
            metadata = metadata_fixture()
            package = next(p for p in metadata["packages"] if p["name"] == name)
            package["dependencies"].append({"name": "skin-editor-gameplay"})
            self.assertTrue(validate_metadata(metadata, ROOT), name)

    def test_gameplay_source_cannot_hide_runtime_or_project_imports(self):
        for source in ["use skin_editor_domain::SkinProject;", "use skin_editor_application as hidden;",
                       "use egui::Ui;", "use wgpu::Device;", "use std::time::Instant;", "use std::fs;"]:
            self.assertTrue(source_issues(source, "skin-editor-gameplay"), source)
        self.assertEqual(source_issues("use core::num::NonZeroU128; use skin_editor_gameplay::MapTime;", "skin-editor-gameplay"), [])

    def test_reviewed_graph_is_allowed(self):
        self.assertEqual(validate_metadata(metadata_fixture(), ROOT), [])

    def test_reverse_dependency_is_rejected(self):
        metadata = metadata_fixture()
        metadata["packages"][0]["dependencies"].append({"name": "skin-editor-application"})
        self.assertTrue(any("forbidden dependency" in e for e in validate_metadata(metadata, ROOT)))

    def test_external_and_p0_packages_are_rejected(self):
        for name in ("egui", "p0-preview-contract"):
            metadata = metadata_fixture()
            metadata["packages"][1]["dependencies"].append({"name": name})
            self.assertTrue(validate_metadata(metadata, ROOT))

    def test_domain_name_cannot_hide_an_external_path(self):
        metadata = metadata_fixture()
        metadata["packages"][1]["dependencies"][0]["path"] = str(ROOT / "experiments/p0-preview-contract")
        self.assertTrue(validate_metadata(metadata, ROOT))

    def test_optional_and_platform_dependencies_require_review(self):
        for field, value in (("optional", True), ("target", 'cfg(windows)'), ("kind", "build"), ("rename", "hidden")):
            metadata = metadata_fixture()
            metadata["packages"][1]["dependencies"][0][field] = value
            self.assertTrue(validate_metadata(metadata, ROOT), field)

    def test_build_scripts_and_features_are_rejected(self):
        for target in ("custom-build", "proc-macro"):
            metadata = metadata_fixture()
            metadata["packages"][0]["targets"].append({"kind": [target]})
            self.assertTrue(validate_metadata(metadata, ROOT))
        metadata = metadata_fixture()
        metadata["packages"][0]["features"] = {"gui": []}
        self.assertTrue(validate_metadata(metadata, ROOT))

    def test_io_includes_and_obscured_imports_are_rejected(self):
        for source in ["use std::fs;", "use std :: net;", "use std::{fs, fmt};", "use std as hidden;",
                       'include_str!("outside");', '#[path = "outside.rs"] mod x;', "use std::time::Instant;"]:
            self.assertTrue(source_issues(source), source)

    def test_pure_value_sources_are_allowed(self):
        self.assertEqual(source_issues("use core::num::NonZeroU128; use std::collections::BTreeMap;"), [])

    def test_real_cargo_graph_with_an_injected_ui_crate_is_rejected(self):
        cargo = shutil.which("cargo")
        self.assertIsNotNone(cargo, "Run through tools/check_core.py with the pinned toolchain")
        parent = (ROOT / "artifacts/p1").resolve()
        self.assertIn(ROOT, parent.parents)
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="dependency-canary-", dir=parent) as directory:
            base = Path(directory)
            (base / "Cargo.toml").write_text('[workspace]\nmembers=["crates/domain","crates/application","crates/gameplay","crates/ui"]\nresolver="3"\n', encoding="utf-8")
            for name, relative in [("skin-editor-domain", "domain"), ("skin-editor-application", "application"), ("skin-editor-gameplay", "gameplay"), ("forbidden-ui", "ui")]:
                folder = base / "crates" / relative
                (folder / "src").mkdir(parents=True)
                text = '[package]\nname="' + name + '"\nversion="0.1.0"\nedition="2024"\nrust-version="1.95"\npublish=false\n'
                if relative == "application":
                    text += '[dependencies]\nskin-editor-domain={path="../domain"}\nforbidden-ui={path="../ui"}\n'
                (folder / "Cargo.toml").write_text(text, encoding="utf-8")
                (folder / "src/lib.rs").write_text("", encoding="utf-8")
            result = subprocess.check_output([cargo, "metadata", "--manifest-path", str(base / "Cargo.toml"),
                                              "--offline", "--format-version", "1"], cwd=base, text=True)
            errors = validate_metadata(json.loads(result), base)
            self.assertTrue(any("forbidden dependency forbidden-ui" in e for e in errors))


if __name__ == "__main__":
    unittest.main()
