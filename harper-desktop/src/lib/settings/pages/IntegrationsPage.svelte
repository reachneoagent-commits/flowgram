<script lang="ts">
import {
	Button,
	Checkbox,
	CheckIcon,
	IconButton,
	Panel,
	SettingRow,
	Toggle,
	TrashIcon,
} from 'components';
import { onMount } from 'svelte';
import { Client, type BrowserProfileStatus, type Integration } from '$lib/client';
import AppIcon from '../components/AppIcon.svelte';
import AppPickerModal from '../components/AppPickerModal.svelte';

interface IntegrationRow extends Integration {
	name: string;
}

let integrations: Integration[] = [];
let integrationApps: IntegrationRow[] = [];
let integrationsError = '';
let isIntegrationsLoading = true;
let isIntegrationsSaving = false;
let autoEnableNewApps = false;
let isAutoEnableLoading = true;
let isAutoEnableSaving = false;
let autoEnableError = '';
let appPickerOpen = false;
let newBundleId = '';
let browserProfiles: BrowserProfileStatus[] = [];
let isProfilesLoading = true;
let extensionError = '';
let isPreparingExtension = false;
let isPreparingFirefox = false;

const BROWSER_LABELS: Record<string, string> = {
	chrome: 'Chrome',
	edge: 'Edge',
	brave: 'Brave',
};

$: existingBundleIds = integrations.map((integration) => integration.bundle_id);

onMount(() => {
	const refresh = () => {
		if (!isIntegrationsSaving) void loadIntegrations();
		if (!isAutoEnableSaving) void loadAutoEnableNewApps();
		void loadBrowserProfiles();
	};
	refresh();
	window.addEventListener('focus', refresh);
	return () => window.removeEventListener('focus', refresh);
});

async function loadBrowserProfiles() {
	try {
		browserProfiles = await Client.getBrowserExtensionProfiles();
	} catch (error) {
		extensionError = `Unable to check browser profiles: ${error}`;
	} finally {
		isProfilesLoading = false;
	}
}

async function openProfileExtensionsPage(profile: BrowserProfileStatus) {
	extensionError = '';
	try {
		await Client.openBrowserExtensionsPage(profile.browser, profile.profile_dir);
	} catch (error) {
		extensionError = `${error}`;
	}
}

/** Versions are dotted numbers; compare piece by piece so 3.0.10 is newer than 3.0.9. */
function isOlder(installed: string | null, bundled: string): boolean {
	if (!installed) return false;
	const a = installed.split('.').map(Number);
	const b = bundled.split('.').map(Number);

	for (let i = 0; i < Math.max(a.length, b.length); i++) {
		const diff = (a[i] ?? 0) - (b[i] ?? 0);
		if (diff !== 0) return diff < 0;
	}

	return false;
}

let profileNotice: Record<string, string> = {};

function profileKey(profile: BrowserProfileStatus) {
	return `${profile.browser}:${profile.profile_dir}`;
}

/** Copies the newest extension files over the old ones, then opens the extension's own page. */
async function updateProfileExtension(profile: BrowserProfileStatus) {
	extensionError = '';
	try {
		await Client.prepareBrowserExtension(false);
		await Client.openBrowserExtensionsPage(
			profile.browser,
			profile.profile_dir,
			profile.extension_id ?? undefined,
		);
		profileNotice = {
			...profileNotice,
			[profileKey(profile)]: 'Files updated. On the page that opened, click the reload arrow (↻) on Gramflow to finish.',
		};
		await loadBrowserProfiles();
	} catch (error) {
		extensionError = `${error}`;
	}
}

/** Chrome does not let other apps delete extensions, so open the extension's page where Remove is one click. */
async function removeProfileExtension(profile: BrowserProfileStatus) {
	extensionError = '';
	try {
		await Client.openBrowserExtensionsPage(
			profile.browser,
			profile.profile_dir,
			profile.extension_id ?? undefined,
		);
		profileNotice = {
			...profileNotice,
			[profileKey(profile)]: 'On the page that opened, click Remove, then confirm. This list updates when you come back.',
		};
	} catch (error) {
		extensionError = `${error}`;
	}
}

async function showExtensionFolder() {
	if (isPreparingExtension) return;
	isPreparingExtension = true;
	extensionError = '';
	try {
		await Client.prepareBrowserExtension();
	} catch (error) {
		extensionError = `${error}`;
	} finally {
		isPreparingExtension = false;
	}
}

async function setUpFirefox() {
	if (isPreparingFirefox) return;
	isPreparingFirefox = true;
	extensionError = '';
	try {
		await Client.prepareFirefoxExtension();
		await Client.openBrowserExtensionsPage('firefox');
	} catch (error) {
		extensionError = `${error}`;
	} finally {
		isPreparingFirefox = false;
	}
}

async function loadAutoEnableNewApps() {
	isAutoEnableLoading = true;
	autoEnableError = '';
	try {
		autoEnableNewApps = await Client.getAutoEnableNewApps();
	} catch (error) {
		autoEnableError = `Unable to load automatic app enablement: ${error}`;
	} finally {
		isAutoEnableLoading = false;
	}
}

async function setAutoEnableNewApps(enabled: boolean) {
	const previous = autoEnableNewApps;
	autoEnableNewApps = enabled;
	isAutoEnableSaving = true;
	autoEnableError = '';
	try {
		await Client.setAutoEnableNewApps(enabled);
	} catch (error) {
		autoEnableNewApps = previous;
		autoEnableError = `Unable to save automatic app enablement: ${error}`;
	} finally {
		isAutoEnableSaving = false;
	}
}

async function loadIntegrations() {
	isIntegrationsLoading = true;
	integrationsError = '';

	try {
		integrations = await Client.getIntegrations();
		integrationApps = integrations.map(toIntegrationRow);
	} catch (error) {
		integrationsError = `Unable to load integrations: ${error}`;
	} finally {
		isIntegrationsLoading = false;
	}
}

function toIntegrationRow(integration: Integration): IntegrationRow {
	return {
		...integration,
		name: integration.display_name,
	};
}

async function setIntegrationEnabled(bundleId: string, enabled: boolean) {
	const previousIntegrations = integrations;
	const previousIntegrationApps = integrationApps;

	integrations = integrations.map((integration) =>
		integration.bundle_id === bundleId ? { ...integration, enabled } : integration,
	);
	integrationApps = integrationApps.map((app) =>
		app.bundle_id === bundleId ? { ...app, enabled } : app,
	);
	isIntegrationsSaving = true;
	integrationsError = '';

	try {
		await Client.setIntegrationEnabled(bundleId, enabled);
	} catch (error) {
		integrations = previousIntegrations;
		integrationApps = previousIntegrationApps;
		integrationsError = `Unable to update integration: ${error}`;
	} finally {
		isIntegrationsSaving = false;
	}
}

async function removeIntegration(bundleId: string) {
	const previousIntegrations = integrations;
	const previousIntegrationApps = integrationApps;

	integrations = integrations.filter((integration) => integration.bundle_id !== bundleId);
	integrationApps = integrationApps.filter((app) => app.bundle_id !== bundleId);
	isIntegrationsSaving = true;
	integrationsError = '';

	try {
		await Client.removeIntegration(bundleId);
	} catch (error) {
		integrations = previousIntegrations;
		integrationApps = previousIntegrationApps;
		integrationsError = `Unable to remove integration: ${error}`;
	} finally {
		isIntegrationsSaving = false;
	}
}

async function addIntegration(bundleId: string) {
	const trimmedBundleId = bundleId.trim();

	if (
		!trimmedBundleId ||
		integrations.some((integration) => integration.bundle_id === trimmedBundleId)
	) {
		return;
	}

	const previousIntegrations = integrations;
	const previousIntegrationApps = integrationApps;

	integrations = [
		...integrations,
		{ bundle_id: trimmedBundleId, enabled: true, display_name: trimmedBundleId },
	];
	integrationApps = integrations.map(toIntegrationRow);
	isIntegrationsSaving = true;
	integrationsError = '';

	try {
		await Client.addIntegration(trimmedBundleId);
		await loadIntegrations();
		closeAppPicker();
	} catch (error) {
		integrations = previousIntegrations;
		integrationApps = previousIntegrationApps;
		integrationsError = `Unable to add integration: ${error}`;
	} finally {
		isIntegrationsSaving = false;
	}
}

function closeAppPicker() {
	appPickerOpen = false;
	newBundleId = '';
}
</script>

<section>
  <div class="stanza">
    <div class="eyebrow">Selected apps</div>
    <p class="section-copy">Gramflow will only watch the apps you enable here.</p>

    {#if isIntegrationsLoading}
      <p class="result-summary">Loading integrations...</p>
    {:else if integrationsError}
      <p class="result-summary">{integrationsError}</p>
    {:else if isIntegrationsSaving}
      <p class="result-summary">Saving integrations...</p>
    {/if}

    <Panel>
      {#if !isIntegrationsLoading && integrationApps.length === 0}
        <div class="empty">No configured app integrations.</div>
      {:else}
        {#each integrationApps as app}
          <div class="app-row">
            <AppIcon bundleId={app.bundle_id} name={app.name} />
            <div class="grow">
              <strong>{app.name}</strong>
              <p>{app.bundle_id}</p>
            </div>
            <IconButton
              danger
              disabled={isIntegrationsLoading || isIntegrationsSaving}
              aria-label={`Remove ${app.name}`}
              on:click={() => removeIntegration(app.bundle_id)}
            >
              <TrashIcon className="control-icon" />
            </IconButton>
            <Toggle
              appearance="settings"
              checked={app.enabled}
              disabled={isIntegrationsLoading || isIntegrationsSaving}
              aria-label={`Toggle ${app.name}`}
              on:click={() => setIntegrationEnabled(app.bundle_id, !app.enabled)}
            />
          </div>
        {/each}
      {/if}
    </Panel>

    <div class="actions-row">
      <Button
        unstyled
        class="button"
        type="button"
        disabled={isIntegrationsLoading || isIntegrationsSaving}
        on:click={() => (appPickerOpen = true)}
      >Add application...</Button>
      <span class="muted">Choose any app from your Applications folder.</span>
    </div>
  </div>

  <div class="divider"></div>

  <div class="stanza">
    <div class="eyebrow">Browser extension</div>
    <p class="section-copy">Each browser profile has its own extensions, so add Gramflow to every profile you want checked.</p>

    <Panel>
      {#if isProfilesLoading}
        <div class="empty">Checking your browser profiles...</div>
      {:else if browserProfiles.length === 0}
        <div class="empty">No Chrome, Edge or Brave profiles found.</div>
      {:else}
        {#each browserProfiles as profile (profileKey(profile))}
          {@const outdated = profile.installed && isOlder(profile.folder_version, profile.bundled_version)}
          <div class="app-row">
            <div class="grow">
              <strong>{BROWSER_LABELS[profile.browser]} · {profile.profile_name}</strong>
              <p>
                {profile.email ?? profile.profile_dir} —
                {#if profile.installed && profile.enabled}Extension active
                {:else if profile.installed}Extension installed but turned off
                {:else}Extension not added{/if}
                {#if profile.installed && profile.folder_version}
                  · version {profile.folder_version}
                {/if}
              </p>
              {#if outdated}
                <p><span class="pill amber">Update available</span> Version {profile.bundled_version} ships with this Gramflow.</p>
              {:else if profile.installed}
                <p><span class="pill">Up to date</span></p>
              {/if}
              {#if profileNotice[profileKey(profile)]}
                <p role="status">{profileNotice[profileKey(profile)]}</p>
              {/if}
            </div>
            {#if profile.installed}
              {#if outdated}
                <Button size="sm" on:click={() => updateProfileExtension(profile)}>Update</Button>
              {/if}
              <Button color="white" size="sm" on:click={() => removeProfileExtension(profile)}>Remove</Button>
            {:else}
              <Button color="white" size="sm" on:click={() => openProfileExtensionsPage(profile)}>Set up</Button>
            {/if}
          </div>
        {/each}
      {/if}
    </Panel>

    <div class="actions-row">
      <Button unstyled class="button" type="button" disabled={isPreparingExtension} on:click={showExtensionFolder}>
        {isPreparingExtension ? 'Preparing...' : 'Show extension folder'}
      </Button>
      <span class="muted">Set up opens that profile's extensions page: turn on Developer mode, click Load unpacked, then pick the Gramflow Browser Extension folder in Documents.</span>
    </div>
    <Panel>
      <div class="app-row">
        <div class="grow">
          <strong>Opera</strong>
          <p>Uses the same extension as Chrome. Set up opens its extensions page; turn on Developer mode, then Load unpacked.</p>
        </div>
        <Button color="white" size="sm" on:click={() => Client.openBrowserExtensionsPage('opera').catch((error) => (extensionError = `${error}`))}>Set up</Button>
      </div>
      <div class="app-row">
        <div class="grow">
          <strong>Firefox</strong>
          <p>Needs its own copy. Set up opens Firefox; choose Load Temporary Add-on and pick manifest.json in the Gramflow Firefox Extension folder. Firefox forgets it on quit.</p>
        </div>
        <Button color="white" size="sm" disabled={isPreparingFirefox} on:click={setUpFirefox}>{isPreparingFirefox ? 'Preparing...' : 'Set up'}</Button>
      </div>
    </Panel>
    {#if extensionError}
      <p class="result-summary" role="alert">{extensionError}</p>
    {/if}
  </div>

  <div class="divider"></div>

  <div class="stanza">
    <div class="eyebrow">New apps</div>
    <SettingRow top>
      <strong>Enable new apps automatically</strong>
      <p>Automatically add and enable new apps when Gramflow encounters them. Disabled apps stay disabled; removed apps can be added again.</p>
      <Toggle slot="control" appearance="settings"
        checked={autoEnableNewApps}
        disabled={isAutoEnableLoading || isAutoEnableSaving}
        aria-label="Enable new apps automatically"
        on:click={() => setAutoEnableNewApps(!autoEnableNewApps)}
       />
    </SettingRow>
    {#if autoEnableError}
      <p class="result-summary" role="alert">{autoEnableError}</p>
    {:else if isAutoEnableLoading}
      <p class="result-summary">Loading automatic app enablement...</p>
    {:else if isAutoEnableSaving}
      <p class="result-summary">Saving automatic app enablement...</p>
    {/if}
  </div>
</section>

{#if appPickerOpen}
  <AppPickerModal
    bind:bundleId={newBundleId}
    {existingBundleIds}
    isSaving={isIntegrationsSaving}
    close={closeAppPicker}
    add={addIntegration}
  />
{/if}
