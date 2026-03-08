#!/usr/bin/env bash
# Load dev/build.toml into shell variables.
# shellcheck shell=bash

set -euo pipefail

load_config() {
  local cfg="${1:?config path required}"
  [[ -f "$cfg" ]] || die "config file not found: $cfg"

  local assigns
  assigns="$(python3 - "$cfg" <<'PY'
import sys, tomllib, shlex
p = sys.argv[1]
with open(p, "rb") as f:
    t = tomllib.load(f)

proj = t.get("project", {})
out = t.get("output", {})
musl = t.get("container", {}).get("musl", {})
glibc = t.get("container", {}).get("glibc", {})
build = t.get("build", {})

def emit(k, v):
    if v is None:
        return
    if isinstance(v, bool):
        v = "1" if v else "0"
    print(f"{k}={shlex.quote(str(v))}")

emit("CFG_PROJECT_NAME", proj.get("name", "crushr"))
emit("CFG_CRATE_DIR", proj.get("crate_dir", "crushr"))
emit("CFG_BINARY", proj.get("binary", "crushr"))

emit("CFG_DIST_DIR", out.get("dist_dir", "dist"))
docs = out.get("docs", [])
print("CFG_DOCS__NL=" + shlex.quote("\n".join(map(str, docs))))

emit("CFG_MUSL_IMAGE_TAG", musl.get("image_tag", "crushr-build:musl"))
emit("CFG_MUSL_CONTAINERFILE", musl.get("containerfile", "dev/Containerfile.build"))
emit("CFG_MUSL_TARGET", musl.get("target", "x86_64-unknown-linux-musl"))

emit("CFG_GLIBC_IMAGE_TAG", glibc.get("image_tag", "crushr-build:glibc"))
emit("CFG_GLIBC_CONTAINERFILE", glibc.get("containerfile", "dev/Containerfile.build.debian"))
emit("CFG_GLIBC_TARGET", glibc.get("target", "x86_64-unknown-linux-gnu"))

emit("CFG_DEFAULT_PROFILE", build.get("profile", "release"))
emit("CFG_STRIP", bool(build.get("strip", True)))
PY
)"
  eval "$assigns"

  IFS=$'\n' read -r -d '' -a CFG_DOCS <<<"${CFG_DOCS__NL}"$'\0' || true

  export CFG_PROJECT_NAME CFG_CRATE_DIR CFG_BINARY CFG_DIST_DIR          CFG_MUSL_IMAGE_TAG CFG_MUSL_CONTAINERFILE CFG_MUSL_TARGET          CFG_GLIBC_IMAGE_TAG CFG_GLIBC_CONTAINERFILE CFG_GLIBC_TARGET          CFG_DEFAULT_PROFILE CFG_STRIP CFG_DOCS__NL
}
