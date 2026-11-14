import { invoke } from '@tauri-apps/api/core';
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart';
import type {
	Dialect,
	Lint,
	LintConfig,
	Linter,
	LocalLinter as LocalLinterType,
	StructuredLintConfig,
} from 'harper.js';

type RustDialect = 'American' | 'British' | 'Australian' | 'Canadian' | 'Indian';

const DialectValue = {
	American: 0,
	British: 1,
	Australian: 2,
	Canadian: 3,
	Indian: 4,
} as const;

export type HumanizerLevel = 'Ready' | 'Limited' | 'Unavailable';

export interface HumanizerOption {
	id: string;
	label: string;
	blurb: string;
	size_bytes: number;
	min_ram_gb: number;
	fits_memory: boolean;
	fits_disk: boolean;
	recommended: boolean;
	installed: boolean;
}

export interface HumanizerSettings {
	skipped: boolean;
	active: string | null;
	idle_minutes: number;
	warn_numbers: boolean;
}

export interface HumanizerStatus {
	verdict: {
		level: HumanizerLevel;
		headline: string;
		detail: string;
		facts: { os: string; arch: string; total_ram_gb: number; free_disk_gb: number };
		options: HumanizerOption[];
	};
	settings: HumanizerSettings;
	engine_installed: boolean;
	downloading: string | null;
	engine_running: boolean;
	active: string | null;
}

export interface HumanizerRewrite {
	text: string;
	numbers: { missing: string[]; added: string[] } | null;
}

export interface HumanizerDownloadProgress {
	tier: string;
	downloaded: number;
	total: number;
}

export interface HumanizerDownloadDone {
	tier: string;
	error: string | null;
}

const ACCESSIBILITY_PERMISSION_TIMEOUT_MS = 6_000;

/** Fired on `window` after the rule settings are saved. */
export const LINT_CONFIG_CHANGED = 'flowgram:lint-config-changed';

let configLinterPromise: Promise<LocalLinterType> | null = null;

function getConfigLinter(): Promise<LocalLinterType> {
	configLinterPromise ??= Promise.all([
		import('harper.js'),
		import('harper.js/slimBinaryInlined'),
	]).then(
		([{ LocalLinter }, { slimBinaryInlined }]) => new LocalLinter({ binary: slimBinaryInlined }),
	);

	return configLinterPromise;
}

export interface Integration {
	bundle_id: string;
	enabled: boolean;
	display_name: string;
}

export type BrowserId = 'chrome' | 'edge' | 'brave' | 'opera' | 'firefox';

export interface BrowserProfileStatus {
	browser: BrowserId;
	profile_dir: string;
	profile_name: string;
	email: string | null;
	installed: boolean;
	enabled: boolean;
	extension_id: string | null;
	folder_version: string | null;
	bundled_version: string;
}

export interface AppSearchResult {
	name: string;
	bundle_id: string;
}

export type OnboardingState =
	| 'welcome'
	| 'accessibility'
	| 'integration'
	| 'apps'
	| 'browser'
	| 'humanizer'
	| 'test-drive'
	| 'ready'
	| 'complete';

export type AccessibilityPermissionStatus = 'Granted' | 'NotGranted' | 'Unsupported';

/** Serializable Rust updater outcome; an installed update may still require a restart. */
export interface UpdateResult {
	status: 'up-to-date' | 'updated' | 'error';
	currentVersion: string | null;
	latestVersion: string | null;
	message: string;
	error: string | null;
}

/** Provides an easy-to-use interface for interacting with the main Rust Tauri process.
 * Relevant because that is where the canonical state is stored. */
export class Client {
	static async getLintConfig(): Promise<LintConfig> {
		return await invoke<LintConfig>('get_lint_config');
	}

	static async getDefaultLintConfig(): Promise<LintConfig> {
		const configLinter = await getConfigLinter();

		return JSON.parse(await configLinter.getDefaultLintConfigAsJSON()) as LintConfig;
	}

	/** Plain-text one-line descriptions of each rule, keyed by rule name. */
	static async getLintDescriptions(): Promise<Record<string, string>> {
		const configLinter = await getConfigLinter();

		return await configLinter.getLintDescriptions();
	}

	static async getStructuredLintConfig(): Promise<StructuredLintConfig> {
		const configLinter = await getConfigLinter();

		await configLinter.setLintConfigWithJSON(JSON.stringify(await Client.getLintConfig()));

		return JSON.parse(await configLinter.getStructuredLintConfigJSON()) as StructuredLintConfig;
	}

	static async getDialect(): Promise<Dialect> {
		return rustDialectToDialect(await invoke<RustDialect>('get_dialect'));
	}

	static async setDialect(dialect: Dialect): Promise<void> {
		await invoke('set_dialect', { dialect: dialectToRustDialect(dialect) });
	}

	static async getDebounceMs(): Promise<number> {
		return await invoke<number>('get_debounce_ms');
	}

	static async setDebounceMs(debounceMs: number): Promise<void> {
		await invoke('set_debounce_ms', { debounceMs });
	}

	/** Check if the app is configured to auto update. */
	static async getAutoUpdate(): Promise<boolean> {
		return await invoke<boolean>('get_auto_update');
	}

	/** Configure the app to auto update. */
	static async setAutoUpdate(autoUpdate: boolean): Promise<void> {
		await invoke('set_auto_update', { autoUpdate });
	}

	/** Get the preferred UI theme: "light", "dark", or "system". */
	static async getTheme(): Promise<string> {
		return await invoke<string>('get_theme');
	}

	/** Set the preferred UI theme: "light", "dark", or "system". */
	static async setTheme(theme: string): Promise<void> {
		await invoke('set_theme', { theme });
	}

	static async humanizerStatus(): Promise<HumanizerStatus> {
		return await invoke<HumanizerStatus>('humanizer_status');
	}

	static async humanizerSaveSettings(settings: HumanizerSettings): Promise<void> {
		await invoke('humanizer_save_settings', { settings });
	}

	/** Starts a background download; progress and completion arrive as events. */
	static async humanizerDownload(tier: string): Promise<void> {
		await invoke('humanizer_download', { tier });
	}

	static async humanizerCancelDownload(): Promise<void> {
		await invoke('humanizer_cancel_download');
	}

	static async humanizerDeleteModel(tier: string): Promise<void> {
		await invoke('humanizer_delete_model', { tier });
	}

	static async humanizerSetActiveModel(tier: string): Promise<void> {
		await invoke('humanizer_set_active_model', { tier });
	}

	static async humanizerRewrite(text: string): Promise<HumanizerRewrite> {
		return await invoke<HumanizerRewrite>('humanizer_rewrite', { text });
	}

	static async humanizerCancelRewrite(): Promise<void> {
		await invoke('humanizer_cancel_rewrite');
	}

	static async humanizerStopEngine(): Promise<void> {
		await invoke('humanizer_stop_engine');
	}

	static async humanizerOpenModelsFolder(): Promise<void> {
		await invoke('humanizer_open_models_folder');
	}

	/** Text the shortcut captured before the Humanize window loaded, if any. */
	static async humanizerTakePending(): Promise<string | null> {
		return await invoke<string | null>('humanizer_take_pending');
	}

	static async humanizerOpenWindow(): Promise<void> {
		await invoke('humanizer_open_window');
	}

	static async humanizerOpenSettings(): Promise<void> {
		await invoke('humanizer_open_settings');
	}

	/** Hides the Humanize window and pastes `text` over the selection in the app underneath. */
	static async humanizerPasteBack(text: string): Promise<void> {
		await invoke('humanizer_paste_back', { text });
	}

	static async getCurrentVersion(): Promise<string> {
		return await invoke<string>('get_current_version');
	}

	static async getLatestVersion(): Promise<string> {
		return await invoke<string>('get_latest_version');
	}

	static async updateToLatest(): Promise<UpdateResult> {
		return await invoke<UpdateResult>('update_to_latest');
	}

	static async restartHarper(): Promise<void> {
		await invoke('restart_harper');
	}

	static async getOnboardingState(): Promise<OnboardingState> {
		return await invoke<OnboardingState>('get_onboarding_state');
	}

	static async setOnboardingState(onboardingState: OnboardingState): Promise<void> {
		await invoke('set_onboarding_state', { onboardingState });
	}

	static async getLaunchAtStartup(): Promise<boolean> {
		return await isEnabled();
	}

	static async setLaunchAtStartup(enabled: boolean): Promise<void> {
		if (enabled) {
			await enable();
		} else {
			await disable();
		}
	}

	static async setLintConfig(lintConfig: LintConfig): Promise<void> {
		await invoke('set_lint_config', { lintConfig });
		// Lets other parts of Settings (such as the "spell checking is off" banner) update at once.
		window.dispatchEvent(new CustomEvent(LINT_CONFIG_CHANGED));
	}

	static async getDictionary(): Promise<string[]> {
		return await invoke<string[]>('get_dictionary');
	}

	static async setDictionary(words: string[]): Promise<void> {
		await invoke('set_dictionary', { words });
	}

	static async disableRule(ruleName: string): Promise<LintConfig> {
		const lintConfig = await Client.getLintConfig();
		lintConfig[ruleName] = false;

		await Client.setLintConfig(lintConfig);

		return lintConfig;
	}

	static async ignoreLint(linter: Linter, source: string, lint: Lint): Promise<void> {
		await linter.ignoreLint(source, lint);
		const ignoredLints = await linter.exportIgnoredLints();

		await invoke('ignore_lint', { ignoredLints });
	}

	static async addToDictionary(word: string): Promise<void> {
		await invoke('add_to_dictionary', { word });
	}

	static async getIntegrations(): Promise<Integration[]> {
		return await invoke<Integration[]>('get_integrations');
	}

	static async getAutoEnableNewApps(): Promise<boolean> {
		return await invoke<boolean>('get_auto_enable_new_apps');
	}

	static async setAutoEnableNewApps(autoEnableNewApps: boolean): Promise<void> {
		await invoke('set_auto_enable_new_apps', { autoEnableNewApps });
	}

	static async addIntegration(bundleId: string): Promise<void> {
		await invoke('add_integration', { bundleId });
	}

	static async removeIntegration(bundleId: string): Promise<void> {
		await invoke('remove_integration', { bundleId });
	}

	static async setIntegrationEnabled(bundleId: string, enabled: boolean): Promise<void> {
		await invoke('set_integration_enabled', { bundleId, enabled });
	}

	static async getApplicationIconDataUrl(bundleId: string): Promise<string> {
		return await invoke<string>('get_application_icon_data_url', { bundleId });
	}

	static async searchApps(query: string): Promise<AppSearchResult[]> {
		return await invoke<AppSearchResult[]>('search_apps', { query });
	}

	static async launchApp(bundleId: string): Promise<void> {
		await invoke('launch_app', { bundleId });
	}

	/** Copy the bundled browser extension to Documents and open that folder. Returns its path. */
	static async prepareBrowserExtension(reveal = true): Promise<string> {
		return await invoke<string>('prepare_browser_extension', { reveal });
	}

	/** Copy the bundled Firefox extension to Documents and open that folder. Returns its path. */
	static async prepareFirefoxExtension(): Promise<string> {
		return await invoke<string>('prepare_firefox_extension');
	}

	static async openBrowserExtensionsPage(
		browser: BrowserId,
		profileDirectory?: string,
		extensionId?: string,
	): Promise<void> {
		await invoke('open_browser_extensions_page', { browser, profileDirectory, extensionId });
	}

	static async getBrowserExtensionProfiles(): Promise<BrowserProfileStatus[]> {
		return await invoke<BrowserProfileStatus[]>('get_browser_extension_profiles');
	}

	static async getAccessibilityPermissionStatus(): Promise<AccessibilityPermissionStatus> {
		return await withTimeout(
			invoke<AccessibilityPermissionStatus>('get_accessibility_permission_status'),
			ACCESSIBILITY_PERMISSION_TIMEOUT_MS,
			'Accessibility permission check timed out',
		);
	}

	static async requestAccessibilityPermission(): Promise<AccessibilityPermissionStatus> {
		return await withTimeout(
			invoke<AccessibilityPermissionStatus>('request_accessibility_permission'),
			ACCESSIBILITY_PERMISSION_TIMEOUT_MS,
			'Accessibility permission request timed out',
		);
	}

	static async startHighlighterService(): Promise<boolean> {
		return await invoke<boolean>('start_highlighter_service');
	}

	static async stopHighlighterService(): Promise<boolean> {
		return await invoke<boolean>('stop_highlighter_service');
	}
}

async function withTimeout<T>(promise: Promise<T>, timeoutMs: number, message: string): Promise<T> {
	let timeoutId: ReturnType<typeof setTimeout> | undefined;

	const timeout = new Promise<never>((_, reject) => {
		timeoutId = setTimeout(() => reject(new Error(message)), timeoutMs);
	});

	try {
		return await Promise.race([promise, timeout]);
	} finally {
		if (timeoutId) {
			clearTimeout(timeoutId);
		}
	}
}

function rustDialectToDialect(dialect: RustDialect): Dialect {
	switch (dialect) {
		case 'British':
			return DialectValue.British as Dialect;
		case 'Australian':
			return DialectValue.Australian as Dialect;
		case 'Canadian':
			return DialectValue.Canadian as Dialect;
		case 'Indian':
			return DialectValue.Indian as Dialect;
		default:
			return DialectValue.American as Dialect;
	}
}

function dialectToRustDialect(dialect: Dialect): RustDialect {
	switch (dialect) {
		case DialectValue.British:
			return 'British';
		case DialectValue.Australian:
			return 'Australian';
		case DialectValue.Canadian:
			return 'Canadian';
		case DialectValue.Indian:
			return 'Indian';
		default:
			return 'American';
	}
}
