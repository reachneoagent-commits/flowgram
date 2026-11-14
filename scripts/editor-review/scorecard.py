#!/usr/bin/env python3
"""Scores Gramflow's grammar engine against sentences with known right answers.

Gold files are JSON lines: {"category": "...", "text": "...", "errors": [{"text": "corect", "fix": "correct"}]}
A sentence with an empty "errors" list is correct writing, so anything flagged in it is a false alarm.

Usage:
  python3 scripts/editor-review/scorecard.py                       # score every gold file
  python3 scripts/editor-review/scorecard.py --failures            # also list what went wrong
  python3 scripts/editor-review/scorecard.py --save before.json    # keep these numbers
  python3 scripts/editor-review/scorecard.py --compare before.json # show what a change did
"""
import argparse
import glob
import json
import subprocess
import sys
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CLI = ROOT / "target/release/harper-cli"


def lint(text: str) -> list[dict]:
    out = subprocess.run(
        [str(CLI), "lint", "--format", "json", "--quiet", text],
        capture_output=True, text=True, cwd=ROOT,
    ).stdout
    try:
        return [item for file in json.loads(out) for item in file["lints"]]
    except (json.JSONDecodeError, KeyError):
        return []


def clean_suggestion(raw: str) -> str:
    return raw.replace("Replace with:", "").strip().strip("“”\"")


def find_span(text: str, needle: str, start: int = 0) -> tuple[int, int] | None:
    import re
    before = r"(?<![A-Za-z])" if needle[:1].isalpha() else ""
    after = r"(?![A-Za-z])" if needle[-1:].isalpha() else ""
    match = re.search(before + re.escape(needle) + after, text[start:], re.IGNORECASE)
    return None if match is None else (start + match.start(), start + match.end())


def overlaps(a: tuple[int, int], b: tuple[int, int]) -> bool:
    return a[0] < b[1] and b[0] < a[1]


def score_sentence(entry: dict) -> dict:
    text = entry["text"]
    lints = lint(text)
    spans = [(l["span"]["char_start"], l["span"]["char_end"]) for l in lints]
    used = set()
    result = {"expected": 0, "found": 0, "first_ok": 0, "top3_ok": 0, "missed": [], "bad_first": [], "false_alarms": []}

    for error in entry["errors"]:
        result["expected"] += 1
        target = find_span(text, error["text"])
        hit = None
        if target:
            for i, span in enumerate(spans):
                if i not in used and overlaps(span, target):
                    hit = i
                    break
        if hit is None:
            result["missed"].append({"text": text, "wanted": error})
            continue

        used.add(hit)
        result["found"] += 1
        suggestions = [clean_suggestion(s) for s in lints[hit]["suggestions"]]
        fix = error.get("fix")
        if fix is None or (suggestions and suggestions[0].lower() == fix.lower()):
            result["first_ok"] += 1
            result["top3_ok"] += 1
        elif fix.lower() in [s.lower() for s in suggestions[:3]]:
            result["top3_ok"] += 1
            result["bad_first"].append({"text": text, "wanted": fix, "got": suggestions[:3], "rule": lints[hit]["rule"]})
        else:
            result["bad_first"].append({"text": text, "wanted": fix, "got": suggestions[:3], "rule": lints[hit]["rule"]})

    for i, lint_item in enumerate(lints):
        if i not in used:
            result["false_alarms"].append({"text": text, "rule": lint_item["rule"], "flagged": lint_item["matched_text"]})

    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="*")
    parser.add_argument("--failures", action="store_true")
    parser.add_argument("--save")
    parser.add_argument("--compare")
    args = parser.parse_args()

    subprocess.run(["cargo", "build", "-q", "--release", "--bin", "harper-cli"], cwd=ROOT, check=True)

    files = args.files or sorted(glob.glob(str(ROOT / "scripts/editor-review/gold/*.jsonl")))
    entries = [json.loads(line) for f in files for line in open(f, encoding="utf-8") if line.strip()]

    with ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(score_sentence, entries))

    cats = defaultdict(lambda: defaultdict(int))
    failures = defaultdict(list)
    for entry, result in zip(entries, results):
        c = cats[entry["category"]]
        c["sentences"] += 1
        for key in ("expected", "found", "first_ok", "top3_ok"):
            c[key] += result[key]
        c["false_alarms"] += len(result["false_alarms"])
        failures["missed"] += result["missed"]
        failures["bad_first"] += result["bad_first"]
        failures["false_alarms"] += result["false_alarms"]

    total = defaultdict(int)
    for c in cats.values():
        for key, value in c.items():
            total[key] += value
    cats["ALL"] = total

    def pct(a, b):
        return "  -  " if b == 0 else f"{100 * a / b:4.0f}%"

    previous = json.load(open(args.compare)) if args.compare else None
    print(f"{'category':<15}{'sent':>5}{'caught':>8}{'1st ok':>8}{'false alarms':>14}" + ("   vs before (caught / 1st ok / alarms)" if previous else ""))
    summary = {}
    for name, c in cats.items():
        summary[name] = dict(c)
        line = f"{name:<15}{c['sentences']:>5}{pct(c['found'], c['expected']):>8}{pct(c['first_ok'], c['found']):>8}{c['false_alarms']:>14}"
        if previous and name in previous:
            p = previous[name]
            def delta(now, before_num, before_den, now_den):
                if before_den == 0 or now_den == 0:
                    return "  -"
                return f"{100 * now / now_den - 100 * before_num / before_den:+.0f}"
            line += f"   {delta(c['found'], p['found'], p['expected'], c['expected'])} / {delta(c['first_ok'], p['first_ok'], p['found'], c['found'])} / {c['false_alarms'] - p['false_alarms']:+d}"
        print(line)

    if args.failures:
        for kind, label in (("missed", "MISSED"), ("bad_first", "WRONG FIRST SUGGESTION"), ("false_alarms", "FALSE ALARM")):
            if failures[kind]:
                print(f"\n{label} ({len(failures[kind])})")
                for item in failures[kind]:
                    print("  ", json.dumps(item, ensure_ascii=False))

    if args.save:
        json.dump(summary, open(args.save, "w"), indent=2)
        print(f"\nsaved to {args.save}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
