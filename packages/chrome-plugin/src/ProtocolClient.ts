import type { Dialect, LintConfig, LintOptions, StructuredLintConfig } from 'harper.js';
import type { UnpackedLintGroups } from 'lint-framework';
import { LRUCache } from 'lru-cache';
import type { ActivationKey, Hotkey, WeirpackMeta } from './protocol';

/**
 * Sends a message to the extension's background worker.
 *
 * When the extension is reloaded or updated, page scripts from the old version keep running but
 * can no longer reach the worker ("Extension context invalidated"). That is expected and fixed by
 * refreshing the page, so go quiet instead of throwing errors on every keystroke.
 */
// biome-ignore lint/suspicious/noExplicitAny: messages and replies are untyped across the extension.
async function sendToExtension(message: unknown): Promise<any> {
	const stale = () => new Promise<never>(() => {});

	if (!chrome.runtime?.id) return stale();

	try {
		return await chrome.runtime.sendMessage(message);
	} catch (error) {
		if (String((error as Error)?.message).includes('Extension context invalidated')) {
			return stale();
		}
		throw error;
	}
}

export default class ProtocolClient {
	private static readonly lintCache = new LRUCache<string, Promise<UnpackedLintGroups>>({
		max: 5000,
		ttl: 60_000,
		ttlAutopurge: true,
	});

	private static cacheKey(text: string, domain: string, options?: LintOptions): string {
		return JSON.stringify([
			domain,
			text,
			options?.language ?? 'markdown',
			options?.forceAllHeadings ?? false,
			options?.regex_mask ?? null,
			options?.dedup ?? true,
			options?.isolateEnglish ?? false,
		]);
	}

	public static async lint(
		text: string,
		domain: string,
		options?: LintOptions,
	): Promise<UnpackedLintGroups> {
		const key = this.cacheKey(text, domain, options);
		let p = this.lintCache.get(key);
		if (!p) {
			p = sendToExtension({ kind: 'lint', text, domain, options })
				.then((r) => r.lints as UnpackedLintGroups);
			this.lintCache.set(key, p);
		}
		return p;
	}

	public static async getLintConfig(): Promise<LintConfig> {
		return (await sendToExtension({ kind: 'getConfig' })).config;
	}

	public static async getStructuredLintConfig(): Promise<StructuredLintConfig> {
		return (await sendToExtension({ kind: 'getStructuredConfig' })).config;
	}

	public static async setLintConfig(lintConfig: LintConfig): Promise<void> {
		this.lintCache.clear();
		await sendToExtension({ kind: 'setConfig', config: lintConfig });
	}

	public static async setRuleEnabled(ruleId: string, enabled: boolean): Promise<void> {
		const config = await this.getLintConfig();
		const nextConfig: LintConfig = { ...config, [ruleId]: enabled };
		await this.setLintConfig(nextConfig);
	}

	public static async getLintDescriptions(): Promise<Record<string, string>> {
		return (await sendToExtension({ kind: 'getLintDescriptions' })).descriptions;
	}

	public static async getDialect(): Promise<Dialect> {
		return (await sendToExtension({ kind: 'getDialect' })).dialect;
	}

	public static async setDialect(dialect: Dialect): Promise<void> {
		await sendToExtension({ kind: 'setDialect', dialect });
	}

	public static async getIsolateEnglish(): Promise<boolean> {
		return (await sendToExtension({ kind: 'getIsolateEnglish' })).isolateEnglish;
	}

	public static async setIsolateEnglish(isolateEnglish: boolean): Promise<void> {
		this.lintCache.clear();
		await sendToExtension({ kind: 'setIsolateEnglish', isolateEnglish });
	}

	/** Read whether pointer hover should open suggestion popups. */
	public static async getOpenSuggestionsOnHover(): Promise<boolean> {
		return (await sendToExtension({ kind: 'getOpenSuggestionsOnHover' })).enabled;
	}

	/** Persist the optional hover behavior for all extension tabs. */
	public static async setOpenSuggestionsOnHover(enabled: boolean): Promise<void> {
		await sendToExtension({ kind: 'setOpenSuggestionsOnHover', enabled });
	}

	public static async getDelay(): Promise<number> {
		return (await sendToExtension({ kind: 'getDelay' })).delay;
	}

	public static async setDelay(delay: number): Promise<void> {
		await sendToExtension({ kind: 'setDelay', delay });
	}

	public static async getDomainEnabled(domain: string): Promise<boolean> {
		this.lintCache.clear();
		return (await sendToExtension({ kind: 'getDomainStatus', domain })).enabled;
	}

	/** Set whether Harper is enabled for a given domain.
	 *
	 * @param overrideValue dictates whether this should override a previous setting.
	 * */
	public static async setDomainEnabled(
		domain: string,
		enabled: boolean,
		overrideValue = true,
	): Promise<void> {
		await sendToExtension({
			kind: 'setDomainStatus',
			enabled,
			domain,
			overrideValue,
		});
	}

	public static async getDefaultEnabled(): Promise<boolean> {
		this.lintCache.clear();
		return (await sendToExtension({ kind: 'getDefaultStatus' })).enabled;
	}

	public static async getEnabledDomains(): Promise<string[]> {
		return (await sendToExtension({ kind: 'getEnabledDomains' })).domains;
	}

	public static async setDefaultEnabled(enabled: boolean): Promise<void> {
		await sendToExtension({ kind: 'setDefaultStatus', enabled });
	}

	public static async getActivationKey(): Promise<ActivationKey> {
		return (await sendToExtension({ kind: 'getActivationKey' })).key;
	}

	public static async getHotkey(): Promise<Hotkey> {
		return (await sendToExtension({ kind: 'getHotkey' })).hotkey;
	}

	public static async setHotkey(hotkey: Hotkey): Promise<void> {
		const modifiers = hotkey.modifiers;
		const hotkeyCopy = {
			modifiers: [...modifiers], // Create a new array
			key: hotkey.key,
		};
		await sendToExtension({ kind: 'setHotkey', hotkey: hotkeyCopy });
	}

	public static async setActivationKey(key: ActivationKey): Promise<void> {
		await sendToExtension({ kind: 'setActivationKey', key });
	}

	public static async addToUserDictionary(words: string[]): Promise<void> {
		this.lintCache.clear();
		await sendToExtension({ kind: 'addToUserDictionary', words });
	}

	public static async setUserDictionary(words: string[]): Promise<void> {
		this.lintCache.clear();
		await sendToExtension({ kind: 'setUserDictionary', words });
	}

	public static async getUserDictionary(): Promise<string[]> {
		return (await sendToExtension({ kind: 'getUserDictionary' })).words;
	}

	public static async getInstalledOn(): Promise<string | null> {
		return (await sendToExtension({ kind: 'getInstalledOn' })).installedOn;
	}

	public static async getReviewed(): Promise<boolean> {
		return (await sendToExtension({ kind: 'getReviewed' })).reviewed;
	}

	public static async setReviewed(reviewed: boolean): Promise<void> {
		await sendToExtension({ kind: 'setReviewed', reviewed });
	}

	/** Clear and persist ignored suggestions for all websites. Open pages may need a reload. */
	public static async clearIgnoredLints(): Promise<void> {
		const response = await sendToExtension({ kind: 'clearIgnoredLints' });
		if (response?.kind !== 'clearIgnoredLints' || !response.success) {
			throw new Error('Could not clear ignored suggestions. Please try again.');
		}
	}

	public static async ignoreHash(hash: string): Promise<void> {
		await sendToExtension({ kind: 'ignoreLint', contextHash: hash });
		this.lintCache.clear();
	}

	public static async openReportError(
		example: string,
		ruleId: string,
		feedback: string,
	): Promise<void> {
		await sendToExtension({
			kind: 'openReportError',
			example,
			rule_id: ruleId,
			feedback,
		});
	}

	public static async openOptions(): Promise<void> {
		// Use background to open options to support content scripts reliably
		await sendToExtension({ kind: 'openOptions' });
	}

	public static async postFormData(
		url: string,
		formData: Record<string, string>,
	): Promise<boolean> {
		return (await sendToExtension({ kind: 'postFormData', url, formData })).success;
	}

	public static async getWeirpacks(): Promise<WeirpackMeta[]> {
		return (await sendToExtension({ kind: 'getWeirpacks' })).weirpacks;
	}

	public static async addWeirpack(filename: string, bytes: Uint8Array): Promise<void> {
		this.lintCache.clear();
		await sendToExtension({
			kind: 'addWeirpack',
			filename,
			bytes: Array.from(bytes),
		});
	}

	public static async removeWeirpack(id: string): Promise<void> {
		this.lintCache.clear();
		await sendToExtension({ kind: 'removeWeirpack', id });
	}
}
