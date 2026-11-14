"""Shared helpers for the Humanizer upstream tools (stdlib only)."""
import json
import os
import urllib.request

HF_REPO = "jialinyyzz/humanizer"
GITHUB_REPO = "sgaofen/humanizer-local-model"
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STATE_PATH = os.path.join(ROOT, "humanizer", "upstream-state.json")
MANIFEST_PATH = os.path.join(ROOT, "humanizer", "manifest.json")

# Model size id -> file name. Keep in step with harper-desktop/src-tauri/src/humanizer/models.rs.
TIER_FILES = {
    "q8": "humanizer-12b-Q8_0.gguf",
    "q6": "humanizer-12b-Q6_K.gguf",
    "q4": "humanizer-12b-Q4_K_M.gguf",
    "q3": "humanizer-12b-Q3-QAT.gguf",
    "q2": "humanizer-12b-IQ2_XS-QAT.gguf",
}
WATCHED_SMALL_FILES = ["prompt_format.json", "config.json", "generation_config.json", "tokenizer_config.json", "LICENSE", "NOTICE"]


def get(url, accept="application/json"):
    request = urllib.request.Request(url, headers={"User-Agent": "Gramflow-humanizer-tools", "Accept": accept})
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read()


def get_json(url):
    return json.loads(get(url))


def hf_info():
    """Current upstream revision and the files we care about, with sizes and checksums."""
    data = get_json(f"https://huggingface.co/api/models/{HF_REPO}?blobs=true")
    files = {}
    for sibling in data.get("siblings", []):
        name = sibling["rfilename"]
        if name in TIER_FILES.values() or name in WATCHED_SMALL_FILES:
            files[name] = {"size": sibling.get("size"), "sha256": (sibling.get("lfs") or {}).get("sha256", "")}
    return {"revision": data["sha"], "modified": data.get("lastModified", ""), "files": files}


def hf_text(revision, name):
    return get(f"https://huggingface.co/{HF_REPO}/resolve/{revision}/{name}", accept="*/*").decode("utf-8", "replace")


def load(path, default):
    try:
        with open(path) as handle:
            return json.load(handle)
    except FileNotFoundError:
        return default


def save(path, value):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as handle:
        json.dump(value, handle, indent=2)
        handle.write("\n")
