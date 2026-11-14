---
name: gramflow-rules
description: How to find gaps in Gramflow's grammar and spelling checks, write or fix a rule safely, and prove the change helped. Use when adding or changing grammar, spelling or style rules, reviewing what the checker misses, or after merging Harper updates.
---

# Improving Gramflow's rules

Gramflow's checking engine is `harper-core` (Rust). Rules are either **Rust linters** or **Weir files**
(`.weir`, plain-text pattern rules). Prefer Weir for phrase and word-choice rules and Rust for anything
that needs grammar structure. Read `AGENTS.md` ("On Writing New Rules") and
`packages/web/src/routes/docs/weir/+page.md` before writing one.

## 1. Measure first
Run the scorecard before touching anything:

```bash
python3 scripts/editor-review/scorecard.py --failures --save /tmp/before.json
```

It scores `scripts/editor-review/gold/*.jsonl`: sentences with the exact mistake and the right fix. It
reports, per category, how many mistakes were **caught**, how often the **first suggestion** was right,
and how many **false alarms** fired on correct writing. A change is only an improvement if caught or
first-suggestion goes up and false alarms do not.

## 2. Add the evidence
Before writing a rule, add sentences that show the gap to a gold file (one JSON object per line):

```json
{"category": "confused", "text": "I would rather go then stay home.", "errors": [{"text": "then", "fix": "than"}]}
```

Add correct sentences that look similar (`"Go then stay"` is fine), so a rule that is too eager fails.
Keep gold sentences natural, the way people type in email and chat.

## 3. Write the rule
- Put Weir rules in `harper-core/src/linting/weir_rules/` (a folder groups related rules under one name).
- Match the narrowest pattern that is still correct. A missed edge case is cheaper than a false alarm.
- Message: short, plain, non-judgmental. Say what to change, not what the writer did wrong.
- Include at least 15 tests: true positives, true negatives, near-misses, casing, punctuation, spacing.
- Enable the rule in `harper-core/default_config.json` unless it is a matter of taste (see
  `scripts/editor-review/DECISIONS.md`).
- Check the engine does not already cover it: `cargo run --release --bin harper-cli -- lint "your sentence"`.

## 4. Prove it
```bash
cargo test -p harper-core          # nothing regressed
python3 scripts/editor-review/scorecard.py --failures --compare /tmp/before.json
```
If an existing snapshot changed, read each diff. Update snapshots only when the new output is better.

## 5. Record decisions
Debatable usage (split infinitives, singular "they", Oxford comma) goes in
`scripts/editor-review/DECISIONS.md` with the reason, so it is not reopened.

## Known gaps (from the first scorecard)
- "Your welcome" is not caught (your/you're before a verb phrase).
- "go then stay" (then/than after "rather") is not caught.
- "There is three reasons" (existential there + plural) is not caught.
- "He seen the accident" (past participle used as past tense) is not caught.
- A lowercase first word ("my brother lives...") is flagged as a misspelling or not at all instead of a
  capitalization fix.
- "righ to ignore" suggests "righto" (MergeWords) instead of "right" (SpellCheck).
