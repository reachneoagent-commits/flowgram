use super::{Lint, LintKind, Linter, Suggestion};
use crate::Document;
use crate::char_string::CharStringExt;

const SUBJECTS: &[&str] = &["i", "you", "he", "she", "we", "they"];

const OBJECT_STARTERS: &[&str] = &[
    "the", "a", "an", "it", "that", "this", "him", "her", "them", "me", "us", "what", "who", "my",
    "his", "their", "our", "your",
];

/// Words that make `subject + seen` a legitimate perfect form ("have you seen").
const HELPERS: &[&str] = &[
    "have",
    "has",
    "had",
    "having",
    "haven't",
    "hasn't",
    "hadn't",
    "ve",
    "d",
    "s",
    "never",
    "ever",
    "not",
    "n't",
    "be",
    "been",
    "being",
    "is",
    "are",
    "was",
    "were",
    "get",
    "got",
    "gets",
    "getting",
    "if",
    "when",
    "that",
    "where",
    "which",
    "whom",
    "than",
    "as",
    "thing",
    "things",
    "all",
    "what",
    "who",
    "everything",
    "anything",
    "nothing",
    "something",
    "one",
    "ones",
];

/// Finds `he seen the accident`, where the past participle stands in for `saw`.
#[derive(Default)]
pub struct SubjectSeen;

impl Linter for SubjectSeen {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let tokens = document.get_tokens();

        for (i, window) in tokens.windows(5).enumerate() {
            let [subject, space_a, seen, space_b, object] = window else {
                continue;
            };

            if !subject.kind.is_word()
                || !space_a.kind.is_whitespace()
                || !seen.kind.is_word()
                || !space_b.kind.is_whitespace()
                || !object.kind.is_word()
            {
                continue;
            }

            let subject_chars = document.get_span_content(&subject.span);
            let seen_chars = document.get_span_content(&seen.span);
            let object_chars = document.get_span_content(&object.span);

            if !seen_chars.eq_any_ignore_ascii_case_str(&["seen"])
                || !subject_chars.eq_any_ignore_ascii_case_str(SUBJECTS)
                || !object_chars.eq_any_ignore_ascii_case_str(OBJECT_STARTERS)
            {
                continue;
            }

            // "Have you seen the news?", "the film I seen" and similar are not this mistake.
            let previous = tokens[..i]
                .iter()
                .rev()
                .find(|t| !t.kind.is_whitespace())
                .filter(|t| t.kind.is_word() || t.kind.is_apostrophe());
            if previous.is_some_and(|t| {
                document
                    .get_span_content(&t.span)
                    .eq_any_ignore_ascii_case_str(HELPERS)
            }) {
                continue;
            }

            lints.push(Lint {
                span: seen.span,
                lint_kind: LintKind::Grammar,
                suggestions: vec![Suggestion::replace_with_match_case(
                    "saw".chars().collect(),
                    seen_chars,
                )],
                message: "Use the simple past `saw` here, or add `have` or `has`.".to_owned(),
                ..Default::default()
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "`Seen` is a past participle and needs a helper such as `have`. For a plain past event, use `saw`."
    }
}

#[cfg(test)]
mod tests {
    use super::SubjectSeen;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fixes_he_seen() {
        assert_suggestion_result("He seen the accident.", SubjectSeen, "He saw the accident.");
    }

    #[test]
    fn fixes_she_seen() {
        assert_suggestion_result("She seen a ghost.", SubjectSeen, "She saw a ghost.");
    }

    #[test]
    fn fixes_they_seen() {
        assert_suggestion_result("They seen it happen.", SubjectSeen, "They saw it happen.");
    }

    #[test]
    fn fixes_we_seen() {
        assert_suggestion_result("We seen that movie.", SubjectSeen, "We saw that movie.");
    }

    #[test]
    fn fixes_i_seen() {
        assert_suggestion_result("I seen him yesterday.", SubjectSeen, "I saw him yesterday.");
    }

    #[test]
    fn fixes_you_seen() {
        assert_suggestion_result("You seen the news.", SubjectSeen, "You saw the news.");
    }

    #[test]
    fn fixes_lowercase() {
        assert_suggestion_result(
            "he seen her at the store.",
            SubjectSeen,
            "he saw her at the store.",
        );
    }

    #[test]
    fn fixes_after_comma() {
        assert_suggestion_result(
            "Yesterday, I seen my teacher downtown.",
            SubjectSeen,
            "Yesterday, I saw my teacher downtown.",
        );
    }

    #[test]
    fn allows_have_you_seen() {
        assert_lint_count("Have you seen the Mock Turtle yet?", SubjectSeen, 0);
    }

    #[test]
    fn allows_has_he_seen() {
        assert_lint_count("Has he seen the report?", SubjectSeen, 0);
    }

    #[test]
    fn allows_have_seen() {
        assert_lint_count("I have seen that movie.", SubjectSeen, 0);
    }

    #[test]
    fn allows_had_seen() {
        assert_lint_count("They had seen it before.", SubjectSeen, 0);
    }

    #[test]
    fn allows_saw() {
        assert_lint_count("He saw the accident.", SubjectSeen, 0);
    }

    #[test]
    fn allows_passive() {
        assert_lint_count("She is seen as a leader.", SubjectSeen, 0);
    }

    #[test]
    fn allows_relative_clause() {
        assert_lint_count(
            "The best film I ever seen the end of is old.",
            SubjectSeen,
            0,
        );
    }

    #[test]
    fn allows_having_seen() {
        assert_lint_count("Having seen the results, we agreed.", SubjectSeen, 0);
    }
}
