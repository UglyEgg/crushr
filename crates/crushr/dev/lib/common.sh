#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

# Common helpers for dev/build.sh
# shellcheck shell=bash

set -euo pipefail

supports_color() {
  [[ -t 1 ]] && [[ "${NO_COLOR:-}" != "1" ]] && [[ "${TERM:-}" != "dumb" ]]
}

c() { # c CODE TEXT...
  local code="$1"; shift
  if supports_color; then
    printf '\033[%sm%s\033[0m' "$code" "$*"
  else
    printf '%s' "$*"
  fi
}

log_info()  { printf '%s %s\n' "$(c '1;34' '[INFO]')" "$*"; }
log_warn()  { printf '%s %s\n' "$(c '1;33' '[WARN]')" "$*" >&2; }
log_error() { printf '%s %s\n' "$(c '1;31' '[ERR ]')" "$*" >&2; }
log_ok()    { printf '%s %s\n' "$(c '1;32' '[ OK ]')" "$*"; }

die() { log_error "$*"; exit 1; }

require_cmd() {
  local cmd
  for cmd in "$@"; do
    command -v "$cmd" >/dev/null 2>&1 || die "missing required command: $cmd"
  done
}

abspath() { # abspath <path>
  python3 - "$1" <<'PY'
import os, sys
print(os.path.abspath(sys.argv[1]))
PY
}
