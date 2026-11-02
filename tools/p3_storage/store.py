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
                     AddressBinding, StateView, fixture_projection, inspect_block,
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
    try:
        for statement in ("foreign_keys=ON", "recursive_triggers=ON", "trusted_schema=OFF",
                          "cache_size=-2048", "mmap_size=0", "temp_store=FILE", "synchronous=EXTRA"):
            db.execute("PRAGMA " + statement)
        if db.execute("PRAGMA foreign_keys").fetchone()[0] != 1:
            raise StorageError("foreign key support required")
        if db.execute("PRAGMA synchronous").fetchone()[0] != 3:
            raise StorageError("EXTRA synchronous setting unavailable")
        if not writable:
            db.execute("PRAGMA query_only=ON")
        return db
    except BaseException:
        db.close()
        raise


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
    before = path.stat()
    with path.open("rb") as stream:
        raw = stream.read(100)
    if (len(raw) != 100 or raw[:16] != b"SQLite format 3\0"
            or int.from_bytes(raw[68:72], "big") != APP_ID
            or int.from_bytes(raw[60:64], "big") != FORMAT_VERSION):
        raise StorageError("unknown container header; no write attempted")
    if raw[18:20] != b"\x01\x01" or int.from_bytes(raw[16:18], "big") != 4096:
        raise StorageError("unsupported page/journal format")
    after = path.stat()
    if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino):
        raise StorageError("file replaced while admitting header")
    return before.st_dev, before.st_ino


class Store:
    def __init__(self, path, db, limits, writable):
        self.path, self._db, self.limits, self.writable = Path(path), db, limits, writable
        self.epoch = new_identity()
        row = db.execute("SELECT family,profile,container_version,project_schema,history_schema,project_id,history_id,lineage_id,root_id FROM header WHERE singleton=1").fetchone()
        if row is None or row[:5] != ("skin-editor-project", "p3-0-r2", 2, 1, 2):
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
            db.execute("PRAGMA user_version=2")
            db.executescript("BEGIN IMMEDIATE;\n" + schema.script())
            db.execute("INSERT INTO header VALUES(1,?,?,?,?,?,?,?,?,?)",
                       ("skin-editor-project", "p3-0-r2", 2, 1, 2, project, history, lineage, root))
            db.executemany("INSERT INTO features VALUES(?,?,1)", REQUIRED_FEATURES)
            db.execute("INSERT INTO control VALUES(1,?,NULL,NULL,NULL,0,0,0,0,?)", (u64(0, zero=True), new_identity()))
            db.execute("COMMIT")
            return cls(path, db, limits, True)
        except BaseException:
            if db.in_transaction:
                db.execute("ROLLBACK")
            db.close()
            raise

    @classmethod
    def open(cls, path, *, writable=True, recover=False, verify=True, limits=None,
             reference=False, cancel=None, checkpoint=None):
        limits = limits or Limits()
        admitted_file = _raw_header(path, limits)
        # Read-only probe must not silently recover an unknown/future file.
        db = _connection(path, bool(recover))
        try:
            db.execute("BEGIN")
            if _layout(db) != _expected_layout():
                raise StorageError("unknown or modified schema; writes refused")
            probe = cls(path, db, limits, False)
            if writable and not probe.unknown_required:
                db.close()
                db = _connection(path, True)
                db.execute("BEGIN")
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
                result.verify(reference=reference, cancel=cancel, checkpoint=checkpoint)
            if result._file_id != admitted_file or (result.path.stat().st_dev, result.path.stat().st_ino) != admitted_file:
                raise StorageError("file replaced during validation")
            db.execute("COMMIT")
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
        row = self._db.execute("SELECT generation,save_id,manual_id,protected_id,node_count,block_count,blob_bytes,metadata_bytes,address_epoch FROM control WHERE singleton=1").fetchone()
        if row is None:
            raise StorageError("missing control record")
        return dict(zip(("generation", "save_id", "manual_id", "protected_id", "nodes", "blocks", "blob_bytes", "metadata_bytes", "address_epoch"),
                        (int.from_bytes(row[0], "big"), *row[1:])))

    def captured(self, origin, **values):
        """Prototype capture value. Does not access a real ProjectSession."""
        status = self.status()
        return Batch(self.epoch, new_identity(), status["save_id"], origin,
                     address_epoch=status["address_epoch"], **values)

    def block(self, sha):
        length = self._db.execute("SELECT length(bytes) FROM blocks WHERE sha=?", (digest(sha),)).fetchone()
        if length is None or length[0] > MAX_JSON_BYTES:
            raise StorageError("missing or oversized state block")
        row = self._db.execute("SELECT bytes FROM blocks WHERE sha=?", (digest(sha),)).fetchone()
        if row is None:
            raise StorageError("missing state block")
        return Block(row[0])

    def node(self, value):
        length = self._db.execute("SELECT length(action),record_version FROM nodes WHERE id=?", (identity(value),)).fetchone()
        if length is None or length[0] > 64 * 1024 or length[1] != 2:
            raise StorageError("missing or oversized node action")
        row = self._db.execute("SELECT n.id,n.parent,n.state,a.alias,a.branch,a.ordinal,n.action FROM nodes n JOIN addresses a ON a.node_id=n.id WHERE n.id=?", (identity(value),)).fetchone()
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

    def resolve(self, text, *, epoch, project, history, address_epoch=None):
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
            if address_epoch != self.status()["address_epoch"]:
                raise StorageError("stale address mapping epoch")
            row = self._db.execute("SELECT node_id FROM addresses WHERE alias=?", (u64(parse_alias(text)),)).fetchone()
            result = row[0] if row else None
        elif text.startswith("b"):
            if address_epoch != self.status()["address_epoch"]:
                raise StorageError("stale address mapping epoch")
            branch, ordinal = parse_branch(text)
            row = self._db.execute("SELECT node_id FROM addresses WHERE branch=? AND ordinal=?", (u64(branch), u64(ordinal, zero=True))).fetchone()
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
        project_id = (envelope["value"].get("project_id") if envelope["kind"] == "fixture.project"
                      and type(envelope["value"]) is dict else None)
        if project_id is not None and (type(project_id) is not str or not re.fullmatch(r"[0-9a-f]{32}", project_id)):
            raise StorageError("invalid project identity index")
        self._db.execute("INSERT INTO blocks VALUES(?,?,?,?,?,?)",
                         (block.sha, u64(sequence), block.raw, envelope["kind"], envelope["schema"], project_id))
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
            if row[0] != node.checksum or self.node(node.id).record() != node.record():
                raise StorageError("immutable node conflict")
            if self.node(node.id).address != node.address:
                raise StorageError("display binding requires an explicit remap")
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
        size = len(node.record()) + len(node.address.record()) + 32
        if usage["metadata_bytes"] + size > self.limits.metadata_bytes:
            raise StorageError("history metadata budget")
        self._db.execute("INSERT INTO nodes VALUES(?,?,?,?,?,?,2)",
                         (node.id, node.parent, node.state, depth, node.action, node.checksum))
        self._put_address(node.address)
        usage["nodes"] += 1
        usage["metadata_bytes"] += size
        return True

    def _put_address(self, binding):
        self._db.execute("INSERT INTO addresses VALUES(?,?,?,?,?)",
                         (binding.node, u64(binding.alias), u64(binding.branch),
                          u64(binding.ordinal, zero=True), binding.checksum))

    def _validate_view(self, view):
        source, target = self.block(view.origin), self.block(view.target)
        if source.sha != view.origin or target.sha != view.target:
            raise StorageError("state view resource checksum")
        expected = fixture_projection(source)
        if expected.envelope() != target.envelope():
            raise StorageError("state view changes original semantics/dependencies")

    def _put_view(self, view, usage):
        self._validate_view(view)
        prior = self._db.execute("SELECT target,converter,checksum FROM state_views WHERE origin=? AND target_schema=2 AND converter=?", (view.origin, view.converter)).fetchone()
        if prior:
            if prior != (view.target, view.converter, view.checksum):
                raise StorageError("immutable state view conflict")
            return False
        self._meta(usage, len(view.record()) + 32)
        self._db.execute("INSERT INTO state_views VALUES(?,2,?,?,?)",
                         (view.origin, view.target, view.converter, view.checksum))
        return True

    def state_for(self, node_id, *, target_schema=None, converter="fixture.step-to-index.v1"):
        """Origin stays in the node; a known, append-only view is selected separately."""
        original = self.block(self.node(node_id).state)
        if target_schema is None or original.envelope()["schema"] == target_schema:
            return original
        row = self._db.execute("SELECT target,converter,checksum FROM state_views WHERE origin=? AND target_schema=? AND converter=?",
                               (original.sha, target_schema, converter)).fetchone()
        if row is None:
            raise StorageError("missing verified schema view")
        view = StateView(original.sha, row[0], row[1])
        if row[2] != view.checksum:
            raise StorageError("state view checksum")
        self._validate_view(view)
        return self.block(view.target)

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
        if (sum(len(b.raw) for b in batch.blocks)
                + sum(len(n.record()) + len(n.address.record()) + 32 for n in batch.nodes)
                + sum(len(raw) for _, raw in batch.extensions)
                + sum(16 + len(t.key.encode("utf8")) + len((t.value or "").encode("utf8")) for t in batch.tags)
                + len(batch.identities) * 64
                + sum(len(v.record()) + 32 for v in batch.views)
                + sum(len(a.record()) + 32 for a in batch.remap) > self.limits.batch_metadata):
            raise StorageError("batch metadata admission limit")
        if len(batch.tags) > 4096 or len(batch.identities) > 4096 or len(batch.extensions) > 64:
            raise StorageError("batch annotation/identity limit")
        if len(batch.views) > 1024 or len(batch.remap) > self.limits.nodes:
            raise StorageError("view/address batch limit")
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
            if batch.address_epoch != old["address_epoch"]:
                raise StorageError("stale address mapping epoch")
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
            for view in batch.views:
                changed |= self._put_view(view, usage)
            times["blocks_ms"] = (time.perf_counter() - mark) * 1000
            if hook:
                hook("blocks", self)
            for node in batch.nodes:
                changed |= self._put_node(node, usage)
            address_epoch = old["address_epoch"]
            if batch.new_address_epoch is not None:
                if batch.new_address_epoch == address_epoch or len(batch.remap) != usage["nodes"]:
                    raise StorageError("replacement mapping must cover all nodes with a fresh epoch")
                # This deletes display bindings, never a history/content record.
                self._db.execute("DELETE FROM addresses")
                for binding in batch.remap:
                    self._put_address(binding)
                address_epoch = batch.new_address_epoch
                changed = True
            elif batch.remap:
                raise StorageError("remap needs a fresh address epoch")
            if self._db.execute("SELECT count(*) FROM addresses").fetchone()[0] != usage["nodes"]:
                raise StorageError("missing node display binding")
            if usage["metadata_bytes"] > self.limits.metadata_bytes:
                raise StorageError("address/history metadata budget")
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
                          batch.head, batch.navigation, fingerprint, address_epoch)
            self._meta(usage, self._commit_size(commit_row))
            self._db.execute("INSERT INTO commits VALUES(?,?,?,?,?,?,?,?)", commit_row)
            manual_id = batch.commit_id if batch.origin == "manual" else old["manual_id"]
            protected_id = batch.commit_id if batch.head is not None else old["protected_id"]
            if batch.origin == "manual":
                self._meta(usage, 32)
                self._db.execute("INSERT INTO manual_marks VALUES(?,?)", (batch.commit_id, batch.head))
            self._db.execute("UPDATE control SET generation=?,save_id=?,manual_id=?,protected_id=?,node_count=?,block_count=?,blob_bytes=?,metadata_bytes=?,address_epoch=? WHERE singleton=1",
                             (u64(generation), batch.commit_id, manual_id, protected_id, usage["nodes"],
                              usage["blocks"], usage["blob_bytes"], usage["metadata_bytes"], address_epoch))
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

    def verify(self, *, reference=False, cancel=None, checkpoint=None, trace_sql=False):
        """One read snapshot, cancellable bounded scans; no project activation."""
        owns_transaction = not self._db.in_transaction
        if owns_transaction:
            self._db.execute("BEGIN")
        started = time.perf_counter()
        self.verification_report = {"reference": reference, "phases_ms": {}, "sql_statements": None}
        last_time, last_phase = started, "admission"
        statements = [0]
        def trace(_):
            statements[0] += 1
        def pulse(phase, units=0):
            nonlocal last_time, last_phase
            now = time.perf_counter()
            if phase != last_phase:
                self.verification_report["phases_ms"][last_phase] = (now - last_time) * 1000
                last_time, last_phase = now, phase
            if cancel and cancel():
                raise InterruptedError("verification cancelled")
            if checkpoint:
                checkpoint(phase, units)
        if trace_sql:
            self._db.set_trace_callback(trace)
        if cancel:
            self._db.set_progress_handler(lambda: int(bool(cancel())), 1000)
        try:
            result = self._verify_body(reference, pulse)
            pulse("complete")
            if owns_transaction:
                self._db.execute("COMMIT")
            self.verification_report.update(total_ms=(time.perf_counter() - started) * 1000,
                                            counts=result, sql_statements=statements[0] if trace_sql else None)
            return result
        except BaseException:
            self._db.set_progress_handler(None, 0)
            if owns_transaction and self._db.in_transaction:
                self._db.execute("ROLLBACK")
            raise
        finally:
            self._db.set_progress_handler(None, 0)
            if trace_sql:
                self._db.set_trace_callback(None)

    def _verify_body(self, reference, pulse):
        if self.unknown_required:
            raise StorageError("unknown required capability cannot be fully interpreted")
        pulse("sqlite_integrity")
        if self._db.execute("PRAGMA integrity_check").fetchall() != [("ok",)]:
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
        pulse("blocks")
        if self._db.execute("SELECT 1 FROM blocks WHERE length(bytes)>? OR length(kind)>64 OR length(project_id)>32 LIMIT 1", (MAX_JSON_BYTES,)).fetchone():
            raise StorageError("stored block byte limit")
        if self._db.execute("SELECT 1 FROM nodes WHERE length(action)>65536 LIMIT 1").fetchone():
            raise StorageError("stored action byte limit")
        streams = {} if reference else {
            key: iter(self._db.execute("SELECT owner,target FROM " + table + " ORDER BY owner,target"))
            for key, table in (("blocks", "block_refs"), ("blobs", "blob_refs"))}
        pending = {key: next(cursor, None) for key, cursor in streams.items()}
        for index, (sha, raw, kind, version, project_id) in enumerate(self._db.execute(
                "SELECT sha,bytes,kind,schema_version,project_id FROM blocks ORDER BY sha")):
            if index % 256 == 0:
                pulse("blocks", index)
            if type(raw) is not bytes or len(raw) > MAX_JSON_BYTES:
                raise StorageError("block byte limit")
            envelope = Block(raw).envelope() if reference else inspect_block(raw)
            if hashlib.sha256(raw).digest() != sha or envelope["kind"] not in BLOCK_KINDS:
                raise StorageError("block hash/codec mismatch")
            expected_project = (envelope["value"].get("project_id")
                                if kind == "fixture.project" and type(envelope["value"]) is dict else None)
            if (kind, version, project_id) != (envelope["kind"], envelope["schema"], expected_project):
                raise StorageError("derived state index mismatch")
            for key, table in (("blocks", "block_refs"), ("blobs", "blob_refs")):
                if reference:
                    actual = sorted(r[0].hex() for r in self._db.execute("SELECT target FROM " + table + " WHERE owner=?", (sha,)))
                else:
                    actual = []
                    while pending[key] is not None and pending[key][0] == sha:
                        actual.append(pending[key][1].hex())
                        if len(actual) > 8192:
                            raise StorageError("dependency row budget")
                        pending[key] = next(streams[key], None)
                if actual != envelope[key]:
                    raise StorageError("dependency index mismatch")
            metadata_bytes += len(raw)
        if any(row is not None for row in pending.values()):
            raise StorageError("stray dependency rows")
        if self._db.execute("SELECT 1 FROM block_refs r JOIN blocks a ON r.owner=a.sha JOIN blocks b ON r.target=b.sha WHERE b.sequence>=a.sequence LIMIT 1").fetchone():
            raise StorageError("block graph cycle/topological order")
        if self._db.execute("SELECT count(*) FROM nodes WHERE parent IS NULL").fetchone()[0] != 1:
            raise StorageError("invalid root count")
        pulse("nodes")
        if self._db.execute("SELECT count(*) FROM addresses").fetchone()[0] != counts[0]:
            raise StorageError("incomplete address mapping")
        query = """SELECT n.id,n.checksum,n.depth,n.parent,n.state,n.action,
                    a.alias,a.branch,a.ordinal,a.checksum,p.depth,b.kind,b.schema_version,b.project_id,n.record_version
                    FROM nodes n JOIN addresses a ON n.id=a.node_id
                    LEFT JOIN nodes p ON n.parent=p.id JOIN blocks b ON n.state=b.sha"""
        for index, row in enumerate(self._db.execute(query)):
            if index % 256 == 0:
                pulse("nodes", index)
            value, checksum, depth = row[:3]
            if row[14] != 2:
                raise StorageError("unsupported node record version")
            node = (self.node(value) if reference else Node(value, row[3], row[4],
                    *(int.from_bytes(v, "big") for v in row[6:9]), row[5]))
            if node.checksum != checksum:
                raise StorageError("node checksum mismatch")
            if reference:
                state = self.block(node.state).envelope()
                valid_state = (state["kind"] == "fixture.project" and type(state["value"]) is dict
                               and state["value"].get("project_id") == self.project.hex())
            else:
                valid_state = row[11] == "fixture.project" and row[12] in (1, 2) and row[13] == self.project.hex()
            if not valid_state:
                raise StorageError("node state identity mismatch")
            if node.parent is None:
                if value != self.root or depth != 0:
                    raise StorageError("invalid declared root")
            else:
                parent_depth = (self._db.execute("SELECT depth FROM nodes WHERE id=?", (node.parent,)).fetchone()[0]
                                if reference else row[10])
                if parent_depth is None or parent_depth + 1 != depth:
                    raise StorageError("history cycle/depth mismatch")
            if node.address.checksum != row[9]:
                raise StorageError("display binding checksum")
            metadata_bytes += len(node.record()) + len(node.address.record()) + 32
        total = 0
        pulse("originals")
        for sha, byte_len in self._db.execute("SELECT sha,byte_len FROM blobs"):
            if byte_len > self.limits.per_blob:
                raise StorageError("stored original byte limit")
            for index, _ in enumerate(self.blob_parts(sha)):
                pulse("originals", index)
            total += byte_len
        pulse("metadata")
        for row in self._db.execute("SELECT node_id,name,value FROM tags"):
            from .values import TagEdit
            TagEdit(*row)
            metadata_bytes += 16 + len(row[1].encode("utf8")) + len(row[2].encode("utf8"))
        if self._db.execute("SELECT 1 FROM tags GROUP BY node_id HAVING count(*)>64 LIMIT 1").fetchone():
            raise StorageError("stored tag count budget")
        previous, generation, manual, protected = None, 0, None, None
        for row in self._db.execute("SELECT id,previous,generation,origin,head,navigation,request_hash,address_epoch FROM commits ORDER BY generation"):
            identity(row[7])
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
        if row[7] != status["address_epoch"]:
            raise StorageError("stale address epoch/control")
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
        for origin, target_schema, target, converter, checksum in self._db.execute(
                "SELECT origin,target_schema,target,converter,checksum FROM state_views"):
            view = StateView(origin, target, converter)
            if target_schema != 2 or view.checksum != checksum:
                raise StorageError("state view checksum/schema")
            self._validate_view(view)
            metadata_bytes += len(view.record()) + 32
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
