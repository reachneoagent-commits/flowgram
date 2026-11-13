#!/usr/bin/env python3
"""Publishes an approved Humanizer release to humanizer/manifest.json.

Run this only after reviewing `humanizer-upstream-report.py` and trying the new model. The manifest
pins each model to the exact upstream revision and checksum, so what users download can never
change after approval. Gramflow reads this file, never the upstream repository.
"""
import argparse
import datetime
import sys

from humanizer_upstream import HF_REPO, MANIFEST_PATH, STATE_PATH, TIER_FILES, get_json, hf_info, hf_text, load, save


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--version", required=True, help="label for this release, e.g. 2026-11-20")
    parser.add_argument("--notes", required=True, help="what changed, in plain words; users read this before updating")
    parser.add_argument("--tiers", default=",".join(TIER_FILES), help="model sizes to publish (default: all)")
    parser.add_argument("--min-app", default="3.0.12", help="oldest Gramflow version that can use this release")
    parser.add_argument("--approve", action="store_true", help="required: confirms you reviewed the changes and tested the model")
    args = parser.parse_args()

    if not args.approve:
        sys.exit("Refusing to publish without --approve. Review the upstream report and test the model first.")

    live = hf_info()
    tiers = [t.strip() for t in args.tiers.split(",") if t.strip()]
    unknown = [t for t in tiers if t not in TIER_FILES]
    if unknown:
        sys.exit(f"Unknown model sizes: {', '.join(unknown)}")

    manifest = load(MANIFEST_PATH, {"schema": 1, "published": "", "models": []})
    released = (live["modified"] or datetime.date.today().isoformat())[:10]
    by_tier = {m["tier"]: m for m in manifest["models"]}
    for tier in tiers:
        name = TIER_FILES[tier]
        info = live["files"].get(name)
        if not info or not info.get("sha256") or not info.get("size"):
            sys.exit(f"Upstream has no usable file for {tier} ({name}).")
        by_tier[tier] = {
            "tier": tier,
            "version": args.version,
            "released": released,
            "notes": args.notes,
            "file": name,
            "url": f"https://huggingface.co/{HF_REPO}/resolve/{live['revision']}/{name}",
            "size_bytes": info["size"],
            "sha256": info["sha256"],
            "min_app_version": args.min_app,
        }
    manifest["models"] = [by_tier[t] for t in TIER_FILES if t in by_tier]
    manifest["published"] = datetime.date.today().isoformat()
    save(MANIFEST_PATH, manifest)

    head = get_json("https://api.github.com/repos/sgaofen/humanizer-local-model/commits/main")["sha"]
    prompt = hf_text(live["revision"], "prompt_format.json") if "prompt_format.json" in live["files"] else ""
    save(STATE_PATH, {"hf_revision": live["revision"], "github_commit": head, "files": live["files"], "prompt_format": prompt, "reviewed": datetime.date.today().isoformat()})
    print(f"Published {len(tiers)} model(s) as version {args.version}, pinned to {live['revision'][:8]}.")
    print("Now commit humanizer/manifest.json and humanizer/upstream-state.json.")


if __name__ == "__main__":
    main()
