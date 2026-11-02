"""Run bounded P3-0 tests/experiments; no product integration or native devices."""

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import queue
import shutil
import sqlite3
import subprocess
import sys
import threading
import time
import unittest

from p3_storage.fixtures import make_store, project_block, write_media
from p3_storage.identity import new_identity
from p3_storage.metrics import difference, snapshot
from p3_storage.store import Store
from p3_storage.values import Block, Node, TagEdit, encode_json

ROOT = Path(__file__).resolve().parents[1]


def write_json(path, value):
    with Path(path).open("w", encoding="utf8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def checked_output(path, *, create=False):
    path = Path(path).resolve()
    allowed = (ROOT / "artifacts/p3-0").resolve()
    if not path.is_relative_to(allowed) or path == allowed:
        raise ValueError("experiment output must be a named subdirectory of artifacts/p3-0")
    if create:
        path.mkdir(parents=True, exist_ok=False)
    return path


def percentiles(values):
    ordered = sorted(values)
    return {"count": len(values), "median_ms": ordered[len(ordered) // 2],
            "p95_ms": ordered[max(0, math.ceil(.95 * len(ordered)) - 1)],
            "max_ms": ordered[-1]}


def benchmark(directory):
    directory = checked_output(directory)
    media = write_media(directory, large_mib=128)
    before = snapshot()
    store, root, initial_receipt = make_store(directory / "scale.oskproj", media)
    initial = difference(before, snapshot())
    common = root.state
    last, alias = root, 1
    phases = {}
    for shape in ("deep", "wide"):
        before = snapshot()
        receipts = []
        for offset in range(0, 10000, 250):
            blocks, nodes = [], []
            for index in range(offset, min(offset + 250, 10000)):
                alias += 1
                state = project_block(store.project, alias, blocks=(common,))
                node = Node(new_identity(), last.id if shape == "deep" else root.id,
                            state.sha, alias, 1 if shape == "deep" else index + 2,
                            index + 1 if shape == "deep" else 0,
                            encode_json({"kind": "fixture-edit", "shape": shape, "step": index}))
                blocks.append(state)
                nodes.append(node)
                if shape == "deep":
                    last = node
            receipts.append(store.commit(store.captured("automatic", head=nodes[-1].id,
                                                        nodes=tuple(nodes), blocks=tuple(blocks))))
        phases[shape] = difference(before, snapshot())
        phases[shape]["batch_latency"] = percentiles([r.timings["total_ms"] for r in receipts])
    original_state = root.record()
    def old_nodes_digest():
        hasher = hashlib.sha256()
        for value, checksum in store._db.execute("SELECT n.id,n.checksum FROM nodes n JOIN addresses a ON a.node_id=n.id WHERE a.alias<=? ORDER BY a.alias",
                                                ((20001).to_bytes(8, "big"),)):
            hasher.update(value + checksum)
        return hasher.hexdigest()
    originals = old_nodes_digest()
    before = snapshot()
    times = []
    for index in range(300):
        alias += 1
        state = project_block(store.project, alias, blocks=(common,))
        node = Node(new_identity(), last.id, state.sha, alias, 1, last.ordinal + 1,
                    encode_json({"kind": "fixture-edit", "shape": "single", "step": index}))
        receipt = store.commit(store.captured("automatic", head=node.id, nodes=(node,), blocks=(state,)))
        times.append(receipt.timings["total_ms"])
        last = node
    phases["content_300"] = difference(before, snapshot())
    phases["content_300"]["latency"] = percentiles(times)
    before = snapshot()
    times = []
    for index in range(300):
        receipt = store.commit(store.captured("metadata", tags=(TagEdit(root.id, "user:note", str(index)),)))
        times.append(receipt.timings["total_ms"])
    phases["tags_300"] = difference(before, snapshot())
    phases["tags_300"]["latency"] = percentiles(times)
    assert store.node(root.id).record() == original_state
    assert store.status()["nodes"] == 20301
    assert old_nodes_digest() == originals
    # Compare three equivalent tag commits with and without whole-file copy.
    baseline_path = directory / "rewrite-baseline.oskproj"
    shutil.copyfile(store.path, baseline_path)
    before = snapshot()
    for index in range(3):
        store.commit(store.captured("metadata", tags=(TagEdit(root.id, "user:comparison", str(index)),)))
    phases["incremental_3"] = difference(before, snapshot())
    before = snapshot()
    for index in range(3):
        temporary = directory / ("rewrite-" + str(index) + ".oskproj")
        shutil.copyfile(baseline_path, temporary)
        # Trusted, exact copy of the fully verified fixture; opening/guard checks
        # remain enabled, but avoid a second whole-media validation in comparison.
        with Store.open(temporary, verify=False) as other:
            other.commit(other.captured("metadata", tags=(TagEdit(root.id, "user:comparison", str(index)),)))
        os.replace(temporary, baseline_path)
    phases["rewrite_3"] = difference(before, snapshot())
    before = snapshot()
    store.verify()
    phases["verify_full"] = difference(before, snapshot())
    status = store.status()
    project, history, lineage = store.project, store.history, store.lineage
    main_path = store.path
    store.close()
    closed = snapshot()
    cycles = []
    for _ in range(10):
        before = snapshot()
        with Store.open(main_path) as opened:
            assert (opened.project, opened.history, opened.lineage) == (project, history, lineage)
            assert opened.status()["nodes"] == 20301
            assert opened.node(root.id).record() == original_state
        cycles.append(difference(before, snapshot()))
    assert not Path(str(main_path) + "-journal").exists()
    result = {"runtime": {"python": platform.python_version(), "sqlite": sqlite3.sqlite_version,
                           "platform": platform.platform(), "process_clock": str(time.get_clock_info("process_time"))},
              "fixture": {"raw_bytes": sum(b.byte_len for b in media), "nodes": status["nodes"],
                          "depth": 10300, "wide_children": 10001, "tag_edits": 303,
                          "single_content_edits": 300, "old_nodes_unchanged": 20001,
                          "old_node_records_sha256": originals,
                          "file_bytes": main_path.stat().st_size, "blocks": status["blocks"],
                          "metadata_payload_bytes": status["metadata_bytes"]},
              "initial": initial, "initial_commit_ms": initial_receipt.timings["total_ms"],
              "phases": phases, "closed": closed, "open_close_cycles": cycles,
              "limits": {"device_gpu_audio_opened": False, "is_product_cpu_acceptance": False,
                         "fixture_state_is_full_domain_codec": False,
                         "io_includes_journal_and_process_overhead": True,
                         "power_loss_tested": False}}
    write_json(directory / "result.json", result)
    print(json.dumps({"completed": True, "file_bytes": main_path.stat().st_size,
                      "tag_p95_ms": phases["tags_300"]["latency"]["p95_ms"]}), flush=True)


def crash_child(path, media_directory, phase):
    path = checked_output(path)
    with Store.open(path) as store:
        media = write_media(Path(media_directory), large_mib=16)
        state = project_block(store.project, 2, blocks=(store.node(store.root).state,),
                              blobs=[b.sha for b in media])
        node = Node(new_identity(), store.root, state.sha, 2, 1, 1, encode_json({"kind": "crash-fixture"}))
        batch = store.captured("exit", head=node.id, nodes=(node,), blocks=(state,), blobs=media,
                               tags=(TagEdit(store.root, "user:crash", "new"),))
        def gate(actual, _):
            if actual == phase:
                print(json.dumps({"phase": phase, "journal_exists": Path(str(path) + "-journal").exists(),
                                  "main_bytes": path.stat().st_size}), flush=True)
                sys.stdin.readline()  # Parent kills this process at a known boundary, never sleep guessing.
                raise RuntimeError("parent did not terminate crash child")
        store.commit(batch, hook=gate)


def crash_matrix(directory):
    directory = checked_output(directory, create=True)
    baseline = directory / "baseline.oskproj"
    store, root, _ = make_store(baseline)
    original = store.node(root.id).record()
    store.close()
    results = []
    for phase in ("begin", "originals", "blocks", "nodes", "tags", "before_commit", "after_commit"):
        case = directory / phase
        case.mkdir()
        path = case / "crash.oskproj"
        shutil.copyfile(baseline, path)
        process = subprocess.Popen([sys.executable, "-B", str(Path(__file__).resolve()),
                                    "--crash-child", str(path), "--media", str(case), "--phase", phase],
                                   cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True, encoding="utf8")
        # Deterministic gate with a bounded wait; no scheduling sleep guesses.
        messages = queue.Queue(maxsize=1)
        threading.Thread(target=lambda: messages.put(process.stdout.readline()), daemon=True).start()
        try:
            marker = messages.get(timeout=30)
        except queue.Empty:
            process.kill()
            process.communicate(timeout=15)
            raise RuntimeError("crash child did not reach its gate within 30s")
        if not marker:
            _, error = process.communicate(timeout=15)
            raise RuntimeError("crash child failed before gate: " + error)
        observed = json.loads(marker)
        process.kill()
        process.communicate(timeout=15)
        with Store.open(path, recover=True) as recovered:
            recovered.verify()
            assert recovered.node(root.id).record() == original
            expected = 2 if phase == "after_commit" else 1
            assert recovered.status()["nodes"] == expected
            assert recovered.tags(root.id) == ({"user:crash": "new"} if expected == 2 else {})
            assert recovered.open_position()[0].id == root.id
            result = {"phase": phase, "gate": observed, "recovered_nodes": expected,
                      "generation": recovered.status()["generation"],
                      "root_record_sha256": hashlib.sha256(original).hexdigest(),
                      "journal_after_recovery": Path(str(path) + "-journal").exists()}
            results.append(result)
        print(json.dumps({"crash_phase": phase, "passed": True}), flush=True)
    write_json(directory / "result.json", {"python": platform.python_version(), "sqlite": sqlite3.sqlite_version,
                                           "cases": results, "controlled_process_kill": True,
                                           "kill_inside_commit_syscall": False, "power_loss": False})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bench")
    parser.add_argument("--crashes")
    parser.add_argument("--crash-child")
    parser.add_argument("--media")
    parser.add_argument("--phase")
    args = parser.parse_args()
    if args.crash_child:
        crash_child(args.crash_child, args.media, args.phase)
    elif args.bench:
        directory = checked_output(args.bench, create=True)
        benchmark(directory)
    elif args.crashes:
        crash_matrix(args.crashes)
    else:
        suite = unittest.defaultTestLoader.discover(str(ROOT / "tools"), pattern="test_check_p3_storage.py")
        result = unittest.TextTestRunner(verbosity=2).run(suite)
        raise SystemExit(not result.wasSuccessful())


if __name__ == "__main__":
    main()
