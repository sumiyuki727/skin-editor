"""Generate original, deterministic PNGs for the isolated P0 standard workload."""

import argparse
import hashlib
import json
from pathlib import Path
import struct
import tempfile
import zlib

from p0_evidence import ARTIFACTS, output_path, write_json


def png(side, identity):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    # Asymmetric colored tiles and straight alpha, including transparent colored pixels.
    rows = []
    tile = max(1, side // 32)
    for y in range(side):
        row = bytearray(b"\0")
        for x in range(0, side, tile):
            rgba = bytes(((identity * 43 + x // tile * 7) % 256,
                          (identity * 71 + y // tile * 11) % 256,
                          (x // tile * 19 + y // tile * 3) % 256,
                          (0, 64, 160, 255)[(x // tile + y // tile) % 4]))
            row.extend(rgba * min(tile, side - x))
        rows.append(row)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", side, side, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(b"".join(rows), 3)) + chunk(b"IEND", b""))


def make_workload(destination, asset_count=100, document_count=10, side=1024):
    destination = output_path(destination)
    if asset_count % document_count or not 1 <= side <= 1024:
        raise ValueError("Use equally sized documents and images no larger than 1024")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=destination.parent) as temporary:
        assert ARTIFACTS.resolve() in Path(temporary).resolve().parents
        staging = Path(temporary) / "fixture"
        (staging / "素材 日本語").mkdir(parents=True)
        assets, files = [], []
        for number in range(asset_count):
            entry = {"id": number, "path": f"素材 日本語/asset-{number:03}.png",
                     "thumbnail": f"素材 日本語/thumb-{number:03}.png", "width": side, "height": side}
            for key, size in (("path", side), ("thumbnail", min(64, side))):
                data = png(size, number)
                (staging / entry[key]).write_bytes(data)
                files.append({"path": entry[key], "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
            assets.append(entry)
        documents = []
        per_document = asset_count // document_count
        for document in range(document_count):
            # Five nested groups, ten leaves each in the reference workload.
            groups = []
            for group in range(5):
                leaves = [{"asset": document * per_document + n,
                           "x": (n % 5) * 104, "y": (n // 5) * 100 + group * 67,
                           "scale": 0.1, "opacity": 0.7 + group * 0.05}
                          for n in range(per_document)]
                groups.append({"x": group * 3, "y": group * 2, "scale": 1.0,
                               "opacity": 0.95, "children": [{"x": 0, "y": 0, "scale": 1.0,
                                                               "opacity": 0.9, "children": leaves}]})
            documents.append({"id": document, "children": groups})
        manifest = {"schema_version": 1, "origin": "independently generated P0 fixture; no stable resources",
                    "assets": assets, "documents": documents,
                    "composition": "ordered source-over; positive uniform scale, translation, inherited opacity; no isolated group effects"}
        write_json(staging / "workload.json", manifest)
        data = (staging / "workload.json").read_bytes()
        files.append({"path": "workload.json", "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
        inventory = {"schema_version": 1, "asset_count": asset_count, "document_count": document_count,
                     "layer_count": asset_count * 5, "side": side, "files": files,
                     "stable_compatibility_verified": False}
        write_json(staging / "inventory.json", inventory)
        staging.rename(destination)
    return inventory


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    print(json.dumps(make_workload(arguments.output), ensure_ascii=False))
