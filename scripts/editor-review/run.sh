#!/usr/bin/env bash
# Runs Flowgram's grammar engine over a text file, one sentence per line, and prints each lint
# with its rule, the flagged text and the suggestions in the order a user would see them.
# Usage: bash scripts/editor-review/run.sh scripts/editor-review/corpus/spelling.txt
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
file="${1:?Give a text file with one sentence per line}"

cargo build -q --release --bin harper-cli 2>/dev/null
cli=target/release/harper-cli

while IFS= read -r line || [ -n "$line" ]; do
  [ -z "$line" ] && continue
  echo "SENTENCE: $line"
  "$cli" lint --format json --quiet "$line" 2>/dev/null | python3 -c '
import json, sys
try:
    lints = [l for f in json.load(sys.stdin) for l in f["lints"]]
except Exception:
    lints = []
if not lints:
    print("  (nothing flagged)")
for l in lints:
    sug = " | ".join(s.replace("Replace with: ", "").strip("“”") for s in l["suggestions"][:3]) or "-"
    print(f"  [{l[\"rule\"]}] \"{l[\"matched_text\"]}\" -> {sug}")
'
done < "$file"
