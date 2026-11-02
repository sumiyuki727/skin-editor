"""Explicit desktop package edges; complements the stricter independent core gate."""
from pathlib import Path
import re

PACKAGES = {
    "skin-editor-domain": ("crates/domain", set()),
    "skin-editor-application": ("crates/application", {"skin-editor-domain"}),
    "skin-editor-audio": ("desktop/audio", {"skin-editor-domain"}),
    "skin-editor-rendering": ("desktop/rendering", {"skin-editor-domain", "wgpu"}),
    "skin-editor-preview": ("desktop/preview", set()),
    "skin-editor-infrastructure": ("desktop/infrastructure", {"skin-editor-application", "skin-editor-rendering", "skin-editor-audio", "image", "sha2", "cpal", "rtrb", "serde", "serde_json", "getrandom"}),
    "skin-editor-ui": ("desktop/ui", {"skin-editor-application", "egui"}),
    "skin-editor-desktop": ("desktop/host", {"skin-editor-application", "skin-editor-audio", "skin-editor-rendering", "skin-editor-preview", "skin-editor-infrastructure", "skin-editor-ui", "eframe", "pollster", "getrandom", "profiling", "tracing", "winit"}),
}
VERSIONS = {"wgpu": "=30.0.1", "egui": "=0.37.0", "eframe": "=0.37.0", "image": "=0.25.10", "sha2": "=0.10.9", "pollster": "=1.0.1", "getrandom": "=0.4.3", "cpal": "=0.18.2", "rtrb": "=0.4.0"}
FEATURES = {"wgpu": {"std", "wgsl"}, "egui": {"default_fonts"}, "eframe": {"default_fonts", "wgpu", "x11", "wayland", "system_fonts", "clipboard"}, "image": {"png"}, "sha2": set(), "pollster": set(), "getrandom": set(), "cpal": set(), "rtrb": set()}
VERSIONS.update(profiling='=1.0.18',tracing='=0.1.44',winit='=0.30.13')
FEATURES.update(profiling={'profile-with-tracing'},tracing={'std'},winit=set())
VERSIONS.update(serde='=1.0.229',serde_json='=1.0.154')
FEATURES.update(serde={'std'},serde_json={'std','float_roundtrip'})
HOST_FEATURES={'framework-trace':['dep:profiling','dep:tracing','dep:winit']}

def validate_metadata(metadata, root):
    root = Path(root).resolve()
    errors = []
    local = {p["name"]: p for p in metadata["packages"] if p.get("source") is None}
    if set(local) != set(PACKAGES):
        errors.append("unexpected local package, including P0 or unreviewed adapter")
    if Path(metadata["workspace_root"]).resolve() != root / "desktop":
        errors.append("desktop workspace root changed")
    members = {p["id"] for name, p in local.items() if name in PACKAGES and PACKAGES[name][0].startswith("desktop/")}
    if set(metadata["workspace_members"]) != members:
        errors.append("desktop must have exactly the reviewed explicit members")
    for name, package in local.items():
        if name not in PACKAGES:
            continue
        directory, allowed = PACKAGES[name]
        if Path(package["manifest_path"]).resolve() != root / directory / "Cargo.toml":
            errors.append(name + ": wrong source path")
        if package.get("features",{}) != (HOST_FEATURES if name=='skin-editor-desktop' else {}) or package.get("publish") != []:
            errors.append(name + ": unreviewed feature or publication")
        if any("custom-build" in t["kind"] or "proc-macro" in t["kind"] for t in package.get("targets", [])):
            errors.append(name + ": product build script/proc macro is unreviewed")
        dependencies = package.get("dependencies", [])
        if {d["name"] for d in dependencies} != allowed:
            errors.append(name + ": dependency direction changed")
        for d in dependencies:
            dep = d["name"]
            diagnostic = name=='skin-editor-desktop' and dep in {'profiling','tracing','winit'}
            if d.get("rename") or bool(d.get("optional")) != diagnostic or d.get("kind") or d.get("target"):
                errors.append(name + ": aliased/conditional dependency is unreviewed")
            if dep in PACKAGES:
                if d.get("path") is None or Path(d["path"]).resolve() != root / PACKAGES[dep][0] or d.get("source"):
                    errors.append(name + ": wrong local edge " + dep)
            elif dep in VERSIONS:
                if d.get("path") or d.get("req") != VERSIONS[dep] or not (d.get("source") or "").startswith("registry+") or set(d.get("features", [])) != FEATURES[dep]:
                    errors.append(name + ": unreviewed version/source/features " + dep)
                if dep in {"wgpu", "egui", "eframe", "image", "cpal", "rtrb", "profiling", "tracing", "winit", "serde", "serde_json"} and d.get("uses_default_features"):
                    errors.append(name + ": broad default features " + dep)
            else:
                errors.append(name + ": unknown direct dependency " + dep)
    return errors

def source_issues(module, source):
    errors = []
    if re.search(r"\b(?:include|include_str|include_bytes)\s*!|#\s*\[\s*path\s*=", source):
        errors.append("external source inclusion")
    if re.search(r"experiments[/\\]|osu-stable-leak|\.p0-tools", source):
        errors.append("external/P0 source dependency")
    if module in {"ui", "preview", "rendering", "audio"} and re.search(r"\bstd\s*::\s*(fs|net|process|env|thread|ffi)\b|\buse\s+std\s*::\s*\{", source):
        errors.append("I/O/platform capability outside adapter")
    if module in {"ui", "preview"} and re.search(r"\b(wgpu|eframe|skin_editor_rendering|skin_editor_infrastructure)\s*::", source):
        errors.append("runtime/GPU dependency in UI or simulation")
    if module in {"ui", "preview", "rendering", "audio"} and re.search(r"\b(cpal|rtrb)\s*::", source):
        errors.append("native audio boundary outside infrastructure")
    return errors

def validate_sources(root):
    root = Path(root)
    errors = []
    for directory in ("audio", "rendering", "preview", "infrastructure", "ui", "host"):
        for path in (root / "desktop" / directory / "src").rglob("*.rs"):
            errors.extend(str(path.relative_to(root)) + ": " + issue for issue in source_issues(directory, path.read_text(encoding="utf-8")))
        entry = root / "desktop" / directory / "src" / ("main.rs" if directory == "host" else "lib.rs")
        if not entry.is_file() or "#![forbid(unsafe_code)]" not in entry.read_text(encoding="utf-8"):
            errors.append(directory + ": missing unsafe prohibition")
    return errors
