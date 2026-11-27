#!/usr/bin/env bash
# Builds the primary contract to Wasm with the Soroban toolchain and copies the
# artifact into artifacts/. soroban-sdk 30 requires the build to be driven by
# `stellar contract build` (it sets STELLAR_CLI_VERSION and the target flags).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PACKAGE="${PACKAGE:-primary}"
WASM_TARGET="${WASM_TARGET:-wasm32v1-none}"
OUT_DIR="${OUT_DIR:-artifacts}"

if ! command -v stellar >/dev/null 2>&1; then
  echo "error: stellar CLI not found on PATH (required by soroban-sdk 30 build)" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

stellar contract build \
  --package "$PACKAGE" \
  --out-dir "$OUT_DIR" \
  --optimize=false \
  --locked

WASM="${OUT_DIR}/${PACKAGE}.wasm"
SIZE="$(wc -c < "$WASM" | tr -d '[:space:]')"
echo "built ${WASM} (${SIZE} bytes, target ${WASM_TARGET})"
