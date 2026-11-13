#!/usr/bin/env bash
# Fails if a Harper merge changed Flowgram's identity or put Harper's name in the app UI.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
fail=0
conf=harper-desktop/src-tauri/tauri.conf.json

expect() { # file, pattern, message
  grep -q "$2" "$1" || { echo "BRANDING: $3"; fail=1; }
}

expect "$conf" '"productName": "Flowgram"' 'productName is not Flowgram'
expect "$conf" '"identifier": "com.flowgram.app"' 'app identifier changed'
expect "$conf" 'reachneoagent-commits/gramflow/releases/latest/download/latest.json' 'updater endpoint no longer points at our releases'
expect "$conf" "$(cat scripts/upstream/updater_pubkey.txt)" 'updater public key changed'
expect harper-desktop/src-tauri/Cargo.toml 'name = "flowgram-desktop"' 'crate name changed'

hits=$(grep -rniE "harper" harper-desktop/src harper-desktop/src-tauri/src 2>/dev/null \
  | grep -viE "import |from '|harper\.js|harper-editor|harper_|harper-core|harper-dictionary|@harper|harper-lint|\.harper-|harper-btn|use harper|extern crate|harper-desktop|// |/// |^\S+:[0-9]+:\s*\*" || true)
if [ -n "$hits" ]; then
  echo "BRANDING: Harper's name appears in app text:"; echo "$hits" | head -20; fail=1
fi
exit $fail
