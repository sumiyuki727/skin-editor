"""Run the product core checks offline; never install tools or change global settings."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

from check_core_architecture import validate_metadata, validate_sources


ROOT = Path(__file__).resolve().parents[1]
RUST_VERSION = "1.95.0"


def tool_environment(directory=None):
    environment = dict(os.environ)
    directory = directory or environment.get("SKIN_EDITOR_RUST_TOOLCHAIN_DIR")
    if directory:
        binary_directory = Path(directory).resolve()
    else:
        rustup = shutil.which("rustup")
        if not rustup:
            raise RuntimeError("Install Rust 1.95.0 with rustfmt/clippy, or pass --toolchain-dir <bin-directory>")
        installed = subprocess.check_output([rustup, "toolchain", "list"], cwd=ROOT, text=True)
        if not any(line.split()[0].startswith(RUST_VERSION + "-") or line.split()[0] == RUST_VERSION
                   for line in installed.splitlines() if line.strip()):
            raise RuntimeError("Rust 1.95.0 is not registered; install it or pass --toolchain-dir. No download was attempted.")
        cargo_path = subprocess.check_output(
            [rustup, "which", "--toolchain", RUST_VERSION, "cargo"], cwd=ROOT, text=True,
        ).strip()
        binary_directory = Path(cargo_path).parent
    suffix = ".exe" if os.name == "nt" else ""
    for tool in ("cargo", "rustc", "rustdoc", "rustfmt", "cargo-clippy", "cargo-fmt"):
        if not (binary_directory / (tool + suffix)).is_file():
            raise RuntimeError("Missing tool component: " + tool)
    environment["PATH"] = str(binary_directory) + os.pathsep + environment.get("PATH", "")
    environment["RUSTC"] = str(binary_directory / ("rustc" + suffix))
    environment["RUSTDOC"] = str(binary_directory / ("rustdoc" + suffix))
    environment["CARGO_NET_OFFLINE"] = "true"
    for variable in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS",
                     "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"):
        if environment.get(variable):
            raise RuntimeError("Unset compiler override before reproducible checks: " + variable)
    version = subprocess.check_output([environment["RUSTC"], "--version"], env=environment, cwd=ROOT, text=True)
    if version.split()[1] != RUST_VERSION:
        raise RuntimeError("Expected rustc " + RUST_VERSION + "; toolchain was not changed")
    return str(binary_directory / ("cargo" + suffix)), environment


def linker_arguments(environment):
    if os.name != "nt" or environment.get("LIB"):
        return []
    program_files = Path(environment.get("ProgramFiles(x86)", "C:/Program Files (x86)"))
    vswhere = program_files / "Microsoft Visual Studio/Installer/vswhere.exe"
    if not vswhere.is_file():
        raise RuntimeError("MSVC x64 and Windows SDK are required; no tools were installed")
    installation = subprocess.check_output([
        str(vswhere), "-latest", "-products", "*", "-requires",
        "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath",
    ], text=True).strip()
    if not installation:
        raise RuntimeError("No MSVC x64 installation found")
    toolsets = [p for p in (Path(installation) / "VC/Tools/MSVC").iterdir()
                if (p / "bin/Hostx64/x64/link.exe").is_file()]
    sdks = [p for p in (program_files / "Windows Kits/10/Lib").iterdir()
            if (p / "um/x64/kernel32.lib").is_file() and (p / "ucrt/x64/ucrt.lib").is_file()]
    if not toolsets or not sdks:
        raise RuntimeError("MSVC x64 libraries or Windows SDK libraries are missing")
    toolset, sdk = sorted(toolsets)[-1], sorted(sdks)[-1]
    environment["LIB"] = ";".join(str(p) for p in (toolset / "lib/x64", sdk / "um/x64", sdk / "ucrt/x64"))
    linker = (toolset / "bin/Hostx64/x64/link.exe").as_posix()
    return ["--config", 'target.x86_64-pc-windows-msvc.linker="' + linker + '"']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolchain-dir", help="Existing Rust 1.95.0 bin directory; no installation or P0 import")
    parser.add_argument("--architecture-only", action="store_true", help="Check the actual Cargo graph and core sources")
    options = parser.parse_args()
    cargo, environment = tool_environment(options.toolchain_dir)
    common = ["--manifest-path", str(ROOT / "Cargo.toml"), "--offline", "--locked"]
    metadata = json.loads(subprocess.check_output([
        cargo, "metadata", *common, "--format-version", "1", "--all-features",
    ], cwd=ROOT, env=environment, text=True))
    errors = validate_metadata(metadata, ROOT) + validate_sources(ROOT)
    if errors:
        raise RuntimeError("\n".join(errors))
    print("PASS: three pure core packages; application -> domain; gameplay independent; no third-party/P0 dependencies", flush=True)
    if options.architecture_only:
        return
    linker = linker_arguments(environment)
    steps = [
        [sys.executable, "-m", "unittest", "discover", "-s", "tools", "-p", "test_check_*.py", "-v"],
        [cargo, "fmt", "--all", "--", "--check"],
        [cargo, *linker, "clippy", *common, "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"],
        [cargo, *linker, "test", *common, "--workspace", "--all-features"],
        [cargo, *linker, "build", *common, "--workspace", "--release"],
    ]
    for arguments in steps:
        subprocess.run(arguments, cwd=ROOT, env=environment, check=True)
    print("PASS: core checks completed without GUI windows, devices, network or P0 packages", flush=True)


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print("Core checks failed: " + str(error), file=sys.stderr)
        sys.exit(1)
