#!/usr/bin/env python3
"""Prints a Markdown summary of Harper commits we have not merged yet, grouped by area.

Usage: summarize.py <base-sha> <upstream-ref>
"""
import subprocess
import sys

UPSTREAM = "https://github.com/Automattic/harper"
AREAS = [
    ("Grammar rules and engine", ("harper-core/", "harper-pos-utils/", "harper-brill/", "harper-wasm/", "harper-thesaurus/", "harper-dictionary-wordlist/", "harper.js/")),
    ("Browser extension", ("packages/chrome-plugin/", "packages/lint-framework/")),
    ("Desktop app", ("harper-desktop/", "packages/components/", "packages/harper-editor/")),
]


def git(*args: str) -> str:
    return subprocess.run(["git", *args], check=True, capture_output=True, text=True).stdout


def area_of(paths: list[str]) -> str:
    for name, prefixes in AREAS:
        if any(p.startswith(prefix) or f"/{prefix}" in f"/{p}" for p in paths for prefix in prefixes):
            return name
    return "Everything else (website, docs, other editors, CI)"


base, ref = sys.argv[1], sys.argv[2]
shas = git("rev-list", "--reverse", f"{base}..{ref}").split()
groups: dict[str, list[str]] = {}
for sha in shas:
    subject = git("show", "-s", "--format=%s", sha).strip()
    paths = [p for p in git("show", "--name-only", "--format=", sha).splitlines() if p]
    groups.setdefault(area_of(paths), []).append(f"- [{subject}]({UPSTREAM}/commit/{sha}) (`{sha[:8]}`)")

print(f"**{len(shas)} new commit(s)** in Harper since our last merge.\n")
print(f"[Compare on GitHub]({UPSTREAM}/compare/{base[:12]}...{ref.split('/')[-1] if '/' in ref else ref[:12]})\n")
order = [a[0] for a in AREAS] + ["Everything else (website, docs, other editors, CI)"]
for name in order:
    if name in groups:
        print(f"### {name} ({len(groups[name])})")
        print("\n".join(groups[name]) + "\n")
