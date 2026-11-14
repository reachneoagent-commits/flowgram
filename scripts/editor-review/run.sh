#!/usr/bin/env bash
# Runs Gramflow's grammar engine over a text file, one sentence per line, and prints each lint
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
  "$cli" lint --format json --quiet "$line" 2>/dev/null | python3 scripts/editor-review/format_lints.py
done < "$file"
