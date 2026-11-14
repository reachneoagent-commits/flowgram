<script lang="ts">
import { Button, Input, Select, SettingRow, Toggle } from 'components';
import type { Dialect } from 'harper.js';
import { onMount } from 'svelte';
import { Client } from '$lib/client';
import { applyTheme } from '$lib/theme';
import { DIALECT_OPTIONS } from '../settings-data';

const DialectValue = {
	American: 0,
	British: 1,
	Australian: 2,
	Canadian: 3,
	Indian: 4,
} as const;

let launchAtStartup = false;
let autoUpdate = true;
let dialect = 'american';
let isDialectLoading = true;
let isDialectSaving = false;
let dialectError = '';
let isLaunchAtStartupLoading = true;
let isLaunchAtStartupSaving = false;
let launchAtStartupError = '';
let isAutoUpdateLoading = true;
let isAutoUpdateSaving = false;
let isCheckingForUpdates = false;
let autoUpdateError = '';
let updateStatus = '';
let currentVersion = '';
let latestVersion = '';
let debounceMs = 0;
let debounceMsInput = '0';
let isDebounceLoading = true;
let isDebounceSaving = false;
let debounceError = '';
let theme = 'system';
let isThemeLoading = true;
let isThemeSaving = false;
let themeError = '';

onMount(() => {
	void loadDialect();
	void loadLaunchAtStartup();
	void loadAutoUpdate();
	void loadUpdateVersions();
	void loadDebounceMs();
	void loadTheme();
	applyTheme(theme);

	// Listen for system theme changes when theme is "system"
	const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
	const handleSystemThemeChange = () => {
		if (theme === 'system') {
			applyTheme(theme);
		}
	};
	mediaQuery.addEventListener('change', handleSystemThemeChange);

	return () => {
		mediaQuery.removeEventListener('change', handleSystemThemeChange);
	};
});

async function loadDialect() {
	isDialectLoading = true;
	dialectError = '';

	try {
		dialect = dialectToSettingsValue(await Client.getDialect());
	} catch (error) {
		dialectError = `Unable to load dialect: ${error}`;
	} finally {
		isDialectLoading = false;
	}
}

async function setDialect(value: string) {
	const previousDialect = dialect;

	dialect = value;
	isDialectSaving = true;
	dialectError = '';

	try {
		await Client.setDialect(settingsValueToDialect(value));
	} catch (error) {
		dialect = previousDialect;
		dialectError = `Unable to save dialect: ${error}`;
	} finally {
		isDialectSaving = false;
	}
}

async function loadLaunchAtStartup() {
	isLaunchAtStartupLoading = true;
	launchAtStartupError = '';

	try {
		launchAtStartup = await Client.getLaunchAtStartup();
	} catch (error) {
		launchAtStartupError = `Unable to load startup setting: ${error}`;
	} finally {
		isLaunchAtStartupLoading = false;
	}
}

async function setLaunchAtStartup(enabled: boolean) {
	const previousLaunchAtStartup = launchAtStartup;

	launchAtStartup = enabled;
	isLaunchAtStartupSaving = true;
	launchAtStartupError = '';

	try {
		await Client.setLaunchAtStartup(enabled);
	} catch (error) {
		launchAtStartup = previousLaunchAtStartup;
		launchAtStartupError = `Unable to save startup setting: ${error}`;
	} finally {
		isLaunchAtStartupSaving = false;
	}
}

async function loadAutoUpdate() {
	isAutoUpdateLoading = true;
	autoUpdateError = '';

	try {
		autoUpdate = await Client.getAutoUpdate();
	} catch (error) {
		autoUpdateError = `Unable to load update setting: ${error}`;
	} finally {
		isAutoUpdateLoading = false;
	}
}

async function setAutoUpdate(enabled: boolean) {
	const previousAutoUpdate = autoUpdate;

	autoUpdate = enabled;
	isAutoUpdateSaving = true;
	autoUpdateError = '';
	updateStatus = '';

	try {
		await Client.setAutoUpdate(enabled);
	} catch (error) {
		autoUpdate = previousAutoUpdate;
		autoUpdateError = `Unable to save update setting: ${error}`;
	} finally {
		isAutoUpdateSaving = false;
	}
}

async function loadUpdateVersions() {
	try {
		const [current, latest] = await Promise.all([
			Client.getCurrentVersion(),
			Client.getLatestVersion(),
		]);
		currentVersion = current;
		latestVersion = latest;
	} catch (error) {
		console.error('Unable to load update versions.', error);
	}
}

async function checkForUpdates() {
	isCheckingForUpdates = true;
	autoUpdateError = '';
	updateStatus = 'Checking for updates...';

	try {
		const result = await Client.updateToLatest();
		updateStatus = result.message;

		if (result.latestVersion != null) {
			latestVersion = result.latestVersion;
		}

		currentVersion = result.currentVersion ?? (await Client.getCurrentVersion());
	} catch (error) {
		autoUpdateError = `Unable to check for updates: ${error}`;
		updateStatus = '';
	} finally {
		isCheckingForUpdates = false;
	}
}

async function loadDebounceMs() {
	isDebounceLoading = true;
	debounceError = '';

	try {
		debounceMs = await Client.getDebounceMs();
		debounceMsInput = String(debounceMs);
	} catch (error) {
		debounceError = `Unable to load this setting: ${error}`;
	} finally {
		isDebounceLoading = false;
	}
}

async function saveDebounceMs() {
	const parsedDebounceMs = Number(debounceMsInput);

	if (!Number.isInteger(parsedDebounceMs) || parsedDebounceMs < 0) {
		debounceError = 'Enter a whole number of 0 or more.';
		debounceMsInput = String(debounceMs);
		return;
	}

	const previousDebounceMs = debounceMs;
	debounceMs = parsedDebounceMs;
	debounceMsInput = String(parsedDebounceMs);
	isDebounceSaving = true;
	debounceError = '';

	try {
		await Client.setDebounceMs(parsedDebounceMs);
	} catch (error) {
		debounceMs = previousDebounceMs;
		debounceMsInput = String(previousDebounceMs);
		debounceError = `Unable to save debounce delay: ${error}`;
	} finally {
		isDebounceSaving = false;
	}
}

async function loadTheme() {
	isThemeLoading = true;
	themeError = '';

	try {
		theme = await Client.getTheme();
		applyTheme(theme);
	} catch (error) {
		themeError = `Unable to load theme: ${error}`;
	} finally {
		isThemeLoading = false;
	}
}

async function setTheme(value: string) {
	const previousTheme = theme;

	theme = value;
	isThemeSaving = true;
	themeError = '';

	try {
		await Client.setTheme(value);
		applyTheme(value);
	} catch (error) {
		theme = previousTheme;
		themeError = `Unable to save theme: ${error}`;
	} finally {
		isThemeSaving = false;
	}
}

function dialectToSettingsValue(dialect: Dialect): string {
	switch (dialect) {
		case DialectValue.British:
			return 'british';
		case DialectValue.Canadian:
			return 'canadian';
		case DialectValue.Australian:
			return 'australian';
		case DialectValue.Indian:
			return 'indian';
		default:
			return 'american';
	}
}

function settingsValueToDialect(value: string): Dialect {
	switch (value) {
		case 'british':
			return DialectValue.British as Dialect;
		case 'canadian':
			return DialectValue.Canadian as Dialect;
		case 'australian':
			return DialectValue.Australian as Dialect;
		case 'indian':
			return DialectValue.Indian as Dialect;
		default:
			return DialectValue.American as Dialect;
	}
}
</script>

<section>
        <div class="stanza">
          <div class="eyebrow">General</div>
          <div class="rows">
            <SettingRow top>
              <strong>Launch Flowgram at startup</strong>
              <p>Flowgram will start silently when you log in.</p>
              <Toggle slot="control" appearance="settings"
                checked={launchAtStartup}
                aria-label="Launch Flowgram at startup"
                disabled={isLaunchAtStartupLoading || isLaunchAtStartupSaving}
                on:click={() => setLaunchAtStartup(!launchAtStartup)}
               />
            </SettingRow>
            {#if isLaunchAtStartupLoading}
              <p class="result-summary">Loading startup setting...</p>
            {:else if launchAtStartupError}
              <p class="result-summary">{launchAtStartupError}</p>
            {:else if isLaunchAtStartupSaving}
              <p class="result-summary">Saving startup setting...</p>
            {/if}

            <SettingRow top>
              <strong>Automatically check for updates</strong>
              <p>Flowgram will check for new versions daily.</p>
              <p class="result-summary">
                Current version: {currentVersion || 'loading...'} · Latest version: {latestVersion || 'loading...'}
              </p>
              <div class="inline-row">
                <Button
                  unstyled
                  class="button"
                  type="button"
                  disabled={isCheckingForUpdates}
                  on:click={checkForUpdates}
                >
                  {isCheckingForUpdates ? 'Checking...' : 'Check for updates'}
                </Button>
              </div>
              <Toggle slot="control" appearance="settings"
                checked={autoUpdate}
                aria-label="Automatically check for updates"
                disabled={isAutoUpdateLoading || isAutoUpdateSaving}
                on:click={() => setAutoUpdate(!autoUpdate)}
               />
            </SettingRow>
            {#if isAutoUpdateLoading}
              <p class="result-summary">Loading update setting...</p>
            {:else if autoUpdateError}
              <p class="result-summary">{autoUpdateError}</p>
            {:else if isAutoUpdateSaving}
              <p class="result-summary">Saving update setting...</p>
            {:else if updateStatus}
              <p class="result-summary">{updateStatus}</p>
            {/if}
          </div>
        </div>

        <div class="divider"></div>

        <div class="stanza">
          <div class="eyebrow">Language</div>
          <p class="section-copy">
                      Choose the dialect Flowgram uses to check spelling and grammar.
                    </p>
          <div class="inline-row">
            <label for="dialect">English dialect:</label>
            <Select
              unstyled
              id="dialect"
              class="select wide"
              disabled={isDialectLoading || isDialectSaving}
              bind:value={dialect}
              on:change={(event) => setDialect((event.detail.currentTarget as HTMLSelectElement).value)}
            >
              {#each DIALECT_OPTIONS as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </Select>
          </div>
          {#if isDialectLoading}
            <p class="result-summary">Loading dialect...</p>
          {:else if dialectError}
            <p class="result-summary">{dialectError}</p>
          {:else if isDialectSaving}
            <p class="result-summary">Saving dialect...</p>
          {/if}
        </div>

        <div class="stanza">
          <div class="eyebrow">Appearance</div>
          <p class="section-copy">
            Choose how Flowgram looks on your screen.
          </p>
          <div class="inline-row">
            <label for="theme">Theme:</label>
            <Select
              unstyled
              id="theme"
              class="select wide"
              disabled={isThemeLoading || isThemeSaving}
              bind:value={theme}
              on:change={(event) => setTheme((event.detail.currentTarget as HTMLSelectElement).value)}
            >
              <option value="light">Light</option>
              <option value="dark">Dark</option>
              <option value="system">System</option>
            </Select>
          </div>
          {#if isThemeLoading}
            <p class="result-summary">Loading theme...</p>
          {:else if themeError}
            <p class="result-summary">{themeError}</p>
          {:else if isThemeSaving}
            <p class="result-summary">Saving theme...</p>
          {/if}
        </div>

        <div class="divider"></div>

        <div class="stanza">
          <div class="eyebrow">Writing</div>
          <p class="section-copy">
            How long Flowgram waits after you stop typing before it checks your text. Leave it at 0
            for instant checking; raise it if underlines feel distracting while you type.
          </p>
          <div class="inline-row">
            <label for="debounce-ms">Wait before checking:</label>
            <Input
              unstyled
              id="debounce-ms"
              class="select"
              type="number"
              min="0"
              step="50"
              disabled={isDebounceLoading || isDebounceSaving}
              bind:value={debounceMsInput}
              on:change={saveDebounceMs}
            />
            <span>milliseconds</span>
          </div>
          {#if isDebounceLoading}
            <p class="result-summary">Loading...</p>
          {:else if debounceError}
            <p class="result-summary">{debounceError}</p>
          {:else if isDebounceSaving}
            <p class="result-summary">Saved.</p>
          {/if}
        </div>
      </section>
