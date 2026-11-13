# Language-intelligence roadmap (from the audit)

Findings from testing Gramflow against Grammarly-style categories. Items marked DONE shipped in 3.0.7.

## Done
- Spelling suggestion ranking (names, short typos, doubled-letter typos); modern vocabulary added.
- Context homophones: break/brake, herd/heard, plain/plane, site/cite, know/no, whether/weather, to/too, your/you're, its/it's.
- Eggcorns: for granted, shoo-in, toe the line, nip in the bud, scot-free, dog-eat-dog.
- Redundancy: 27 classic pairs (past history, free gift, reply back, end result, revert back, join together ...).
- Agreement: "a number of X is", "news were", "one of the X are", "each + plural", "he and I is", "Mathematics are".
- Verbs: "has already ate" (15 verbs), "was lead by", "I use to", "said me that"; "between you and I"; fewer/less.
- Natural phrasing: "according to me", "do the needful", "explain you".
- Fixed false alarm: "Whom did you invite?".

## Next (needs engine work, not just phrase rules)
1. Comma rules: missing comma after an introductory clause, comma splices, serial-comma consistency (style choice, see DECISIONS).
2. Passive voice and weak-verb detection (opt-in; upstream PR #4502 is in progress).
3. Dangling and misplaced modifiers ("Walking down the street, the trees were tall").
4. Parallelism in lists ("running, to swim, and bike").
5. Pronoun case in compounds ("Me and him went"): needs a clause-aware check.
6. Sentence fragments and run-ons.
7. Tone layer: formal/casual/hedging, filler words ("very", "really", "basically"), reading-level score.
8. Dialect awareness inside Weir rules (whilst, amongst, learnt flagged only for American).
9. Real benchmark data (JFLEG, BEA-2019) for a bigger scorecard; the current sets are hand-written.
