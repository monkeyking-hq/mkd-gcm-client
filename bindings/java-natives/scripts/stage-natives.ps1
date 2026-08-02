# Copyright (c) 2026 MonkeyKing.dev
# SPDX-License-Identifier: MIT
#
# Stage shared libraries for mkd-gcm-natives packaging.
# Default targets right now: windows-x86_64 + linux-x86_64.
#
# Usage (from repo root or any cwd):
#   pwsh bindings/java-natives/scripts/stage-natives.ps1
#   pwsh .../stage-natives.ps1 -HostOnly
#   pwsh .../stage-natives.ps1 -SkipCargo   # only copy already-built artifacts
#
# Linux build uses WSL with a separate CARGO_TARGET_DIR so it does not clobber
# the Windows target/release tree.

[CmdletBinding()]
param(
    [switch]$HostOnly,
    [switch]$SkipCargo,
    [string[]]$Platforms = @('windows-x86_64', 'linux-x86_64')
)

$ErrorActionPreference = 'Stop'

# Maven passes NATIVE_HOST_ONLY / NATIVE_SKIP_CARGO as env vars ("true"/"false").
if (-not $HostOnly -and $env:NATIVE_HOST_ONLY -eq 'true') { $HostOnly = $true }
if (-not $SkipCargo -and $env:NATIVE_SKIP_CARGO -eq 'true') { $SkipCargo = $true }

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$NativesDir = (Resolve-Path (Join-Path $ScriptDir '..')).Path
$RepoRoot = (Resolve-Path (Join-Path $NativesDir '..\..')).Path
$StageRoot = Join-Path $NativesDir 'target\native-staging'

function Write-Info([string]$msg) { Write-Host "[stage-natives] $msg" }

function Ensure-Dir([string]$path) {
    if (-not (Test-Path $path)) {
        New-Item -ItemType Directory -Path $path | Out-Null
    }
}

function Stage-File([string]$platform, [string]$source, [string]$destName) {
    if (-not (Test-Path $source)) {
        throw "Missing native library for ${platform}: $source"
    }
    $destDir = Join-Path $StageRoot $platform
    Ensure-Dir $destDir
    $dest = Join-Path $destDir $destName
    Copy-Item -Force $source $dest
    $len = (Get-Item $dest).Length
    Write-Info "staged $platform/$destName ($len bytes)"
}

Write-Info "repo=$RepoRoot"
Write-Info "stage=$StageRoot"
if (Test-Path $StageRoot) {
    Remove-Item -Recurse -Force $StageRoot
}
Ensure-Dir $StageRoot

Push-Location $RepoRoot
try {
    $wantWindows = $Platforms -contains 'windows-x86_64'
    $wantLinux = $Platforms -contains 'linux-x86_64'

    if ($HostOnly) {
        $os = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
            [System.Runtime.InteropServices.OSPlatform]::Windows)
        if ($os) {
            $wantWindows = $true
            $wantLinux = $false
        } else {
            $wantWindows = $false
            $wantLinux = $true
        }
        Write-Info "HostOnly: windows=$wantWindows linux=$wantLinux"
    }

    # --- Windows x86_64 ---
    if ($wantWindows) {
        if (-not $SkipCargo) {
            Write-Info "cargo build -p mkd-gcm-ffi --release (host / Windows)"
            & cargo build -p mkd-gcm-ffi --release
            if ($LASTEXITCODE -ne 0) { throw "cargo build (Windows) failed: $LASTEXITCODE" }
        }
        $winLib = Join-Path $RepoRoot 'target/release/mkd_gcm_ffi.dll'
        Stage-File 'windows-x86_64' $winLib 'mkd_gcm_ffi.dll'
    }

    # --- Linux x86_64 (via WSL; separate target dir) ---
    if ($wantLinux) {
        $linuxTargetDir = Join-Path $RepoRoot 'target/linux-x86_64'
        $linuxLib = Join-Path $linuxTargetDir 'release/libmkd_gcm_ffi.so'

        if (-not $SkipCargo) {
            $wsl = Get-Command wsl -ErrorAction SilentlyContinue
            if (-not $wsl) {
                throw @"
Linux x86_64 packaging requires WSL (or pre-stage the .so).

Options:
  1) Install WSL + Rust in the distro, then re-run this script
  2) On a Linux machine:
       CARGO_TARGET_DIR=target/linux-x86_64 cargo build -p mkd-gcm-ffi --release
     then copy target/linux-x86_64/release/libmkd_gcm_ffi.so into
       bindings/java-natives/target/native-staging/linux-x86_64/
     and run with -SkipCargo
  3) Host-only packaging (Windows DLL only):
       -Dnative.hostOnly=true
"@
            }

            # Convert Windows path to WSL path (quote so drive letters survive)
            $wslRoot = (& wsl -e wslpath -a "$RepoRoot" 2>&1 | Out-String).Trim()
            if (-not $wslRoot -or $wslRoot -notmatch '^/') {
                throw "wslpath failed for '$RepoRoot' (got: '$wslRoot')"
            }
            Write-Info "WSL root=$wslRoot"

            Write-Info "WSL cargo build (CARGO_TARGET_DIR=target/linux-x86_64)"
            # Single-quoted bash -c body; expand only WSL paths we control
            $bashCmd = "set -euo pipefail; cd `"$wslRoot`"; export CARGO_TARGET_DIR=`"$wslRoot/target/linux-x86_64`"; command -v cargo >/dev/null; cargo build -p mkd-gcm-ffi --release"
            & wsl -e bash -lc $bashCmd
            if ($LASTEXITCODE -ne 0) { throw "WSL cargo build (Linux) failed: $LASTEXITCODE" }
        }

        Stage-File 'linux-x86_64' $linuxLib 'libmkd_gcm_ffi.so'
    }
}
finally {
    Pop-Location
}

# Manifest for Maven / humans
$manifest = @()
Get-ChildItem $StageRoot -Directory | ForEach-Object {
    $files = (Get-ChildItem $_.FullName -File | ForEach-Object { $_.Name }) -join ', '
    $manifest += "$($_.Name): $files"
}
$manifestPath = Join-Path $StageRoot 'STAGED.txt'
$manifest | Set-Content -Path $manifestPath -Encoding utf8
Write-Info "done. platforms staged:"
$manifest | ForEach-Object { Write-Info "  $_" }
