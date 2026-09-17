# -*- mode: python ; coding: utf-8 -*-


a = Analysis(
    ['src/git_manager/__main__.py'],
    pathex=['src'],
    binaries=[],
    datas=[('src/git_manager', 'git_manager')],
    hiddenimports=['git_manager', 'git_manager.desktop', 'git_manager.web', 'git_manager.cli', 'git_manager.core', 'git_manager.models', 'git_manager.utils'],
    hookspath=['scripts/hooks'],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name='git-manager.exe',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=['src/git_manager/desktop/resources/icons/app.ico'],
)
