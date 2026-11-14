"""Explicit pure-core package/source rules; not a Rust security sandbox."""

from pathlib import Path
import re


PACKAGES = {
    "skin-editor-domain": ("crates/domain", set()),
    "skin-editor-application": ("crates/application", {"skin-editor-domain"}),
    "skin-editor-gameplay": ("crates/gameplay", set()),
}
FORBIDDEN_SOURCE = (
    ("core I/O or platform module", r"\bstd\s*::\s*(?:fs|io|os|net|process|env|thread|ffi)\b"),
    ("wall clock", r"\bstd\s*::\s*time\s*::\s*(?:Instant|SystemTime)\b"),
    ("opaque clock import", r"\buse\s+std\s*::\s*time\s*(?:as\b|;|::\s*(?:\*|\{))"),
    ("opaque std import", r"\buse\s+std\s*(?:as\b|::\s*(?:\*|\{))"),
    ("extern crate alias", r"\bextern\s+crate\b"),
    ("external include or environment", r"\b(?:include|include_str|include_bytes|env|option_env)\s*!"),
    ("external module path", r"#\s*\[\s*path\s*="),
)

# SaveCoord-Control-v1 owns fixed slots, with one reviewed box, a persistent
# port, one task and two independent cancellation blocks. Capture payloads use
# the separate pre-existing capture budget. This is a source gate, not a Rust
# allocator/security sandbox; size/copy/drop proofs and behavioral tests remain
# required when these allocation sites or their types change.
SAVE_CONTROL_ALLOCATIONS = {
    "src/save_coordination.rs": {"Box::new": 1, "Arc::new": 2},
    "src/save_coordination/work.rs": {"CaptureCancellation::default": 2},
}


def save_control_issues(text, relative):
    errors = []
    if re.search(r"\b(?:Vec|String|VecDeque|HashMap|BTreeMap|HashSet|BTreeSet|Rc)\b|\b(?:vec|format)\s*!", text):
        errors.append("save control storage must remain fixed and statically accounted")
    calls = re.findall(r"\b(Box|Arc|CaptureCancellation)\s*::\s*(new|default|from|try_new|new_uninit|new_zeroed|new_cyclic)\s*\(", text)
    allowed = SAVE_CONTROL_ALLOCATIONS.get(relative, {})
    counts = {}
    for owner, method in calls:
        key = owner + "::" + method
        counts[key] = counts.get(key, 0) + 1
    if any(count > allowed.get(key, 0) for key, count in counts.items()):
        errors.append("save control allocation requires a reviewed capacity/ownership proof")
    return errors


def validate_metadata(metadata, root):
    root = Path(root).resolve()
    errors = []
    packages = metadata.get("packages", [])
    by_name = {p["name"]: p for p in packages}
    if len(packages) != len(PACKAGES) or set(by_name) != set(PACKAGES):
        errors.append("core workspace must contain domain, application and independent gameplay; no third-party or P0 packages")
    if Path(metadata["workspace_root"]).resolve() != root:
        errors.append("unexpected workspace root")
    if set(metadata.get("workspace_members", [])) != {p["id"] for p in packages}:
        errors.append("all resolved packages must be explicit core workspace members")
    for name, package in by_name.items():
        if name not in PACKAGES:
            continue
        directory, allowed = PACKAGES[name]
        if Path(package["manifest_path"]).resolve() != root / directory / "Cargo.toml":
            errors.append(name + ": unexpected package path")
        if package.get("source") is not None or package.get("edition") != "2024" or package.get("rust_version") != "1.95":
            errors.append(name + ": expected local Rust 1.95 / edition 2024 package")
        if package.get("publish") != []:
            errors.append(name + ": product foundation must not be published")
        if package.get("features"):
            errors.append(name + ": new features require an explicit policy review")
        if any("custom-build" in t["kind"] or "proc-macro" in t["kind"] for t in package.get("targets", [])):
            errors.append(name + ": build scripts and proc macros are not core dependencies")
        actual = set()
        for dependency in package.get("dependencies", []):
            dependency_name = dependency["name"]
            actual.add(dependency_name)
            if dependency_name not in allowed:
                errors.append(name + ": forbidden dependency " + dependency_name)
                continue
            expected = root / PACKAGES[dependency_name][0]
            if (dependency.get("path") is None or Path(dependency["path"]).resolve() != expected
                    or dependency.get("source") is not None or dependency.get("kind") is not None
                    or dependency.get("target") is not None or dependency.get("optional")
                    or dependency.get("rename") is not None):
                errors.append(name + ": dependency must be direct, unconditional and local: " + dependency_name)
        if actual != allowed:
            errors.append(name + ": dependency set differs from the reviewed direction")
    return errors


def source_issues(text, package=None):
    # Conservatively inspect source text, including comments. Keep opaque import
    # forms out of the small core; compiler visibility tests provide another gate.
    errors = [description for description, expression in FORBIDDEN_SOURCE if re.search(expression, text)]
    if package == "skin-editor-gameplay" and re.search(
            r"\b(?:skin_editor_(?:domain|application|preview|audio|rendering|infrastructure|ui|desktop)|egui|wgpu|cpal)\s*(?:::|as\b)", text):
        errors.append("gameplay must not import project, adapter or device runtimes")
    return errors


def validate_sources(root):
    root = Path(root)
    errors = []
    for name, (directory, _) in PACKAGES.items():
        base = root / directory
        entry = base / "src/lib.rs"
        if not entry.is_file() or "#![forbid(unsafe_code)]" not in entry.read_text(encoding="utf-8"):
            errors.append(directory + ": crate must forbid unsafe code")
        for source in base.rglob("*.rs"):
            content = source.read_text(encoding="utf-8")
            for issue in source_issues(content, name):
                errors.append(source.relative_to(root).as_posix() + ": " + issue)
            relative = source.relative_to(base).as_posix()
            if name == "skin-editor-application" and (relative == "src/save_coordination.rs"
                    or relative.startswith("src/save_coordination/")):
                for issue in save_control_issues(content, relative):
                    errors.append(source.relative_to(root).as_posix() + ": " + issue)
            if (name == "skin-editor-gameplay" and relative.startswith("src/")
                    and relative != "src/storage.rs"
                    and re.search(r"\bVec\b|\bvec\s*!|\.\s*(?:reserve|try_reserve|resize|sort)(?:_exact|_with|_unstable)?\s*\(", content)):
                errors.append(source.relative_to(root).as_posix() + ": gameplay object buffers must use reviewed storage module")
    return errors
