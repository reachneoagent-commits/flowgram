#!/usr/bin/env python3
"""Resolves the merge conflicts we expect when merging Harper into Gramflow.

Only Gramflow's identity (names, versions, app id) conflicts routinely, so those files keep
our side. Cargo.lock is regenerated. Any other conflict is left for a human and fails the run.
"""
import re
import subprocess
import sys

KEEP_OURS = {
    "harper-desktop/package.json",
    "harper-desktop/src-tauri/Cargo.toml",
    "harper-desktop/src-tauri/tauri.conf.json",
}
LOCK = "harper-desktop/src-tauri/Cargo.lock"
HUNK = re.compile(r"<<<<<<< [^\n]*\n(.*?)=======\n(.*?)>>>>>>> [^\n]*\n", re.S)


def run(*args: str) -> str:
    return subprocess.run(args, check=True, capture_output=True, text=True).stdout


conflicted = run("git", "diff", "--name-only", "--diff-filter=U").split()
unresolved = []

for path in conflicted:
    if path in KEEP_OURS:
        text = open(path, encoding="utf-8").read()
        open(path, "w", encoding="utf-8").write(HUNK.sub(lambda m: m.group(1), text))
        run("git", "add", path)
        print(f"kept our side: {path}")
    elif path == LOCK:
        run("git", "checkout", "--ours", path)
        run("git", "add", path)
        print(f"reset {path} to ours; it is refreshed below")
    else:
        unresolved.append(path)

if LOCK in conflicted:
    # Re-resolve the lockfile against the merged manifests without building anything.
    run("cargo", "metadata", "--manifest-path", "harper-desktop/src-tauri/Cargo.toml", "--format-version", "1")
    run("git", "add", LOCK)

if unresolved:
    print("NEEDS A HUMAN:\n  " + "\n  ".join(unresolved))
    sys.exit(1)
