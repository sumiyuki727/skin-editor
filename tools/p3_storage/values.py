"""Bounded value inputs for an isolated storage experiment, not Commands."""

from dataclasses import dataclass, field
import hashlib
import json
import math
from pathlib import Path
import re

from .identity import ContractError, identity, u64

CHUNK_BYTES = 64 * 1024
MAX_JSON_BYTES = 1024 * 1024
MAX_ACTION_BYTES = 64 * 1024
MAX_JSON_DEPTH = 32
APP_ID = 0x534B4550
FORMAT_VERSION = 2
PROJECT_EXTENSION = ".oskproj"
REQUIRED_FEATURES = (("storage.core", "2"), ("state.p3-fixture", "2"))


def digest(value):
    if type(value) is not bytes or len(value) != 32:
        raise ContractError("digest must be 32 bytes")
    return value


def optional(value):
    return b"\0" if value is None else b"\1" + value


def sized(value):
    return u64(len(value), zero=True) + value


def optional_text(value):
    return b"\0" if value is None else b"\1" + sized(value.encode("utf8"))


def _json_value(value, depth=0):
    if depth > MAX_JSON_DEPTH:
        raise ContractError("metadata nesting limit")
    if type(value) is str:
        try:
            value.encode("utf8")
        except UnicodeError as error:
            raise ContractError("JSON text must contain Unicode scalar values") from error
        return
    if value is None or type(value) is bool:
        return
    if type(value) is int:
        if not -(1 << 53) < value < (1 << 53):
            raise ContractError("large integers must use their text encoding")
        return
    if type(value) is float and math.isfinite(value):
        return
    if type(value) is list:
        for child in value:
            _json_value(child, depth + 1)
        return
    if type(value) is dict and all(type(k) is str for k in value):
        for child in value.values():
            _json_value(child, depth + 1)
        return
    raise ContractError("unsupported or non-finite JSON value")


def encode_json(value, limit=MAX_JSON_BYTES):
    _json_value(value)
    raw = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"),
                     allow_nan=False).encode("utf8")
    if len(raw) > limit:
        raise ContractError("metadata byte limit")
    return raw


def decode_json(raw, limit=MAX_JSON_BYTES):
    if type(raw) is not bytes or len(raw) > limit:
        raise ContractError("metadata byte limit")

    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ContractError("duplicate JSON key")
            result[key] = value
        return result

    try:
        value = json.loads(raw.decode("utf8"), object_pairs_hook=pairs)
        _json_value(value)
    except (UnicodeError, ValueError, RecursionError) as error:
        raise ContractError("invalid JSON: " + type(error).__name__) from error
    return value


@dataclass(frozen=True)
class Block:
    raw: bytes

    def __post_init__(self):
        inspect_block(self.raw)

    @classmethod
    def make(cls, kind, value, blocks=(), blobs=(), *, schema=1):
        return cls(encode_json({"kind": kind, "schema": schema,
                               "blocks": sorted({digest(v).hex() for v in blocks}),
                               "blobs": sorted({digest(v).hex() for v in blobs}), "value": value}))

    @property
    def sha(self):
        return hashlib.sha256(self.raw).digest()

    def envelope(self):
        return decode_json(self.raw)


def inspect_block(raw):
    """Parse and validate exactly once; no retained whole-file JSON cache."""
    value = decode_json(raw)
    if set(value) != {"kind", "schema", "blocks", "blobs", "value"}:
        raise ContractError("invalid block envelope")
    if (type(value["kind"]) is not str
            or not re.fullmatch(r"[a-z][a-z0-9.-]{0,63}", value["kind"])
            or type(value["schema"]) is not int or value["schema"] not in (1, 2)
            or (value["schema"] == 2 and value["kind"] != "fixture.project")):
        raise ContractError("unsupported block envelope")
    for key in ("blocks", "blobs"):
        refs = value[key]
        if (type(refs) is not list or len(refs) > 8192
                or any(type(r) is not str or not re.fullmatch(r"[0-9a-f]{64}", r) for r in refs)
                or refs != sorted(set(refs))):
            raise ContractError("invalid block references")

    return value

@dataclass(frozen=True)
class BlobInput:
    path: Path
    sha: bytes
    byte_len: int
    signature: tuple

    @classmethod
    def prepare(cls, path):
        path = Path(path)
        if not path.is_file() or path.is_symlink():
            raise ContractError("not an explicit regular fixture file")
        before = path.stat()
        hasher = hashlib.sha256()
        size = 0
        with path.open("rb") as stream:
            for part in iter(lambda: stream.read(CHUNK_BYTES), b""):
                hasher.update(part)
                size += len(part)
        after = path.stat()
        signature = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns)
        if signature(before) != signature(after) or size != before.st_size:
            raise ContractError("source changed while preparing")
        return cls(path, hasher.digest(), size, signature(after))

    def parts(self):
        stat = self.path.stat()
        if (stat.st_dev, stat.st_ino, stat.st_size, stat.st_mtime_ns) != self.signature:
            raise ContractError("prepared source changed")
        hasher = hashlib.sha256()
        size = 0
        with self.path.open("rb") as stream:
            for part in iter(lambda: stream.read(CHUNK_BYTES), b""):
                hasher.update(part)
                size += len(part)
                yield part
        if size != self.byte_len or hasher.digest() != self.sha:
            raise ContractError("source content changed")


@dataclass(frozen=True)
class Node:
    """Query/capture entry pairing immutable history with a display binding."""
    id: bytes
    parent: object
    state: bytes
    alias: int
    branch: int
    ordinal: int
    action: bytes

    def __post_init__(self):
        identity(self.id)
        if self.parent is not None:
            identity(self.parent)
        digest(self.state)
        u64(self.alias)
        u64(self.branch)
        u64(self.ordinal, zero=True)
        if type(decode_json(self.action, MAX_ACTION_BYTES)) is not dict:
            raise ContractError("action must be a structured record")

    def record(self):
        # Hash original action bytes, never a language-specific JSON re-encoding.
        return (b"skin-editor.node.v2\0" + self.id + optional(self.parent)
                + self.state + sized(self.action))

    @property
    def address(self):
        return AddressBinding(self.id, self.alias, self.branch, self.ordinal)

    @property
    def checksum(self):
        return hashlib.sha256(self.record()).digest()


@dataclass(frozen=True)
class AddressBinding:
    node: bytes
    alias: int
    branch: int
    ordinal: int

    def __post_init__(self):
        identity(self.node)
        u64(self.alias)
        u64(self.branch)
        u64(self.ordinal, zero=True)

    def record(self):
        return (b"skin-editor.address.v1\0" + self.node + u64(self.alias)
                + u64(self.branch) + u64(self.ordinal, zero=True))

    @property
    def checksum(self):
        return hashlib.sha256(self.record()).digest()


def fixture_projection(source):
    """Known self-authored 1 -> 2 conversion; never a general migration engine."""
    envelope = source.envelope()
    value = envelope["value"]
    if (envelope["kind"] != "fixture.project" or envelope["schema"] != 1
            or type(value) is not dict or type(value.get("step")) is not int
            or "edit_index" in value):
        raise ContractError("unsupported fixture conversion")
    projected = dict(value)
    projected["edit_index"] = projected.pop("step")
    return Block.make("fixture.project", projected,
                      [bytes.fromhex(v) for v in envelope["blocks"]],
                      [bytes.fromhex(v) for v in envelope["blobs"]], schema=2)


@dataclass(frozen=True)
class StateView:
    origin: bytes
    target: bytes
    converter: str = "fixture.step-to-index.v1"

    def __post_init__(self):
        digest(self.origin)
        digest(self.target)
        if self.converter != "fixture.step-to-index.v1":
            raise ContractError("unsupported converter")

    def record(self):
        return (b"skin-editor.state-view.v1\0" + self.origin + self.target
                + u64(2) + sized(self.converter.encode("ascii")))

    @property
    def checksum(self):
        return hashlib.sha256(self.record()).digest()


@dataclass(frozen=True)
class TagEdit:
    node: bytes
    key: str
    value: object

    def __post_init__(self):
        identity(self.node)
        if type(self.key) is not str or not re.fullmatch(r"user:[a-z0-9][a-z0-9._-]{0,63}", self.key):
            raise ContractError("only custom user tags can be edited")
        if self.value is not None and (type(self.value) is not str
                                      or len(self.value.encode("utf8")) > 4096):
            raise ContractError("invalid tag value")


@dataclass(frozen=True)
class Batch:
    epoch: bytes
    commit_id: bytes
    expected_save: object
    origin: str
    address_epoch: object = None
    head: object = None
    navigation: object = None
    nodes: tuple = ()
    blocks: tuple = ()
    blobs: tuple = ()
    tags: tuple = ()
    identities: tuple = ()
    extensions: tuple = ()
    new_address_epoch: object = None
    remap: tuple = ()
    views: tuple = ()

    def fingerprint(self):
        identity(self.epoch)
        identity(self.commit_id)
        identity(self.address_epoch)
        if self.new_address_epoch is not None:
            identity(self.new_address_epoch)
        if self.expected_save is not None:
            identity(self.expected_save)
        if self.head is not None:
            identity(self.head)
        if self.navigation is not None:
            digest(self.navigation)
        if self.origin not in {"manual", "automatic", "exit", "metadata"}:
            raise ContractError("invalid commit origin")
        if self.origin == "manual" and self.head is None:
            raise ContractError("manual save needs a captured head")
        if self.origin == "metadata" and (self.head is not None or self.navigation is not None):
            raise ContractError("metadata edit cannot move a checkpoint")
        prefix = (b"skin-editor.batch.v2\0" + self.commit_id + optional(self.expected_save)
                  + bytes(({"manual": 1, "automatic": 2, "exit": 3, "metadata": 4}[self.origin],))
                  + optional(self.head) + optional(self.navigation)
                  + self.address_epoch + optional(self.new_address_epoch))
        sequences = (
            [n.checksum + n.address.checksum for n in self.nodes],
            [b.sha for b in self.blocks],
            [b.sha + u64(b.byte_len, zero=True) for b in self.blobs],
            [t.node + sized(t.key.encode("utf8")) + optional_text(t.value) for t in self.tags],
            [sized(k.encode("utf8")) + identity(i) + identity(n) for k, i, n in self.identities],
            [sized(k.encode("utf8")) + u64(len(v), zero=True) + hashlib.sha256(v).digest()
             for k, v in self.extensions],
            [b.checksum for b in self.remap],
            [v.checksum for v in self.views])
        hasher = hashlib.sha256(prefix)
        for values in sequences:
            hasher.update(len(values).to_bytes(4, "big"))
            for value in values:
                hasher.update(value)
        return hasher.digest()


@dataclass(frozen=True)
class Limits:
    nodes: int = 65536
    blocks: int = 250000
    blob_bytes: int = 512 * 1024 * 1024
    per_blob: int = 128 * 1024 * 1024
    metadata_bytes: int = 128 * 1024 * 1024
    batch_metadata: int = 4 * 1024 * 1024
    batch_nodes: int = 1024
    batch_blocks: int = 4096
    file_bytes: int = 1024 * 1024 * 1024

    def __post_init__(self):
        if any(type(v) is not int or v <= 0 for v in self.__dict__.values()):
            raise ContractError("invalid experiment limits")


@dataclass(frozen=True)
class Receipt:
    commit_id: object
    generation: int
    changed: bool
    replay: bool = False
    new_nodes: int = 0
    new_blocks: int = 0
    new_blob_bytes: int = 0
    timings: dict = field(default_factory=dict)
