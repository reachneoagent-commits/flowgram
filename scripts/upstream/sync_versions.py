#!/usr/bin/env python3
"""Makes the browser extension's version match the Flowgram app's version.

Harper bumps its own version numbers in merges. Flowgram has one version, set in
harper-desktop/src-tauri/tauri.conf.json, and the extension should show the same number.
"""
import json
import re

app = re.search(r'"version":\s*"([^"]+)"', open("harper-desktop/src-tauri/tauri.conf.json").read()).group(1)
path = "packages/chrome-plugin/package.json"
text = open(path).read()
open(path, "w").write(re.sub(r'("version":\s*")[^"]+(")', rf"\g<1>{app}\g<2>", text, count=1))
print(f"extension version set to {app}")
