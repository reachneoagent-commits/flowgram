#!/usr/bin/env python3
"""Makes every Gramflow version number match the app version in tauri.conf.json.

Covers the browser extension and the download links in the README.

Harper bumps its own version numbers in merges. Gramflow has one version, set in
harper-desktop/src-tauri/tauri.conf.json, and the extension should show the same number.
"""
import json
import re

app = re.search(r'"version":\s*"([^"]+)"', open("harper-desktop/src-tauri/tauri.conf.json").read()).group(1)
path = "packages/chrome-plugin/package.json"
text = open(path).read()
open(path, "w").write(re.sub(r'("version":\s*")[^"]+(")', rf"\g<1>{app}\g<2>", text, count=1))

readme = open("README.md", encoding="utf-8").read()
readme = re.sub(r"/download/v\d+\.\d+\.\d+/Gramflow_\d+\.\d+\.\d+_", f"/download/v{app}/Gramflow_{app}_", readme)
readme = re.sub(r"latest version, \d+\.\d+\.\d+\)", f"latest version, {app})", readme)
open("README.md", "w", encoding="utf-8").write(readme)
print(f"extension and README set to {app}")
