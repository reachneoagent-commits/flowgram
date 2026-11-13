# Harper vs Grammarly: gap analysis for Gramflow

Sources: Harper from this repo (`harper-core`, `harper-desktop`, `harper-ls`, `packages/*`, `COMPARISON.md`); Grammarly from grammarly.com/features (official page). Third-party pricing pages contradict each other, so pricing is excluded. I did not read every one of Harper's ~378 rule files; counts come from directory listings and greps.

## What Harper is
- **Engine:** Rust, rule-based, offline, ~10 ms per suggestion (repo comparison table: Grammarly 4000 ms).
- **Rules:** ~370 Rust linters plus ~370 declarative Weir rule files. Lint kinds: Agreement, Capitalization, Grammar, Punctuation, Readability, Redundancy, Repetition, Spelling, Style, Usage, WordChoice and others.
- **Spelling:** 55k-entry dictionary with 5 English dialects (American, British, Canadian, Australian, Indian), plus a user dictionary.
- **Extras:** thesaurus crate, language detection, title-case, ignore lists, local stats (`harper-stats`), Weirpacks (installable rule bundles), harper.js (WASM), LSP server.
- **Where it runs:** Chrome and Firefox extension, Obsidian, VS Code, Neovim/Helix/Emacs/Zed (LSP), WordPress, Desktop (Tauri overlay, macOS and early Windows).
- **License:** Apache-2.0.

## What Grammarly does (official features page)
- **Correctness, clarity, tone** suggestions, plus whole-paragraph rewrites.
- **AI agents:** paraphraser, proofreader, reader reactions, AI grader, citation finder.
- **Generative AI:** prompts, chat, humanizer, AI detector, "Go" assistant across tabs and apps.
- **Plagiarism checker** and citation tools.
- **20+ languages**, inline translation.
- **Everywhere:** browsers (including Safari), Windows and Mac apps, iOS and Android keyboard, Office add-ins, Google Docs, 1M+ apps and sites.
- **Teams:** style guides, brand tones, knowledge share, snippets, analytics dashboards.
- **Free utilities:** word, character and sentence counters, tone detector, passive-voice checker.

## Gap table
| Feature | Harper today | Verdict for Gramflow |
|---|---|---|
| Spelling, grammar, punctuation | Strong, fast, offline | **Keep.** This is the moat |
| Clarity (wordiness, filler, hedging, redundancy, long sentences) | Partial (filler, hedging, redundancy rules, `long_sentences`) | **Expand** with more Weir rules |
| Passive voice | Some passive-related code, no clear user-facing check | **Add** as a toggle |
| Overall writing score | None | **Add** (derive from lint density per 100 words) |
| Readability stats (grade level, reading time) | `Readability` kind exists, no metrics | **Add** (Flesch-Kincaid is pure math) |
| Goals (audience, formality, domain) | None | **Add** as presets that switch rule sets |
| Explain *why* | Each lint has a message | **Surface** it in the popup |
| Fix all, per-category accept | Per-lint only | **Add** |
| Style guides and brand rules | Weirpacks exist in Desktop | **Rename and polish** as Style Guides. Free parity with Grammarly Business |
| Snippets | None | **Add** (local text expansion) |
| Personal insights | `harper-stats` logs locally | **Add** weekly local report. No cloud |
| Dictionary backup and sync | Local file | **Add** export/import, optional folder sync |
| Tone detection and rewrite | None | **Optional AI layer** (below) |
| Paraphrase, shorten, change tone | None | **Optional AI layer** |
| Multiple languages | English only | **Later.** Spell-check-only for other languages first |
| Plagiarism, AI detector, citations | None | **Skip.** Needs server-side corpora, and AI detectors are unreliable |
| Mobile keyboard | None | **Out of scope for v1** |
| Word, Outlook, Google Docs | None | Office add-in and Docs handling (see platform plan) |
| Safari | None | Safari Web Extension |

## The one big decision: AI
Harper has no machine-learning rewrites. Grammarly's headline features (tone, rewrite, paraphrase) need a language model. Options:
1. **Rule-based only.** Fastest, fully private, matches Harper's identity. No rewrites.
2. **Optional bring-your-own model (recommended).** Off by default. The user points Gramflow at a local model (Ollama or similar) or their own API key. Text only leaves the device when they choose it, and the UI says so. Rules stay the fast first pass.
3. **Hosted Gramflow AI.** Needs accounts, servers, billing and a privacy policy. Not recommended for a family-and-friends release.

## Strengths to keep and advertise
Offline, instant, open source, no account, no subscription, text never leaves the device by default.

## Suggested build order
1. Score, readability panel, explanations in the popup, Fix all.
2. Goals presets and Style Guides (Weirpacks) UI.
3. Passive voice, wordiness and inclusive-language rule packs.
4. Snippets, dictionary export, local insights report.
5. Optional AI layer (tone, rewrite, paraphrase).
6. Word, Outlook, Safari and Google Docs support.

## Not verified
- Exact Grammarly free-tier limits and prices (sources conflict).
- Which Harper rules overlap with the proposed packs. Check with `harper-cli lint` before writing any new rule, per `AGENTS.md`.
