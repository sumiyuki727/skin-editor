"""Self-authored storage fixtures, not a stable compatibility implementation."""

import hashlib
import struct
import wave
import zlib

from .identity import new_identity
from .store import Store
from .values import Block, BlobInput, Node, encode_json


def png_bytes():
    def chunk(kind, raw):
        return struct.pack(">I", len(raw)) + kind + raw + struct.pack(">I", zlib.crc32(kind + raw) & 0xffffffff)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 1, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(b"\0\xff\x60\0\xff\0\x80\xff\0")) + chunk(b"IEND", b""))


def write_media(directory, large_mib=0):
    png = directory / "self-authored.png"
    png.write_bytes(png_bytes())
    audio = directory / "self-authored.wav"
    with wave.open(str(audio), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(48000)
        output.writeframes(b"\0\0" * 4800)
    opaque = directory / "unsupported-audio.bin"
    opaque.write_bytes(b"self-authored unknown codec\0\xff\x81\r\n")
    paths = [png, audio, opaque]
    if large_mib:
        large = directory / "self-authored-load.bin"
        # Small deterministic buffer; never manufacture a whole large blob in RAM.
        part = b"".join(hashlib.sha256(str(i).encode("ascii")).digest() for i in range(2048))
        with large.open("wb") as output:
            for _ in range(large_mib * 16):
                output.write(part)
        paths.append(large)
    return tuple(BlobInput.prepare(path) for path in paths)


def project_block(project, step=0, blocks=(), blobs=()):
    return Block.make("fixture.project", {"project_id": project.hex(), "step": step}, blocks, blobs)


def make_store(path, blobs=()):
    project, history, root = new_identity(), new_identity(), new_identity()
    store = Store.create(path, project, history, root)
    state = project_block(project, blobs=[b.sha for b in blobs])
    node = Node(root, None, state.sha, 1, 1, 0, encode_json({"kind": "root"}))
    receipt = store.commit(store.captured("manual", head=root, nodes=(node,), blocks=(state,), blobs=blobs))
    return store, node, receipt


def append(store, parent, alias, step=1, branch=1, ordinal=None, origin="automatic", blobs=()):
    state = project_block(store.project, step, blocks=(parent.state,), blobs=[b.sha for b in blobs])
    node = Node(new_identity(), parent.id, state.sha, alias, branch,
                parent.ordinal + 1 if ordinal is None else ordinal,
                encode_json({"kind": "fixture-edit", "step": step}))
    receipt = store.commit(store.captured(origin, head=node.id, nodes=(node,), blocks=(state,), blobs=blobs))
    return node, receipt
