# Gramflow: recommendations and license register

**Rule: everything we ship must be open source** (OSI-approved licenses: MIT, Apache-2.0, BSD; fonts under SIL OFL). Gramflow inherits Harper's **Apache-2.0**, so MIT, Apache-2.0 and BSD can be bundled with attribution. Not legal advice.

## Recommendations (in build order)
| # | Feature | Source | License | Status |
|---|---|---|---|---|
| 1 | **AI-sounding-writing rule pack** (26 patterns as offline Weir rules) | [blader/humanizer](https://github.com/blader/humanizer) | MIT | Cleared. Write rules ourselves, credit the project and Wikipedia's guide |
| 2 | **Readability panel** (grade level, reading ease, reading time) | [textstat](https://docs.rs/crate/textstat/latest) Rust crate | MIT OR Apache-2.0 | Cleared |
| 3 | **Inclusive-language pack** | Ideas from [alex](https://github.com/get-alex/alex) and [retext-equality](https://github.com/retextjs/retext-equality) | MIT, MIT | Repo-level license confirmed MIT. Data folder `data/en` has no separate license on the repo page; confirm in the `license` file |
| 4 | **Profanity list** for a "keep it professional" check | [cuss](https://github.com/words/cuss) | MIT | Cleared |
| 5 | **Optional "Make it sound human" rewrite** (uses humanizer's `SKILL.md` as the prompt) | humanizer | MIT | Cleared. Runs only through a model the user connects |
| 6 | **Optional AI layer** (rewrite, shorten, paraphrase, tone) through the user's own local model | [Ollama](https://github.com/ollama/ollama) (REST API at `localhost:11434`) | MIT | Cleared. Models the user picks carry their own licenses; we only list ones that are open |
| 7 | **Small on-device grammar model** | [t5-efficient-mini-grammar-correction](https://huggingface.co/visheratin/t5-efficient-mini-grammar-correction) | Model MIT | **Not cleared.** Base model and training-data (C4_200M) licenses not stated |
| 8 | **Style-guide import** (Vale-style packs as Weirpacks) | [Vale](https://github.com/errata-ai/vale) (engine), [Microsoft style pack](https://github.com/errata-ai/Microsoft) | MIT, MIT | Engine and pack are MIT. The pack is derived from Microsoft's Writing Style Guide, whose own license is not stated. Write our own rules; don't copy |
| 9 | **Tone/formality detection** | [laya](https://github.com/NandhaKishorM/laya) | Apache-2.0 (code) | Defer. Model-weight license unconfirmed, ~2.3 GB, CPU speed unknown |

## Dropped because they are not open source
- **CoEdIT** ([model](https://huggingface.co/grammarly/coedit-large)): card says CC-BY-NC-4.0, which is non-commercial and not an open-source license. The [GitHub repo](https://github.com/vipulraheja/coedit) states no license at all. Excluded. Third-party "Apache-2.0" copies are unreliable.
- **LanguageTool** (LGPL-2.1): open source, but copyleft and Java-heavy. Skipped by choice, not by rule.
- **Grammarly itself**: proprietary. We study its features only; never copy text, rules or assets.

## Things still to check before shipping
1. `retext-equality` `license` file and `data/en` folder: confirm there are no extra terms.
2. Microsoft Writing Style Guide license: needed only if we ever import its rule text.
3. Base-model and dataset licenses for the T5 grammar model (google/t5-efficient-mini, C4_200M). Until confirmed, option 7 is not allowed.
4. Laya model-weight license (separate from its Apache-2.0 code).
5. Fonts: Inter Tight, Instrument Serif, JetBrains Mono are expected to be SIL OFL. Confirm the LICENSE files and bundle them locally instead of loading from Google Fonts (works offline, no network call from a privacy-first app).
6. Wikipedia's "signs of AI writing" content is expected to be CC BY-SA. We write original rules, not copied text; credit it in NOTICE.
7. Dependencies: run `cargo deny` / `pnpm licenses list` once to confirm every dependency is on an open-source allowlist.

## Housekeeping required by Harper's license (Apache-2.0)
- Keep `LICENSE` and add a `NOTICE`/`THIRD_PARTY_LICENSES` file listing every project above that we bundle.
- Keep copyright notices in files we modify and mark that we changed them.
- Do not use the Harper or Automattic names or logos. Gramflow's own logo and name only.
- Read `AGENT_POLICY.md` before any code change.

## Honest limits
Licenses above were read from GitHub and Hugging Face pages this session; several pages did not show the full license text. Treat "Cleared" as "no conflict found on the public page", and re-check the actual LICENSE file when we add each dependency.
