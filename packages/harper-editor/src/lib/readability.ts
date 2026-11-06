/**
 * Offline readability statistics (Flesch Reading Ease and Flesch-Kincaid grade).
 *
 * Pure functions with no state. Syllables are estimated with a vowel-group
 * heuristic, so figures are approximate (typically within about one grade level).
 */

export interface ReadabilityStats {
	words: number;
	sentences: number;
	syllables: number;
	/** 0-100, higher is easier. */
	readingEase: number;
	/** US school grade needed to follow the text. */
	gradeLevel: number;
	/** Silent reading time in minutes at an average adult pace. */
	readingMinutes: number;
}

/** Below this many words the formulas are too noisy to be useful. */
export const MIN_WORDS_FOR_READABILITY = 30;

const READING_WORDS_PER_MINUTE = 238;

export function countSyllables(word: string): number {
	const cleaned = word.toLowerCase().replace(/[^a-z]/g, '');
	if (cleaned.length === 0) {
		return 0;
	}
	if (cleaned.length <= 3) {
		return 1;
	}

	const trimmed = cleaned.replace(/(?:[^laeiouy]es|ed|[^laeiouy]e)$/, '').replace(/^y/, '');
	const groups = trimmed.match(/[aeiouy]{1,2}/g);

	return Math.max(1, groups?.length ?? 1);
}

export function countSentences(text: string): number {
	const marked = text.match(/[^.!?]+[.!?]+(?=\s|$)|[^.!?\s][^.!?]*$/g);
	return marked?.length ?? 0;
}

export function readabilityStats(text: string): ReadabilityStats | null {
	const words = text.match(/[A-Za-z][A-Za-z'’-]*/g) ?? [];
	if (words.length < MIN_WORDS_FOR_READABILITY) {
		return null;
	}

	const sentences = Math.max(1, countSentences(text));
	const syllables = words.reduce((total, word) => total + countSyllables(word), 0);
	const wordsPerSentence = words.length / sentences;
	const syllablesPerWord = syllables / words.length;

	const readingEase = 206.835 - 1.015 * wordsPerSentence - 84.6 * syllablesPerWord;
	const gradeLevel = 0.39 * wordsPerSentence + 11.8 * syllablesPerWord - 15.59;

	return {
		words: words.length,
		sentences,
		syllables,
		readingEase: clamp(readingEase, 0, 100),
		gradeLevel: Math.max(0, gradeLevel),
		readingMinutes: words.length / READING_WORDS_PER_MINUTE,
	};
}

/** Plain-language band for a Flesch Reading Ease score. */
export function readingEaseLabel(readingEase: number): string {
	if (readingEase >= 80) return 'Easy';
	if (readingEase >= 60) return 'Plain';
	if (readingEase >= 50) return 'Fairly difficult';
	if (readingEase >= 30) return 'Difficult';
	return 'Very difficult';
}

export function formatReadingTime(minutes: number): string {
	if (minutes < 1) {
		return '<1 min read';
	}
	return `${Math.round(minutes)} min read`;
}

function clamp(value: number, min: number, max: number): number {
	return Math.min(max, Math.max(min, value));
}
