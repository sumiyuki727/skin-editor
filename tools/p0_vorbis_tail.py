"""Offline Vorbis oracle and page-layout control; no client files or audio device."""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import time
import uuid
import wave

from check_p0 import ROOT
from check_p0_compressed import compare_float_files, identity_paths, validate_pull
from p0_compressed_fixture import digest, encoder_path


def ogg_crc(data):
    value = 0
    for byte in data:
        value ^= byte << 24
        for _ in range(8):
            value = ((value << 1) ^ (0x04c11db7 if value & 0x80000000 else 0)) & 0xffffffff
    return value


def inspect_ogg(data):
    """Strict single-serial fixture framing, not a production Ogg reader."""
    if not 58 <= len(data) <= 1024 * 1024:
        raise ValueError('Ogg fixture size')
    pages = []; packets = []; partial = bytearray(); position = 0; serial = None
    while position < len(data):
        if len(pages) >= 4096 or position + 27 > len(data) or data[position:position + 5] != b'OggS\0':
            raise ValueError('Ogg page header')
        flags = data[position + 5]; count = data[position + 26]
        table_end = position + 27 + count
        if table_end > len(data) or flags & ~7:
            raise ValueError('Ogg lacing header')
        sizes = data[position + 27:table_end]; end = table_end + sum(sizes)
        if end > len(data):
            raise ValueError('Truncated Ogg payload')
        page = bytearray(data[position:end]); stored = struct.unpack_from('<I', page, 22)[0]; page[22:26] = b'\0' * 4
        if ogg_crc(page) != stored:
            raise ValueError('Ogg CRC')
        current = struct.unpack_from('<I', page, 14)[0]
        if serial is None: serial = current
        if (current != serial or struct.unpack_from('<I', page, 18)[0] != len(pages)
                or bool(flags & 1) != bool(partial) or bool(flags & 2) != (not pages)
                or bool(flags & 4) != (end == len(data))):
            raise ValueError('Not a complete single-serial fixture')
        cursor = table_end
        for size in sizes:
            partial.extend(data[cursor:cursor + size]); cursor += size
            if size < 255:
                packets.append(bytes(partial)); partial.clear()
        pages.append({'offset': position, 'flags': flags, 'granule': struct.unpack_from('<q', page, 6)[0]})
        position = end
    if partial or len(packets) < 4 or any(packets[n][:7] != bytes([kind]) + b'vorbis' for n, kind in enumerate((1, 3, 5))):
        raise ValueError('Incomplete Vorbis headers/packets')
    h = hashlib.sha256()
    for packet in packets[3:]:
        h.update(struct.pack('<I', len(packet))); h.update(packet)
    return {'pages': pages, 'audio_packets': len(packets) - 3, 'audio_packet_sha256': h.hexdigest(),
            'final_granule': pages[-1]['granule']}


class Info(ctypes.Structure):
    _fields_ = [('frames', ctypes.c_int64), ('rate', ctypes.c_int), ('channels', ctypes.c_int),
                ('format', ctypes.c_int), ('sections', ctypes.c_int), ('seekable', ctypes.c_int)]


def reference_library(path):
    if sys.platform != 'win32' or ctypes.sizeof(ctypes.c_void_p) != 8 or sys.byteorder != 'little':
        raise RuntimeError('Private oracle currently verified on 64-bit little-endian Windows only')
    lib = ctypes.CDLL(str(path))
    lib.sf_wchar_open.argtypes = [ctypes.c_wchar_p, ctypes.c_int, ctypes.POINTER(Info)]
    lib.sf_wchar_open.restype = ctypes.c_void_p
    lib.sf_readf_float.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_float), ctypes.c_int64]
    lib.sf_readf_float.restype = ctypes.c_int64
    lib.sf_close.argtypes = [ctypes.c_void_p]; lib.sf_close.restype = ctypes.c_int
    lib.sf_error.argtypes = [ctypes.c_void_p]; lib.sf_error.restype = ctypes.c_int
    lib.sf_version_string.restype = ctypes.c_char_p
    return lib


def decode_reference(lib, source, output):
    info = Info(); handle = lib.sf_wchar_open(str(source), 0x10, ctypes.byref(info))  # SFM_READ
    if not handle: raise RuntimeError('Independent reference open failed')
    try:
        if info.rate != 48000 or info.channels != 2 or not 0 < info.frames <= 144000:
            raise RuntimeError('Unexpected independent reference scope')
        frames = 0; buffer = (ctypes.c_float * 8192)()
        with output.open('xb') as stream:
            while True:
                count = lib.sf_readf_float(handle, buffer, 4096)
                if count < 0 or count > 4096: raise RuntimeError('Independent reference read failed')
                if not count: break
                frames += count
                if frames > info.frames: raise RuntimeError('Independent reference length overflow')
                stream.write(ctypes.string_at(buffer, count * 8))
        if lib.sf_error(handle) or frames != info.frames:
            raise RuntimeError('Independent reference length/error')
        return frames
    finally:
        lib.sf_close(handle)


def run_pull(source):
    out = ROOT / 'artifacts/p0' / ('vorbis-tail-pull-' + uuid.uuid4().hex[:8])
    exe = ROOT / 'experiments/p0-media/target/release/compressed-proof.exe'
    with out.with_suffix('.log').open('xb') as log:
        process = subprocess.Popen([str(exe), str(source), str(out)], stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 30
            while process.poll() is None and not (out / 'ready').exists():
                if time.monotonic() > deadline: raise RuntimeError('Pull handshake timeout')
                time.sleep(.005)
            if not (out / 'ready').exists(): raise RuntimeError('Pull binary exited before handshake')
            (out / 'go').write_text('1'); process.wait(timeout=30)
            if process.returncode: raise RuntimeError('Pull rejected: ' + str(out))
        finally:
            if process.poll() is None: process.kill(); process.wait(timeout=5)
    return out


def checked_paths(fixture, output):
    allowed = (ROOT / 'artifacts/p0').resolve()
    fixture = fixture.resolve(strict=True); output = output.resolve()
    if allowed not in fixture.parents or output.parent != allowed or output.exists():
        raise ValueError('Use an owned fixture and a new direct artifacts/p0 output directory')
    return fixture, output


def audit(fixture, output):
    fixture, output = checked_paths(fixture, output)
    manifest = json.loads((fixture / 'manifest.json').read_text())
    if manifest['generator'] != 'self_authored_stereo_sines_compressed_v1': raise ValueError('Wrong fixture')
    for name, row in manifest['files'].items():
        p = (fixture / name).resolve(strict=True)
        if fixture not in p.parents or digest(p) != row['sha256']: raise ValueError('Fixture path/hash')
    output.mkdir()
    frozen = {p.relative_to(ROOT).as_posix(): p.read_bytes() for p in identity_paths()}
    for name, data in frozen.items():
        p = output / 'source-snapshot' / name; p.parent.mkdir(parents=True, exist_ok=True); p.write_bytes(data)
    exe = ROOT / 'experiments/p0-media/target/release/compressed-proof.exe'; exe_hash = digest(exe)
    shutil.copyfile(exe, output / 'source-snapshot' / exe.name)
    dll = ROOT / '.p0-tools/audio-reference/_soundfile_data/libsndfile_x64.dll'; dll_hash = digest(dll)
    lib = reference_library(dll); ff = encoder_path(); rows = []
    for frames in (480, 960, 1023, 1024, 1025, 4096, 17760, 47999, 48000, 48001, 52800, 144000):
        wav = output / ('source-%d.wav' % frames)
        with wave.open(str(fixture / 'source-3.wav'), 'rb') as source, wave.open(str(wav), 'wb') as dest:
            dest.setparams((2, 2, 48000, frames, 'NONE', 'not compressed')); dest.writeframes(source.readframes(frames))
        pair = []
        for layout, options in [('default', []), ('paged', ['-page_duration', '20000'])]:
            name = '%d-%s' % (frames, layout); encoded = output / (name + '.ogg')
            command = [str(ff), '-nostdin', '-hide_banner', '-loglevel', 'error', '-n', '-i', str(wav),
                       '-map_metadata', '-1', '-threads', '1', '-c:a', 'libvorbis', '-q:a', '4', *options, str(encoded)]
            subprocess.run(command, check=True)
            framing = inspect_ogg(encoded.read_bytes())
            if framing['final_granule'] != frames: raise RuntimeError('Unexpected EOS granule')
            independent = output / (name + '.sndfile.f32'); decoded = decode_reference(lib, encoded, independent)
            ff_reference = output / (name + '.ffmpeg.f32')
            with ff_reference.open('xb') as stream:
                subprocess.run([str(ff), '-nostdin', '-hide_banner', '-loglevel', 'error', '-xerror', '-i', str(encoded),
                                '-c:a', 'pcm_f32le', '-f', 'f32le', '-'], stdout=stream, check=True)
            pull = run_pull(encoded); result = json.loads((pull / 'result.json').read_text())
            comparison = validate_pull(result, pull, independent)
            if decoded != frames or not comparison['frames_match']: raise RuntimeError('Independent length mismatch')
            ff_comparison = compare_float_files(pull / 'decoded.f32', ff_reference)
            if ff_comparison['samples_outside_tolerance']: raise RuntimeError('FFmpeg common-prefix mismatch')
            row = {'frames': frames, 'layout': layout, 'encoded': encoded.name, 'encoded_sha256': digest(encoded),
                   'framing': framing, 'pull': pull.relative_to(ROOT).as_posix(), 'result_sha256': digest(pull / 'result.json'),
                   'reference': independent.name, 'reference_sha256': digest(independent),
                   'independent_comparison': comparison, 'ffmpeg_comparison': ff_comparison, 'encode_command': command}
            pair.append(row); rows.append(row)
            print(name, 'independent matched;', 'FFmpeg frames', ff_comparison['reference_frames'], flush=True)
        if pair[0]['framing']['audio_packet_sha256'] != pair[1]['framing']['audio_packet_sha256']:
            raise RuntimeError('Page-layout control changed audio packets')
    if digest(exe) != exe_hash or digest(dll) != dll_hash or any(digest(ROOT / n) != hashlib.sha256(b).hexdigest() for n, b in frozen.items()):
        raise RuntimeError('Oracle/probe/source changed')
    result = {'schema_version': 1, 'status': 'passed_scoped_independent_reference_and_page_layout_control',
              'source_sha256': {n: hashlib.sha256(b).hexdigest() for n, b in frozen.items()},
              'executable_sha256': exe_hash, 'reference_library': {'package': 'soundfile==0.13.1 tool DLL only',
              'version': lib.sf_version_string().decode(), 'sha256': dll_hash, 'bytes': dll.stat().st_size},
              'encoder_sha256': digest(ff), 'fixture_manifest_sha256': digest(fixture / 'manifest.json'),
              'rows': rows, 'audio_device_opened': False, 'performance_measurement': False}
    (output / 'audit.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture', type=Path, required=True); parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args(); audit(args.fixture, args.output)
