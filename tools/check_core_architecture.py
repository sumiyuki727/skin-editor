"""Small, explicit P1 package and source rules; not a Rust security sandbox."""

from pathlib import Path
import re


PACKAGES = {
    "skin-editor-domain": ("crates/domain", set()),
    "skin-editor-application": ("crates/application", {"skin-editor-domain"}),
}
FORBIDDEN_SOURCE = (
    ("core I/O or platform module", r"\bstd\s*::\s*(?:fs|net|process|env|thread|ffi)\b"),
    ("wall clock", r"\bstd\s*::\s*time\s*::\s*(?:Instant|SystemTime)\b"),
    ("opaque std import", r"\buse\s+std\s*(?:as\b|::\s*(?:\*|\{))"),
    ("extern crate alias", r"\bextern\s+crate\b"),
    ("external include or environment", r"\b(?:include|include_str|include_bytes|env|option_env)\s*!"),
    ("external module path", r"#\s*\[\s*path\s*="),
)


def validate_metadata(metadata, root):
    root = Path(root).resolve()
    errors = []
    packages = metadata.get("packages", [])
    by_name = {p["name"]: p for p in packages}
    if len(packages) != len(PACKAGES) or set(by_name) != set(PACKAGES):
        errors.append("P1 workspace must contain only domain and application; no third-party or P0 packages")
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


def source_issues(text):
    # Conservatively inspect source text, including comments. Keep opaque import
    # forms out of the small core; compiler visibility tests provide another gate.
    return [description for description, expression in FORBIDDEN_SOURCE if re.search(expression, text)]


def validate_sources(root):
    root = Path(root)
    errors = []
    for directory, _ in PACKAGES.values():
        base = root / directory
        entry = base / "src/lib.rs"
        if not entry.is_file() or "#![forbid(unsafe_code)]" not in entry.read_text(encoding="utf-8"):
            errors.append(directory + ": crate must forbid unsafe code")
        for source in base.rglob("*.rs"):
            for issue in source_issues(source.read_text(encoding="utf-8")):
                errors.append(source.relative_to(root).as_posix() + ": " + issue)
    return errors
