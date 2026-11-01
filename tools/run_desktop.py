"""Run the local desktop with the same existing toolchain/linker as its checks."""
import argparse
import subprocess
import sys
from check_core import ROOT, tool_environment, linker_arguments

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolchain-dir")
    mode=parser.add_mutually_exclusive_group()
    mode.add_argument("--smoke-window", action="store_true", help="Open own window for three seconds, then exit")
    mode.add_argument("--candidate-audio", action="store_true", help="Explicit separated audio candidate; device remains opt-in in the UI")
    args=parser.parse_args()
    cargo,env=tool_environment(args.toolchain_dir)
    command=[cargo,*linker_arguments(env),"run","--manifest-path",str(ROOT/"desktop/Cargo.toml"),"--locked","--offline","-p","skin-editor-desktop"]
    if args.smoke_window:
        command.extend(["--", "--smoke-window"])
    elif args.candidate_audio:
        command.extend(["--", "--candidate-audio"])
    subprocess.run(command,cwd=ROOT,env=env,check=True)

if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        print("Desktop launch failed: "+str(error),file=sys.stderr)
        sys.exit(1)
