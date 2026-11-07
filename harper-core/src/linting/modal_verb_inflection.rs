use super::{Lint, LintKind, Linter, Suggestion};
use crate::char_string::CharStringExt;
use crate::spell::Dictionary;
use crate::{Document, Span};

const MODALS: &[&str] = &[
    "can", "could", "will", "would", "should", "must", "may", "might", "shall",
];

const SUBJECTS: &[&str] = &[
    "i", "you", "he", "she", "it", "we", "they", "who", "that", "this", "there",
];

/// Finds a modal verb followed by a verb carrying a third-person `-s` ending ("she can sings").
pub struct ModalVerbInflection<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> ModalVerbInflection<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    fn base_form(&self, word: &[char]) -> Option<Vec<char>> {
        if word.len() < 3 || !word.ends_with(&['s']) {
            return None;
        }

        let mut candidates = vec![word[..word.len() - 1].to_vec()];
        if word.ends_with(&['e', 's']) {
            candidates.push(word[..word.len() - 2].to_vec());
        }
        if word.ends_with(&['i', 'e', 's']) {
            let mut stem = word[..word.len() - 3].to_vec();
            stem.push('y');
            candidates.push(stem);
        }

        candidates.into_iter().find(|stem| {
            self.dictionary
                .get_word_metadata(stem)
                .is_some_and(|m| m.is_verb())
        })
    }
}

impl<T: Dictionary> Linter for ModalVerbInflection<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let tokens = document.get_tokens();

        for i in 2..tokens.len().saturating_sub(2) {
            let (subject, space_a, modal, space_b, verb) = match tokens.get(i - 2..=i + 2) {
                Some([a, b, c, d, e]) => (a, b, c, d, e),
                _ => continue,
            };

            if !subject.kind.is_word()
                || !space_a.kind.is_whitespace()
                || !modal.kind.is_word()
                || !space_b.kind.is_whitespace()
                || !verb.kind.is_word()
            {
                continue;
            }

            let subject_chars = document.get_span_content(&subject.span);
            let modal_chars = document.get_span_content(&modal.span);
            let verb_chars = document.get_span_content(&verb.span);

            if !subject_chars.eq_any_ignore_ascii_case_str(SUBJECTS)
                || !modal_chars.eq_any_ignore_ascii_case_str(MODALS)
            {
                continue;
            }

            // "must needs" is an old but correct idiom.
            if verb_chars.eq_any_ignore_ascii_case_str(&["needs"]) {
                continue;
            }

            // Already a real word that is not just a verb with an -s ending (such as "news").
            let Some(base) = self.base_form(verb_chars) else {
                continue;
            };

            lints.push(Lint {
                span: Span::new(verb.span.start, verb.span.end),
                lint_kind: LintKind::Grammar,
                suggestions: vec![Suggestion::ReplaceWith(base)],
                message:
                    "After a modal verb such as `can` or `will`, use the base form of the verb."
                        .to_owned(),
                ..Default::default()
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "Modal verbs such as `can`, `will` and `should` are followed by the base form of a verb, never one ending in `-s`."
    }
}

#[cfg(test)]
mod tests {
    use super::ModalVerbInflection;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};
    use crate::spell::FstDictionary;

    fn linter() -> ModalVerbInflection<std::sync::Arc<FstDictionary>> {
        ModalVerbInflection::new(FstDictionary::curated())
    }

    #[test]
    fn fixes_can_sings() {
        assert_suggestion_result(
            "She can sings very well.",
            linter(),
            "She can sing very well.",
        );
    }

    #[test]
    fn fixes_will_goes() {
        assert_suggestion_result("He will goes home soon.", linter(), "He will go home soon.");
    }

    #[test]
    fn fixes_should_takes() {
        assert_suggestion_result(
            "It should takes a minute.",
            linter(),
            "It should take a minute.",
        );
    }

    #[test]
    fn fixes_must_finishes() {
        assert_suggestion_result(
            "She must finishes today.",
            linter(),
            "She must finish today.",
        );
    }

    #[test]
    fn fixes_could_tries() {
        assert_suggestion_result("He could tries again.", linter(), "He could try again.");
    }

    #[test]
    fn fixes_would_wants() {
        assert_suggestion_result(
            "They would wants more time.",
            linter(),
            "They would want more time.",
        );
    }

    #[test]
    fn fixes_might_comes() {
        assert_suggestion_result("She might comes later.", linter(), "She might come later.");
    }

    #[test]
    fn fixes_lowercase_subject() {
        assert_suggestion_result("it can works fine.", linter(), "it can work fine.");
    }

    #[test]
    fn allows_must_needs() {
        assert_lint_count("They must needs come back.", linter(), 0);
    }

    #[test]
    fn allows_base_form() {
        assert_lint_count("She can sing very well.", linter(), 0);
    }

    #[test]
    fn allows_question_with_plural_noun() {
        assert_lint_count("Can dogs swim?", linter(), 0);
    }

    #[test]
    fn allows_noun_after_modal_subject_phrase() {
        assert_lint_count("It will news be good.", linter(), 0);
    }

    #[test]
    fn allows_third_person_without_modal() {
        assert_lint_count("She sings very well.", linter(), 0);
    }

    #[test]
    fn allows_modal_with_adverb() {
        assert_lint_count("She can easily sing.", linter(), 0);
    }

    #[test]
    fn allows_will_as_noun() {
        assert_lint_count("It was his will to leave.", linter(), 0);
    }

    #[test]
    fn allows_that_can_be_used() {
        assert_lint_count("That can be used later.", linter(), 0);
    }

    #[test]
    fn allows_may_as_month() {
        assert_lint_count("It may rain in May.", linter(), 0);
    }

    #[test]
    fn allows_it_must_be() {
        assert_lint_count("It must be true.", linter(), 0);
    }
}
