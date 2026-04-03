#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT_DIR"

rm -rf dist pkg
wasm-pack build --target web --out-dir pkg --release --no-opt

mkdir -p dist/pkg
cp web/index.html dist/index.html
cp web/main.js dist/main.js
cp web/styles.css dist/styles.css
cp -r pkg/* dist/pkg/
touch dist/.nojekyll

echo "Built static demo bundle at: $ROOT_DIR/dist"
