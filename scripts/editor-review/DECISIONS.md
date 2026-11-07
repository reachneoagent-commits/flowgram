# Style decisions

Rules we leave on, off or unchecked on purpose, so reviews do not reopen them every time.
Add a line whenever a rule is switched or a debated usage is decided.

| Topic | Decision | Why |
|---|---|---|
| Serial (Oxford) comma | Flag its absence only in the "Style" group, off by default | Chicago wants it, AP drops it; neither is an error |
| Spelling dialect | American by default, user can change it in Settings | British forms like "neighbour" are flagged under American English |
| Split infinitives, ending a sentence with a preposition | Do not flag | Long accepted by Garner, Chicago and Merriam-Webster |
| Singular "they" | Do not flag | Accepted by AP, Chicago and Merriam-Webster |
| "which" vs "that" (restrictive) | Do not flag by default | Real usage varies; high false-alarm risk |
| "AvoidContractions", "BoringWords", "SpelledNumbers" | Off by default, optional | Matters of taste, not errors |
| Passive voice | Flag only when a rule can tell it hurts clarity | A blanket flag mostly produces false alarms |

## Engine changes after the first scorecard (3.0.6 work)

- A lone short line ending in `.`, `!` or `?` is a sentence and needs a capital letter. Lines ending in `;`, `:` or nothing stay treated as labels or list items.
- `SpellCheck` puts the properly capitalized entry first for a lowercase name (`paris` becomes `Paris`) only when the lowercase form is not a word in any dialect and the word has 4+ letters. Short words such as `si` stay on the typo path.
- `MergeWords` never merges a misspelling with a small function word (`righ to` stays a typo for `right`).
- `WrongNegative` ignores doubled-letter typos (`untill` is `until`, not `distill`).
- New checks: `ModalVerbInflection` (`can sings`), `PluralNounSingularBe` (`the books is`, skips `of`/preposition phrases and singular-looking plurals such as `news`), `SubjectSeen` (`he seen`, skips `have you seen`), `EachOfHave`, `EveryoneHave` (only with perfect-tense or `to` follow-ups, so `Let everyone have a say` is safe), `BeGoing`, `ToBuy`, `AcceptFor`, `WontContraction`, `ThereIsPluralNumber`, `ThereWasPluralNumber`, `YourWelcomeTo`, `RatherThen`, `LookForwardToGerund`.
- Deliberately not added: `The reason is because` (debated style), `principle`/`principal` (needs meaning), `french`/`google` lowercase (valid lowercase words), `I is` suggestion (needs a pronoun-aware fix).
