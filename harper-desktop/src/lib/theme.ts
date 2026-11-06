export type ThemePreference = 'light' | 'dark' | 'system';

/** Resolves a saved preference ("system" follows the OS) to a concrete theme. */
export function resolveTheme(preference: string): 'light' | 'dark' {
	if (preference === 'dark' || preference === 'light') {
		return preference;
	}

	return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/** The stylesheet keys off `body[data-theme]`; the root gets it too for native widgets. */
export function applyTheme(preference: string): void {
	const theme = resolveTheme(preference);
	document.body.setAttribute('data-theme', theme);
	document.documentElement.setAttribute('data-theme', theme);
	// The shared component library uses Tailwind's `dark:` variant, which keys off `.dark`.
	document.documentElement.classList.toggle('dark', theme === 'dark');
	document.body.classList.toggle('dark', theme === 'dark');
	document.documentElement.style.colorScheme = theme;
}
