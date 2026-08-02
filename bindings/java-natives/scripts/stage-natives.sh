#!/usr/bin/env bash
# Copyright (c) 2026 MonkeyKing.dev
# SPDX-License-Identifier: MIT
#
# Stage shared libraries for mkd-gcm-natives packaging (Linux/macOS host).
# Default: windows-x86_64 (if prebuilt) + linux-x86_64 (cargo host).
#
# Usage:
#   ./bindings/java-natives/scripts/stage-natives.sh
#   HOST_ONLY=1 ./bindings/java-natives/scripts/stage-natives.sh
#   SKIP_CARGO=1 ./bindings/java-natives/scripts/stage-natives.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NATIVES_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$NATIVES_DIR/../.." && pwd)"
STAGE_ROOT="$NATIVES_DIR/target/native-staging"

HOST_ONLY="${HOST_ONLY:-0}"
SKIP_CARGO="${SKIP_CARGO:-0}"

info() { echo "[stage-natives] $*"; }

stage_file() {
  local platform="$1" source="$2" dest_name="$3"
  if [[ ! -f "$source" ]]; then
    echo "Missing native library for ${platform}: $source" >&2
    exit 1
  fi
  local dest_dir="$STAGE_ROOT/$platform"
  mkdir -p "$dest_dir"
  cp -f "$source" "$dest_dir/$dest_name"
  info "staged $platform/$dest_name ($(wc -c <"$dest_dir/$dest_name") bytes)"
}

info "repo=$REPO_ROOT"
info "stage=$STAGE_ROOT"
rm -rf "$STAGE_ROOT"
mkdir -p "$STAGE_ROOT"

cd "$REPO_ROOT"

want_windows=1
want_linux=1
if [[ "$HOST_ONLY" == "1" ]]; then
  case "$(uname -s)" in
    Linux*) want_windows=0; want_linux=1 ;;
    Darwin*) want_windows=0; want_linux=0; info "HostOnly on macOS: set platforms explicitly"; exit 1 ;;
    MINGW*|MSYS*|CYGWIN*) want_windows=1; want_linux=0 ;;
    *) want_windows=0; want_linux=1 ;;
  esac
  info "HostOnly: windows=$want_windows linux=$want_linux"
fi

if [[ "$want_linux" == "1" ]]; then
  linux_target_dir="$REPO_ROOT/target/linux-x86_64"
  linux_lib="$linux_target_dir/release/libmkd_gcm_ffi.so"
  if [[ "$SKIP_CARGO" != "1" ]]; then
    info "cargo build -p mkd-gcm-ffi --release (CARGO_TARGET_DIR=target/linux-x86_64)"
    export CARGO_TARGET_DIR="$linux_target_dir"
    cargo build -p mkd-gcm-ffi --release
  fi
  stage_file linux-x86_64 "$linux_lib" libmkd_gcm_ffi.so
fi

if [[ "$want_windows" == "1" ]]; then
  win_lib="$REPO_ROOT/target/release/mkd_gcm_ffi.dll"
  if [[ ! -f "$win_lib" ]]; then
    echo "Windows DLL not found at $win_lib — build on Windows or copy into staging." >&2
    exit 1
  fi
  stage_file windows-x86_64 "$win_lib" mkd_gcm_ffi.dll
fi

{
  for d in "$STAGE_ROOT"/*/; do
    [[ -d "$d" ]] || continue
    base="$(basename "$d")"
    files="$(ls -1 "$d" | tr '\n' ' ')"
    echo "$base: $files"
  done
} | tee "$STAGE_ROOT/STAGED.txt"

info "done."
