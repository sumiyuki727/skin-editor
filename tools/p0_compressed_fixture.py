"""Self-authored WAV -> pinned external encoder/reference, never client/music files."""
import hashlib
import json
import math
from pathlib import Path
import subprocess
import struct
import wave

from check_p0 import ROOT


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1048576), b''): h.update(block)
    return h.hexdigest()


def encoder_path():
    paths = list((ROOT / '.p0-tools/encoder/imageio_ffmpeg/binaries').glob('ffmpeg*.exe'))
    if len(paths) != 1: raise RuntimeError('Provide/install the documented private imageio-ffmpeg 0.6.0 encoder')
    return paths[0]


def make_fixture(output, durations=(.37, 3, 180), encoder=None):
    if not durations or any(not math.isfinite(s) or not 0 < s <= 600 or round(s * 48000) != s * 48000 for s in durations):
        raise ValueError('Bounded whole sample durations required')
    output.mkdir(exist_ok=False); exe = encoder or encoder_path()
    version = subprocess.check_output([str(exe), '-version'], text=True).splitlines()
    second = bytearray()
    for n in range(48000):
        t = n / 48000; envelope = min(1, t / .02, (1 - t) / .02)
        left = .32 * math.sin(math.tau * 440 * t) + .08 * math.sin(math.tau * 997 * t)
        right = .25 * math.sin(math.tau * 660 * t) - .05 * math.sin(math.tau * 131 * t)
        second.extend(struct.pack('<hh', round(left * envelope * 32767), round(right * envelope * 32767)))
    items = []
    for seconds in durations:
        name = str(seconds).replace('.', '_'); frames = round(seconds * 48000)
        wav = output / ('source-' + name + '.wav')
        with wave.open(str(wav), 'wb') as f:
            f.setparams((2, 2, 48000, frames, 'NONE', 'not compressed'))
            remaining = frames
            while remaining:
                n = min(remaining, 48000); f.writeframesraw(second[:n * 4]); remaining -= n
        for ext, options in [('mp3', ['-c:a', 'libmp3lame', '-b:a', '192k', '-write_xing', '1', '-id3v2_version', '0', '-write_id3v1', '0']),
                             ('ogg', ['-c:a', 'libvorbis', '-q:a', '4'])]:
            compressed = output / ('audio-' + name + '.' + ext); reference = output / (compressed.name + '.reference.f32')
            command = [str(exe), '-nostdin', '-hide_banner', '-loglevel', 'error', '-n', '-i', str(wav), '-map_metadata', '-1', '-threads', '1', *options, str(compressed)]
            subprocess.run(command, check=True)
            with reference.open('xb') as f:
                subprocess.run([str(exe), '-nostdin', '-hide_banner', '-loglevel', 'error', '-xerror', '-i', str(compressed),
                                '-map', '0:a:0', '-vn', '-threads', '1', '-c:a', 'pcm_f32le', '-f', 'f32le', '-'], stdout=f, check=True)
            items.append({'codec': ext, 'seconds': seconds, 'input_frames': frames, 'source': wav.name,
                          'encoded': compressed.name, 'reference': reference.name, 'reference_frames': reference.stat().st_size // 8,
                          'encode_arguments': command[1:]})
    files = {p.name: {'bytes': p.stat().st_size, 'sha256': digest(p)} for p in sorted(output.iterdir()) if p.is_file()}
    manifest = {'schema_version': 1, 'generator': 'self_authored_stereo_sines_compressed_v1', 'rate': 48000, 'channels': 2,
                'encoder': {'package': 'imageio-ffmpeg==0.6.0', 'version': version[0], 'sha256': digest(exe),
                            'reference_implementation': 'FFmpeg native floating point MP3/Vorbis decoder',
                            'runtime_product_dependency': False}, 'items': items, 'files': files}
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    return manifest


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument('output', type=Path)
    args = parser.parse_args(); target = args.output.resolve()
    if (ROOT / 'artifacts/p0').resolve() not in target.parents: raise RuntimeError('Use a new fixture directory inside artifacts/p0')
    result = make_fixture(target)
    for item in result['items']: print(item['encoded'], 'source/reference frames', item['input_frames'], item['reference_frames'])
