#!/usr/bin/env bash
# dev/build.sh - containerized build for crushr

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# shellcheck source=dev/lib/common.sh
source "dev/lib/common.sh"
# shellcheck source=dev/lib/args.sh
source "dev/lib/args.sh"
# shellcheck source=dev/lib/config.sh
source "dev/lib/config.sh"
# shellcheck source=dev/lib/container.sh
source "dev/lib/container.sh"
# shellcheck source=dev/lib/rust.sh
source "dev/lib/rust.sh"

require_cmd bash python3
parse_args "$@"
load_config "dev/build.toml"
detect_engine

TARGET="${TARGET:-$CFG_MUSL_TARGET}"
if [[ "$TARGET" == *"musl"* ]]; then
  CONTAINERFILE="${CONTAINERFILE:-$CFG_MUSL_CONTAINERFILE}"
  IMAGE_TAG="${IMAGE_TAG:-$CFG_MUSL_IMAGE_TAG}"
else
  CONTAINERFILE="${CONTAINERFILE:-$CFG_GLIBC_CONTAINERFILE}"
  IMAGE_TAG="${IMAGE_TAG:-$CFG_GLIBC_IMAGE_TAG}"
fi

case "$BIN" in
  crushr|all) ;;
  *) die "--bin must be crushr|all" ;;
esac

DIST_DIR="$ROOT_DIR/$CFG_DIST_DIR"
mkdir -p "$DIST_DIR"

APP_ID="$CFG_PROJECT_NAME"
APP_VER="$(cargo_version)"

TARGET_CACHE_VOL="crushr_target_cache"
REGISTRY_CACHE_VOL="crushr_registry_cache"

if [[ "$DO_CLEAN" == "1" ]]; then
  log_warn "clean requested; removing caches (volumes + image)"
  rm_volume "$TARGET_CACHE_VOL"
  rm_volume "$REGISTRY_CACHE_VOL"
  rm_image "$IMAGE_TAG"
  log_ok "clean complete"
  exit 0
fi

ensure_volume "$TARGET_CACHE_VOL"
ensure_volume "$REGISTRY_CACHE_VOL"
ensure_image "$IMAGE_TAG" "$CONTAINERFILE" "$ROOT_DIR/dev"

log_info "project: $APP_ID $APP_VER"
log_info "engine:  $ENGINE"
log_info "image:   $IMAGE_TAG"
log_info "target:  $TARGET"
log_info "profile: $BUILD_PROFILE"
log_info "bin:     $BIN"

COMMON_ARGS=(
  -e "TERM=${TERM:-xterm-256color}"
  -e "CARGO_TERM_COLOR=always"
  -e "RUST_BACKTRACE=1"
  -v "$TARGET_CACHE_VOL:/work/target:Z"
  -v "$REGISTRY_CACHE_VOL:/usr/local/cargo/registry:Z"
)

in_container_prelude='set -euo pipefail; cd /work;'
if [[ "$VERBOSE" == "1" ]]; then
  in_container_prelude+=' set -x;'
fi

log_info "building in container..."
build_cmd="$(cargo_build_cmd "$TARGET" "$BUILD_PROFILE" "$CFG_BINARY" "")"
run_in_container "$IMAGE_TAG" "${COMMON_ARGS[@]}" -- bash -lc "$in_container_prelude $build_cmd"
log_ok "build complete"

if [[ "$BUILD_PROFILE" == "release" ]] && [[ "$CFG_STRIP" == "1" ]]; then
  log_info "stripping (best effort)..."
  run_in_container "$IMAGE_TAG" "${COMMON_ARGS[@]}" -- bash -lc "$in_container_prelude $(strip_script "$TARGET" "$CFG_BINARY")"
  log_ok "strip step complete"
fi

if [[ "$DO_TEST" == "1" ]]; then
  log_info "running tests..."
  run_in_container "$IMAGE_TAG" "${COMMON_ARGS[@]}" -- bash -lc "$in_container_prelude $(cargo_test_script)"
  log_ok "tests complete"
fi

log_info "packaging dist..."
pkg="$(package_dist_script "$CFG_DIST_DIR" "$APP_ID" "$APP_VER" "$TARGET" "$CFG_BINARY" "$CFG_DOCS__NL")"
run_in_container "$IMAGE_TAG" "${COMMON_ARGS[@]}" -- bash -lc "$in_container_prelude $pkg"
log_ok "dist package ready"
