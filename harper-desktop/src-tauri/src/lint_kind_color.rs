use harper_core::linting::LintKind;

use crate::color::Color;

pub const LINT_KINDS: [LintKind; 20] = [
    LintKind::Agreement,
    LintKind::BoundaryError,
    LintKind::Capitalization,
    LintKind::Eggcorn,
    LintKind::Enhancement,
    LintKind::Formatting,
    LintKind::Grammar,
    LintKind::Malapropism,
    LintKind::Miscellaneous,
    LintKind::Nonstandard,
    LintKind::Punctuation,
    LintKind::Readability,
    LintKind::Redundancy,
    LintKind::Regionalism,
    LintKind::Repetition,
    LintKind::Spelling,
    LintKind::Style,
    LintKind::Typo,
    LintKind::Usage,
    LintKind::WordChoice,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextColor {
    Black,
    White,
}

/// Gramflow's four issue categories. Tuned to stay visible over both light and dark app
/// backgrounds, since the overlay draws on top of other apps.
const SPELLING: Color = Color {
    r: 0xE5,
    g: 0x53,
    b: 0x2D,
};
const GRAMMAR: Color = Color {
    r: 0x3D,
    g: 0x8B,
    b: 0xFF,
};
const STYLE: Color = Color {
    r: 0x8F,
    g: 0x63,
    b: 0xF0,
};
const CLARITY: Color = Color {
    r: 0xE6,
    g: 0x9A,
    b: 0x0B,
};

pub fn lint_kind_color(lint_kind: LintKind) -> Color {
    match lint_kind {
        // Spelling: coral
        LintKind::Spelling | LintKind::Typo | LintKind::Eggcorn | LintKind::Malapropism => SPELLING,
        // Grammar: sky
        LintKind::Agreement
        | LintKind::BoundaryError
        | LintKind::Capitalization
        | LintKind::Formatting
        | LintKind::Grammar
        | LintKind::Nonstandard
        | LintKind::Punctuation
        | LintKind::Usage
        | LintKind::WordOrder => GRAMMAR,
        // Clarity: amber
        LintKind::Readability
        | LintKind::Redundancy
        | LintKind::Repetition
        | LintKind::WordChoice => CLARITY,
        // Style: violet
        LintKind::Enhancement
        | LintKind::Miscellaneous
        | LintKind::Regionalism
        | LintKind::Style => STYLE,
    }
}

pub fn lint_kind_color_from_key(lint_kind_key: &str) -> Option<Color> {
    LintKind::from_string_key(lint_kind_key).map(lint_kind_color)
}

pub fn lint_kind_text_color(lint_kind: LintKind) -> TextColor {
    contrasting_text_color(lint_kind_color(lint_kind))
}

pub fn contrasting_text_color(color: Color) -> TextColor {
    let luminance =
        0.299 * f64::from(color.r) + 0.587 * f64::from(color.g) + 0.114 * f64::from(color.b);

    if luminance > 186.0 {
        TextColor::Black
    } else {
        TextColor::White
    }
}
