/**
 * Shared display configuration and helpers for the Gramflow editor UI.
 *
 * This module keeps editor-only presentation choices, such as lint-kind styles,
 * lint-kind ordering, font options, and small formatting helpers, out of Svelte
 * components. Canonical linting concepts still come from `harper.js` and
 * `lint-framework`; this file only describes how those values should appear in
 * the editor.
 */
import type { LintKind } from 'harper.js';
import type { UnpackedSuggestion } from './types.js';

export type EditorFontFamily = 'sans' | 'serif' | 'mono';

export type EditorFontSize = 'default' | number;

export type LintKindCounts = Record<LintKind, number>;

export const FONT_OPTIONS: {
	value: EditorFontFamily;
	label: string;
	sample: string;
	stack: string;
}[] = [
	{
		value: 'sans',
		label: 'Sans',
		sample: 'Aa',
		stack:
			"'Inter', -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif",
	},
	{
		value: 'serif',
		label: 'Serif',
		sample: 'Aa',
		stack: "'Iowan Old Style', 'New York', Georgia, serif",
	},
	{
		value: 'mono',
		label: 'Mono',
		sample: 'Aa',
		stack: "'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace",
	},
];

export const FONT_SIZES = [12, 13, 14, 15, 16, 17, 18, 20, 22];
export const DEFAULT_FONT_SIZE = 'default';

export const LINT_KIND_ORDER: LintKind[] = [
	'Spelling',
	'Typo',
	'Capitalization',
	'Grammar',
	'Agreement',
	'BoundaryError',
	'Punctuation',
	'Usage',
	'WordChoice',
	'WordOrder',
	'Style',
	'Readability',
	'Enhancement',
	'Redundancy',
	'Repetition',
	'Formatting',
	'Regionalism',
	'Nonstandard',
	'Eggcorn',
	'Malapropism',
	'Miscellaneous',
];

/**
 * Presentation metadata for each lint kind, mapped onto Gramflow's four categories
 * (spelling, grammar, clarity, style). Colours come from the shared `cat-*` tokens.
 *
 * Keeping these classes here lets the editor share one visual language across
 * cards, counts, and active states without spreading Tailwind class maps through
 * Svelte components. Every class string except `dotClass` also carries its
 * `dark:` variant, so the palette follows a `.dark` ancestor without extra
 * logic in components.
 */
export const LINT_KIND_STYLES: Record<
	LintKind,
	{
		label: string;
		dotClass: string;
		haloClass: string;
		textClass: string;
		softClass: string;
		activeClass: string;
	}
> = {
	Agreement: {
		label: 'Agreement',
		dotClass: 'bg-cat-grammar',
		haloClass: 'bg-cat-grammar/20',
		textClass: 'text-cat-grammar',
		softClass: 'bg-cat-grammar/12',
		activeClass: 'border-cat-grammar/70 shadow-none',
	},
	BoundaryError: {
		label: 'Boundary',
		dotClass: 'bg-cat-grammar',
		haloClass: 'bg-cat-grammar/20',
		textClass: 'text-cat-grammar',
		softClass: 'bg-cat-grammar/12',
		activeClass: 'border-cat-grammar/70 shadow-none',
	},
	Capitalization: {
		label: 'Capitalization',
		dotClass: 'bg-cat-grammar',
		haloClass: 'bg-cat-grammar/20',
		textClass: 'text-cat-grammar',
		softClass: 'bg-cat-grammar/12',
		activeClass: 'border-cat-grammar/70 shadow-none',
	},
	Eggcorn: {
		label: 'Eggcorn',
		dotClass: 'bg-cat-spelling',
		haloClass: 'bg-cat-spelling/20',
		textClass: 'text-cat-spelling',
		softClass: 'bg-cat-spelling/12',
		activeClass: 'border-cat-spelling/70 shadow-none',
	},
	Enhancement: {
		label: 'Enhancement',
		dotClass: 'bg-cat-style',
		haloClass: 'bg-cat-style/20',
		textClass: 'text-cat-style',
		softClass: 'bg-cat-style/12',
		activeClass: 'border-cat-style/70 shadow-none',
	},
	Formatting: {
		label: 'Formatting',
		dotClass: 'bg-cat-grammar',
		haloClass: 'bg-cat-grammar/20',
		textClass: 'text-cat-grammar',
		softClass: 'bg-cat-grammar/12',
		activeClass: 'border-cat-grammar/70 shadow-none',
	},
	Grammar: {
		label: 'Grammar',
		dotClass: 'bg-cat-grammar',
		haloClass: 'bg-cat-grammar/20',
		textClass: 'text-cat-grammar',
		softClass: 'bg-cat-grammar/12',
		activeClass: 'border-cat-grammar/70 shadow-none',
	},
	Malapropism: {
		label: 'Malapropism',
		dotClass: 'bg-cat-spelling',
		haloClass: 'bg-cat-spelling/20',
		textClass: 'text-cat-spelling',
		softClass: 'bg-cat-spelling/12',
		activeClass: 'border-cat-spelling/70 shadow-none',
	},
	Miscellaneous: {
		label: 'Miscellaneous',
		dotClass: 'bg-cat-style',
		haloClass: 'bg-cat-style/20',
		textClass: 'text-cat-style',
		softClass: 'bg-cat-style/12',
		activeClass: 'border-cat-style/70 shadow-none',
	},
	Nonstandard: {
		label: 'Nonstandard',
		dotClass: 'bg-cat-grammar',
		haloClass: 'bg-cat-grammar/20',
		textClass: 'text-cat-grammar',
		softClass: 'bg-cat-grammar/12',
		activeClass: 'border-cat-grammar/70 shadow-none',
	},
	Punctuation: {
		label: 'Punctuation',
		dotClass: 'bg-cat-grammar',
		haloClass: 'bg-cat-grammar/20',
		textClass: 'text-cat-grammar',
		softClass: 'bg-cat-grammar/12',
		activeClass: 'border-cat-grammar/70 shadow-none',
	},
	Readability: {
		label: 'Readability',
		dotClass: 'bg-cat-clarity',
		haloClass: 'bg-cat-clarity/20',
		textClass: 'text-cat-clarity',
		softClass: 'bg-cat-clarity/12',
		activeClass: 'border-cat-clarity/70 shadow-none',
	},
	Redundancy: {
		label: 'Redundancy',
		dotClass: 'bg-cat-clarity',
		haloClass: 'bg-cat-clarity/20',
		textClass: 'text-cat-clarity',
		softClass: 'bg-cat-clarity/12',
		activeClass: 'border-cat-clarity/70 shadow-none',
	},
	Regionalism: {
		label: 'Regionalism',
		dotClass: 'bg-cat-style',
		haloClass: 'bg-cat-style/20',
		textClass: 'text-cat-style',
		softClass: 'bg-cat-style/12',
		activeClass: 'border-cat-style/70 shadow-none',
	},
	Repetition: {
		label: 'Repetition',
		dotClass: 'bg-cat-clarity',
		haloClass: 'bg-cat-clarity/20',
		textClass: 'text-cat-clarity',
		softClass: 'bg-cat-clarity/12',
		activeClass: 'border-cat-clarity/70 shadow-none',
	},
	Spelling: {
		label: 'Spelling',
		dotClass: 'bg-cat-spelling',
		haloClass: 'bg-cat-spelling/20',
		textClass: 'text-cat-spelling',
		softClass: 'bg-cat-spelling/12',
		activeClass: 'border-cat-spelling/70 shadow-none',
	},
	Style: {
		label: 'Style',
		dotClass: 'bg-cat-style',
		haloClass: 'bg-cat-style/20',
		textClass: 'text-cat-style',
		softClass: 'bg-cat-style/12',
		activeClass: 'border-cat-style/70 shadow-none',
	},
	Typo: {
		label: 'Typo',
		dotClass: 'bg-cat-spelling',
		haloClass: 'bg-cat-spelling/20',
		textClass: 'text-cat-spelling',
		softClass: 'bg-cat-spelling/12',
		activeClass: 'border-cat-spelling/70 shadow-none',
	},
	Usage: {
		label: 'Usage',
		dotClass: 'bg-cat-grammar',
		haloClass: 'bg-cat-grammar/20',
		textClass: 'text-cat-grammar',
		softClass: 'bg-cat-grammar/12',
		activeClass: 'border-cat-grammar/70 shadow-none',
	},
	WordChoice: {
		label: 'Word Choice',
		dotClass: 'bg-cat-clarity',
		haloClass: 'bg-cat-clarity/20',
		textClass: 'text-cat-clarity',
		softClass: 'bg-cat-clarity/12',
		activeClass: 'border-cat-clarity/70 shadow-none',
	},
	WordOrder: {
		label: 'Word Order',
		dotClass: 'bg-cat-grammar',
		haloClass: 'bg-cat-grammar/20',
		textClass: 'text-cat-grammar',
		softClass: 'bg-cat-grammar/12',
		activeClass: 'border-cat-grammar/70 shadow-none',
	},
};

export const LINT_KIND_STYLE_ENTRIES = LINT_KIND_ORDER.map(
	(key) => [key, LINT_KIND_STYLES[key]] as const,
);

/**
 * Create a zero-filled count map for every lint kind shown by the editor.
 *
 * Components use this as a stable accumulator shape so lint-kind count rendering
 * does not need to handle missing keys.
 */
export function createEmptyLintKindCounts(): LintKindCounts {
	return Object.fromEntries(LINT_KIND_ORDER.map((key) => [key, 0])) as LintKindCounts;
}

/**
 * Convert persisted or caller-provided font family values to a supported option.
 *
 * This keeps the editor resilient to stale settings or arbitrary input while
 * preserving a predictable default.
 */
export function normalizeFontFamily(value: string): EditorFontFamily {
	if (value === 'serif' || value === 'mono') {
		return value;
	}

	return 'sans';
}

/**
 * Resolve an editor font family option to the CSS stack used by the editor.
 *
 * Components call this instead of duplicating font-stack strings or fallback
 * behavior.
 */
export function fontStackFor(value: EditorFontFamily): string {
	return FONT_OPTIONS.find((option) => option.value === value)?.stack ?? FONT_OPTIONS[0].stack;
}

/**
 * Convert persisted or caller-provided font size values to a supported editor size.
 *
 * Numeric values are rounded and clamped so user input cannot produce unreadable
 * or layout-breaking editor text.
 */
export function normalizeFontSize(value: EditorFontSize | string): EditorFontSize {
	if (value === DEFAULT_FONT_SIZE || value == null || value === '') {
		return DEFAULT_FONT_SIZE;
	}

	const numericValue = typeof value === 'number' ? value : Number(value);
	if (!Number.isFinite(numericValue)) {
		return DEFAULT_FONT_SIZE;
	}

	const rounded = Math.round(numericValue);
	return Math.min(28, Math.max(11, rounded));
}

/**
 * Return the user-facing text for an unpacked suggestion.
 *
 * Replacement suggestions show their replacement text, while non-replacement
 * suggestions fall back to their kind so buttons still have meaningful labels.
 */
export function suggestionText(suggestion: UnpackedSuggestion): string {
	return suggestion.replacement_text !== '' ? suggestion.replacement_text : String(suggestion.kind);
}

/**
 * Count whitespace-delimited words for the editor status bar.
 *
 * This lightweight count is intentionally display-oriented and avoids coupling
 * status text to Harper's parser internals.
 */
export function wordCount(text: string): number {
	return text.trim().match(/\S+/g)?.length ?? 0;
}
