<#
.SYNOPSIS
    Build the Chronicler backend PyInstaller binary and place it in the Tauri
    externalBin directory with the correct Rust target-triple suffix.

.DESCRIPTION
    Run from the repo root on Windows:
        powershell -ExecutionPolicy Bypass -File scripts/build-backend.ps1

    Requires:
        - Python 3.11+ with PyInstaller and backend requirements installed
        - Rust toolchain (rustc) for target-triple detection
#>

$ErrorActionPreference = "Stop"

# Resolve paths relative to this script's location
$repoRoot   = Resolve-Path (Join-Path $PSScriptRoot "..")
$backendDir = Join-Path $repoRoot "backend"
$binDir     = Join-Path $repoRoot "frontend\src-tauri\binaries"

# Detect the Rust target triple (e.g. x86_64-pc-windows-msvc)
$targetTriple = (rustc -vV | Select-String "host:").ToString().Split(":")[1].Trim()
$outputName   = "chronicler-backend-$targetTriple.exe"

Write-Host "Target triple : $targetTriple"
Write-Host "Output binary : $outputName"

# Ensure the binaries directory exists
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

# Build with PyInstaller
Push-Location $backendDir
try {
    python -m PyInstaller chronicler-backend.spec --clean --noconfirm
} finally {
    Pop-Location
}

# Copy with the target-triple suffix that Tauri expects
$src = Join-Path $backendDir "dist\chronicler-backend.exe"
$dst = Join-Path $binDir $outputName

if (-not (Test-Path $src)) {
    Write-Error "PyInstaller output not found: $src"
    exit 1
}

Copy-Item $src $dst -Force
Write-Host "Placed at: $dst"
