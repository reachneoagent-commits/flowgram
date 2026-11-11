//! Compares the numbers in a draft with the numbers in its rewrite, so changed facts get flagged.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct NumberCheck {
    /// Numbers in the draft that the rewrite dropped.
    pub missing: Vec<String>,
    /// Numbers in the rewrite that the draft didn't have.
    pub added: Vec<String>,
}

impl NumberCheck {
    pub fn is_clean(&self) -> bool {
        self.missing.is_empty() && self.added.is_empty()
    }
}

/// Pulls out runs of Arabic digits, keeping internal `,` and `.` so "1,084" and "3.5" stay whole.
fn numbers(text: &str) -> BTreeMap<String, usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = BTreeMap::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || ((chars[i] == ',' || chars[i] == '.')
                        && chars.get(i + 1).is_some_and(|c| c.is_ascii_digit())))
            {
                i += 1;
            }
            // Thousands separators don't change the value, so compare without them.
            let number: String = chars[start..i].iter().filter(|c| **c != ',').collect();
            *found.entry(number).or_insert(0) += 1;
        } else {
            i += 1;
        }
    }
    found
}

pub fn check(draft: &str, rewrite: &str) -> NumberCheck {
    let before = numbers(draft);
    let after = numbers(rewrite);

    let diff = |from: &BTreeMap<String, usize>, against: &BTreeMap<String, usize>| {
        from.iter()
            .filter(|(number, count)| against.get(*number).copied().unwrap_or(0) < **count)
            .map(|(number, _)| number.clone())
            .collect()
    };

    NumberCheck {
        missing: diff(&before, &after),
        added: diff(&after, &before),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_numbers_are_clean() {
        assert!(
            check(
                "Budget is $180K by March 3.",
                "By March 3 the budget is $180K."
            )
            .is_clean()
        );
    }

    #[test]
    fn a_dropped_number_is_reported() {
        let result = check(
            "We hired 32 students in 2024.",
            "We hired students in 2024.",
        );
        assert_eq!(result.missing, vec!["32"]);
        assert!(result.added.is_empty());
    }

    #[test]
    fn an_invented_number_is_reported() {
        let result = check("We have six laptops.", "We have 6 laptops.");
        assert_eq!(result.added, vec!["6"]);
    }

    #[test]
    fn a_changed_number_is_both_missing_and_added() {
        let result = check("Deadline is March 3.", "Deadline is March 5.");
        assert_eq!(result.missing, vec!["3"]);
        assert_eq!(result.added, vec!["5"]);
    }

    #[test]
    fn thousands_separators_do_not_matter() {
        assert!(check("1,084 chars", "1084 chars").is_clean());
    }

    #[test]
    fn decimals_stay_whole() {
        assert!(check("It grew 3.5 percent.", "Growth was 3.5 percent.").is_clean());
        assert!(!check("It grew 3.5 percent.", "It grew 35 percent.").is_clean());
    }

    #[test]
    fn a_trailing_period_is_not_part_of_the_number() {
        assert!(check("Born in 1990.", "I was born in 1990, a long time ago.").is_clean());
    }

    #[test]
    fn repeated_numbers_are_counted() {
        let result = check("2 and 2", "just 2");
        assert_eq!(result.missing, vec!["2"]);
    }

    #[test]
    fn text_without_numbers_is_clean() {
        assert!(check("No digits here.", "Still none.").is_clean());
    }
}
