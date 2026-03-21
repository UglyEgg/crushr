#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_TAG="crushr-musl-build:local"
OUT_DIR="${ROOT_DIR}/dist/musl-release"
TARGET="x86_64-unknown-linux-musl"

VERSION="$(tr -d '\r\n' < "${ROOT_DIR}/VERSION")"
GIT_COMMIT="$(git -C "${ROOT_DIR}" rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
BUILD_TIMESTAMP="${CRUSHR_BUILD_TIMESTAMP:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}"
RUSTC_VERSION="${CRUSHR_RUSTC_VERSION:-$(rustc --version 2>/dev/null || echo unknown)}"

mkdir -p "${OUT_DIR}"

podman build -f "${ROOT_DIR}/Containerfile.musl" -t "${IMAGE_TAG}" "${ROOT_DIR}"

podman run --rm \
  -v "${ROOT_DIR}:/work:Z" \
  -w /work \
  -e CRUSHR_VERSION="${VERSION}" \
  -e CRUSHR_GIT_COMMIT="${GIT_COMMIT}" \
  -e CRUSHR_BUILD_TIMESTAMP="${BUILD_TIMESTAMP}" \
  -e CRUSHR_TARGET_TRIPLE="${TARGET}" \
  -e CRUSHR_RUSTC_VERSION="${RUSTC_VERSION}" \
  "${IMAGE_TAG}" \
  bash -lc 'cargo build --release --target x86_64-unknown-linux-musl -p crushr'

cp -f "${ROOT_DIR}/target/${TARGET}/release/crushr" "${OUT_DIR}/crushr"

(
  cd "${OUT_DIR}"
  sha256sum crushr > SHA256SUMS
  if command -v b3sum >/dev/null 2>&1; then
    b3sum crushr > B3SUMS
  fi
)

file "${OUT_DIR}/crushr"
"${OUT_DIR}/crushr" about

echo "musl release artifact: ${OUT_DIR}/crushr"
