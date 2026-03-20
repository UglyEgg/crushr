#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION_FILE="$ROOT_DIR/VERSION"
CARGO_FILE="$ROOT_DIR/Cargo.toml"

version_raw="$(cat "$VERSION_FILE")"
version="${version_raw%$'\n'}"

python3 - "$version" <<'PY'
import re
import sys

value = sys.argv[1]
pattern = r"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:0|[1-9]\d*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
if not re.fullmatch(pattern, value):
    raise SystemExit(f"VERSION is not strict SemVer: {value!r}")
PY

workspace_version="$(python3 - "$CARGO_FILE" <<'PY'
import pathlib
import sys

text = pathlib.Path(sys.argv[1]).read_text(encoding='utf-8')
in_section = False
for line in text.splitlines():
    stripped = line.strip()
    if stripped.startswith('[') and stripped.endswith(']'):
        in_section = stripped == '[workspace.package]'
        continue
    if in_section and stripped.startswith('version'):
        print(stripped.split('=', 1)[1].strip().strip('"'))
        raise SystemExit(0)
raise SystemExit('workspace.package.version not found')
PY
)"

if [[ "$workspace_version" != "$version" ]]; then
  echo "version drift detected: VERSION=$version workspace.package.version=$workspace_version" >&2
  exit 1
fi

echo "version sync ok: $version"
