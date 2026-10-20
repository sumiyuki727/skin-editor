"""Prepare self-authored first stable comparisons; never install or open a client."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import zipfile

from p0_evidence import make_fixture, png, inspect_client
from check_p0 import ROOT


def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def prepare(destination, client=None):
    destination = destination.resolve(); allowed = (ROOT / 'artifacts/p0').resolve()
    if destination.parent != allowed or destination.exists(): raise ValueError('Use a new direct artifacts/p0 child')
    # Staging and final paths both remain in the project; client is read-only.
    with tempfile.TemporaryDirectory(prefix='.stable-batch-', dir=allowed) as temporary:
        staging = Path(temporary); make_fixture(staging / 'base')
        packages = staging / 'packages'; packages.mkdir(); skins = staging / 'skins'; skins.mkdir()
        names = [('baseline', 'P0 First Batch Baseline'), ('transparent', 'P0 First Batch Transparent'), ('missing', 'P0 First Batch Missing')]
        for key, title in names:
            skin = skins / key; shutil.copytree(staging / 'base/skin', skin)
            (skin / 'skin.ini').write_text('[General]\nName: ' + title + '\nAuthor: skin-editor independent P0\nVersion: 2.0\n', encoding='utf-8')
            # Asymmetric corner marks and a centre cross are test variables.
            def circle(x, y):
                if x < 9 and y < 9: return (255, 80, 40, 255)
                if x >= 55 and y >= 55: return (30, 210, 250, 255)
                if x in (31, 32) or y in (31, 32): return (255, 255, 255, 255)
                return (70, 170, 230, 210) if (x - 31.5) ** 2 + (y - 31.5) ** 2 < 27 ** 2 else (0, 0, 0, 0)
            (skin / 'hitcircle.png').write_bytes(png(64, 64, circle))
            if key == 'transparent': (skin / 'hitcircleoverlay.png').write_bytes(png(64, 64, lambda x, y: (0, 0, 0, 0)))
            if key == 'missing': (skin / 'hitcircleoverlay.png').unlink()
            with zipfile.ZipFile(packages / (title + '.osk'), 'x', zipfile.ZIP_DEFLATED) as archive:
                for path in sorted(skin.iterdir()): archive.write(path, path.name)
        # One identical map for all skin variants: eight separated circles.
        beatmap = staging / 'beatmap'; shutil.copytree(staging / 'base/beatmap', beatmap)
        old = beatmap / 'P0 Circles.osu'; content = old.read_text(encoding='utf-8')
        content = content[:content.index('[HitObjects]')] + '[HitObjects]\n' + '\n'.join(
            f'{x},{y},{2000 + n * 3000},1,0,0:0:0:0:' for n, (x, y) in enumerate([(128,128),(384,128),(128,256),(384,256)] * 2)) + '\n'
        content = content.replace('P0 Circle Fixture', 'P0 First Batch').replace('Version:Circles', 'Version:Static Circles').replace('ApproachRate:5', 'ApproachRate:0')
        old.unlink(); (beatmap / 'P0 First Batch.osu').write_text(content, encoding='utf-8')
        # The map lasts 24 seconds; replace only our generated silent audio.
        import wave
        with wave.open(str(beatmap / 'silence.wav'), 'wb') as f:
            f.setparams((1, 2, 22050, 22050 * 26, 'NONE', 'not compressed')); f.writeframes(b'\0\0' * 22050 * 26)
        with zipfile.ZipFile(packages / 'P0 First Batch.osz', 'x', zipfile.ZIP_DEFLATED) as archive:
            for path in sorted(beatmap.iterdir()): archive.write(path, path.name)
        patch = staging / 'refresh-B'; patch.mkdir()
        def changed_ring(x, y):
            r = (x - 31.5) ** 2 + (y - 31.5) ** 2
            return (120, 255, 60, 255) if 25 ** 2 < r < 29 ** 2 else (0, 0, 0, 0)
        for name in ('cursor.png', 'hitcircleoverlay.png'): (patch / name).write_bytes(png(64, 64, changed_ring))
        if client is not None:
            (staging / 'client-inspection.json').write_text(json.dumps(inspect_client(client), indent=2) + '\n', encoding='utf-8')
        instructions = '''# 首批 stable 对照操作卡

本包没有安装文件或更改游戏设置。只使用自制图像、数字、静音音频和谱面。

先执行 C00，再做后面的素材案例。每次 F12 后记录对应文件的完整路径；
只提供本轮明确拍摄的图片，不扫描旧截图。不要使用 Shift+F12（会上传）。
F12 的格式／保存选项如有自定义，请同时记录。不要用图片登记代替案例判定。

## C00：确认客户端身份

1. 手动运行 D:\\osu!\\osu!.exe，确认是 stable，避免文件关联误开 lazer。
2. 在主菜单打开设置，找到更新通道／版本信息；将完整发布编号抄到 observation-template.json。
   若这处未显示完整版本，先记录可见文字，不从 exe 文件版本或历史日志推断。
3. 保持版本／通道信息可见，按 F12。把这张新截图的完整路径发给 Codex。
   默认本地位置是 D:\\osu!\\Screenshots；无需提供该目录中的其他图片。

## C01：静态标记基线

1. 将 packages/P0 First Batch Baseline.osk 和 packages/P0 First Batch.osz
   分别拖到刚确认的 stable 窗口；不要双击依赖未知的文件关联。
2. 记录当前皮肤、窗口／全屏、分辨率、Windows 显示缩放以及背景暗化设置。
   在游戏设置中选择 P0 First Batch Baseline，关闭 Mods 并记录原状态。
3. 搜索 P0 First Batch，选择 Static Circles，开始游玩。
   第一个圆圈在约两秒处；圆圈出现后、尚未消失时按 F12，不需要击打它。
   后续每三秒还有圆圈，若错过可以重试。记录这是第几个圆圈及是否开启游戏内暂停。
4. 提供一张清晰截图和以上显示设置。这只准备几何／标记观察，不预先规定显示尺度。

## C03：透明文件与缺失文件

1. 分别拖入另外两个 osk，选择 P0 First Batch Transparent 和 P0 First Batch Missing。
2. 保持 C01 的同一显示设置和谱面；每种皮肤拍一张对应圆圈截图，记录对象序号。
3. 不要将“看起来一样”直接视为相同回退规则；保留三个输入 manifest 与截图对应关系。

## C08：菜单中的刷新，再验证游玩／普通暂停

1. 重新选择 Baseline，回到主菜单，按 F12 留下圆环光标 A 的画面。
2. 在游戏设置中打开当前皮肤文件夹，确认它对应本包 Baseline（原文件可与 skins/baseline 比对）。
   仅将 refresh-B/cursor.png 和 hitcircleoverlay.png 复制到这个本包皮肤目录；
   不向自己的其他皮肤复制。需要恢复时复制 skins/baseline 中同名的两个原图。
3. 在主菜单按 Ctrl+Alt+Shift+S，待画面稳定后按 F12，记录光标是否变成绿色。
4. 游戏／普通暂停下的刷新单独执行并记录“何时发生变化”；官方快捷键说明提示
   游玩／结算中的重载可能延后至返回选歌，本实验不预先保证暂停时即时刷新。
5. 完成后恢复本包 A 的两个文件并刷新，重新选择原皮肤与原设置。
   本轮不要求删除测试内容；如需清理，只处理本包明确命名的皮肤和谱面。

请先提供 C00 和 C01。后续案例不依赖立刻完成，不影响压缩音频原型继续验证。
以上均为待执行步骤，compatibility_cases_executed 仍为 0。
'''
        (staging / 'STEPS.md').write_text(instructions, encoding='utf-8')
        (staging / 'observation-template.json').write_text(json.dumps({'schema_version': 1, 'status': 'not_executed', 'case': None,
            'client_release': None, 'display': {'resolution': None, 'window_mode': None, 'windows_scale': None},
            'skin': None, 'object_index': None, 'steps': [], 'observation': None, 'explicit_f12_paths': [], 'compatibility_passed': None}, indent=2) + '\n', encoding='utf-8')
        files = {p.relative_to(staging).as_posix(): digest(p) for p in sorted(staging.rglob('*')) if p.is_file()}
        manifest = {'schema_version': 1, 'kind': 'self_authored_stable_first_batch', 'files': files, 'client_installation_modified': False,
            'compatibility_cases_executed': 0, 'cases_prepared': ['C00', 'C01', 'C03', 'C08'], 'source_snapshot_used': False,
            'source_references': ['https://osu.ppy.sh/wiki/en/Client/Keyboard_shortcuts', 'https://osu.ppy.sh/wiki/en/Client/File_formats/osz_(file_format)',
                                  'https://osu.ppy.sh/wiki/en/Skinning/skin.ini', 'https://osu.ppy.sh/wiki/en/Skinning/osu%21']}
        (staging / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8'); staging.rename(destination)
    return manifest


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--output', type=Path, required=True); p.add_argument('--client-dir', type=Path)
    args = p.parse_args(); manifest = prepare(args.output, args.client_dir)
    print('Prepared: ' + str(args.output.resolve())); print('Compatibility executed: ' + str(manifest['compatibility_cases_executed']))
