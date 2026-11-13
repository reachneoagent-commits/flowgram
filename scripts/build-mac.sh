#!/usr/bin/env bash
# Builds the Apple Silicon Gramflow installer without `just`. Run from the repo root.
set -eo pipefail
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
ROOT="$(pwd)"

cd "$ROOT/harper-wasm"
wasm-pack build --target web --out-name harper_wasm
wasm-pack build --target web --out-name harper_wasm_slim --no-default-features
perl -pi -e 's/new URL\(.*\)/new URL()/g' pkg/harper_wasm.js pkg/harper_wasm_slim.js

for pkg in harper.js lint-framework components harper-editor; do
  cd "$ROOT/packages/$pkg"; pnpm install --engine-strict=false; pnpm build
done

cd "$ROOT/packages/chrome-plugin"
pnpm install --engine-strict=false
pnpm zip-for-chrome
pnpm build-for-firefox

cd "$ROOT/harper-desktop"
pnpm install
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/gramflow.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$(cat ~/.tauri/gramflow.key.password)"
pnpm tauri build -b app,dmg --target aarch64-apple-darwin
