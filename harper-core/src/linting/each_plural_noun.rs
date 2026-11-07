use super::{Lint, LintKind, Linter, Suggestion};
use crate::Document;
use crate::char_string::CharStringExt;
use crate::spell::Dictionary;

const SKIP: &[&str] = &[
    "others", "one", "ones", "news", "series", "species", "means", "lens", "gas", "bus", "kudos",
    "pros", "cons", "thanks", "less",
];

/// Finds `each` followed directly by a plural noun ("each employees must sign").
pub struct EachPluralNoun<T: Dictionary> {
    dictionary: T,
}

impl<T: Dictionary> EachPluralNoun<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }
}

impl<T: Dictionary> Linter for EachPluralNoun<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for window in document.get_tokens().windows(3) {
            let [each, space, noun] = window else {
                continue;
            };

            if !each.kind.is_word() || !space.kind.is_whitespace() || !noun.kind.is_word() {
                continue;
            }

            let each_chars = document.get_span_content(&each.span);
            let noun_chars = document.get_span_content(&noun.span);

            if !each_chars.eq_any_ignore_ascii_case_str(&["each", "every"])
                || noun_chars.eq_any_ignore_ascii_case_str(SKIP)
                || noun_chars.len() < 4
                || !noun_chars.ends_with(&['s'])
                || noun_chars.ends_with(&['s', 's'])
            {
                continue;
            }

            let stem = &noun_chars[..noun_chars.len() - 1];

            let is_plural_of_noun = noun
                .kind
                .as_word()
                .and_then(|m| m.as_ref())
                .is_some_and(|m| m.is_plural_noun() && !m.is_singular_noun())
                && self
                    .dictionary
                    .get_word_metadata(stem)
                    .is_some_and(|m| m.is_singular_noun());

            if !is_plural_of_noun {
                continue;
            }

            lints.push(Lint {
                span: noun.span,
                lint_kind: LintKind::Agreement,
                suggestions: vec![Suggestion::ReplaceWith(stem.to_vec())],
                message: "After `each` or `every`, use a singular noun.".to_owned(),
                ..Default::default()
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "`Each` and `every` are followed by a singular noun, as in `each employee`."
    }
}

#[cfg(test)]
mod tests {
    use super::EachPluralNoun;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};
    use crate::spell::FstDictionary;

    fn linter() -> EachPluralNoun<std::sync::Arc<FstDictionary>> {
        EachPluralNoun::new(FstDictionary::curated())
    }

    #[test]
    fn fixes_each_employees() {
        assert_suggestion_result(
            "Each employees must sign.",
            linter(),
            "Each employee must sign.",
        );
    }

    #[test]
    fn fixes_every_students() {
        assert_suggestion_result("Every students passed.", linter(), "Every student passed.");
    }

    #[test]
    fn fixes_each_books() {
        assert_suggestion_result(
            "Each books has a cover.",
            linter(),
            "Each book has a cover.",
        );
    }

    #[test]
    fn fixes_lowercase() {
        assert_suggestion_result(
            "we reviewed each documents.",
            linter(),
            "we reviewed each document.",
        );
    }

    #[test]
    fn fixes_each_cars() {
        assert_suggestion_result("Each cars was checked.", linter(), "Each car was checked.");
    }

    #[test]
    fn fixes_every_customers() {
        assert_suggestion_result(
            "Every customers matters.",
            linter(),
            "Every customer matters.",
        );
    }

    #[test]
    fn allows_each_employee() {
        assert_lint_count("Each employee must sign.", linter(), 0);
    }

    #[test]
    fn allows_each_other() {
        assert_lint_count("They help each other.", linter(), 0);
    }

    #[test]
    fn allows_each_others() {
        assert_lint_count("They read each others letters.", linter(), 0);
    }

    #[test]
    fn allows_each_of_the() {
        assert_lint_count("Each of the employees signed.", linter(), 0);
    }

    #[test]
    fn allows_every_day() {
        assert_lint_count("I walk every day.", linter(), 0);
    }

    #[test]
    fn allows_each_one() {
        assert_lint_count("Each one counts.", linter(), 0);
    }

    #[test]
    fn allows_every_time() {
        assert_lint_count("Every time I try, it works.", linter(), 0);
    }

    #[test]
    fn allows_each_class() {
        assert_lint_count("Each class starts at nine.", linter(), 0);
    }

    #[test]
    fn allows_every_news() {
        assert_lint_count("Every news story matters.", linter(), 0);
    }

    #[test]
    fn allows_each_has() {
        assert_lint_count("Each has a pen.", linter(), 0);
    }
}
