use super::{Lint, LintKind, Linter, Suggestion};
use crate::Document;
use crate::char_string::CharStringExt;

const DETERMINERS: &[&str] = &[
    "the", "these", "those", "my", "our", "your", "his", "her", "their", "some", "many", "few",
    "several", "all", "both",
];

/// A plural noun inside a prepositional phrase ("the pattern on their backs was") is not the subject.
const PREPOSITIONS: &[&str] = &[
    "of", "on", "in", "at", "by", "for", "with", "from", "to", "about", "under", "over", "near",
    "behind", "beside", "between", "among", "through", "across", "along", "inside", "outside",
    "into", "onto", "upon", "within", "without", "against", "around", "above", "below", "toward",
];

/// Nouns that look plural but take a singular verb.
const SINGULAR_LOOKING_PLURALS: &[&str] = &[
    "news",
    "means",
    "series",
    "species",
    "measles",
    "mumps",
    "crossroads",
    "headquarters",
    "barracks",
    "works",
    "gallows",
    "innings",
    "billiards",
    "darts",
    "checkers",
    "dominoes",
    "summons",
    "lens",
    "bus",
    "gas",
    "class",
    "glass",
    "grass",
    "boss",
    "business",
    "process",
];

/// Finds a plural noun directly followed by a singular `to be` ("the books is on the table").
#[derive(Default)]
pub struct PluralNounSingularBe;

impl Linter for PluralNounSingularBe {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let tokens = document.get_tokens();

        for (index, window) in tokens.windows(5).enumerate() {
            let [det, space_a, noun, space_b, be] = window else {
                continue;
            };

            // "Neither of the options was" and "One of my friends is" take a singular verb.
            let after_of = tokens[..index]
                .iter()
                .rev()
                .find(|t| !t.kind.is_whitespace())
                .is_some_and(|t| {
                    document
                        .get_span_content(&t.span)
                        .eq_any_ignore_ascii_case_str(PREPOSITIONS)
                });
            if after_of {
                continue;
            }

            if !det.kind.is_word()
                || !space_a.kind.is_whitespace()
                || !noun.kind.is_word()
                || !space_b.kind.is_whitespace()
                || !be.kind.is_word()
            {
                continue;
            }

            let det_chars = document.get_span_content(&det.span);
            let noun_chars = document.get_span_content(&noun.span);
            let be_chars = document.get_span_content(&be.span);

            if !det_chars.eq_any_ignore_ascii_case_str(DETERMINERS) {
                continue;
            }

            // Proper nouns such as "the Netherlands is" are singular in meaning.
            if noun_chars.first().is_none_or(|c| !c.is_lowercase()) {
                continue;
            }

            let lower: String = noun_chars.iter().collect::<String>().to_lowercase();
            if SINGULAR_LOOKING_PLURALS.contains(&lower.as_str()) || lower.ends_with("ics") {
                continue;
            }

            let Some(metadata) = noun.kind.as_word().and_then(|m| m.as_ref()) else {
                continue;
            };

            if !metadata.is_plural_noun() || metadata.is_singular_noun() {
                continue;
            }

            let replacement = if be_chars.eq_any_ignore_ascii_case_str(&["is"]) {
                "are"
            } else if be_chars.eq_any_ignore_ascii_case_str(&["was"]) {
                "were"
            } else {
                continue;
            };

            lints.push(Lint {
                span: be.span,
                lint_kind: LintKind::Agreement,
                suggestions: vec![Suggestion::replace_with_match_case(
                    replacement.chars().collect(),
                    be_chars,
                )],
                message: "A plural noun needs a plural verb.".to_owned(),
                ..Default::default()
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "A plural subject such as `the books` takes `are` or `were`, not `is` or `was`."
    }
}

#[cfg(test)]
mod tests {
    use super::PluralNounSingularBe;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fixes_books_is() {
        assert_suggestion_result(
            "The books is on the table.",
            PluralNounSingularBe,
            "The books are on the table.",
        );
    }

    #[test]
    fn fixes_cats_was() {
        assert_suggestion_result(
            "The cats was asleep.",
            PluralNounSingularBe,
            "The cats were asleep.",
        );
    }

    #[test]
    fn fixes_my_friends_is() {
        assert_suggestion_result(
            "My friends is coming over.",
            PluralNounSingularBe,
            "My friends are coming over.",
        );
    }

    #[test]
    fn fixes_these_apples_is() {
        assert_suggestion_result(
            "These apples is fresh.",
            PluralNounSingularBe,
            "These apples are fresh.",
        );
    }

    #[test]
    fn fixes_their_children_is() {
        assert_suggestion_result(
            "Their children is quiet.",
            PluralNounSingularBe,
            "Their children are quiet.",
        );
    }

    #[test]
    fn fixes_those_shoes_was() {
        assert_suggestion_result(
            "Those shoes was expensive.",
            PluralNounSingularBe,
            "Those shoes were expensive.",
        );
    }

    #[test]
    fn fixes_all_lowercase() {
        assert_suggestion_result(
            "the tickets is sold out.",
            PluralNounSingularBe,
            "the tickets are sold out.",
        );
    }

    #[test]
    fn fixes_our_neighbors_is() {
        assert_suggestion_result(
            "Our neighbors is nice.",
            PluralNounSingularBe,
            "Our neighbors are nice.",
        );
    }

    #[test]
    fn allows_neither_of_the_options_was() {
        assert_lint_count(
            "Neither of the options was perfect.",
            PluralNounSingularBe,
            0,
        );
    }

    #[test]
    fn allows_one_of_my_friends_is() {
        assert_lint_count("One of my friends is late.", PluralNounSingularBe, 0);
    }

    #[test]
    fn allows_plural_inside_prepositional_phrase() {
        assert_lint_count(
            "The pattern on their backs was the same.",
            PluralNounSingularBe,
            0,
        );
    }

    #[test]
    fn allows_correct_plural() {
        assert_lint_count("The books are on the table.", PluralNounSingularBe, 0);
    }

    #[test]
    fn allows_singular_noun() {
        assert_lint_count("The book is on the table.", PluralNounSingularBe, 0);
    }

    #[test]
    fn allows_news() {
        assert_lint_count("The news is good.", PluralNounSingularBe, 0);
    }

    #[test]
    fn allows_series() {
        assert_lint_count("The series is popular.", PluralNounSingularBe, 0);
    }

    #[test]
    fn allows_proper_noun() {
        assert_lint_count("The Netherlands is flat.", PluralNounSingularBe, 0);
    }

    #[test]
    fn allows_the_united_states() {
        assert_lint_count("The United States is large.", PluralNounSingularBe, 0);
    }

    #[test]
    fn allows_plural_with_adjective() {
        assert_lint_count("The red books are mine.", PluralNounSingularBe, 0);
    }

    #[test]
    fn allows_physics() {
        assert_lint_count("The physics is hard.", PluralNounSingularBe, 0);
    }

    #[test]
    fn allows_the_means() {
        assert_lint_count("The means is justified.", PluralNounSingularBe, 0);
    }
}
