# PyInstaller spec for the Chronicler backend sidecar.
# Run from the backend/ directory: pyinstaller chronicler-backend.spec
# Output: dist/chronicler-backend[.exe]
# Then rename with the Rust target triple (see scripts/build-backend.ps1).

from PyInstaller.utils.hooks import collect_all  # type: ignore[import]

block_cipher = None

# Collect all sub-packages for frameworks that use dynamic imports
_datas: list = []
_binaries: list = []
_hiddenimports: list = []

for _pkg in ("uvicorn", "fastapi", "starlette", "anyio", "aiosqlite"):
    d, b, h = collect_all(_pkg)
    _datas += d
    _binaries += b
    _hiddenimports += h

a = Analysis(  # type: ignore[name-defined]
    ["run.py"],
    pathex=["."],
    binaries=_binaries,
    datas=_datas,
    hiddenimports=_hiddenimports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)  # type: ignore[name-defined]

exe = EXE(  # type: ignore[name-defined]
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name="chronicler-backend",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
