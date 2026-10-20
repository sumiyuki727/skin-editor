"""Native own-renderer smoke and optional manual Windows desktop acceptance session."""
import argparse
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import uuid

from check_p0 import ROOT
from check_p0_transport import digest, sources
from p0_process_metrics import environment_info

EVENTS = {"zx_presses", "pointer_presses", "focus_lost", "focus_returned", "focus_cancelled_draft", "escape_cancelled_draft",
          "ime_preedit", "ime_commit", "text_zx_presses", "text_zx_while_playing", "keyboard_game_accepts", "text_game_input_leaks",
          "fullscreen_entered", "fullscreen_returned", "drop_accepted", "drop_rejected"}
CLAIMS = {"font_legibility", "ime_candidate_commit_escape", "chinese_ime", "japanese_ime", "drop_thumbnail", "fullscreen_return"}
FIELDS = {"schema_version", "status", "source", "clock", "simulation_us", "paused", "history_nodes", "position", "draft_active",
          "accepted_game_inputs", "native_event_observations", "text_summary", "layouts_visited", "scale_observations", "pending_fullscreen",
          "human_confirmations", "unavailable", "errors", "limits", "elapsed_ms", "ended_by", "system_capture_used",
          "input_text_persisted", "foreign_file_bytes_read"}


def validate(result, source):
    if (set(result) != FIELDS or result.get("schema_version") != 1 or result.get("status") != "recorded" or result.get("source") != source
            or source not in {"native_smoke_no_manual_input", "native_user_session"}
            or any(result.get(k) is not False for k in ["system_capture_used", "input_text_persisted", "foreign_file_bytes_read"])
            or result.get("errors")):
        raise RuntimeError("Desktop record has wrong source, errors, missing fields or unexpected text/file payload")
    events, claims = result["native_event_observations"], result["human_confirmations"]
    if (set(events) != EVENTS or any(type(n) is not int or n < 0 for n in events.values())
            or set(claims) != CLAIMS or any(type(v) is not bool for v in claims.values())
            or set(result["text_summary"]) != {"characters", "matches_fixture"}
            or type(result["text_summary"]["matches_fixture"]) is not bool or not 0 <= result["text_summary"]["characters"] <= 64
            or len(result["layouts_visited"]) != 2 or any(type(v) is not bool for v in result["layouts_visited"])
            or set(result["unavailable"]) != {"japanese_ime", "cross_native_dpi"}
            or any(type(v) is not bool for v in result["unavailable"].values())):
        raise RuntimeError("Malformed desktop observation or human confirmation")
    if claims["japanese_ime"] and result["unavailable"]["japanese_ime"]:
        raise RuntimeError("Unavailable IME cannot also be confirmed")
    if result["history_nodes"] < 1 or len(result["position"]) != 2 or any(not math.isfinite(v) for v in result["position"]):
        raise RuntimeError("Invalid gesture/history observation")
    if len(result["scale_observations"]) > 64:
        raise RuntimeError("Unbounded desktop scale trace")
    for row in result["scale_observations"]:
        if (set(row) != {"native_pixels_per_point", "effective_pixels_per_point", "ui_zoom"}
                or any(not math.isfinite(v) or v <= 0 for v in row.values())
                or abs(row["native_pixels_per_point"]*row["ui_zoom"]-row["effective_pixels_per_point"]) > .001):
            raise RuntimeError("DPI/zoom observations are conflated or invalid")
    if source == "native_smoke_no_manual_input" and (any(claims.values()) or events["ime_commit"] or events["drop_accepted"]):
        raise RuntimeError("Smoke cannot carry manual acceptance")


def assess(result):
    """Coverage, not a product/physical-input pass based on a window opening."""
    manual = result["source"] == "native_user_session"
    e, c = result["native_event_observations"], result["human_confirmations"]
    native_scales = {round(s["native_pixels_per_point"], 4) for s in result["scale_observations"]}
    observed = {
        "drag_commit": manual and result["history_nodes"] > 1 and e["pointer_presses"] > 0,
        "escape_cancel": manual and e["escape_cancelled_draft"] > 0,
        "focus_cancel_return": manual and e["focus_cancelled_draft"] > 0 and e["focus_returned"] > 0,
        "keyboard_editor_isolation": manual and e["keyboard_game_accepts"] > 0 and e["text_zx_while_playing"] > 0 and e["text_game_input_leaks"] == 0,
        "ime_events": manual and e["ime_preedit"] > 0 and e["ime_commit"] > 0,
        "known_file_drop": manual and e["drop_accepted"] > 0,
        "fullscreen_return": manual and e["fullscreen_entered"] > 0 and e["fullscreen_returned"] > 0,
        "both_layouts": manual and all(result["layouts_visited"]),
        "native_scale_observed": manual and bool(native_scales),
        "native_dpi_transition": manual and len(native_scales) > 1,
    }
    confirmed = {
        "font_legibility": manual and c["font_legibility"],
        "chinese_ime": observed["ime_events"] and c["ime_candidate_commit_escape"] and c["chinese_ime"],
        "japanese_ime": observed["ime_events"] and c["ime_candidate_commit_escape"] and c["japanese_ime"] and not result["unavailable"]["japanese_ime"],
        "drop_thumbnail": observed["known_file_drop"] and c["drop_thumbnail"],
        "fullscreen_return": observed["fullscreen_return"] and c["fullscreen_return"],
    }
    return {"schema_version": 1, "scope": "native_user_session" if manual else "renderer_smoke_only",
            "observed": observed, "user_confirmed": confirmed,
            "unverified_observations": [k for k, v in observed.items() if not v],
            "unconfirmed_user_checks": [k for k, v in confirmed.items() if not v],
            "input_leak_detected": e["text_game_input_leaks"] > 0,
            "unavailable": result["unavailable"], "product_acceptance_passed": False}


def run(exe, interactive):
    output = ROOT/"artifacts/p0"/("desktop-user-" if interactive else "desktop-smoke-")
    output = output.with_name(output.name+uuid.uuid4().hex[:8])
    paths = sources()+[Path(__file__).resolve()]
    identity = {p.relative_to(ROOT).as_posix(): digest(p) for p in paths}
    executable_sha = digest(exe)
    command = [str(exe), "--interactive" if interactive else "--self-test", str(output)]
    log_path = output.with_suffix(".log")
    print(("Manual desktop session: " if interactive else "Desktop renderer smoke: ")+str(output), flush=True)
    with log_path.open("wb") as log, subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT) as process:
        deadline = time.monotonic()+(1800 if interactive else 45)
        while process.poll() is None:
            if time.monotonic()>deadline:
                process.kill(); process.wait()
                raise RuntimeError("Desktop recorder timeout: "+str(output))
            time.sleep(.25)
    if process.returncode or not (output/"result.json").is_file():
        raise RuntimeError("Desktop recorder failed: "+str(log_path))
    shutil.copyfile(log_path, output/"run.log")
    result = json.loads((output/"result.json").read_text(encoding="utf-8"))
    validate(result, "native_user_session" if interactive else "native_smoke_no_manual_input")
    if not interactive and not (output/"smoke-own-renderer.png").is_file():
        raise RuntimeError("Own renderer readback missing")
    if digest(exe) != executable_sha or any(digest(p) != identity[p.relative_to(ROOT).as_posix()] for p in paths):
        raise RuntimeError("Desktop executable/source changed during session")
    for p in paths:
        dest = output/"sources"/p.relative_to(ROOT)
        dest.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(p, dest)
    evidence = {"schema_version": 1, "executable_sha256": executable_sha, "source_sha256": identity,
                "output_sha256": {p.name: digest(p) for p in sorted(output.glob("*.png"))}, "environment": environment_info(),
                "note": "Own native window only; manual records contain counters/claims, not typed text or foreign file data. Smoke is not desktop acceptance."}
    write = lambda p, value: p.write_text(json.dumps(value, indent=2)+"\n", encoding="utf-8")
    write(output/"evidence.json", evidence); write(output/"assessment.json", assess(result))
    frozen_outputs = {p.name: digest(p) for p in output.iterdir() if p.is_file()}
    for location in [ROOT/("desktop-escape-"+uuid.uuid4().hex[:8]), output]:
        response = subprocess.run([str(exe), "--self-test", str(location)], cwd=ROOT, capture_output=True)
        if response.returncode == 0 or (location != output and location.exists()):
            raise RuntimeError("Desktop output boundary/overwrite rejection failed")
    if {p.name: digest(p) for p in output.iterdir() if p.is_file()} != frozen_outputs:
        raise RuntimeError("Overwrite rejection modified existing desktop evidence")
    print("Desktop record saved: "+str(output), flush=True)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--interactive", action="store_true", help="After smoke, show guided manual recorder; finish with its record button")
    args = parser.parse_args()
    if os.name != "nt":
        raise RuntimeError("Only Windows desktop sessions are currently verified")
    subprocess.run([sys.executable, str(ROOT/"tools/check_p0_gui.py")], check=True)
    exe = ROOT/"experiments/p0-egui/target/release/desktop-proof.exe"
    run(exe, False)
    if args.interactive:
        run(exe, True)


if __name__ == "__main__":
    main()
