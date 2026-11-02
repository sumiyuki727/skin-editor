"""Deterministic prototype regressions; no product, window, device or network."""

from dataclasses import replace
from contextlib import closing
import hashlib
import json
from pathlib import Path
import shutil
import sqlite3
import tempfile
import unittest
from unittest.mock import patch

from p3_storage.fixtures import append, make_store, project_block, write_media
from p3_storage.identity import (ContractError, NodeReference, U64_MAX, U128_MAX,
                                 branch_address, new_identity, node_key, parse_alias,
                                 parse_branch, parse_identity, parse_node_key, short_alias)
from p3_storage.schema import IMMUTABLE
from p3_storage.store import StorageError, Store
from p3_storage.values import (Batch, BlobInput, Block, Limits, Node, TagEdit,
                              AddressBinding, StateView, FORMAT_VERSION, fixture_projection,
                              decode_json, encode_json, optional, sized)


class IdentityTests(unittest.TestCase):
    def test_display_binding_does_not_change_permanent_history_checksum(self):
        node = Node(new_identity(), None, b"s" * 32, 1, 1, 0, b'{"kind":"root"}')
        other = replace(node, alias=999, branch=12, ordinal=345)
        self.assertEqual(node.record(), other.record())
        self.assertEqual(node.checksum, other.checksum)
        self.assertNotEqual(node.address.checksum, other.address.checksum)

    def test_current_frozen_vectors(self):
        v = json.loads((Path(__file__).parent / "p3_storage/golden_v2.json").read_text())
        node = Node(bytes.fromhex(v["node"]), None, bytes.fromhex(v["state_sha"]),
                    1, 1, 0, v["action_utf8"].encode())
        self.assertEqual(node.record().hex(), v["node_frame_hex"])
        self.assertEqual(node.checksum.hex(), v["node_sha256"])
        self.assertEqual(node.address.checksum.hex(), v["address_sha256"])
        batch = Batch(bytes.fromhex("05" * 16), bytes.fromhex(v["batch_commit"]), None,
                      "manual", address_epoch=bytes.fromhex(v["address_epoch"]), head=node.id, nodes=(node,))
        self.assertEqual(batch.fingerprint().hex(), v["batch_sha256"])
    def test_frozen_language_neutral_vectors(self):
        v = json.loads((Path(__file__).parent / "p3_storage/golden_v1.json").read_text(encoding="utf8"))
        p, h, n = (bytes.fromhex(v[k]) for k in ("project", "history", "node"))
        self.assertEqual(NodeReference(p, h, n).text(), v["qualified"])
        self.assertEqual(node_key(n), v["node_key"])
        self.assertEqual(short_alias(U64_MAX), v["alias"])
        self.assertEqual(branch_address(U64_MAX, U64_MAX), v["branch"])
        # Legacy vector stays unchanged and explicitly uses its original frame.
        action = v["action_utf8"].encode("utf8")
        legacy = (b"skin-editor.node.v1\0" + n + b"\0" + bytes.fromhex(v["state_sha"])
                  + (1).to_bytes(8, "big") * 2 + (0).to_bytes(8, "big") + sized(action))
        self.assertEqual(legacy.hex(), v["node_frame_hex"])
        self.assertEqual(hashlib.sha256(legacy).hexdigest(), v["node_sha256"])

    def test_fixed_width_persistent_and_qualified_addresses(self):
        for value in (1, 35, 36, (1 << 64), U128_MAX):
            raw = value.to_bytes(16, "big")
            self.assertEqual(parse_identity(raw.hex()), raw)
            self.assertEqual(parse_node_key(node_key(raw)), raw)
            self.assertLessEqual(len(node_key(raw)), 26)
        ref = NodeReference(new_identity(), new_identity(), new_identity())
        self.assertEqual(NodeReference.parse(ref.text()), ref)
        self.assertEqual(len(ref.text()), 102)

    def test_local_u64_addresses_remain_bounded(self):
        self.assertEqual(parse_alias(short_alias(U64_MAX)), U64_MAX)
        self.assertEqual(len(short_alias(U64_MAX)), 14)
        value = branch_address(U64_MAX, U64_MAX)
        self.assertEqual(len(value), 28)
        self.assertEqual(parse_branch(value), (U64_MAX, U64_MAX))

    def test_noncanonical_or_wrong_version_rejected(self):
        for value in ("k0", "k01", "kA", "n1", "k" + "z" * 26, "k-1"):
            with self.subTest(value=value), self.assertRaises(ContractError):
                parse_node_key(value)
        for value in ("n0", "n01", "nA", "n" + "z" * 14):
            with self.subTest(value=value), self.assertRaises(ContractError):
                parse_alias(value)
        for value in ("b0:1", "b1:00", "b1:1:1", "bA:0"):
            with self.subTest(value=value), self.assertRaises(ContractError):
                parse_branch(value)
        for value in ("0" * 32, "A" * 32, "1" * 31, 1.0):
            with self.subTest(value=value), self.assertRaises(ContractError):
                parse_identity(value)

    def test_json_rules_preserve_unknown_bytes_elsewhere(self):
        for raw in (b'{"x":1,"x":2}', b'{"x":NaN}', b'"\xff"'):
            with self.assertRaises(ContractError):
                decode_json(raw)
        with self.assertRaises(ContractError):
            encode_json({"large": 1 << 64})
        self.assertEqual(decode_json(b'{ "x": 1e-07 }'), {"x": 0.0000001})
        self.assertEqual(decode_json(b'{"x":0.0000001}'), {"x": 0.0000001})

    def test_node_hash_keeps_original_json_lexical_bytes(self):
        args = (bytes.fromhex("01" * 16), None, bytes.fromhex("02" * 32), 1, 1, 0)
        first, second = Node(*args, b'{"x":1}'), Node(*args, b'{ "x": 1 }')
        self.assertEqual(decode_json(first.action), decode_json(second.action))
        self.assertNotEqual(first.checksum, second.checksum)
        self.assertTrue(first.record().endswith(b'{"x":1}'))


class StorageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.directory = Path(self.temp.name)
        self.path = self.directory / "fixture.oskproj"
        self.store, self.root, _ = make_store(self.path)

    def tearDown(self):
        self.store.close()
        self.temp.cleanup()

    def reopen(self):
        self.store.close()
        self.store = Store.open(self.path)

    def test_initial_file_is_self_contained_and_context_is_fresh(self):
        epoch = self.store.epoch
        self.reopen()
        self.assertNotEqual(epoch, self.store.epoch)
        self.assertEqual(self.store.open_position()[0], self.root)
        self.assertEqual(self.store.verify()[0], 1)
        self.assertFalse(Path(str(self.path) + "-journal").exists())

    def test_all_addresses_survive_reopen_but_old_context_is_rejected(self):
        epoch = self.store.epoch
        reference = NodeReference(self.store.project, self.store.history, self.root.id)
        self.reopen()
        for address in (reference.text(), node_key(self.root.id), "n1", "b1:0"):
            self.assertEqual(self.store.resolve(address, epoch=self.store.epoch,
                             project=self.store.project, history=self.store.history,
                             address_epoch=self.store.status()["address_epoch"]), self.root.id)
            with self.assertRaises(StorageError):
                self.store.resolve(address, epoch=epoch, project=self.store.project, history=self.store.history)

    def test_same_alias_in_other_project_is_not_a_reference(self):
        other, _, _ = make_store(self.directory / "other.oskproj")
        try:
            reference = NodeReference(other.project, other.history, other.root)
            with self.assertRaises(StorageError):
                self.store.resolve(reference.text(), epoch=self.store.epoch,
                                   project=self.store.project, history=self.store.history)
        finally:
            other.close()

    def test_copy_diverges_without_durable_node_collisions(self):
        copy_path = self.directory / "copy.oskproj"
        shutil.copyfile(self.path, copy_path)
        with Store.open(copy_path) as other:
            a, _ = append(self.store, self.root, 2)
            b, _ = append(other, self.root, 2)
            self.assertNotEqual(a.id, b.id)
            self.assertEqual(a.alias, b.alias)
            self.assertEqual(other.root, self.root.id)
            self.assertEqual(self.store.project, other.project)

    def test_branches_and_manual_versus_latest_are_retained(self):
        a, _ = append(self.store, self.root, 2, origin="manual")
        b, _ = append(self.store, a, 3, step=2)
        c, _ = append(self.store, a, 4, step=3, branch=2, ordinal=0, origin="exit")
        self.reopen()
        self.assertEqual(self.store.open_position()[0].id, a.id)
        self.assertEqual(self.store.open_position(latest=True)[0].id, c.id)
        self.assertEqual(self.store.node(b.id), b)
        before = self.store.status()
        no_change = self.store.commit(self.store.captured("automatic"))
        self.assertFalse(no_change.changed)
        self.assertEqual(before, self.store.status())

    def test_older_content_manual_save_never_replaces_newer_tree(self):
        a, _ = append(self.store, self.root, 2)
        b, _ = append(self.store, a, 3, step=2)
        self.store.commit(self.store.captured("manual", head=self.root.id))
        self.reopen()
        self.assertEqual(self.store.open_position()[0].id, self.root.id)
        self.assertEqual(self.store.status()["nodes"], 3)
        self.assertEqual(self.store.node(b.id), b)

    def test_tags_change_without_changing_node_or_checkpoint(self):
        original = self.store.node(self.root.id)
        manual = self.store.status()["manual_id"]
        for value in ("important", "reviewed", None):
            self.store.commit(self.store.captured("metadata", tags=(TagEdit(self.root.id, "user:note", value),)))
            self.assertEqual(self.store.node(self.root.id), original)
            self.assertEqual(self.store.status()["nodes"], 1)
            self.assertEqual(self.store.status()["manual_id"], manual)
        self.reopen()
        self.assertEqual(self.store.tags(self.root.id), {})

    def test_system_saved_tag_cannot_be_forged(self):
        with self.assertRaises(ContractError):
            TagEdit(self.root.id, "system:saved", "yes")

    def test_same_node_identity_with_other_state_or_action_is_rejected(self):
        before = self.store.status()
        bad = replace(self.root, action=encode_json({"kind": "forged"}))
        with self.assertRaises(StorageError):
            self.store.commit(self.store.captured("automatic", nodes=(bad,)))
        self.assertEqual(before, self.store.status())
        self.assertEqual(self.root, self.store.node(self.root.id))

    def test_sql_update_delete_and_replace_guards(self):
        for sql, values in (("DELETE FROM nodes WHERE id=?", (self.root.id,)),
                            ("UPDATE nodes SET action=? WHERE id=?", (b"{}", self.root.id)),
                            ("INSERT OR REPLACE INTO nodes SELECT * FROM nodes WHERE id=?", (self.root.id,))):
            with self.subTest(sql=sql), self.assertRaises(sqlite3.IntegrityError):
                self.store._db.execute(sql, values)
        for name in IMMUTABLE:
            triggers = self.store._db.execute("SELECT count(*) FROM sqlite_master WHERE type='trigger' AND tbl_name=?", (name,)).fetchone()[0]
            self.assertEqual(triggers, 3, name)

    def test_no_public_delete_replace_or_application_dependency(self):
        self.assertFalse(any(name in Store.__dict__ for name in ("delete_node", "trim", "replace_node", "clear")))
        root = Path(__file__).resolve().parents[1]
        for path in [*root.glob("crates/**/src/*.rs"), *root.glob("desktop/**/src/*.rs")]:
            self.assertNotIn("p3_storage", path.read_text(encoding="utf8"))

    def test_original_media_reused_and_unknown_audio_bytes_preserved(self):
        media = write_media(self.directory)
        a, _ = append(self.store, self.root, 2, blobs=media)
        initial = self.store.status()["blob_bytes"]
        append(self.store, self.root, 3, branch=2, ordinal=0, blobs=media)
        self.assertEqual(initial, self.store.status()["blob_bytes"])
        self.reopen()
        for blob in media:
            self.assertEqual(b"".join(self.store.blob_parts(blob.sha)), blob.path.read_bytes())
        self.assertEqual(a, self.store.node(a.id))

    def test_missing_reference_rolls_back_whole_batch(self):
        before = self.store.status()
        block = project_block(self.store.project, blocks=(b"X" * 32,))
        with self.assertRaises(StorageError):
            self.store.commit(self.store.captured("automatic", blocks=(block,)))
        self.assertEqual(before, self.store.status())

    def test_budget_failure_releases_transaction_without_trimming(self):
        self.store.limits = replace(self.store.limits, nodes=1)
        with self.assertRaises(StorageError):
            append(self.store, self.root, 2)
        self.assertEqual(self.store.status()["nodes"], 1)
        self.assertFalse(self.store._db.in_transaction)
        self.assertEqual(self.store.node(self.root.id), self.root)

    def test_commit_request_replay_and_reuse(self):
        batch = self.store.captured("metadata", tags=(TagEdit(self.root.id, "user:note", "x"),))
        first = self.store.commit(batch)
        replay = self.store.commit(batch)
        self.assertTrue(replay.replay)
        self.assertEqual(first.commit_id, replay.commit_id)
        with self.assertRaises(StorageError):
            self.store.commit(replace(batch, tags=(TagEdit(self.root.id, "user:note", "y"),)))

    def test_stale_epoch_and_old_metadata_capture_cannot_overwrite(self):
        batch = self.store.captured("metadata", tags=(TagEdit(self.root.id, "user:note", "old"),))
        self.store.commit(self.store.captured("metadata", tags=(TagEdit(self.root.id, "user:note", "new"),)))
        with self.assertRaises(StorageError):
            self.store.commit(batch)
        self.reopen()
        with self.assertRaises(StorageError):
            self.store.commit(batch)
        self.assertEqual(self.store.tags(self.root.id), {"user:note": "new"})

    def test_faults_at_every_staging_boundary_are_atomic(self):
        for phase in ("begin", "originals", "blocks", "nodes", "tags", "before_commit"):
            before = self.store.status()
            state = project_block(self.store.project, 9)
            node = Node(new_identity(), self.root.id, state.sha, 2, 1, 1, encode_json({"kind": "test"}))
            batch = self.store.captured("automatic", head=node.id, nodes=(node,), blocks=(state,),
                                        tags=(TagEdit(self.root.id, "user:fault", "never"),))
            def fail(actual, _):
                if actual == phase:
                    raise RuntimeError("controlled failure")
            with self.subTest(phase=phase), self.assertRaises(RuntimeError):
                self.store.commit(batch, hook=fail)
            self.assertEqual(before, self.store.status())
            self.assertEqual(self.store.tags(self.root.id), {})
            self.assertFalse(self.store._db.in_transaction)
            self.store.verify()

    def test_lost_receipt_after_real_commit_is_resolved_by_id(self):
        batch = self.store.captured("metadata", tags=(TagEdit(self.root.id, "user:receipt", "kept"),))
        def fail(phase, _):
            if phase == "after_commit":
                raise RuntimeError("receipt lost")
        with self.assertRaises(RuntimeError):
            self.store.commit(batch, hook=fail)
        self.assertEqual(self.store.tags(self.root.id), {"user:receipt": "kept"})
        self.assertTrue(self.store.commit(batch).replay)

    def test_unknown_required_capability_is_read_only_and_bytes_unchanged(self):
        self.store._db.execute("INSERT INTO features VALUES('future.codec','2',1)")
        self.store.close()
        before = hashlib.sha256(self.path.read_bytes()).digest()
        self.store = Store.open(self.path)
        self.assertFalse(self.store.writable)
        with self.assertRaises(StorageError):
            self.store.commit(self.store.captured("metadata"))
        self.store.close()
        self.assertEqual(hashlib.sha256(self.path.read_bytes()).digest(), before)
        self.store = Store.open(self.path)

    def test_optional_opaque_extension_survives_edits(self):
        raw = b"\xff\x00future unknown extension\r\n"
        self.store.commit(self.store.captured("metadata", extensions=(("vendor.fixture.notes", raw),)))
        append(self.store, self.root, 2)
        self.reopen()
        self.assertEqual(self.store._db.execute("SELECT bytes FROM extensions").fetchone()[0], raw)

    def test_retired_identity_is_preserved_and_not_reallocated(self):
        value = new_identity()
        self.store.commit(self.store.captured("metadata", identities=(("layer", value, self.root.id),)))
        child, _ = append(self.store, self.root, 2)
        self.reopen()
        with self.assertRaises(StorageError):
            self.store.commit(self.store.captured("metadata", identities=(("layer", value, child.id),)))

    def test_valid_redo_map_and_invalid_direct_child(self):
        a, _ = append(self.store, self.root, 2)
        b, _ = append(self.store, a, 3)
        good = Block.make("navigation", {"redo": [[self.root.id.hex(), a.id.hex()]]})
        self.store.commit(self.store.captured("manual", head=self.root.id, navigation=good.sha, blocks=(good,)))
        bad = Block.make("navigation", {"redo": [[self.root.id.hex(), b.id.hex()]]})
        with self.assertRaises(StorageError):
            self.store.commit(self.store.captured("automatic", head=self.root.id, navigation=bad.sha, blocks=(bad,)))
        self.reopen()
        self.assertEqual(self.store.open_position()[1], good)

    def test_creation_refuses_existing_file(self):
        before = self.path.read_bytes()
        with self.assertRaises(FileExistsError):
            Store.create(self.path, new_identity(), new_identity(), new_identity())
        self.assertEqual(before, self.path.read_bytes())

    def test_remap_preserves_history_and_permanent_references_but_rejects_old_short_context(self):
        child, _ = append(self.store, self.root, 2)
        records = {node.id: node.record() for node in (self.root, child)}
        old_epoch = self.store.status()["address_epoch"]
        request = self.store.captured("metadata", new_address_epoch=new_identity(), remap=(
            AddressBinding(self.root.id, 2, 9, 0), AddressBinding(child.id, 1, 9, 1)))
        self.store.commit(request)
        self.reopen()
        for key, raw in records.items():
            self.assertEqual(self.store.node(key).record(), raw)
        context = dict(epoch=self.store.epoch, project=self.store.project, history=self.store.history)
        with self.assertRaisesRegex(StorageError, "mapping"):
            self.store.resolve("n1", address_epoch=old_epoch, **context)
        self.assertEqual(self.store.resolve("n1", address_epoch=self.store.status()["address_epoch"], **context), child.id)
        self.assertEqual(self.store.resolve(node_key(self.root.id), **context), self.root.id)
        self.assertEqual(self.store.status()["nodes"], 2)

    def test_incomplete_or_colliding_remap_is_atomic(self):
        child, _ = append(self.store, self.root, 2)
        old = self.store.status()
        for bindings in ((AddressBinding(self.root.id, 10, 1, 0),),
                         (AddressBinding(self.root.id, 10, 1, 0), AddressBinding(child.id, 10, 1, 1))):
            with self.assertRaises((StorageError, sqlite3.IntegrityError)):
                self.store.commit(self.store.captured("metadata", new_address_epoch=new_identity(), remap=bindings))
            self.assertEqual(self.store.status(), old)
            self.assertEqual(self.store.node(self.root.id).alias, 1)
            self.assertFalse(self.store._db.in_transaction)

    def test_request_replay_does_not_ignore_changed_display_binding(self):
        child, _ = append(self.store, self.root, 2)
        batch = self.store.captured("automatic", nodes=(child,), tags=(TagEdit(child.id, "user:x", "a"),))
        self.store.commit(batch)
        with self.assertRaisesRegex(StorageError, "reused"):
            self.store.commit(replace(batch, nodes=(replace(child, alias=88),)))

    def test_schema_projection_retains_origin_and_node_checksum_across_branches(self):
        media = write_media(self.directory)
        a, _ = append(self.store, self.root, 2, blobs=media)
        b, _ = append(self.store, self.root, 3, step=9, branch=2, ordinal=0)
        before = {n.id: self.store.node(n.id).record() for n in (self.root, a, b)}
        source = self.store.block(a.state)
        target = fixture_projection(source)
        with self.assertRaisesRegex(StorageError, "missing"):
            self.store.state_for(a.id, target_schema=2)
        self.store.commit(self.store.captured("metadata", blocks=(target,), views=(StateView(source.sha, target.sha),)))
        self.reopen()
        self.assertEqual(self.store.state_for(a.id).raw, source.raw)
        self.assertEqual(self.store.state_for(a.id, target_schema=2).raw, target.raw)
        for key, raw in before.items():
            self.assertEqual(self.store.node(key).record(), raw)
        for blob in media:
            self.assertEqual(b"".join(self.store.blob_parts(blob.sha)), blob.path.read_bytes())
        # Editing after reading schema 2 creates a new node, preserving its old parent.
        next_state = Block.make("fixture.project", {"project_id": self.store.project.hex(), "edit_index": 22}, schema=2)
        next_node = Node(new_identity(), a.id, next_state.sha, 4, 1, 2, b'{"kind":"edit-v2"}')
        self.store.commit(self.store.captured("automatic", head=next_node.id, nodes=(next_node,), blocks=(next_state,)))
        self.assertEqual(self.store.node(a.id).record(), before[a.id])
        self.store.verify(reference=True)

    def test_invalid_or_interrupted_projection_never_rebinds_node(self):
        source = self.store.block(self.root.state)
        target = fixture_projection(source)
        bad = Block.make("fixture.project", {"project_id": self.store.project.hex(), "edit_index": 999}, schema=2)
        before = self.store.status()
        with self.assertRaisesRegex(StorageError, "semantics"):
            self.store.commit(self.store.captured("metadata", blocks=(bad,), views=(StateView(source.sha, bad.sha),)))
        def cancel(phase, _):
            if phase == "blocks":
                raise InterruptedError("controlled conversion cancellation")
        with self.assertRaises(InterruptedError):
            self.store.commit(self.store.captured("metadata", blocks=(target,), views=(StateView(source.sha, target.sha),)), hook=cancel)
        self.assertEqual(self.store.status(), before)
        self.assertEqual(self.store.node(self.root.id).state, source.sha)
        self.assertEqual(self.store._db.execute("SELECT count(*) FROM state_views").fetchone()[0], 0)

    def test_unknown_projection_fields_are_preserved(self):
        source = Block.make("fixture.project", {"project_id": self.store.project.hex(), "step": 1,
                                              "vendor": {"unknown": [1, "text"]}})
        target = fixture_projection(source)
        self.assertEqual(target.envelope()["value"]["vendor"], source.envelope()["value"]["vendor"])

    def test_equivalent_reserialization_does_not_replace_an_existing_schema_view(self):
        source = self.store.block(self.root.state)
        target = fixture_projection(source)
        self.store.commit(self.store.captured("metadata", blocks=(target,), views=(StateView(source.sha, target.sha),)))
        alternate = Block(b" \n" + target.raw)
        self.assertEqual(target.envelope(), alternate.envelope())
        self.assertNotEqual(target.sha, alternate.sha)
        before = self.store.status()
        with self.assertRaisesRegex(StorageError, "immutable state view"):
            self.store.commit(self.store.captured("metadata", blocks=(alternate,), views=(StateView(source.sha, alternate.sha),)))
        self.assertEqual(self.store.status(), before)
        self.assertEqual(self.store.state_for(self.root.id, target_schema=2).raw, target.raw)

    def test_valid_projection_does_not_hide_a_corrupted_original_state(self):
        source = self.store.block(self.root.state)
        target = fixture_projection(source)
        self.store.commit(self.store.captured("metadata", blocks=(target,), views=(StateView(source.sha, target.sha),)))
        path = self.directory / "damaged-origin.oskproj"
        shutil.copyfile(self.path, path)
        from p3_storage.schema import script
        trigger = next(s for s in script().splitlines() if s.startswith("CREATE TRIGGER guard_blocks_update "))
        with closing(sqlite3.connect(str(path))) as db:
            db.execute("DROP TRIGGER guard_blocks_update")
            damaged = source.raw.replace(b'"step":0', b'"step":9')
            self.assertNotEqual(damaged, source.raw)
            db.execute("UPDATE blocks SET bytes=? WHERE sha=?", (damaged, source.sha))
            db.execute(trigger)
            db.commit()
        before = path.read_bytes()
        for reference in (False, True):
            with self.assertRaisesRegex(StorageError, "block hash"):
                Store.open(path, reference=reference)
        self.assertEqual(path.read_bytes(), before)

    def test_cancel_during_original_scan_releases_reader_and_keeps_all_bytes(self):
        media = write_media(self.directory, large_mib=2)
        append(self.store, self.root, 2, blobs=media)
        before = self.path.read_bytes()
        def checkpoint(phase, units):
            if phase == "originals" and units == 1:
                raise InterruptedError("cancel after resource chunks")
        with self.assertRaises(InterruptedError):
            Store.open(self.path, writable=False, checkpoint=checkpoint)
        self.assertEqual(self.path.read_bytes(), before)
        with closing(sqlite3.connect(str(self.path), isolation_level=None, timeout=0)) as db:
            db.execute("BEGIN EXCLUSIVE")
            db.execute("COMMIT")
        self.store.verify()

    def test_full_verification_is_one_read_snapshot_and_cancellation_releases_it(self):
        other = sqlite3.connect(str(self.path), isolation_level=None, timeout=0)
        stages = []
        def checkpoint(phase, _):
            if phase == "blocks" and not stages:
                self.assertTrue(self.store._db.in_transaction)
                stages.append(phase)
                other.execute("BEGIN IMMEDIATE")
                other.execute("UPDATE control SET metadata_bytes=metadata_bytes WHERE singleton=1")
                with self.assertRaisesRegex(sqlite3.OperationalError, "locked"):
                    other.execute("COMMIT")
                other.execute("ROLLBACK")
                raise InterruptedError("controlled load cancellation")
        try:
            with self.assertRaises(InterruptedError):
                self.store.verify(checkpoint=checkpoint)
            self.assertFalse(self.store._db.in_transaction)
            other.execute("BEGIN IMMEDIATE")
            other.execute("COMMIT")
            self.store.verify()
        finally:
            other.close()

    def test_cancel_inside_sql_interrupts_and_never_publishes_an_open_store(self):
        last = self.root
        for alias in range(2, 25):
            last, _ = append(self.store, last, alias)
        state = {"armed": False, "calls": 0}
        def cancelled():
            if state["armed"]:
                state["calls"] += 1
                return True
            return False
        def checkpoint(phase, _):
            if phase == "sqlite_integrity":
                state["armed"] = True
        before = self.path.read_bytes()
        with self.assertRaisesRegex(sqlite3.OperationalError, "interrupted"):
            Store.open(self.path, cancel=cancelled, checkpoint=checkpoint)
        self.assertGreater(state["calls"], 0)
        self.assertEqual(self.path.read_bytes(), before)
        self.reopen()
        self.assertEqual(self.store.status()["nodes"], 24)

    def test_reference_and_streaming_verify_the_same_resources(self):
        last = self.root
        for alias in range(2, 40):
            last, _ = append(self.store, last, alias, step=alias)
        self.assertEqual(self.store.verify(reference=True, trace_sql=True), self.store.verify(trace_sql=True))
        self.assertLess(self.store.verification_report["sql_statements"], 100)

    def test_derived_index_and_binding_corruption_are_not_trusted(self):
        for mutation in ("UPDATE blocks SET project_id='wrong'", "UPDATE addresses SET alias=x'0000000000000033'"):
            path = self.directory / ("bad-" + str(len(mutation)) + ".oskproj")
            shutil.copyfile(self.path, path)
            with closing(sqlite3.connect(str(path))) as db:
                if "blocks" in mutation:
                    from p3_storage.schema import script
                    db.execute("DROP TRIGGER guard_blocks_update")
                    db.execute(mutation)
                    db.execute(next(s for s in script().splitlines() if s.startswith("CREATE TRIGGER guard_blocks_update ")))
                else:
                    db.execute(mutation)
                db.commit()
            for reference in (False, True):
                with self.assertRaisesRegex(StorageError, "index|binding"):
                    Store.open(path, reference=reference)

    def test_sqlite_full_rolls_back_and_next_valid_commit_succeeds(self):
        before = self.store.status()
        media = write_media(self.directory, large_mib=2)
        pages = self.store._db.execute("PRAGMA page_count").fetchone()[0]
        self.store._db.execute("PRAGMA max_page_count=%d" % (pages + 2))
        with self.assertRaisesRegex(sqlite3.OperationalError, "full"):
            append(self.store, self.root, 2, blobs=media)
        self.assertFalse(self.store._db.in_transaction)
        self.assertEqual(before, self.store.status())
        self.assertEqual(self.store._db.execute("SELECT count(*) FROM blobs").fetchone()[0], 0)
        self.store._db.execute("PRAGMA max_page_count=262144")
        append(self.store, self.root, 2)
        self.reopen()
        self.assertEqual(self.store.status()["nodes"], 2)

    def test_reader_lock_refuses_commit_without_partial_tags_or_leaked_lock(self):
        other = sqlite3.connect(str(self.path), isolation_level=None, timeout=0)
        try:
            other.execute("BEGIN")
            other.execute("SELECT * FROM control").fetchone()
            before = self.store.status()
            batch = self.store.captured("metadata", tags=(TagEdit(self.root.id, "user:locked", "new"),))
            with self.assertRaisesRegex(sqlite3.OperationalError, "locked"):
                self.store.commit(batch)
            self.assertFalse(self.store._db.in_transaction)
            self.assertEqual(before, self.store.status())
            self.assertEqual(self.store.tags(self.root.id), {})
            other.execute("ROLLBACK")
            self.assertTrue(self.store.commit(batch).changed)
        finally:
            other.close()

    def test_unknown_container_version_refused_without_touching_bytes(self):
        self.store.close()
        with closing(sqlite3.connect(str(self.path))) as db:
            db.execute("PRAGMA user_version=3")
        before = self.path.read_bytes()
        with self.assertRaisesRegex(StorageError, "unknown container"):
            Store.open(self.path, recover=True)
        self.assertEqual(before, self.path.read_bytes())
        # Restore only this deliberately malformed self-authored test fixture.
        with closing(sqlite3.connect(str(self.path))) as db:
            db.execute("PRAGMA user_version=2")
        self.store = Store.open(self.path)

    def test_old_prototype_version_is_not_silently_reinterpreted_or_overwritten(self):
        self.store.close()
        with closing(sqlite3.connect(str(self.path))) as db:
            db.execute("PRAGMA user_version=1")
        before = self.path.read_bytes()
        with self.assertRaisesRegex(StorageError, "unknown container"):
            Store.open(self.path, recover=True)
        self.assertEqual(before, self.path.read_bytes())
        with closing(sqlite3.connect(str(self.path))) as db:
            db.execute("PRAGMA user_version=2")
        self.store = Store.open(self.path)

    def test_history_schema_selects_reader_instead_of_guessing_from_application_version(self):
        from p3_storage.schema import script
        trigger = next(s for s in script().splitlines() if s.startswith("CREATE TRIGGER guard_header_update "))
        self.store.close()
        def set_version(version):
            with closing(sqlite3.connect(str(self.path))) as db:
                db.execute("DROP TRIGGER guard_header_update")
                db.execute("UPDATE header SET history_schema=?", (version,))
                db.execute(trigger)
                db.commit()
        set_version(3)
        before = self.path.read_bytes()
        with self.assertRaisesRegex(StorageError, "unsupported format/profile"):
            Store.open(self.path)
        self.assertEqual(before, self.path.read_bytes())
        set_version(2)
        self.store = Store.open(self.path)

    def test_connection_configuration_failure_closes_the_native_connection(self):
        from p3_storage.store import _connection
        db = sqlite3.connect(str(self.path), isolation_level=None)
        class FailingConnection:
            def execute(self, _):
                raise sqlite3.OperationalError("controlled pragma failure")
            def close(self):
                db.close()
        with patch("p3_storage.store.sqlite3.connect", return_value=FailingConnection()):
            with self.assertRaisesRegex(sqlite3.OperationalError, "controlled"):
                _connection(self.path, False)
        with self.assertRaises(sqlite3.ProgrammingError):
            db.execute("SELECT 1")

    def test_schema_changes_are_not_silently_migrated(self):
        self.store._db.execute("DROP TRIGGER guard_nodes_update")
        self.store.close()
        before = self.path.read_bytes()
        with self.assertRaisesRegex(StorageError, "schema"):
            Store.open(self.path)
        self.assertEqual(before, self.path.read_bytes())

    def test_corrupt_original_is_reported_and_not_repaired_by_dropping_it(self):
        media = write_media(self.directory)
        append(self.store, self.root, 2, blobs=media)
        self.store._db.execute("DROP TRIGGER guard_blob_chunks_update")
        sha = media[-1].sha
        self.store._db.execute("UPDATE blob_chunks SET bytes=? WHERE sha=?", (b"corrupted", sha))
        from p3_storage.schema import script
        trigger = next(s for s in script().splitlines() if s.startswith("CREATE TRIGGER guard_blob_chunks_update "))
        self.store._db.execute(trigger)
        self.store.close()
        before = self.path.read_bytes()
        with self.assertRaisesRegex(StorageError, "original"):
            Store.open(self.path)
        self.assertEqual(before, self.path.read_bytes())

    def test_changed_prepared_input_cancels_atomic_batch(self):
        media = write_media(self.directory)
        media[0].path.write_bytes(b"changed after capture")
        before = self.store.status()
        with self.assertRaisesRegex(ContractError, "changed"):
            append(self.store, self.root, 2, blobs=media)
        self.assertEqual(before, self.store.status())
        self.assertFalse(self.store._db.in_transaction)
        self.assertEqual(self.store._db.execute("SELECT count(*) FROM blobs").fetchone()[0], 0)

    def test_cancel_after_actual_chunk_insert_releases_partial_original(self):
        media = write_media(self.directory, large_mib=2)
        original = BlobInput.parts
        before = self.store.status()
        def cancelled(source):
            for part in original(source):
                yield part
                if source.byte_len > 65536:
                    raise InterruptedError("controlled stream cancellation")
        with patch.object(BlobInput, "parts", cancelled):
            with self.assertRaises(InterruptedError):
                append(self.store, self.root, 2, blobs=media)
        self.assertEqual(before, self.store.status())
        self.assertEqual(self.store._db.execute("SELECT count(*) FROM blob_chunks").fetchone()[0], 0)
        self.assertFalse(self.store._db.in_transaction)
        append(self.store, self.root, 2, blobs=media)
        self.store.verify()

    def test_read_only_and_tag_budget_refuse_without_node_changes(self):
        with Store.open(self.path, writable=False) as other:
            with self.assertRaisesRegex(StorageError, "read-only"):
                other.commit(other.captured("metadata"))
        self.store.limits = replace(self.store.limits, metadata_bytes=self.store.status()["metadata_bytes"] + 1)
        with self.assertRaisesRegex(StorageError, "budget"):
            self.store.commit(self.store.captured("metadata", tags=(TagEdit(self.root.id, "user:large", "x" * 100),)))
        self.assertEqual(self.store.tags(self.root.id), {})
        self.assertEqual(self.store.node(self.root.id), self.root)

    def test_metadata_cannot_move_checkpoints(self):
        with self.assertRaisesRegex(ContractError, "checkpoint"):
            self.store.commit(self.store.captured("metadata", head=self.root.id))

    def test_invalid_parent_alias_and_project_state_are_atomic(self):
        state = project_block(self.store.project, 8)
        malformed = Block.make("fixture.project", [])
        cases = (
            (Node(new_identity(), new_identity(), state.sha, 2, 1, 1, b'{}'), state),
            (Node(new_identity(), self.root.id, state.sha, 1, 1, 1, b'{}'), state),
            (Node(new_identity(), self.root.id, malformed.sha, 2, 1, 1, b'{}'), malformed))
        for node, block in cases:
            before = self.store.status()
            with self.assertRaises((ContractError, sqlite3.IntegrityError)):
                self.store.commit(self.store.captured("automatic", head=node.id, nodes=(node,), blocks=(block,)))
            self.assertEqual(before, self.store.status())
            self.assertFalse(self.store._db.in_transaction)


if __name__ == "__main__":
    unittest.main()
