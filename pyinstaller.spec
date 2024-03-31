# -*- mode: python ; coding: utf-8 -*-

import argparse

parser = argparse.ArgumentParser()
parser.add_argument("tabula_jar")
args = parser.parse_args()


cli_a = Analysis(
    ['src/travdata/cli/cli.py'],
    pathex=[],
    binaries=[],
    datas=[
        (args.tabula_jar, 'tabula'),
        ('config', './config'),
    ],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[
        "scripts/pyinstaller/hook.py",
    ],
    excludes=[],
    noarchive=False,
)
cli_pyz = PYZ(cli_a.pure)
cli_exe = EXE(
    cli_pyz,
    cli_a.scripts,
    [],
    exclude_binaries=True,
    name='travdata_cli',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)

gui_a = Analysis(
    ['src/travdata/gui/gui.py'],
    pathex=[],
    binaries=[],
    datas=[
        (args.tabula_jar, 'tabula'),
        ('config', './config'),
    ],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[
        "scripts/pyinstaller/hook.py",
    ],
    excludes=[],
    noarchive=False,
)
gui_pyz = PYZ(gui_a.pure)
gui_exe = EXE(
    gui_pyz,
    gui_a.scripts,
    [],
    exclude_binaries=True,
    name='travdata_gui',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
coll = COLLECT(
    cli_exe,
    cli_a.binaries,
    cli_a.datas,
    gui_exe,
    gui_a.binaries,
    gui_a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='travdata',
)
