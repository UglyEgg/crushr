#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="${ROOT_DIR}/dist"

rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

wasm-pack build --target web --no-opt --out-dir "${DIST_DIR}/pkg"
cp "${ROOT_DIR}/web/index.html" "${DIST_DIR}/index.html"
cp "${ROOT_DIR}/web/main.js" "${DIST_DIR}/main.js"
cp "${ROOT_DIR}/web/styles.css" "${DIST_DIR}/styles.css"

# GitHub Pages compatibility for repos without custom Jekyll setup.
touch "${DIST_DIR}/.nojekyll"

echo "Built static demo bundle at ${DIST_DIR}"
