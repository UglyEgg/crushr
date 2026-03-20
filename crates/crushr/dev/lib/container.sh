#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

# Container engine helpers
# shellcheck shell=bash

set -euo pipefail

detect_engine() {
  ENGINE="${ENGINE:-podman}"
  case "$ENGINE" in
    podman|docker) ;;
    *) die "ENGINE must be podman or docker (got: $ENGINE)" ;;
  esac
  require_cmd "$ENGINE"
  export ENGINE
}

ensure_image() { # ensure_image <image_tag> <containerfile> <context_dir>
  local image_tag="${1:?image tag}"
  local containerfile="${2:?containerfile}"
  local context_dir="${3:?context dir}"

  if "$ENGINE" image exists "$image_tag" >/dev/null 2>&1; then
    log_ok "image exists: $image_tag"
    return 0
  fi

  log_info "building image: $image_tag"
  "$ENGINE" build -t "$image_tag" -f "$containerfile" "$context_dir"
  log_ok "built image: $image_tag"
}

ensure_volume() {
  local vol="${1:?volume name}"
  "$ENGINE" volume inspect "$vol" >/dev/null 2>&1 || "$ENGINE" volume create "$vol" >/dev/null
}

rm_volume() { local vol="${1:?volume}"; "$ENGINE" volume rm -f "$vol" >/dev/null 2>&1 || true; }
rm_image()  { local img="${1:?image}"; "$ENGINE" rmi -f "$img" >/dev/null 2>&1 || true; }

run_in_container() {
  local IMAGE="$1"
  shift

  # Optional engine run options may be provided before a `--` separator.
  # Example:
  #   run_in_container "$IMAGE" -e FOO=bar -v /x:/y -- bash -lc '...'
  local -a RUN_OPTS=()
  if [[ "${1:-}" != "" ]]; then
    while [[ "${1:-}" != "" && "${1:-}" != "--" ]]; do
      RUN_OPTS+=("$1")
      shift
    done
    if [[ "${1:-}" == "--" ]]; then shift; fi
  fi

  if [[ "${#RUN_OPTS[@]}" -gt 0 ]]; then
    "${ENGINE}" run "${RUN_OPTS[@]}"       --rm       --userns=keep-id       -v "${PWD}:/work:Z"       -w /work       "${IMAGE}" "$@"
  else
    "${ENGINE}" run       --rm       --userns=keep-id       -v "${PWD}:/work:Z"       -w /work       "${IMAGE}" "$@"
  fi
}
