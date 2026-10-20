"""Opt-in Windows Vulkan allocation diagnostic, separate from product and performance runs.

No window, draw, image decode or injection. Uses Vulkan 1.0 public structures.
The parent samples only its own child while the child waits at API checkpoints.
"""
import argparse
import ctypes as c
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import time
import uuid

from check_p0 import ROOT
from p0_modules import module_snapshot
from p0_process_metrics import MemoryRegion, WindowsProcessMetrics

U, Q, P = c.c_uint32, c.c_uint64, c.c_void_p
MIB = 1024 * 1024


class AppInfo(c.Structure):
    _fields_ = [("sType", U), ("pNext", P), ("name", c.c_char_p), ("version", U),
                ("engine", c.c_char_p), ("engine_version", U), ("api_version", U)]


class InstanceInfo(c.Structure):
    _fields_ = [("sType", U), ("pNext", P), ("flags", U), ("app", c.POINTER(AppInfo)),
                ("layer_count", U), ("layers", P), ("extension_count", U), ("extensions", P)]


class QueueInfo(c.Structure):
    _fields_ = [("sType", U), ("pNext", P), ("flags", U), ("family", U),
                ("count", U), ("priorities", c.POINTER(c.c_float))]


class DeviceInfo(c.Structure):
    _fields_ = [("sType", U), ("pNext", P), ("flags", U), ("queue_count", U),
                ("queues", c.POINTER(QueueInfo)), ("layer_count", U), ("layers", P),
                ("extension_count", U), ("extensions", P), ("features", P)]


class MemoryType(c.Structure):
    _fields_ = [("flags", U), ("heap", U)]


class MemoryHeap(c.Structure):
    _fields_ = [("size", Q), ("flags", U)]


class MemoryProperties(c.Structure):
    _fields_ = [("type_count", U), ("types", MemoryType * 32), ("heap_count", U), ("heaps", MemoryHeap * 16)]


class Family(c.Structure):
    _fields_ = [("flags", U), ("count", U), ("timestamp_bits", U), ("granularity", U * 3)]


class AllocateInfo(c.Structure):
    _fields_ = [("sType", U), ("pNext", P), ("size", Q), ("type_index", U)]


class Extent(c.Structure):
    _fields_ = [("width", U), ("height", U), ("depth", U)]


class ImageInfo(c.Structure):
    _fields_ = [("sType", U), ("pNext", P), ("flags", U), ("type", U), ("format", U),
                ("extent", Extent), ("mips", U), ("layers", U), ("samples", U), ("tiling", U),
                ("usage", U), ("sharing", U), ("family_count", U), ("families", P), ("layout", U)]


class BufferInfo(c.Structure):
    _fields_ = [("sType", U), ("pNext", P), ("flags", U), ("size", Q), ("usage", U),
                ("sharing", U), ("family_count", U), ("families", P)]


class Requirements(c.Structure):
    _fields_ = [("size", Q), ("alignment", Q), ("type_bits", U)]


def select_type(types, bits, required, forbidden=0):
    for index, flags in enumerate(types):
        if bits & (1 << index) and flags & required == required and not flags & forbidden:
            return index
    raise ValueError("Requested memory property combination is unavailable; do not substitute a different class")


def validate_abi():
    if os.name != "nt" or c.sizeof(P) != 8:
        raise RuntimeError("This diagnostic's ABI is validated only on 64-bit Windows")
    expected = {AppInfo: 48, InstanceInfo: 64, QueueInfo: 40, DeviceInfo: 72, MemoryProperties: 520,
                AllocateInfo: 32, ImageInfo: 88, BufferInfo: 56, Requirements: 24, Family: 24}
    if any(c.sizeof(kind) != size for kind, size in expected.items()):
        raise RuntimeError("Unexpected Vulkan C structure layout")


class Vulkan:
    def __init__(self):
        validate_abi()
        self.library = Path(os.environ["SystemRoot"]) / "System32/vulkan-1.dll"
        self.api = c.WinDLL(str(self.library))
        self.instance, self.device = P(), P()
        self.memories, self.images = [], []
        self.buffer, self.host_memory = Q(), Q()
        self.mapped = False
        self.functions = {}
        self.bind("vkCreateInstance", c.c_int32, [c.POINTER(InstanceInfo), P, c.POINTER(P)])
        self.bind("vkDestroyInstance", None, [P, P])
        self.bind("vkEnumeratePhysicalDevices", c.c_int32, [P, c.POINTER(U), c.POINTER(P)])
        self.bind("vkGetPhysicalDeviceProperties", None, [P, P])
        self.bind("vkGetPhysicalDeviceMemoryProperties", None, [P, c.POINTER(MemoryProperties)])
        self.bind("vkGetPhysicalDeviceQueueFamilyProperties", None, [P, c.POINTER(U), c.POINTER(Family)])
        self.bind("vkCreateDevice", c.c_int32, [P, c.POINTER(DeviceInfo), P, c.POINTER(P)])
        self.bind("vkDestroyDevice", None, [P, P])
        self.bind("vkDeviceWaitIdle", c.c_int32, [P])
        self.bind("vkAllocateMemory", c.c_int32, [P, c.POINTER(AllocateInfo), P, c.POINTER(Q)])
        self.bind("vkFreeMemory", None, [P, Q, P])
        self.bind("vkCreateImage", c.c_int32, [P, c.POINTER(ImageInfo), P, c.POINTER(Q)])
        self.bind("vkDestroyImage", None, [P, Q, P])
        self.bind("vkGetImageMemoryRequirements", None, [P, Q, c.POINTER(Requirements)])
        self.bind("vkBindImageMemory", c.c_int32, [P, Q, Q, Q])
        self.bind("vkCreateBuffer", c.c_int32, [P, c.POINTER(BufferInfo), P, c.POINTER(Q)])
        self.bind("vkDestroyBuffer", None, [P, Q, P])
        self.bind("vkGetBufferMemoryRequirements", None, [P, Q, c.POINTER(Requirements)])
        self.bind("vkBindBufferMemory", c.c_int32, [P, Q, Q, Q])
        self.bind("vkMapMemory", c.c_int32, [P, Q, Q, Q, U, c.POINTER(P)])
        self.bind("vkUnmapMemory", None, [P, Q])

    def bind(self, name, restype, arguments):
        function = getattr(self.api, name)
        function.restype, function.argtypes = restype, arguments
        self.functions[name] = function

    def call(self, name, *args):
        result = self.functions[name](*args)
        if result is not None and result != 0:
            raise RuntimeError(name + " failed with VkResult " + str(result))

    def create(self, checkpoint):
        app = AppInfo(0, None, b"P0 allocation diagnostic", 0, b"none", 0, 1 << 22)
        info = InstanceInfo(1, None, 0, c.pointer(app), 0, None, 0, None)
        self.call("vkCreateInstance", c.byref(info), None, c.byref(self.instance))
        checkpoint("instance_created")
        count = U()
        self.call("vkEnumeratePhysicalDevices", self.instance, c.byref(count), None)
        if not 0 < count.value <= 32:
            raise RuntimeError("Unexpected physical device count")
        devices = (P * count.value)()
        self.call("vkEnumeratePhysicalDevices", self.instance, c.byref(count), devices)
        candidates = []
        for device in devices:
            # VkPhysicalDeviceProperties is a fixed Vulkan 1.0 structure. Its
            # immutable prefix is decoded from oversized, 8-byte-aligned storage;
            # no driver-dependent limits tail is interpreted.
            storage = (Q * 512)()
            self.call("vkGetPhysicalDeviceProperties", device, c.byref(storage))
            api, driver, vendor, identifier, kind, name = struct.unpack_from("<5I256s", bytes(storage))
            candidates.append((kind != 2, vendor != 0x10DE, device,
                               dict(name=name.split(b"\0", 1)[0].decode("utf-8"), vendor=vendor,
                                    device_id=identifier, driver_version=driver, api_version=api, type=kind)))
        _, _, physical, self.properties = sorted(candidates, key=lambda item: item[:2])[0]
        self.memory = MemoryProperties()
        self.call("vkGetPhysicalDeviceMemoryProperties", physical, c.byref(self.memory))
        if not 0 < self.memory.type_count <= 32 or not 0 < self.memory.heap_count <= 16:
            raise RuntimeError("Invalid memory properties")
        self.types = [self.memory.types[i].flags for i in range(self.memory.type_count)]
        self.properties["memory_types"] = [dict(flags=self.memory.types[i].flags, heap=self.memory.types[i].heap)
                                           for i in range(self.memory.type_count)]
        self.properties["memory_heaps"] = [dict(size=self.memory.heaps[i].size, flags=self.memory.heaps[i].flags)
                                           for i in range(self.memory.heap_count)]
        count = U()
        self.call("vkGetPhysicalDeviceQueueFamilyProperties", physical, c.byref(count), None)
        if not 0 < count.value <= 128:
            raise RuntimeError("Unexpected queue family count")
        families = (Family * count.value)()
        self.call("vkGetPhysicalDeviceQueueFamilyProperties", physical, c.byref(count), families)
        index = next(i for i, item in enumerate(families) if item.flags & 1 and item.count > 0)
        priority = c.c_float(1)
        queue = QueueInfo(2, None, 0, index, 1, c.pointer(priority))
        info = DeviceInfo(3, None, 0, 1, c.pointer(queue), 0, None, 0, None, None)
        self.call("vkCreateDevice", physical, c.byref(info), None, c.byref(self.device))

    def allocate(self, size, index):
        info, memory = AllocateInfo(5, None, size, index), Q()
        self.call("vkAllocateMemory", self.device, c.byref(info), None, c.byref(memory))
        return memory

    def add_memory(self, images, count):
        while len(self.memories) < count:
            if images:
                # 2D, RGBA8_UNORM, optimal tiling, transfer dst + sampled.
                info = ImageInfo(14, None, 0, 1, 37, Extent(1024, 1024, 1), 1, 1, 1, 0, 0x6, 0, 0, None, 0)
                image = Q()
                self.call("vkCreateImage", self.device, c.byref(info), None, c.byref(image))
                self.images.append(image)
                requirements = Requirements()
                self.call("vkGetImageMemoryRequirements", self.device, image, c.byref(requirements))
                index = select_type(self.types, requirements.type_bits, 1, 2)
                size = requirements.size
            else:
                index, size = select_type(self.types, (1 << len(self.types)) - 1, 1, 2), 16 * MIB
            memory = self.allocate(size, index)
            self.memories.append((memory, size, index))
            if images:
                self.call("vkBindImageMemory", self.device, image, memory, 0)
        self.call("vkDeviceWaitIdle", self.device)

    def release_memory(self):
        if self.device:
            self.call("vkDeviceWaitIdle", self.device)
        for image in self.images:
            self.call("vkDestroyImage", self.device, image, None)
        self.images.clear()
        for memory, _, _ in self.memories:
            self.call("vkFreeMemory", self.device, memory, None)
        self.memories.clear()

    def host_buffer(self):
        info = BufferInfo(12, None, 0, 8 * MIB, 1, 0, 0, None)
        self.call("vkCreateBuffer", self.device, c.byref(info), None, c.byref(self.buffer))
        requirements = Requirements()
        self.call("vkGetBufferMemoryRequirements", self.device, self.buffer, c.byref(requirements))
        index = select_type(self.types, requirements.type_bits, 2 | 4, 1)
        self.host_memory = self.allocate(requirements.size, index)
        self.call("vkBindBufferMemory", self.device, self.buffer, self.host_memory, 0)
        return dict(allocation_bytes=requirements.size, type_index=index, flags=self.types[index])

    def map_host(self):
        pointer = P()
        self.call("vkMapMemory", self.device, self.host_memory, 0, 8 * MIB, 0, c.byref(pointer))
        self.mapped = True
        c.memset(pointer, 0x7F, 8 * MIB)  # Only the buffer allocated by this process.
        return pointer.value

    def release_host(self):
        if self.mapped:
            self.call("vkUnmapMemory", self.device, self.host_memory)
            self.mapped = False
        if self.buffer:
            self.call("vkDestroyBuffer", self.device, self.buffer, None)
            self.buffer = Q()
        if self.host_memory:
            self.call("vkFreeMemory", self.device, self.host_memory, None)
            self.host_memory = Q()

    def close(self):
        self.release_host()
        self.release_memory()
        if self.device:
            self.call("vkDestroyDevice", self.device, None)
            self.device = P()
        if self.instance:
            self.call("vkDestroyInstance", self.instance, None)
            self.instance = P()


def child(kind, settle_seconds=0):
    sequence = 0
    def mark(stage, **payload):
        nonlocal sequence
        sequence += 1
        time.sleep(.1)
        print(json.dumps(dict(schema_version=1, sequence=sequence, stage=stage, payload=payload)), flush=True)
        if sys.stdin.readline().strip() != "continue":
            raise RuntimeError("Parent checkpoint acknowledgement missing")
    mark("python_baseline")
    if kind == "virtual-control":
        kernel = c.WinDLL("kernel32", use_last_error=True)
        kernel.VirtualAlloc.argtypes, kernel.VirtualAlloc.restype = [P, c.c_size_t, U, U], P
        kernel.VirtualFree.argtypes, kernel.VirtualFree.restype = [P, c.c_size_t, U], wintypes.BOOL
        address = kernel.VirtualAlloc(None, 64 * MIB, 0x3000, 4)
        if not address:
            raise c.WinError(c.get_last_error())
        try:
            mark("virtual_commit64", allocation_base=address, allocation_bytes=64 * MIB)
        finally:
            if not kernel.VirtualFree(address, 0, 0x8000):
                raise c.WinError(c.get_last_error())
        mark("virtual_release64")
        return
    vk = Vulkan()
    try:
        mark("loader_loaded", loader_sha256=hashlib.sha256(vk.library.read_bytes()).hexdigest())
        vk.create(mark)
        mark("device_created", **vk.properties)
        for capacity, count in ((80, 20 if kind == "images" else 5), (128, 32 if kind == "images" else 8)):
            vk.add_memory(kind == "images", count)
            mark("device_local_" + str(capacity), allocations=len(vk.memories),
                 allocation_bytes=sum(size for _, size, _ in vk.memories), type_indices=sorted({index for _, _, index in vk.memories}))
        vk.release_memory()
        mark("device_local_freed")
        if settle_seconds:
            time.sleep(settle_seconds)
            mark("device_local_freed_settled")
        host = vk.host_buffer()
        mark("host_buffer_allocated", **host)
        mark("host_buffer_mapped_touched", mapped_address=vk.map_host(), mapped_bytes=8 * MIB)
        vk.release_host()
        mark("host_buffer_freed")
        if settle_seconds:
            time.sleep(settle_seconds)
            mark("host_buffer_freed_settled")
    finally:
        vk.close()
    mark("device_instance_destroyed")
    if settle_seconds:
        time.sleep(settle_seconds)
        mark("device_instance_destroyed_settled")


def region_map(process, metrics):
    query = metrics.kernel.VirtualQueryEx
    query.argtypes, query.restype = [wintypes.HANDLE, P, c.POINTER(MemoryRegion), c.c_size_t], c.c_size_t
    address, rows, complete, terminal = 0, [], False, None
    for _ in range(200_000):
        region = MemoryRegion()
        count = query(wintypes.HANDLE(int(process._handle)), address, c.byref(region), c.sizeof(region))
        if not count:
            terminal = c.get_last_error()
            complete = terminal == 87
            break
        following = (region.BaseAddress or 0) + region.RegionSize
        if count != c.sizeof(region) or following <= address:
            raise RuntimeError("Non-advancing or incomplete VirtualQueryEx result")
        if region.State != 0x10000:  # No free address-space entries.
            rows.append(dict(base=region.BaseAddress, allocation_base=region.AllocationBase, size=region.RegionSize,
                             state=region.State, type=region.Type, protect=region.Protect, allocation_protect=region.AllocationProtect))
        address = following
    return dict(complete=complete, terminal_error=terminal, regions=rows)


def committed_totals(snapshot):
    """VirtualQueryEx page classes, never a partition of process PrivateUsage."""
    if not snapshot["complete"]:
        return None
    totals = dict(private=0, mapped=0, image=0, other=0)
    for row in snapshot["regions"]:
        if row["state"] == 0x1000:
            kind = {0x20000: "private", 0x40000: "mapped", 0x1000000: "image"}.get(row["type"], "other")
            totals[kind] += row["size"]
    return totals


def committed_overlap(snapshot, address, size):
    """Classify a known owned address range, including map splits; no page reads."""
    if not snapshot["complete"]:
        return None
    if address <= 0 or size <= 0:
        raise ValueError("Expected a nonempty owned address range")
    result = dict(private=0, mapped=0, image=0, other=0)
    for row in snapshot["regions"]:
        overlap = min(address + size, row["base"] + row["size"]) - max(address, row["base"])
        if row["state"] == 0x1000 and overlap > 0:
            kind = {0x20000: "private", 0x40000: "mapped", 0x1000000: "image"}.get(row["type"], "other")
            result[kind] += overlap
    return result


def checkpoint_summary(checkpoints):
    if not checkpoints or checkpoints[0]["event"]["stage"] != "python_baseline":
        raise ValueError("Missing baseline checkpoint")
    baseline = checkpoints[0]["after"]
    if baseline is None:
        raise ValueError("Missing baseline process metrics")
    result = []
    for item in checkpoints:
        if item["before"] is None or item["after"] is None:
            raise ValueError("Missing process metrics; do not substitute zero")
        event, sample = item["event"], item["after"]
        payload = event["payload"]
        row = dict(stage=event["stage"], private_bytes=sample["private_bytes"],
                   private_delta_baseline_bytes=sample["private_bytes"] - baseline["private_bytes"],
                   observer_private_change_bytes=sample["private_bytes"] - item["before"]["private_bytes"],
                   working_set_bytes=sample["working_set_bytes"], gpu_dedicated_bytes=sample["gpu_dedicated_bytes"],
                   gpu_shared_bytes=sample["gpu_shared_bytes"], committed_regions=committed_totals(item["regions"]),
                   graphic_modules=item["modules"]["graphic_modules"])
        if event["stage"] == "virtual_commit64":
            row["known_owned_range"] = committed_overlap(item["regions"], payload["allocation_base"], payload["allocation_bytes"])
        elif event["stage"] == "host_buffer_mapped_touched":
            row["known_owned_range"] = committed_overlap(item["regions"], payload["mapped_address"], payload["mapped_bytes"])
        result.append(row)
    return result


def run_case(kind, implicit_off, output, settle_seconds=0):
    environment = dict(os.environ)
    if implicit_off:
        environment["VK_LOADER_LAYERS_DISABLE"] = "~implicit~"
    output.mkdir()
    checkpoints, seen = [], 0
    with (output / "child.jsonl").open("w", encoding="utf-8") as journal, (output / "stderr.txt").open("w", encoding="utf-8") as errors:
        with subprocess.Popen([sys.executable, "-u", str(Path(__file__).resolve()), "--child", kind,
                               "--settle-seconds", str(settle_seconds)],
                              cwd=ROOT, env=environment, stdin=subprocess.PIPE, stdout=journal, stderr=errors, text=True) as process:
            metrics, start = WindowsProcessMetrics(process), time.monotonic()
            try:
                while process.poll() is None:
                    if time.monotonic() - start > 90:
                        raise RuntimeError("Own diagnostic child timed out")
                    lines = (output / "child.jsonl").read_text(encoding="utf-8").splitlines(keepends=True)
                    if len(lines) > seen and lines[seen].endswith("\n"):
                        event = json.loads(lines[seen])
                        if event["sequence"] != seen + 1:
                            raise RuntimeError("Checkpoint order changed")
                        seen += 1
                        before = metrics.sample()
                        regions = region_map(process, metrics)
                        modules = module_snapshot(process)
                        after = metrics.sample()
                        checkpoints.append(dict(event=event, before=before, after=after, regions=regions, modules=modules))
                        process.stdin.write("continue\n")
                        process.stdin.flush()
                    time.sleep(.01)
            finally:
                metrics.close()
                if process.poll() is None:
                    process.kill()
                    process.wait()
            if process.returncode:
                raise RuntimeError("Diagnostic child failed; inspect " + str(output / "stderr.txt"))
    result = dict(schema_version=1, status="passed", kind=kind, pid=process.pid,
                  settle_seconds=settle_seconds,
                  child_environment=dict(VK_LOADER_LAYERS_DISABLE=environment.get("VK_LOADER_LAYERS_DISABLE")),
                  source_sha256={str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                                 for path in (Path(__file__).resolve(), ROOT / "tools/p0_process_metrics.py", ROOT / "tools/p0_modules.py")},
                  checkpoints=checkpoints,
                  summary=checkpoint_summary(checkpoints),
                  note="Checkpoint allocation diagnosis, not application performance; no queue copy, window, draw or PNG. API boundary deltas include loader/layers/driver activity; not a pure driver partition.")
    (output / "allocation.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--child", choices=("memory", "images", "virtual-control"))
    parser.add_argument("--repetitions", type=int, default=1, choices=range(1, 6))
    parser.add_argument("--settle-seconds", type=int, default=0, choices=range(0, 11))
    args = parser.parse_args()
    validate_abi()
    if args.child:
        child(args.child, args.settle_seconds)
        return
    root = ROOT / "artifacts/p0" / ("vulkan-allocation-" + uuid.uuid4().hex[:8])
    root.mkdir()
    cases = []
    order = (("memory", False), ("memory", True), ("images", False), ("images", True))
    schedule = [(0, "virtual-control", False)]
    for repetition in range(1, args.repetitions + 1):
        trial_order = order if repetition % 2 else tuple(reversed(order))
        schedule.extend((repetition, kind, off) for kind, off in trial_order)
    initial_hash = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    for repetition, kind, off in schedule:
        output = root / (str(repetition) + "-" + kind + ("-implicit-off" if off else "-default"))
        result = run_case(kind, off, output, args.settle_seconds)
        if hashlib.sha256(Path(__file__).read_bytes()).hexdigest() != initial_hash:
            raise RuntimeError("Diagnostic script changed during execution")
        cases.append(dict(output=str(output.relative_to(ROOT)), repetition=repetition, kind=kind, implicit_off=off, status=result["status"]))
        (root / "cases.json").write_text(json.dumps(dict(schema_version=1, complete=len(cases) == len(schedule),
                                                        requested_repetitions=args.repetitions, cases=cases), indent=2), encoding="utf-8")
        print("Allocation evidence: " + str(output), flush=True)


if __name__ == "__main__":
    main()
