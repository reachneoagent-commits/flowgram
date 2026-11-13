import type { LintKind } from 'harper.js';
import { getContrastingTextColor } from './utils';

export type { LintKind } from 'harper.js';

// Gramflow's four issue categories. Tuned to stay visible on both light and dark pages.
const SPELLING = '#E5532D'; // Coral
const GRAMMAR = '#3D8BFF'; // Sky
const CLARITY = '#E69A0B'; // Amber
const STYLE = '#8F63F0'; // Violet

const LINT_KIND_COLORS = {
	Agreement: GRAMMAR,
	BoundaryError: GRAMMAR,
	Capitalization: GRAMMAR,
	Eggcorn: SPELLING,
	Enhancement: STYLE,
	Formatting: GRAMMAR,
	Grammar: GRAMMAR,
	Malapropism: SPELLING,
	Miscellaneous: STYLE,
	Nonstandard: GRAMMAR,
	Punctuation: GRAMMAR,
	Readability: CLARITY,
	Redundancy: CLARITY,
	Regionalism: STYLE,
	Repetition: CLARITY,
	Spelling: SPELLING,
	Style: STYLE,
	Typo: SPELLING,
	Usage: GRAMMAR,
	WordChoice: CLARITY,
	WordOrder: GRAMMAR,
} as const satisfies Record<LintKind, string>;

// Export the array of all lint kind names
export const LINT_KINDS = Object.keys(LINT_KIND_COLORS) as LintKind[];

// The main function that uses the map
export function lintKindColor(lintKindKey: string): string {
	const color = LINT_KIND_COLORS[lintKindKey as LintKind];
	if (!color) {
		throw new Error(`Unexpected lint kind: ${lintKindKey}`);
	}
	return color;
}

export function lintKindTextColor(lintKindKeyOrColor: string): 'black' | 'white' {
	const color = LINT_KIND_COLORS[lintKindKeyOrColor as LintKind] ?? lintKindKeyOrColor;
	return getContrastingTextColor(color);
}
