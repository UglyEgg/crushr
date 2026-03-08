#!/usr/bin/env bash
# Argument parsing for dev/build.sh
# shellcheck shell=bash

set -euo pipefail

usage() {
  cat <<'USAGE'
dev/build.sh - containerized build for crushr

Usage:
  ./dev/build.sh [--release|--dev] [--test] [--clean] [--bin NAME]
                [--target TARGET] [--engine podman|docker]
                [--no-color] [--verbose]

Examples:
  ./dev/build.sh --release
  ./dev/build.sh --test
  TARGET=x86_64-unknown-linux-gnu ./dev/build.sh --release
  ENGINE=docker ./dev/build.sh --release

Flags:
  --release          Build optimized release artifact (default)
  --dev              Build debug artifact (faster)
  --test             Run tests inside the container
  --clean            Remove build caches (builder image + cache volumes)
  --bin NAME         Build only one binary: crushr | all (default: all)
  --target TARGET    Override Rust target triple (default from dev/build.toml)
  --engine NAME      podman or docker (default: podman, or $ENGINE)
  --no-color         Disable ANSI colors (also honors NO_COLOR=1)
  --verbose          Print extra detail (enables xtrace in container steps)

Environment:
  ENGINE             Container engine (podman|docker)
  TARGET             Rust target triple
  IMAGE_TAG          Override container image tag
  NO_COLOR=1         Disable colored output

USAGE
}

parse_args() {
  BUILD_PROFILE="release"
  DO_TEST="0"
  DO_CLEAN="0"
  BIN="all"
  VERBOSE="0"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --release) BUILD_PROFILE="release"; shift ;;
      --dev)     BUILD_PROFILE="dev"; shift ;;
      --test)    DO_TEST="1"; shift ;;
      --clean)   DO_CLEAN="1"; shift ;;
      --bin)     BIN="${2:-}"; shift 2 ;;
      --target)  TARGET="${2:-}"; shift 2 ;;
      --engine)  ENGINE="${2:-}"; shift 2 ;;
      --no-color) NO_COLOR="1"; shift ;;
      --verbose) VERBOSE="1"; shift ;;
      -h|--help) usage; exit 0 ;;
      *) die "unknown argument: $1 (use --help)" ;;
    esac
  done

  export BUILD_PROFILE DO_TEST DO_CLEAN BIN VERBOSE
}
