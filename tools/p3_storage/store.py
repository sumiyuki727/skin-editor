"""Single-file transactional experiment. No product model or UI dependency.

The private connection is never a public application write port. SQL guards are
defence against programming mistakes, not protection from an external DB editor.
"""

import hashlib
import os
from pathlib import Path
import re
import sqlite3
import time

from . import schema
from .identity import (ContractError, NodeReference, identity, new_identity,
                       parse_alias, parse_branch, parse_node_key, u64)
from .values import (APP_ID, CHUNK_BYTES, FORMAT_VERSION, MAX_JSON_BYTES,
                     REQUIRED_FEATURES, Batch, Block, Limits, Node, Receipt,
                     decode_json, digest, encode_json)

BLOCK_KINDS = {"fixture.project", "fixture.value", "fixture.assets", "fixture.document",
               "fixture.layer", "fixture.components", "fixture.audio",
               "fixture.configuration", "fixture.outputs", "navigation"}
OBJECT_KINDS = {"asset", "document", "layer", "component", "audio", "output"}


class StorageError(ContractError):
    pass


def _connection(path, writable):
    db = sqlite3.connect(Path(path).resolve().as_uri() + ("?mode=rw" if writable else "?mode=ro"),
                         uri=True, isolation_level=None, timeout=0)
    for statement in ("foreign_keys=ON", "recursive_triggers=ON", "trusted_schema=OFF",
                      "cache_size=-2048", "mmap_size=0", "temp_store=FILE", "synchronous=EXTRA"):
        db.execute("PRAGMA " + statement)
    if db.execute("PRAGMA foreign_keys").fetchone()[0] != 1:
        db.close()
        raise StorageError("foreign key support required")
    if db.execute("PRAGMA synchronous").fetchone()[0] != 3:
        db.close()
        raise StorageError("EXTRA synchronous setting unavailable")
    if not writable:
        db.execute("PRAGMA query_only=ON")
    return db


def _layout(db):
    return [(kind, name, re.sub(r"\s+", " ", text).strip())
            for kind, name, text in db.execute(
                "SELECT type,name,sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY type,name")]


def _expected_layout():
    db = sqlite3.connect(":memory:")
    try:
        db.executescript(schema.script())
        return _layout(db)
    finally:
        db.close()


def _raw_header(path, limits):
    path = Path(path)
    if not path.is_file() or path.is_symlink() or path.stat().st_size > limits.file_bytes:
        raise StorageError("invalid file kind or file byte limit")
    with path.open("rb") as stream:
        raw = stream.read(100)
    if (len(raw) != 100 or raw[:16] != b"SQLite format 3\0"
            or int.from_bytes(raw[68:72], "big") != APP_ID
            or int.from_bytes(raw[60:64], "big") != FORMAT_VERSION):
        raise StorageError("unknown container header; no write attempted")
    if raw[18:20] != b"\x01\x01" or int.from_bytes(raw[16:18], "big") != 4096:
        raise StorageError("unsupported page/journal format")


class Store:
    def __init__(self, path, db, limits, writable):
        self.path, self._db, self.limits, self.writable = Path(path), db, limits, writable
        self.epoch = new_identity()
        row = db.execute("SELECT family,profile,container_version,project_schema,history_schema,project_id,history_id,lineage_id,root_id FROM header WHERE singleton=1").fetchone()
        if row is None or row[:5] != ("skin-editor-project", "p3-0", 1, 1, 1):
            raise StorageError("unsupported format/profile")
        self.project, self.history, self.lineage, self.root = row[5:]
        for value in row[5:]:
            identity(value)
        features = db.execute("SELECT name,version,required FROM features").fetchall()
        self.unknown_required = [(n, v) for n, v, required in features
                                 if required and (n, v) not in REQUIRED_FEATURES]
        if any((n, v, 1) not in features for n, v in REQUIRED_FEATURES):
            raise StorageError("missing core capabilities")
        if self.unknown_required:
            self.writable = False
            db.execute("PRAGMA query_only=ON")
        stat = self.path.stat()
        self._file_id = (stat.st_dev, stat.st_ino)
        if self.writable:
            pages = limits.file_bytes // 4096
            if pages < db.execute("PRAGMA page_count").fetchone()[0]:
                raise StorageError("file page budget")
            db.execute("PRAGMA max_page_count=%d" % pages)

    @classmethod
    def create(cls, path, project, history, root, *, lineage=None, limits=None):
        limits = limits or Limits()
        for value in (project, history, root):
            identity(value)
        lineage = identity(lineage) if lineage is not None else new_identity()
        path = Path(path)
        # Never overwrite an existing destination, including an unrelated file.
        descriptor = os.open(str(path), os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
        os.close(descriptor)
        db = _connection(path, True)
        try:
            db.execute("PRAGMA page_size=4096")
            if db.execute("PRAGMA journal_mode=DELETE").fetchone()[0] != "delete":
                raise StorageError("DELETE journal mode unavailable")
            db.execute("PRAGMA application_id=%d" % APP_ID)
            db.execute("PRAGMA user_version=1")
            db.executescript("BEGIN IMMEDIATE;\n" + schema.script())
            db.execute("INSERT INTO header VALUES(1,?,?,?,?,?,?,?,?,?)",
                       ("skin-editor-project", "p3-0", 1, 1, 1, project, history, lineage, root))
            db.executemany("INSERT INTO features VALUES(?,?,1)", REQUIRED_FEATURES)
            db.execute("INSERT INTO control VALUES(1,?,NULL,NULL,NULL,0,0,0,0)", (u64(0, zero=True),))
            db.execute("COMMIT")
            return cls(path, db, limits, True)
        except BaseException:
            if db.in_transaction:
                db.execute("ROLLBACK")
            db.close()
            raise

    @classmethod
    def open(cls, path, *, writable=True, recover=False, verify=True, limits=None):
        limits = limits or Limits()
        _raw_header(path, limits)
        # Read-only probe must not silently recover an unknown/future file.
        db = _connection(path, bool(recover))
        try:
            if _layout(db) != _expected_layout():
                raise StorageError("unknown or modified schema; writes refused")
            probe = cls(path, db, limits, False)
            if writable and not probe.unknown_required:
                db.close()
                db = _connection(path, True)
                if _layout(db) != _expected_layout():
                    raise StorageError("schema changed while opening")
                result = cls(path, db, limits, True)
                if db.execute("PRAGMA journal_mode").fetchone()[0] != "delete":
                    raise StorageError("journal mode changed")
            else:
                result = probe
                db.execute("PRAGMA query_only=ON")
            if result.status()["save_id"] is None:
                raise StorageError("incomplete initial project")
            if verify and not result.unknown_required:
                result.verify()
            return result
        except BaseException:
            db.close()
            raise

    def close(self):
        self._db.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()

    def status(self):
        row = self._db.execute("SELECT generation,save_id,manual_id,protected_id,node_count,block_count,blob_bytes,metadata_bytes FROM control WHERE singleton=1").fetchone()
        if row is None:
            raise StorageError("missing control record")
        return dict(zip(("generation", "save_id", "manual_id", "protected_id", "nodes", "blocks", "blob_bytes", "metadata_bytes"),
                        (int.from_bytes(row[0], "big"), *row[1:])))

    def captured(self, origin, **values):
        """Prototype capture value. Does not access a real ProjectSession."""
        return Batch(self.epoch, new_identity(), self.status()["save_id"], origin, **values)

    def block(self, sha):
        length = self._db.execute("SELECT length(bytes) FROM blocks WHERE sha=?", (digest(sha),)).fetchone()
        if length is None or length[0] > MAX_JSON_BYTES:
            raise StorageError("missing or oversized state block")
        row = self._db.execute("SELECT bytes FROM blocks WHERE sha=?", (digest(sha),)).fetchone()
        if row is None:
            raise StorageError("missing state block")
        return Block(row[0])

    def node(self, value):
        length = self._db.execute("SELECT length(action) FROM nodes WHERE id=?", (identity(value),)).fetchone()
        if length is None or length[0] > 64 * 1024:
            raise StorageError("missing or oversized node action")
        row = self._db.execute("SELECT id,parent,state,alias,branch,ordinal,action FROM nodes WHERE id=?", (identity(value),)).fetchone()
        if row is None:
            raise StorageError("unknown node")
        return Node(row[0], row[1], row[2], *(int.from_bytes(v, "big") for v in row[3:6]), row[6])

    def blob_parts(self, sha):
        row = self._db.execute("SELECT byte_len FROM blobs WHERE sha=?", (digest(sha),)).fetchone()
        if row is None:
            raise StorageError("unknown original bytes")
        hasher, total, ordinal = hashlib.sha256(), 0, 0
        for index, raw in self._db.execute("SELECT part,bytes FROM blob_chunks WHERE sha=? ORDER BY part", (sha,)):
            if index != ordinal or not 0 < len(raw) <= CHUNK_BYTES:
                raise StorageError("invalid original chunk sequence")
            ordinal += 1
            total += len(raw)
            hasher.update(raw)
            yield raw
        if total != row[0] or hasher.digest() != sha:
            raise StorageError("original bytes checksum mismatch")

    def tags(self, node):
        self.node(node)
        return dict(self._db.execute("SELECT name,value FROM tags WHERE node_id=? ORDER BY name", (node,)))

    def open_position(self, latest=False):
        status = self.status()
        key = status["protected_id" if latest else "manual_id"]
        if key is None:
            return None
        row = self._db.execute("SELECT head,navigation FROM commits WHERE id=?", (key,)).fetchone()
        if row is None or row[0] is None:
            raise StorageError("invalid checkpoint")
        return self.node(row[0]), (self.block(row[1]) if row[1] else None)

    def resolve(self, text, *, epoch, project, history):
        if epoch != self.epoch or project != self.project or history != self.history:
            raise StorageError("stale or wrong address context")
        if text.startswith("hp1:"):
            reference = NodeReference.parse(text)
            if reference.project != project or reference.history != history:
                raise StorageError("reference belongs to another project/history")
            result = reference.node
        elif text.startswith("k"):
            result = parse_node_key(text)
        elif text.startswith("n"):
            row = self._db.execute("SELECT id FROM nodes WHERE alias=?", (u64(parse_alias(text)),)).fetchone()
            result = row[0] if row else None
        elif text.startswith("b"):
            branch, ordinal = parse_branch(text)
            row = self._db.execute("SELECT id FROM nodes WHERE branch=? AND ordinal=?", (u64(branch), u64(ordinal, zero=True))).fetchone()
            result = row[0] if row else None
        else:
            raise StorageError("unknown address version")
        if result is None:
            raise StorageError("address not found")
        return self.node(result).id

    def _put_blob(self, blob, usage):
        if blob.byte_len < 0 or blob.byte_len > self.limits.per_blob:
            raise StorageError("per-original budget")
        row = self._db.execute("SELECT byte_len FROM blobs WHERE sha=?", (digest(blob.sha),)).fetchone()
        if row is not None:
            if row[0] != blob.byte_len:
                raise StorageError("original identity conflict")
            expected = iter(self.blob_parts(blob.sha))
            for part in blob.parts():
                if next(expected, None) != part:
                    raise StorageError("original collision")
            if next(expected, None) is not None:
                raise StorageError("original length conflict")
            return False
        if usage["blob_bytes"] + blob.byte_len > self.limits.blob_bytes:
            raise StorageError("original total budget")
        self._db.execute("INSERT INTO blobs VALUES(?,?)", (blob.sha, blob.byte_len))
        for ordinal, part in enumerate(blob.parts()):
            self._db.execute("INSERT INTO blob_chunks VALUES(?,?,?)", (blob.sha, ordinal, part))
        usage["blob_bytes"] += blob.byte_len
        return True

    def _put_block(self, block, usage):
        row = self._db.execute("SELECT bytes FROM blocks WHERE sha=?", (block.sha,)).fetchone()
        if row:
            if row[0] != block.raw:
                raise StorageError("block collision")
            return False
        envelope = block.envelope()
        if envelope["kind"] not in BLOCK_KINDS:
            raise StorageError("unsupported state codec")
        if (usage["blocks"] >= self.limits.blocks
                or usage["metadata_bytes"] + len(block.raw) > self.limits.metadata_bytes):
            raise StorageError("block metadata budget")
        # References must already exist: a topological insertion order is a
        # bounded, iterative cycle proof. This is not an ancestor-path address.
        for key, table in (("blocks", "blocks"), ("blobs", "blobs")):
            for reference in envelope[key]:
                if self._db.execute("SELECT 1 FROM " + table + " WHERE sha=?", (bytes.fromhex(reference),)).fetchone() is None:
                    raise StorageError("missing block/original dependency")
        last = self._db.execute("SELECT sequence FROM blocks ORDER BY sequence DESC LIMIT 1").fetchone()
        sequence = int.from_bytes(last[0], "big") + 1 if last else 1
        self._db.execute("INSERT INTO blocks VALUES(?,?,?)", (block.sha, u64(sequence), block.raw))
        for reference in envelope["blocks"]:
            self._db.execute("INSERT INTO block_refs VALUES(?,?)", (block.sha, bytes.fromhex(reference)))
        for reference in envelope["blobs"]:
            self._db.execute("INSERT INTO blob_refs VALUES(?,?)", (block.sha, bytes.fromhex(reference)))
        usage["blocks"] += 1
        usage["metadata_bytes"] += len(block.raw)
        return True

    def _put_node(self, node, usage):
        row = self._db.execute("SELECT checksum FROM nodes WHERE id=?", (node.id,)).fetchone()
        if row:
            if row[0] != node.checksum or self.node(node.id) != node:
                raise StorageError("immutable node conflict")
            return False
        if usage["nodes"] >= self.limits.nodes:
            raise StorageError("history node budget; never trim")
        state = self.block(node.state).envelope()
        if (state["kind"] != "fixture.project" or type(state["value"]) is not dict
                or state["value"].get("project_id") != self.project.hex()):
            raise StorageError("wrong project state profile/identity")
        if node.parent is None:
            if node.id != self.root:
                raise StorageError("wrong root")
            depth = 0
        else:
            parent = self._db.execute("SELECT depth FROM nodes WHERE id=?", (node.parent,)).fetchone()
            if parent is None:
                raise StorageError("missing parent")
            depth = parent[0] + 1
        size = len(node.record())
        if usage["metadata_bytes"] + size > self.limits.metadata_bytes:
            raise StorageError("history metadata budget")
        self._db.execute("INSERT INTO nodes VALUES(?,?,?,?,?,?,?,?,?)",
                         (node.id, node.parent, node.state, u64(node.alias), u64(node.branch),
                          u64(node.ordinal, zero=True), depth, node.action, node.checksum))
        usage["nodes"] += 1
        usage["metadata_bytes"] += size
        return True

    def _meta(self, usage, change):
        value = usage["metadata_bytes"] + change
        if not 0 <= value <= self.limits.metadata_bytes:
            raise StorageError("metadata payload budget")
        usage["metadata_bytes"] = value

    def _navigation(self, reference):
        envelope = self.block(reference).envelope()
        if envelope["kind"] != "navigation" or set(envelope["value"]) != {"redo"}:
            raise StorageError("invalid navigation codec")
        pairs = envelope["value"]["redo"]
        if type(pairs) is not list or len(pairs) > self.limits.nodes:
            raise StorageError("invalid redo map")
        parents = set()
        for pair in pairs:
            from .identity import parse_identity
            if type(pair) is not list or len(pair) != 2:
                raise StorageError("invalid redo entry")
            parent, child = (parse_identity(v) for v in pair)
            if parent in parents or self.node(child).parent != parent:
                raise StorageError("redo must name a unique direct child")
            parents.add(parent)

    @staticmethod
    def _commit_size(row):
        return sum(len(v) if type(v) is bytes else len(v.encode("utf8")) if type(v) is str else 0
                   for v in row)

    def commit(self, batch, *, hook=None):
        """hook is private experiment fault injection, never a Command payload."""
        if not self.writable or self.unknown_required:
            raise StorageError("read-only/unknown required capability")
        if batch.epoch != self.epoch:
            raise StorageError("stale opening epoch")
        if len(batch.nodes) > self.limits.batch_nodes or len(batch.blocks) > self.limits.batch_blocks:
            raise StorageError("batch admission limit")
        if (sum(len(b.raw) for b in batch.blocks) + sum(len(n.record()) for n in batch.nodes)
                + sum(len(raw) for _, raw in batch.extensions)
                + sum(16 + len(t.key.encode("utf8")) + len((t.value or "").encode("utf8")) for t in batch.tags)
                + len(batch.identities) * 64 > self.limits.batch_metadata):
            raise StorageError("batch metadata admission limit")
        if len(batch.tags) > 4096 or len(batch.identities) > 4096 or len(batch.extensions) > 64:
            raise StorageError("batch annotation/identity limit")
        if len({(t.node, t.key) for t in batch.tags}) != len(batch.tags):
            raise StorageError("duplicate tag edit")
        fingerprint = batch.fingerprint()
        stat = self.path.stat()
        if (stat.st_dev, stat.st_ino) != self._file_id:
            raise StorageError("target file replaced")
        times = {}
        start = time.perf_counter()
        self._db.execute("BEGIN IMMEDIATE")
        committed = False
        try:
            if hook:
                hook("begin", self)
            old = self.status()
            replay = self._db.execute("SELECT request_hash,generation FROM commits WHERE id=?", (batch.commit_id,)).fetchone()
            if replay:
                if replay[0] != fingerprint:
                    raise StorageError("commit identity reused with other content")
                self._db.execute("ROLLBACK")
                return Receipt(batch.commit_id, int.from_bytes(replay[1], "big"), False, replay=True)
            if batch.expected_save != old["save_id"]:
                raise StorageError("revision/save conflict")
            usage = dict(old)
            changed = False
            for blob in batch.blobs:
                changed |= self._put_blob(blob, usage)
            times["originals_ms"] = (time.perf_counter() - start) * 1000
            if hook:
                hook("originals", self)
            mark = time.perf_counter()
            for block in batch.blocks:
                changed |= self._put_block(block, usage)
            times["blocks_ms"] = (time.perf_counter() - mark) * 1000
            if hook:
                hook("blocks", self)
            for node in batch.nodes:
                changed |= self._put_node(node, usage)
            if hook:
                hook("nodes", self)
            for kind, value, first_node in batch.identities:
                if kind not in OBJECT_KINDS:
                    raise StorageError("unknown object identity kind")
                identity(value)
                identity(first_node)
                prior = self._db.execute("SELECT first_node FROM object_ids WHERE kind=? AND id=?", (kind, value)).fetchone()
                if prior:
                    if prior[0] != first_node:
                        raise StorageError("retired object identity reuse")
                else:
                    self._meta(usage, len(kind) + 32)
                    self._db.execute("INSERT INTO object_ids VALUES(?,?,?)", (kind, value, first_node))
                    changed = True
            for name, raw in batch.extensions:
                if (type(name) is not str or not re.fullmatch(r"vendor\.[a-z0-9.-]{1,100}", name)
                        or type(raw) is not bytes or len(raw) > MAX_JSON_BYTES):
                    raise StorageError("invalid optional extension")
                prior = self._db.execute("SELECT bytes FROM extensions WHERE name=?", (name,)).fetchone()
                if prior:
                    if prior[0] != raw:
                        raise StorageError("opaque extension conflict")
                else:
                    self._meta(usage, len(name.encode("utf8")) + len(raw))
                    self._db.execute("INSERT INTO extensions VALUES(?,?)", (name, raw))
                    changed = True
            for edit in batch.tags:
                self.node(edit.node)
                prior = self._db.execute("SELECT value FROM tags WHERE node_id=? AND name=?", (edit.node, edit.key)).fetchone()
                if edit.value is None:
                    if prior:
                        self._meta(usage, -(16 + len(edit.key.encode("utf8")) + len(prior[0].encode("utf8"))))
                        self._db.execute("DELETE FROM tags WHERE node_id=? AND name=?", (edit.node, edit.key))
                        changed = True
                elif prior is None:
                    if self._db.execute("SELECT count(*) FROM tags WHERE node_id=?", (edit.node,)).fetchone()[0] >= 64:
                        raise StorageError("tag count budget")
                    self._meta(usage, 16 + len(edit.key.encode("utf8")) + len(edit.value.encode("utf8")))
                    self._db.execute("INSERT INTO tags VALUES(?,?,?)", (edit.node, edit.key, edit.value))
                    changed = True
                elif prior[0] != edit.value:
                    self._meta(usage, len(edit.value.encode("utf8")) - len(prior[0].encode("utf8")))
                    self._db.execute("UPDATE tags SET value=? WHERE node_id=? AND name=?", (edit.value, edit.node, edit.key))
                    changed = True
            if hook:
                hook("tags", self)
            if batch.head is not None:
                self.node(batch.head)
            if batch.navigation is not None:
                self._navigation(batch.navigation)
            elif batch.head is None and batch.origin == "manual":
                raise StorageError("manual checkpoint requires head")
            previous_checkpoint = self._db.execute("SELECT head,navigation FROM commits WHERE id=?", (old["protected_id"],)).fetchone()
            checkpoint_changed = batch.head is not None and previous_checkpoint != (batch.head, batch.navigation)
            manual_changed = False
            if batch.origin == "manual":
                previous_manual = self._db.execute("SELECT head,navigation FROM commits WHERE id=?", (old["manual_id"],)).fetchone()
                manual_changed = previous_manual != (batch.head, batch.navigation)
            if not (changed or checkpoint_changed or manual_changed):
                self._db.execute("ROLLBACK")
                return Receipt(old["save_id"], old["generation"], False)
            generation = old["generation"] + 1
            commit_row = (batch.commit_id, old["save_id"], u64(generation), batch.origin,
                          batch.head, batch.navigation, fingerprint)
            self._meta(usage, self._commit_size(commit_row))
            self._db.execute("INSERT INTO commits VALUES(?,?,?,?,?,?,?)", commit_row)
            manual_id = batch.commit_id if batch.origin == "manual" else old["manual_id"]
            protected_id = batch.commit_id if batch.head is not None else old["protected_id"]
            if batch.origin == "manual":
                self._meta(usage, 32)
                self._db.execute("INSERT INTO manual_marks VALUES(?,?)", (batch.commit_id, batch.head))
            self._db.execute("UPDATE control SET generation=?,save_id=?,manual_id=?,protected_id=?,node_count=?,block_count=?,blob_bytes=?,metadata_bytes=? WHERE singleton=1",
                             (u64(generation), batch.commit_id, manual_id, protected_id, usage["nodes"],
                              usage["blocks"], usage["blob_bytes"], usage["metadata_bytes"]))
            if self._db.execute("SELECT 1 FROM nodes WHERE id=? AND parent IS NULL", (self.root,)).fetchone() is None:
                raise StorageError("missing declared root")
            if hook:
                hook("before_commit", self)
            mark = time.perf_counter()
            self._db.execute("COMMIT")
            committed = True
            times["commit_ms"] = (time.perf_counter() - mark) * 1000
            times["total_ms"] = (time.perf_counter() - start) * 1000
            if hook:
                hook("after_commit", self)
            return Receipt(batch.commit_id, generation, True,
                           new_nodes=usage["nodes"] - old["nodes"],
                           new_blocks=usage["blocks"] - old["blocks"],
                           new_blob_bytes=usage["blob_bytes"] - old["blob_bytes"], timings=times)
        except BaseException:
            if not committed and self._db.in_transaction:
                self._db.execute("ROLLBACK")
            raise

    def verify(self):
        if self.unknown_required:
            raise StorageError("unknown required capability cannot be fully interpreted")
        if self._db.execute("PRAGMA quick_check").fetchone()[0] != "ok":
            raise StorageError("SQLite integrity failure")
        if self._db.execute("PRAGMA foreign_key_check").fetchone() is not None:
            raise StorageError("broken foreign key")
        status = self.status()
        counts = (self._db.execute("SELECT count(*) FROM nodes").fetchone()[0],
                  self._db.execute("SELECT count(*) FROM blocks").fetchone()[0])
        if counts[0] > self.limits.nodes or counts[1] > self.limits.blocks:
            raise StorageError("metadata row budget")
        if status["blob_bytes"] > self.limits.blob_bytes or status["metadata_bytes"] > self.limits.metadata_bytes:
            raise StorageError("stored byte budget")
        metadata_bytes = 0
        if self._db.execute("SELECT 1 FROM blocks WHERE length(bytes)>? LIMIT 1", (MAX_JSON_BYTES,)).fetchone():
            raise StorageError("stored block byte limit")
        for sha, raw in self._db.execute("SELECT sha,bytes FROM blocks ORDER BY sequence"):
            if type(raw) is not bytes or len(raw) > MAX_JSON_BYTES:
                raise StorageError("block byte limit")
            block = Block(raw)
            if block.sha != sha or block.envelope()["kind"] not in BLOCK_KINDS:
                raise StorageError("block hash/codec mismatch")
            for key, table in (("blocks", "block_refs"), ("blobs", "blob_refs")):
                actual = sorted(r[0].hex() for r in self._db.execute("SELECT target FROM " + table + " WHERE owner=?", (sha,)))
                if actual != block.envelope()[key]:
                    raise StorageError("dependency index mismatch")
            metadata_bytes += len(raw)
        if self._db.execute("SELECT 1 FROM block_refs r JOIN blocks a ON r.owner=a.sha JOIN blocks b ON r.target=b.sha WHERE b.sequence>=a.sequence LIMIT 1").fetchone():
            raise StorageError("block graph cycle/topological order")
        if self._db.execute("SELECT count(*) FROM nodes WHERE parent IS NULL").fetchone()[0] != 1:
            raise StorageError("invalid root count")
        for value, checksum, depth in self._db.execute("SELECT id,checksum,depth FROM nodes ORDER BY depth"):
            node = self.node(value)
            if node.checksum != checksum:
                raise StorageError("node checksum mismatch")
            state = self.block(node.state).envelope()
            if (state["kind"] != "fixture.project" or type(state["value"]) is not dict
                    or state["value"].get("project_id") != self.project.hex()):
                raise StorageError("node state identity mismatch")
            if node.parent is None:
                if value != self.root or depth != 0:
                    raise StorageError("invalid declared root")
            else:
                parent_depth = self._db.execute("SELECT depth FROM nodes WHERE id=?", (node.parent,)).fetchone()
                if parent_depth is None or parent_depth[0] + 1 != depth:
                    raise StorageError("history cycle/depth mismatch")
            metadata_bytes += len(node.record())
        total = 0
        for sha, byte_len in self._db.execute("SELECT sha,byte_len FROM blobs"):
            if byte_len > self.limits.per_blob:
                raise StorageError("stored original byte limit")
            for _ in self.blob_parts(sha):
                pass
            total += byte_len
        for row in self._db.execute("SELECT node_id,name,value FROM tags"):
            from .values import TagEdit
            TagEdit(*row)
            metadata_bytes += 16 + len(row[1].encode("utf8")) + len(row[2].encode("utf8"))
        if self._db.execute("SELECT 1 FROM tags GROUP BY node_id HAVING count(*)>64 LIMIT 1").fetchone():
            raise StorageError("stored tag count budget")
        previous, generation, manual, protected = None, 0, None, None
        for row in self._db.execute("SELECT id,previous,generation,origin,head,navigation,request_hash FROM commits ORDER BY generation"):
            generation += 1
            if row[1] != previous or int.from_bytes(row[2], "big") != generation:
                raise StorageError("invalid commit lineage")
            if row[3] not in {"manual", "automatic", "exit", "metadata"}:
                raise StorageError("unknown commit origin")
            if row[3] == "metadata" and (row[4] is not None or row[5] is not None):
                raise StorageError("metadata commit moved a checkpoint")
            if row[5] is not None:
                self._navigation(row[5])
            if row[3] == "manual":
                if row[4] is None or self._db.execute("SELECT node_id FROM manual_marks WHERE commit_id=?", (row[0],)).fetchone() != (row[4],):
                    raise StorageError("missing real manual-save fact")
                manual = row[0]
            elif self._db.execute("SELECT 1 FROM manual_marks WHERE commit_id=?", (row[0],)).fetchone():
                raise StorageError("forged manual-save fact")
            if row[4] is not None:
                protected = row[0]
            previous = row[0]
            metadata_bytes += self._commit_size(row)
        metadata_bytes += self._db.execute("SELECT count(*) FROM manual_marks").fetchone()[0] * 32
        for kind, value, first in self._db.execute("SELECT kind,id,first_node FROM object_ids"):
            if kind not in OBJECT_KINDS:
                raise StorageError("unknown identity registry kind")
            identity(value)
            identity(first)
            metadata_bytes += len(kind) + 32
        for name, size in self._db.execute("SELECT name,length(bytes) FROM extensions"):
            if not re.fullmatch(r"vendor\.[a-z0-9.-]{1,100}", name) or size > MAX_JSON_BYTES:
                raise StorageError("invalid opaque extension")
            metadata_bytes += len(name.encode("utf8")) + size
        actual_counts = (*counts, total, metadata_bytes)
        if actual_counts != (status["nodes"], status["blocks"], status["blob_bytes"], status["metadata_bytes"]):
            raise StorageError("stored usage mismatch")
        if (previous, generation, manual, protected) != (status["save_id"], status["generation"], status["manual_id"], status["protected_id"]):
            raise StorageError("stale checkpoint/control index")
        for key in ("manual_id", "protected_id"):
            if status[key] is not None:
                if self._db.execute("SELECT head FROM commits WHERE id=?", (status[key],)).fetchone()[0] is None:
                    raise StorageError("checkpoint has no head")
        return actual_counts
