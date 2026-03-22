#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# SPDX-FileCopyrightText: 2026 Richard Majewski

set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

python3 - "$ROOT_DIR" <<'PY'
import pathlib
import sys
try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib

root = pathlib.Path(sys.argv[1])
cargo_file = root / "Cargo.toml"
workspace = tomllib.loads(cargo_file.read_text(encoding="utf-8"))

expected_resolver = "3"
expected_edition = "2024"
expected_rust_version = "1.88"

workspace_table = workspace.get("workspace", {})
workspace_package = workspace_table.get("package", {})
members = workspace_table.get("members", [])

errors = []

if workspace_table.get("resolver") != expected_resolver:
    errors.append(
        f"workspace resolver drift: expected {expected_resolver}, got {workspace_table.get('resolver')!r}"
    )
if workspace_package.get("edition") != expected_edition:
    errors.append(
        f"workspace.package.edition drift: expected {expected_edition}, got {workspace_package.get('edition')!r}"
    )
if workspace_package.get("rust-version") != expected_rust_version:
    errors.append(
        f"workspace.package.rust-version drift: expected {expected_rust_version}, got {workspace_package.get('rust-version')!r}"
    )

internal_only = {"crushr-cli-common", "crushr-lab", "crushr-tui"}
required_workspace_inheritance = (
    "version",
    "edition",
    "rust-version",
    "license",
    "authors",
    "repository",
    "homepage",
    "documentation",
    "keywords",
    "categories",
)
publishable = []
non_publishable = []

for member in members:
    manifest_path = root / member / "Cargo.toml"
    data = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    package = data.get("package", {})
    crate_name = package.get("name")
    if not crate_name:
        errors.append(f"{manifest_path}: missing package.name")
        continue

    publish_value = package.get("publish", True)
    if publish_value is False:
        non_publishable.append(crate_name)
    else:
        publishable.append(crate_name)

    if crate_name in internal_only and publish_value is not False:
        errors.append(f"{manifest_path}: {crate_name} must set publish = false")

    if publish_value is not False:
        for key in required_workspace_inheritance:
            value = package.get(key)
            if not isinstance(value, dict) or value.get("workspace") is not True:
                errors.append(
                    f"{manifest_path}: publishable crate missing `{key}.workspace = true`"
                )
        readme = package.get("readme")
        if not isinstance(readme, str) or not readme.strip():
            errors.append(f"{manifest_path}: publishable crate missing non-empty readme")
        description = package.get("description")
        if not isinstance(description, str) or not description.strip():
            errors.append(f"{manifest_path}: publishable crate missing crate-specific description")

if errors:
    for error in errors:
        print(f"ERROR: {error}")
    raise SystemExit(1)

print("crate policy check ok")
print(f"publishable crates: {', '.join(sorted(publishable))}")
print(f"non-publishable crates: {', '.join(sorted(non_publishable))}")
PY
