"""Run the isolated P0 probe checks without downloading or installing anything."""

import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "experiments" / "p0-preview-contract" / "Cargo.toml"


def run(arguments, environment):
    subprocess.run(arguments, cwd=ROOT, env=environment, check=True)


def windows_linker(environment):
    """Discover a local MSVC installation; never change user/global settings."""
    if os.name != "nt" or environment.get("LIB"):
        return []
    program_files = Path(environment.get("ProgramFiles(x86)", "C:/Program Files (x86)"))
    vswhere = program_files / "Microsoft Visual Studio" / "Installer" / "vswhere.exe"
    if not vswhere.is_file():
        raise RuntimeError("MSVC environment missing; run from an x64 VS developer shell")
    installation = subprocess.check_output(
        [str(vswhere), "-latest", "-products", "*", "-requires",
         "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"],
        text=True,
    ).strip()
    if not installation:
        raise RuntimeError("No local MSVC x64 tools were found")
    toolsets = sorted((Path(installation) / "VC" / "Tools" / "MSVC").iterdir())
    toolsets = [path for path in toolsets if (path / "bin/Hostx64/x64/link.exe").is_file()]
    kits = program_files / "Windows Kits" / "10" / "Lib"
    sdk_versions = sorted(
        path for path in kits.iterdir()
        if (path / "um/x64/kernel32.lib").is_file() and (path / "ucrt/x64/ucrt.lib").is_file()
    )
    if not toolsets or not sdk_versions:
        raise RuntimeError("MSVC x64 libraries or Windows SDK libraries are missing")
    toolset, sdk = toolsets[-1], sdk_versions[-1]
    environment["LIB"] = ";".join(str(path) for path in (
        toolset / "lib/x64", sdk / "um/x64", sdk / "ucrt/x64",
    ))
    linker = (toolset / "bin/Hostx64/x64/link.exe").as_posix()
    return ["--config", 'target.x86_64-pc-windows-msvc.linker="' + linker + '"']


def check_isolation(cargo, environment):
    metadata = json.loads(subprocess.check_output(
        [cargo, "metadata", "--manifest-path", str(MANIFEST), "--offline", "--locked",
         "--no-deps", "--format-version", "1"],
        cwd=ROOT, env=environment, text=True,
    ))
    packages = metadata["packages"]
    if len(packages) != 1 or len(metadata["workspace_members"]) != 1:
        raise RuntimeError("The P0 probe must stay a separate single-package workspace")
    if packages[0]["dependencies"]:
        raise RuntimeError("The contract probe must have no product or third-party dependencies")
    for source in MANIFEST.parent.rglob("*.rs"):
        if "target" in source.relative_to(MANIFEST.parent).parts:
            continue
        text = source.read_text(encoding="utf-8")
        if re.search(r"\b(?:include|include_str|include_bytes)!", text):
            raise RuntimeError("External source/asset includes are not allowed: " + str(source))
        if re.search(r"^\s*#\s*\[\s*path\s*=", text, re.MULTILINE):
            raise RuntimeError("External Rust module paths are not allowed: " + str(source))
    print("PASS: isolated workspace; no product/third-party dependencies or external includes", flush=True)


def main():
    cargo = shutil.which("cargo")
    if cargo is None:
        raise RuntimeError("Cargo with rustc/rustfmt/clippy 1.80+ is required")
    environment = dict(os.environ)
    linker = windows_linker(environment)
    check_isolation(cargo, environment)
    common = ["--manifest-path", str(MANIFEST), "--offline", "--locked"]
    run([cargo, "fmt", "--manifest-path", str(MANIFEST), "--", "--check"], environment)
    run([cargo, *linker, "clippy", *common, "--all-targets", "--", "-D", "warnings"], environment)
    run([cargo, *linker, "test", *common], environment)
    run([cargo, *linker, "run", *common, "--quiet"], environment)


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print("P0 checks failed: " + str(error), file=sys.stderr)
        sys.exit(1)
