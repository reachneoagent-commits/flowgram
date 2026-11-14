#!/usr/bin/env python3
"""Reports what changed upstream since we last reviewed the Humanizer project.

Reads humanizer/upstream-state.json (what we last approved), compares it with the live Hugging Face
model repository and the GitHub repository, and prints a Markdown report for a human to review.
It changes nothing unless --write-state is given. Exit code 0 means nothing changed, 10 means
there is something to review.
"""
import argparse
import datetime
import difflib
import sys

from humanizer_upstream import GITHUB_REPO, HF_REPO, STATE_PATH, TIER_FILES, get_json, hf_info, hf_text, load, save


def gb(size):
    return f"{(size or 0) / 1e9:.2f} GB"


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--write-state", action="store_true", help="record the current upstream state as reviewed")
    args = parser.parse_args()

    state = load(STATE_PATH, {})
    live = hf_info()
    prompt_now = hf_text(live["revision"], "prompt_format.json") if "prompt_format.json" in live["files"] else ""
    prompt_before = state.get("prompt_format", "")

    old_files = state.get("files", {})
    changed, added, removed = [], [], []
    for name, now in live["files"].items():
        before = old_files.get(name)
        if before is None:
            added.append(name)
        elif before.get("sha256") != now.get("sha256") or before.get("size") != now.get("size"):
            changed.append(name)
    removed = [name for name in old_files if name not in live["files"]]
    models_changed = [n for n in changed + added if n in TIER_FILES.values()]

    commits, files = [], []
    base = state.get("github_commit")
    try:
        head = get_json(f"https://api.github.com/repos/{GITHUB_REPO}/commits/main")["sha"]
        if base and base != head:
            compare = get_json(f"https://api.github.com/repos/{GITHUB_REPO}/compare/{base}...{head}")
            commits = [(c["sha"][:8], c["commit"]["author"]["date"][:10], c["commit"]["message"].splitlines()[0]) for c in compare.get("commits", [])]
            files = [(f["filename"], f["additions"], f["deletions"]) for f in compare.get("files", [])]
    except Exception as error:  # the report is still useful without the GitHub half
        head = base
        print(f"> Could not read the GitHub repository: {error}\n", file=sys.stderr)

    revision_changed = live["revision"] != state.get("hf_revision")
    anything = revision_changed or bool(commits)

    out = ["# Humanizer upstream review", ""]
    out.append(f"Generated {datetime.datetime.now(datetime.timezone.utc):%Y-%m-%d %H:%M} UTC. Last reviewed: {state.get('reviewed', 'never')}.")
    out.append("")
    if not anything:
        out.append("**No changes since the last review.** Nothing to do.")
        print("\n".join(out))
        return 0

    out += ["## Summary", ""]
    out.append(f"- Model repository `{HF_REPO}`: revision `{state.get('hf_revision', 'none')[:8]}` → `{live['revision'][:8]}` (modified {live['modified'][:10]})")
    out.append(f"- GitHub `{GITHUB_REPO}`: {len(commits)} new commit(s)")
    out.append(f"- Model files changed: {len(models_changed)}; added: {len(added)}; removed: {len(removed)}")
    out.append("")

    out += ["## Risk checklist", ""]
    prompt_changed = prompt_now != prompt_before
    out.append(f"- [{'!' if prompt_changed else 'x'}] Prompt format {'**CHANGED**: read the diff below and test before approving' if prompt_changed else 'unchanged'}")
    new_names = [n for n in added if n.endswith('.gguf')]
    out.append(f"- [{'!' if new_names or removed else 'x'}] File names {'changed: ' + ', '.join(new_names + removed) if new_names or removed else 'unchanged'}")
    big = [n for n in changed if n in TIER_FILES.values() and old_files.get(n, {}).get('size') and abs(live['files'][n]['size'] - old_files[n]['size']) > 0.1 * old_files[n]['size']]
    out.append(f"- [{'!' if big else 'x'}] Size changes over 10%: {', '.join(big) if big else 'none'}")
    legal = [n for n in changed + added if n in ('LICENSE', 'NOTICE')]
    out.append(f"- [{'!' if legal else 'x'}] License files {'**CHANGED**: ' + ', '.join(legal) if legal else 'unchanged'}")
    out.append("")

    if changed or added or removed:
        out += ["## Files", "", "| File | Change | Size | SHA-256 |", "|---|---|---|---|"]
        for name in sorted(changed + added + removed):
            kind = "changed" if name in changed else "added" if name in added else "removed"
            now = live["files"].get(name, {})
            out.append(f"| `{name}` | {kind} | {gb(now.get('size'))} | `{(now.get('sha256') or '-')[:12]}` |")
        out.append("")

    if prompt_changed:
        out += ["## prompt_format.json diff", "", "```diff"]
        out += [line.rstrip("\n") for line in difflib.unified_diff(prompt_before.splitlines(), prompt_now.splitlines(), "reviewed", "upstream", lineterm="")]
        out += ["```", ""]

    if commits:
        out += ["## GitHub commits since the last review", ""]
        out += [f"- `{sha}` {date} {message}" for sha, date, message in commits]
        out.append("")
        out += ["### Files touched", ""]
        out += [f"- `{name}` (+{add}/−{dele})" for name, add, dele in files[:60]]
        out.append("")

    out += [
        "## To approve",
        "",
        "1. Download the changed model with the Humanizer app or llama.cpp and try real drafts (numbers, names, English and Chinese).",
        "2. Publish it: `python3 scripts/humanizer-publish.py --version <label> --notes \"<what changed>\" --approve`",
        "3. Commit `humanizer/manifest.json` and `humanizer/upstream-state.json`. Gramflow users then see the update.",
        "",
        "Rejecting a release: do nothing, or run this script with `--write-state` to stop it being reported again.",
    ]
    print("\n".join(out))

    if args.write_state:
        save(STATE_PATH, {"hf_revision": live["revision"], "github_commit": head, "files": live["files"], "prompt_format": prompt_now, "reviewed": datetime.date.today().isoformat()})
        print("\n_State recorded as reviewed._", file=sys.stderr)
    return 10


if __name__ == "__main__":
    sys.exit(main())
