# -*- mode: python ; coding: utf-8 -*-


a = Analysis(
    ['src\\travdata\\cli\\cli.py'],
    pathex=[],
    binaries=[],
    datas=[
        ('.venv/Lib/site-packages/tabula/*.jar', 'tabula'),
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
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
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
coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='travdata',
)
