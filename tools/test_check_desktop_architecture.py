import copy
from pathlib import Path
import unittest
from check_desktop_architecture import PACKAGES, VERSIONS, FEATURES, HOST_FEATURES, validate_metadata, source_issues

ROOT = Path(__file__).resolve().parents[1]
def fixture():
    packages = []
    for name, (directory, edges) in PACKAGES.items():
        deps=[]
        for edge in edges:
            deps.append(dict(name=edge, path=str(ROOT / PACKAGES[edge][0]) if edge in PACKAGES else None,
                             source=None if edge in PACKAGES else "registry+https://github.com/rust-lang/crates.io-index",
                             req=VERSIONS.get(edge, "*"), features=list(FEATURES.get(edge, set())), uses_default_features=False,
                             optional=name=='skin-editor-desktop' and edge in {'profiling','tracing','winit'}))
        packages.append(dict(name=name, id=name, source=None, manifest_path=str(ROOT / directory / "Cargo.toml"),
                             publish=[], features=copy.deepcopy(HOST_FEATURES) if name=='skin-editor-desktop' else {}, targets=[], dependencies=deps))
    return dict(packages=packages, workspace_root=str(ROOT / "desktop"), workspace_members=[p["id"] for p in packages if p["name"] not in {"skin-editor-domain", "skin-editor-application"}])

class Boundaries(unittest.TestCase):
    def test_project_storage_dependencies_are_adapter_only_and_pinned(self):
        for name in ["skin-editor-application", "skin-editor-domain", "skin-editor-ui", "skin-editor-preview", "skin-editor-audio", "skin-editor-rendering"]:
            for edge in ["serde", "serde_json", "getrandom"]:
                data=fixture();package=next(p for p in data['packages'] if p['name']==name)
                package['dependencies'].append(dict(name=edge,req=VERSIONS[edge],features=list(FEATURES[edge]),source='registry+https://github.com/rust-lang/crates.io-index'))
                self.assertTrue(validate_metadata(data,ROOT))
        for field,value in [('req','*'),('uses_default_features',True),('features',['std','unbounded_depth'])]:
            data=fixture();dep=next(d for p in data['packages'] for d in p['dependencies'] if d['name']=='serde_json')
            dep[field]=value;self.assertTrue(validate_metadata(data,ROOT))
    def test_framework_trace_is_explicit_host_only_and_default_off(self):
        for change in [lambda p:p['features'].update(default=['framework-trace']),lambda p:p['features']['framework-trace'].append('eframe/accesskit'),
            lambda p:next(d for d in p['dependencies'] if d['name']=='tracing').update(optional=False),
            lambda p:next(d for d in p['dependencies'] if d['name']=='winit').update(optional=False),
            lambda p:next(d for d in p['dependencies'] if d['name']=='winit').update(uses_default_features=True),
            lambda p:next(d for d in p['dependencies'] if d['name']=='winit').update(features=['x11']),
            lambda p:next(d for d in p['dependencies'] if d['name']=='profiling').update(uses_default_features=True)]:
            data=fixture();host=next(p for p in data['packages'] if p['name']=='skin-editor-desktop');change(host)
            self.assertTrue(validate_metadata(data,ROOT))
    def test_native_audio_dependencies_remain_in_infrastructure(self):
        for name in ["skin-editor-ui", "skin-editor-application", "skin-editor-audio", "skin-editor-preview", "skin-editor-rendering"]:
            for edge in ["cpal", "rtrb"]:
                data=fixture()
                package=next(p for p in data["packages"] if p["name"] == name)
                package["dependencies"].append(dict(name=edge, req=VERSIONS[edge], features=[], source="registry+https://github.com/rust-lang/crates.io-index"))
                self.assertTrue(validate_metadata(data, ROOT))
    def test_native_audio_versions_defaults_and_source_escapes_fail(self):
        for edge in ["cpal", "rtrb"]:
            for field,value in [("req", "*"), ("uses_default_features", True), ("features", ["asio"])]:
                data=fixture()
                dep=next(d for p in data["packages"] for d in p["dependencies"] if d["name"] == edge)
                dep[field]=value
                self.assertTrue(validate_metadata(data,ROOT))
            for module in ["audio", "ui", "preview", "rendering"]:
                self.assertTrue(source_issues(module, edge+"::Stream;"))
    def test_reviewed_graph(self):
        self.assertEqual(validate_metadata(fixture(), ROOT), [])
    def test_reverse_gpu_alias_and_optional_edges_fail(self):
        for field, value in [("name", "wgpu"), ("rename", "hidden_writer"), ("optional", True), ("path", str(ROOT / "experiments/p0"))]:
            data=copy.deepcopy(fixture())
            ui=next(p for p in data["packages"] if p["name"] == "skin-editor-ui")
            ui["dependencies"][0][field]=value
            self.assertTrue(validate_metadata(data,ROOT), field)
    def test_versions_features_and_unregistered_packages_fail(self):
        for field, value in [("req", "*"), ("features", ["jpeg"]), ("uses_default_features", True)]:
            data=fixture()
            dep=next(d for p in data["packages"] for d in p["dependencies"] if d["name"] == "image")
            dep[field]=value
            self.assertTrue(validate_metadata(data,ROOT),field)
        data=fixture(); data["packages"].append(dict(name="p0-hidden",source=None))
        self.assertTrue(validate_metadata(data,ROOT))
    def test_source_includes_io_and_runtime_escape_fail(self):
        for source in ['include_str!("../../experiments/x");', 'std::fs::read("a");', 'eframe::wgpu::Device', 'use std::{fs as storage};']:
            self.assertTrue(source_issues("ui",source), source)
        self.assertEqual(source_issues("ui",'egui::Ui; skin_editor_application::Command;'), [])

    def test_audio_keeps_io_devices_and_visual_adapters_out(self):
        for source in ['std::fs::read("sound");', 'std::thread::spawn(work);', 'use std::{net};']:
            self.assertTrue(source_issues("audio", source), source)
        self.assertEqual(source_issues("audio", 'skin_editor_domain::AudioComponent;'), [])

    def test_audio_edges_cannot_bypass_application_or_import_platforms(self):
        for name, edge in [("skin-editor-ui", "skin-editor-audio"), ("skin-editor-application", "skin-editor-audio"), ("skin-editor-audio", "skin-editor-infrastructure"), ("skin-editor-audio", "skin-editor-application"), ("skin-editor-ui", "skin-editor-infrastructure")]:
            data=fixture()
            package=next(p for p in data["packages"] if p["name"] == name)
            package["dependencies"].append(dict(name=edge, path=str(ROOT / PACKAGES[edge][0])))
            self.assertTrue(validate_metadata(data, ROOT))

if __name__ == "__main__":
    unittest.main()
