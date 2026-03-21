#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(tr -d '\r\n' <"$ROOT_DIR/VERSION")"
CARGO_TOML="$ROOT_DIR/Cargo.toml"

python3 - "$CARGO_TOML" "$VERSION" <<'PY'
import pathlib
import re
import sys

cargo = pathlib.Path(sys.argv[1])
version = sys.argv[2]
text = cargo.read_text(encoding='utf-8')

semver = r"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:0|[1-9]\d*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
if not re.fullmatch(semver, version):
    raise SystemExit(f"VERSION is not strict SemVer: {version!r}")

pattern = re.compile(r'(?ms)(\[workspace\.package\]\s.*?^version\s*=\s*")([^"]+)(")')
updated, n = pattern.subn(lambda m: m.group(1) + version + m.group(3), text, count=1)
if n != 1:
    raise SystemExit("unable to locate workspace.package.version in Cargo.toml")

cargo.write_text(updated, encoding='utf-8')
print(f"updated workspace.package.version -> {version}")
PY
