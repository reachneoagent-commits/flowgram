#!/usr/bin/env python3
"""Reads harper-cli JSON from stdin and prints one readable line per lint."""
import json
import sys

try:
    lints = [lint for file in json.load(sys.stdin) for lint in file["lints"]]
except Exception:
    lints = []

if not lints:
    print("  (nothing flagged)")

for lint in lints:
    suggestions = " | ".join(
        s.replace("Replace with: ", "").strip("“”") for s in lint["suggestions"][:3]
    ) or "-"
    print(f'  [{lint["rule"]}] "{lint["matched_text"]}" -> {suggestions}')
