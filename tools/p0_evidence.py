"""P0 file/log inspection, original fixtures, and explicit local screenshot intake."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import tempfile
from datetime import datetime, timezone
import wave
import zlib

ROOT = Path(__file__).resolve().parents[1]
ARTIFACTS = ROOT / "artifacts/p0"
LOGS = ("runtime.log", "session.log", "update_success.log")
CASES = {f"C{number:02}" for number in range(12)} | {"E01", "E02", "E03"}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def output_path(value):
    path = Path(value).resolve()
    if ARTIFACTS.resolve() not in path.parents:
        raise ValueError("Output must be a new path inside artifacts/p0")
    if path.exists():
        raise ValueError("Existing artifacts are preserved; select a new output path")
    return path


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def inspect_client(directory):
    directory = Path(directory).resolve()
    executable = directory / "osu!.exe"
    executable_hash = sha(executable.read_bytes())
    config = directory / "osu!.cfg"
    fields = {}
    if config.is_file():
        for line in config.read_text(encoding="utf-8-sig", errors="replace").splitlines():
            key, separator, value = line.partition("=")
            if separator and key.strip() in {"_ReleaseStream", "ReleaseStream", "LastVersion"}:
                fields[key.strip()] = value.strip()[:100]
    logs = []
    for name in LOGS:
        path = directory / "Logs" / name
        if not path.is_file():
            continue
        # Bounded reads; never inspect authentication logs or per-user configuration.
        with path.open("rb") as stream:
            data = stream.read(131072)
            if path.stat().st_size > 131072:
                stream.seek(max(131072, path.stat().st_size - 131072))
                data += b"\n" + stream.read(131072)
        text = data.decode("utf-8-sig", errors="replace")
        relevant = [line for line in text.splitlines() if re.search("version|release|build", line, re.I)]
        tokens = sorted({token for line in relevant for token in re.findall(
            r"(?<!\w)b?20\d{6}(?:\.\d+)?(?:stable40|stable|cuttingedge|beta)?(?!\w)", line, re.I
        )})
        logs.append({
            "relative_path": "Logs/" + name,
            "size_bytes": path.stat().st_size,
            "modified_utc": datetime.fromtimestamp(path.stat().st_mtime, timezone.utc).isoformat(),
            "unverified_release_tokens": tokens,
            "tokens_are_current_release_proof": False,
            "bounded_sample_sha256": sha(data),
        })
    return {
        "schema_version": 1,
        "kind": "read_only_client_inspection",
        "inspected_at_utc": datetime.now(timezone.utc).isoformat(),
        "executable_path": str(executable), "executable_sha256": executable_hash,
        "allowed_configuration_fields": fields, "logs": logs,
        "actual_release": None, "compatibility_cases_executed": 0,
        "system_window_capture_used": False,
        "raw_logs_or_authentication_data_included": False,
    }


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def png(width, height, pixel):
    rows = b"".join(b"\0" + bytes(value for x in range(width) for value in pixel(x, y)) for y in range(height))
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b"")


def image_metadata(data):
    if len(data) > 64 * 1024 * 1024:
        raise ValueError("Screenshot exceeds 64 MiB")
    if data.startswith(b"\x89PNG\r\n\x1a\n") and len(data) >= 33:
        if data[12:16] != b"IHDR" or struct.unpack(">I", data[8:12])[0] != 13:
            raise ValueError("Invalid PNG header")
        if zlib.crc32(data[12:29]) != struct.unpack(">I", data[29:33])[0]:
            raise ValueError("Corrupted PNG header")
        width, height = struct.unpack(">II", data[16:24])
        extension = ".png"
    elif data.startswith(b"\xff\xd8"):
        offset, width, height = 2, 0, 0
        while offset + 4 <= len(data):
            if data[offset] != 255:
                raise ValueError("Invalid JPEG marker")
            while offset < len(data) and data[offset] == 255:
                offset += 1
            if offset >= len(data):
                break
            marker = data[offset]
            offset += 1
            if marker in {0xD8, 0xD9, 0x01} or 0xD0 <= marker <= 0xD7:
                continue
            length = int.from_bytes(data[offset:offset + 2], "big")
            if length < 2 or offset + length > len(data):
                raise ValueError("Truncated JPEG segment")
            if marker in {0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7, 0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF}:
                if length < 8:
                    raise ValueError("Invalid JPEG frame header")
                height, width = struct.unpack(">HH", data[offset + 3:offset + 7])
                break
            offset += length
        extension = ".jpg"
    else:
        raise ValueError("Expected a local PNG or JPEG screenshot")
    if not 0 < width <= 16384 or not 0 < height <= 16384:
        raise ValueError("Invalid or excessive image dimensions")
    return {"format": extension[1:], "width": width, "height": height, "extension": extension}


def make_fixture(destination):
    destination = output_path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".fixture-", dir=destination.parent) as temporary:
        staging = Path(temporary).resolve()
        if ARTIFACTS.resolve() not in staging.parents:
            raise ValueError("Invalid staging location")
        skin = staging / "skin"
        skin.mkdir()
        (skin / "skin.ini").write_text("[General]\nName: P0 Synthetic\nAuthor: skin-editor P0\nVersion: 2.0\n", encoding="utf-8")
        def ring(x, y):
            radius = (x - 31.5) ** 2 + (y - 31.5) ** 2
            if 25 ** 2 < radius < 29 ** 2:
                return (250, 80, 130, 255)
            return (0, 0, 0, 0)
        def disk(x, y):
            if (x - 31.5) ** 2 + (y - 31.5) ** 2 < 27 ** 2:
                return (70, 170, 230, 210)
            return (0, 0, 0, 0)
        for name, pattern in [("hitcircle", disk), ("hitcircleoverlay", ring), ("approachcircle", ring), ("cursor", ring)]:
            (skin / (name + ".png")).write_bytes(png(64, 64, pattern))
        # Self-authored 3x5 bitmap numerals; no fonts or resources from the old source.
        digits = ["111101101101111", "010110010010111", "111001111100111", "111001111001111", "101101111001001",
                  "111100111001111", "111100111101111", "111001001001001", "111101111101111", "111101111001111"]
        for number, bits in enumerate(digits):
            def numeral(x, y, bits=bits):
                return (255, 255, 255, 255) if bits[(y // 6) * 3 + x // 6] == "1" else (0, 0, 0, 0)
            (skin / f"default-{number}.png").write_bytes(png(18, 30, numeral))
        map_dir = staging / "beatmap"
        map_dir.mkdir()
        with wave.open(str(map_dir / "silence.wav"), "wb") as audio:
            audio.setnchannels(1)
            audio.setsampwidth(2)
            audio.setframerate(22050)
            audio.writeframes(b"\0\0" * 22050 * 10)
        objects = "\n".join(f"{x},{y},{2000 + index * 750},1,0,0:0:0:0:" for index, (x, y) in enumerate(
            [(128, 128), (256, 192), (384, 128), (256, 256), (128, 256), (384, 256)]
        ))
        map_text = (
            "osu file format v14\n\n[General]\nAudioFilename: silence.wav\nMode: 0\n\n"
            "[Metadata]\nTitle:P0 Circle Fixture\nArtist:Self-authored\nCreator:skin-editor P0\nVersion:Circles\n\n"
            "[Difficulty]\nHPDrainRate:5\nCircleSize:4\nOverallDifficulty:5\nApproachRate:5\nSliderMultiplier:1\nSliderTickRate:1\n\n"
            "[TimingPoints]\n0,500,4,2,0,100,1,0\n\n[HitObjects]\n" + objects + "\n"
        )
        (map_dir / "P0 Circles.osu").write_text(map_text, encoding="utf-8")
        files = [{"path": str(path.relative_to(staging)).replace("\\", "/"), "sha256": sha(path.read_bytes())}
                 for path in sorted(staging.rglob("*")) if path.is_file()]
        manifest = {"schema_version": 1, "kind": "self_authored_fixture", "source": "independently_generated",
                    "files": files, "client_execution_verified": False,
                    "note": "Public format test inputs; dimensions and timing are test choices, not compatibility rules"}
        write_json(staging / "manifest.json", manifest)
        # Both source and destination have been resolved and checked inside artifacts/p0.
        staging.rename(destination)
    return manifest


def register_screenshot(image, client_report, destination, case, release=None, fixture=None):
    if case not in CASES:
        raise ValueError("Unknown validation case")
    destination = output_path(destination)
    image = Path(image).resolve()
    if image.stat().st_size > 64 * 1024 * 1024:
        raise ValueError("Screenshot exceeds 64 MiB")
    data = image.read_bytes()
    metadata = image_metadata(data)
    client_report_bytes = Path(client_report).read_bytes()
    client = json.loads(client_report_bytes.decode("utf-8-sig"))
    if client.get("kind") != "read_only_client_inspection":
        raise ValueError("Use an inspect-client report for provenance")
    fixture_hash = sha(Path(fixture).read_bytes()) if fixture else None
    # Registration is not visual review and never increments executed/passed case counts.
    record = {"schema_version": 1, "kind": "screenshot_registration", "case_id": case,
              "review_status": "awaiting_review", "compatibility_passed": None,
              "declared_client_release": release, "release_automatically_verified": False,
              "registered_at_utc": datetime.now(timezone.utc).isoformat(), "captured_at": None,
              "client_report_sha256": sha(client_report_bytes),
              "client_identity_association": "declared_not_automatically_verified",
              "client_executable_sha256": client["executable_sha256"], "image_sha256": sha(data),
              "image_metadata": metadata, "image_validation": "header_metadata_only",
              "fixture_manifest_sha256": fixture_hash, "system_window_capture_used": False,
              "origin": "explicitly_supplied_local_image"}
    destination.mkdir(parents=True)
    (destination / ("original" + metadata["extension"])).write_bytes(data)
    write_json(destination / "record.json", record)
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    inspect = commands.add_parser("inspect-client")
    inspect.add_argument("--client-dir", required=True)
    inspect.add_argument("--output", required=True)
    fixture = commands.add_parser("make-fixture")
    fixture.add_argument("--output", required=True)
    register = commands.add_parser("register-screenshot")
    for flag in ["image", "client-report", "output", "case"]:
        register.add_argument("--" + flag, required=True)
    register.add_argument("--release")
    register.add_argument("--fixture")
    args = parser.parse_args()
    if args.command == "inspect-client":
        write_json(output_path(args.output), inspect_client(args.client_dir))
    elif args.command == "make-fixture":
        make_fixture(args.output)
    else:
        register_screenshot(args.image, args.client_report, args.output, args.case, args.release, args.fixture)
    print("P0 artifact written: " + args.output)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError) as error:
        raise SystemExit(str(error))
