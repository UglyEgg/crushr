#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

set -Eeuo pipefail

# scripts/build-musl-release.sh
#
# Build static musl binaries for crushr using Podman + Alpine.
#
# Supported profiles:
#   --profile dev
#   --profile release
#   --profile release-lto
#
# Output directories:
#   dev         -> dist/musl-dev
#   release     -> dist/musl
#   release-lto -> dist/musl-release-lto

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

IMAGE_TAG="crushr-musl-build:local"
CONTAINERFILE="Containerfile.musl"
CACHE_ROOT="$ROOT_DIR/.cache/cargo"
BUILD_PROFILE="release"
NETWORK_MODE="none"
TARGET_TRIPLE="x86_64-unknown-linux-musl"

BINS=()

usage() {
    sed -n '1,140p' "$0"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
    --bin)
        BINS+=("$2")
        shift 2
        ;;
    --image)
        IMAGE_TAG="$2"
        shift 2
        ;;
    --containerfile)
        CONTAINERFILE="$2"
        shift 2
        ;;
    --profile)
        BUILD_PROFILE="$2"
        shift 2
        ;;
    --network)
        NETWORK_MODE="$2"
        shift 2
        ;;
    --target)
        TARGET_TRIPLE="$2"
        shift 2
        ;;
    -h | --help)
        usage
        exit 0
        ;;
    *)
        echo "Unknown argument: $1" >&2
        exit 1
        ;;
    esac
done

case "$BUILD_PROFILE" in
dev | release | release-lto) ;;
*)
    echo "Unsupported profile: $BUILD_PROFILE" >&2
    echo "Supported profiles: dev, release, release-lto" >&2
    exit 1
    ;;
esac

if [[ "$BUILD_PROFILE" == "release" ]]; then
    DIST_DIR="$ROOT_DIR/dist/musl"
else
    DIST_DIR="$ROOT_DIR/dist/musl-$BUILD_PROFILE"
fi

if ! command -v podman >/dev/null 2>&1; then
    echo "podman is required but not found in PATH" >&2
    exit 1
fi

if [[ ! -f "$CONTAINERFILE" ]]; then
    echo "Containerfile not found: $CONTAINERFILE" >&2
    exit 1
fi

if [[ ! -f VERSION ]]; then
    echo "Missing root VERSION file" >&2
    exit 1
fi

VERSION="$(tr -d '\n' <VERSION)"
if [[ -z "$VERSION" ]]; then
    echo "VERSION file is empty" >&2
    exit 1
fi

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$ ]]; then
    echo "VERSION does not look like valid SemVer: $VERSION" >&2
    exit 1
fi

GIT_COMMIT="$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
BUILD_TIMESTAMP_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || echo unknown)"
RUSTC_VERSION_HOST="$(rustc --version 2>/dev/null || echo unknown)"

mkdir -p "$DIST_DIR"
mkdir -p "$CACHE_ROOT/registry"
mkdir -p "$CACHE_ROOT/git"
mkdir -p "$CACHE_ROOT/target"

if [[ ${#BINS[@]} -eq 0 ]]; then
    BINS=(
        crushr
        crushr-pack
        crushr-extract
        crushr-info
        crushr-salvage
        crushr-lab
    )
fi

case "$BUILD_PROFILE" in
dev)
    CONTAINER_LTO_VALUE="false"
    CONTAINER_CODEGEN_UNITS_VALUE="256"
    ;;
release)
    CONTAINER_LTO_VALUE="false"
    CONTAINER_CODEGEN_UNITS_VALUE="16"
    ;;
release-lto)
    CONTAINER_LTO_VALUE="thin"
    CONTAINER_CODEGEN_UNITS_VALUE="1"
    ;;
esac

CARGO_NET_OFFLINE_VALUE="false"
if [[ "$NETWORK_MODE" == "none" ]]; then
    CARGO_NET_OFFLINE_VALUE="true"
fi

echo "==> Build configuration"
echo "    image          $IMAGE_TAG"
echo "    containerfile  $CONTAINERFILE"
echo "    profile        $BUILD_PROFILE"
echo "    network        $NETWORK_MODE"
echo "    target         $TARGET_TRIPLE"
echo "    dist           $DIST_DIR"
echo "    version        $VERSION"
echo "    commit         $GIT_COMMIT"
echo "    built          $BUILD_TIMESTAMP_UTC"
echo "    bins           ${BINS[*]}"
echo

echo "==> Building container image"
podman build --no-cache -t "$IMAGE_TAG" -f "$CONTAINERFILE" .

echo
echo "==> Verifying container toolchain"
podman run --rm \
    --network="$NETWORK_MODE" \
    "$IMAGE_TAG" \
    /bin/bash -c 'set -e; echo "PATH=$PATH"; which rustc; which cargo; rustc --version; cargo --version'

BIN_LIST="$(printf "%s " "${BINS[@]}")"

echo
echo "==> Building binaries inside container"
podman run --rm \
    --network="$NETWORK_MODE" \
    -e CRUSHR_VERSION="$VERSION" \
    -e CRUSHR_GIT_COMMIT="$GIT_COMMIT" \
    -e CRUSHR_BUILD_TIMESTAMP="$BUILD_TIMESTAMP_UTC" \
    -e CRUSHR_TARGET_TRIPLE="$TARGET_TRIPLE" \
    -e CRUSHR_RUSTC_VERSION="$RUSTC_VERSION_HOST" \
    -e BUILD_PROFILE="$BUILD_PROFILE" \
    -e TARGET_TRIPLE="$TARGET_TRIPLE" \
    -e BIN_LIST="$BIN_LIST" \
    -e CARGO_PROFILE_RELEASE_LTO="$CONTAINER_LTO_VALUE" \
    -e CARGO_PROFILE_RELEASE_CODEGEN_UNITS="$CONTAINER_CODEGEN_UNITS_VALUE" \
    -e CARGO_NET_OFFLINE="$CARGO_NET_OFFLINE_VALUE" \
    -v "$ROOT_DIR:/src:Z" \
    -v "$DIST_DIR:/out:Z" \
    -v "$CACHE_ROOT/registry:/usr/local/cargo/registry:Z" \
    -v "$CACHE_ROOT/git:/usr/local/cargo/git:Z" \
    -v "$CACHE_ROOT/target:/src/target:Z" \
    -w /src \
    "$IMAGE_TAG" \
    /bin/bash -c '
        set -Eeuo pipefail

        echo "==> Container rust toolchain"
        which rustc
        which cargo
        rustc --version
        cargo --version

        case "$BUILD_PROFILE" in
            dev)
                CARGO_OUTPUT_DIR="debug"
                ;;
            release)
                CARGO_OUTPUT_DIR="release"
                ;;
            *)
                CARGO_OUTPUT_DIR="$BUILD_PROFILE"
                ;;
        esac

        echo "==> Cargo output dir: $CARGO_OUTPUT_DIR"

        for bin in $BIN_LIST; do
            echo
            echo "==> cargo build --profile $BUILD_PROFILE --target $TARGET_TRIPLE --bin $bin"
            cargo build \
                --profile "$BUILD_PROFILE" \
                --target "$TARGET_TRIPLE" \
                --bin "$bin"
        done

        echo
        echo "==> Listing target output directory"
        ls -la "/src/target/$TARGET_TRIPLE/$CARGO_OUTPUT_DIR" || true

        echo
        echo "==> Copying artifacts to /out"
        for bin in $BIN_LIST; do
            src="/src/target/$TARGET_TRIPLE/$CARGO_OUTPUT_DIR/$bin"
            if [[ ! -f "$src" ]]; then
                echo "Missing expected binary: $src" >&2
                exit 1
            fi

            cp "$src" "/out/$bin"

            if command -v strip >/dev/null 2>&1; then
                strip "/out/$bin" || true
            fi

            echo "  copied: /out/$bin"
            file "/out/$bin" || true
        done
    '

echo
echo "==> Verifying outputs"
for bin in "${BINS[@]}"; do
    out="$DIST_DIR/$bin"
    if [[ ! -f "$out" ]]; then
        echo "Expected output not found: $out" >&2
        exit 1
    fi

    printf "  %-18s %s\n" "$bin" "$out"
    if command -v file >/dev/null 2>&1; then
        file "$out" || true
    fi
    echo
done

echo "==> Writing checksums"
(
    cd "$DIST_DIR"
    sha256sum "${BINS[@]}" >SHA256SUMS
    if command -v b3sum >/dev/null 2>&1; then
        b3sum "${BINS[@]}" >B3SUMS
    fi
)

echo
echo "==> Build complete"
echo "Artifacts:"
for bin in "${BINS[@]}"; do
    echo "  $DIST_DIR/$bin"
done
echo "  $DIST_DIR/SHA256SUMS"
if [[ -f "$DIST_DIR/B3SUMS" ]]; then
    echo "  $DIST_DIR/B3SUMS"
fi
