#!/usr/bin/env bash
# Rust build/test packaging helpers
# shellcheck shell=bash

set -euo pipefail

cargo_version() {
  python3 - <<'PY'
import tomllib
with open("Cargo.toml","rb") as f:
    t = tomllib.load(f)
print(t.get("package",{}).get("version","0.0.0"))
PY
}

cargo_build_cmd() { # cargo_build_cmd <target> <profile> <bin> [extra]
  local target="${1:?target}"
  local profile="${2:?profile}" # dev|release
  local bin="${3:?bin}"
  local extra="${4:-}"

  local flags=()
  if [[ "$profile" == "release" ]]; then
    flags+=(--release)
  fi
  if [[ "$bin" != "all" ]]; then
    flags+=(--bin "$bin")
  fi

  printf 'cargo build %s --target %s %s' "${flags[*]}" "$target" "$extra"
}

cargo_test_script() { # emits a sh script
  cat <<'SH'
set -euo pipefail
if command -v cargo-nextest >/dev/null 2>&1; then
  cargo nextest run
else
  cargo test
fi
SH
}

strip_script() { # strip_script <target> <bin>
  local target="${1:?target}"
  local bin="${2:?bin}"
  cat <<SH
set -euo pipefail
if command -v strip >/dev/null 2>&1; then
  strip -s "target/$target/release/$bin" 2>/dev/null || true
fi
SH
}

package_dist_script() { # package_dist_script <dist_dir> <app_id> <app_ver> <target> <bin> <docs_nl>
  local dist_dir="${1:?dist dir}"
  local app_id="${2:?app id}"
  local app_ver="${3:?app ver}"
  local target="${4:?target}"
  local bin="${5:?bin}"
  local docs_nl="${6:-}"

  cat <<SH
set -euo pipefail
OUT_DIR="$dist_dir/${app_id}-${app_ver}-${target}"
rm -rf "\$OUT_DIR"
mkdir -p "\$OUT_DIR/bin" "\$OUT_DIR/spec"

cp -f "target/$target/release/$bin" "\$OUT_DIR/bin/$bin"

# Docs (best effort)
docs=\$(printf '%s' "$docs_nl")
IFS='\n' read -r -d '' -a docarr <<<"\$docs\$'\0'" || true
for p in "\${docarr[@]}"; do
  [[ -z "\$p" ]] && continue
  if [[ -f "/work/\$p" ]]; then
    bn="\$(basename "\$p")"
    if [[ "\$bn" == "SPEC.md" ]]; then
      cp -f "/work/\$p" "\$OUT_DIR/spec/SPEC.md"
    else
      cp -f "/work/\$p" "\$OUT_DIR/\$bn"
    fi
  fi
done

tar -C "$dist_dir" -czf "$dist_dir/${app_id}-${app_ver}-${target}.tar.gz" "${app_id}-${app_ver}-${target}"
echo "dist: $dist_dir/${app_id}-${app_ver}-${target}.tar.gz"
SH
}
