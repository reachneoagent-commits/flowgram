<script lang="ts">
import { isTauri } from '@tauri-apps/api/core';
import { platform } from '@tauri-apps/plugin-os';
import { Button, SlideDeck, Toggle } from 'components';
import { onMount } from 'svelte';
import { type AccessibilityPermissionStatus, type BrowserId, Client, type Integration } from '$lib/client';
import {
	cancelHumanizerDownload,
	formatGb,
	humanizer,
	refreshHumanizer,
	startHumanizerDownload,
	watchHumanizer,
} from '$lib/humanizer';
import AppIcon from '../components/AppIcon.svelte';
import AppPickerModal from '../components/AppPickerModal.svelte';

/** Slide identities determine setup gates; only the welcome and test drive need no action. */
type OnboardingSlide = {
	id: 'welcome' | 'accessibility' | 'apps' | 'browser' | 'humanizer' | 'test-drive' | 'ready';
	title: string;
	lede: string;
};

const allSlides: OnboardingSlide[] = [
	{
		id: 'welcome',
		title: 'Welcome to Gramflow',
		lede: "Before you can start writing with Gramflow, we need to do a little housekeeping.\n\nDon't worry, this should only take a minute.",
	},
	{
		id: 'accessibility',
		title: 'Accessibility',
		lede: 'To be able to read and write text to your favorite text editors, you need to grant Gramflow the permission to do so.\n\nNone of your text will leave your device.',
	},
	{
		id: 'apps',
		title: 'Choose your apps',
		lede: 'Gramflow checks your writing in the apps you switch on. Notes, Mail, Messages and TextEdit are ready by default.\n\nTurn any off or add your own. Click Next to keep what you see; you can change it later in Settings → Integrations.',
	},
	{
		id: 'browser',
		title: 'Add the browser extension',
		lede: 'To check your writing on websites and in webmail, add the Gramflow extension to Chrome, Edge, Brave, Opera or Firefox.\n\nThis step is optional. Click Next to skip it and add the extension later from Settings → Integrations.',
	},
	{
		id: 'humanizer',
		title: 'Rewrite in your voice',
		lede: 'Humanizer rewrites AI-sounding text so it reads like you wrote it. It runs on this computer, so nothing you write leaves it.\n\nThis step is optional. Click Next to skip it and set it up later in Settings → Humanizer.',
	},
	{
		id: 'test-drive',
		title: 'Try Gramflow',
		lede: 'Open TextEdit and write something like, "This is an test."\n\nYou should see Gramflow jump in to fix that mistake.',
	},
	{
		id: 'ready',
		title: 'Ready',
		lede: "Okay! Now we're ready to go.\n\nDon't let your dreams be dreams. Write anything, anywhere and Gramflow will be there to catch your mistakes.",
	},
];

export let onComplete: () => void;

let step = 0;
let selectedTier = '';
let isMacOS = false;
let accessibilityStatus: AccessibilityPermissionStatus | null = null;
let setupError = '';
let copiedResetCommand = false;

/** Clears a stale macOS Accessibility entry (for example after an update changed the app's signature). */
const RESET_COMMAND = 'tccutil reset Accessibility com.gramflow.app';

async function copyResetCommand() {
	try {
		await navigator.clipboard.writeText(RESET_COMMAND);
		copiedResetCommand = true;
		setTimeout(() => (copiedResetCommand = false), 2000);
	} catch (error) {
		console.error('Unable to copy the reset command.', error);
	}
}
let isPreparingService = true;
let hasRequestedAccessibility = false;
let integrations: Integration[] = [];
let integrationsError = '';
let isLoadingIntegrations = true;
let extensionFolder = '';
let firefoxFolder = '';
let isPreparingExtension = false;
let extensionError = '';
let isSavingApps = false;
let appPickerOpen = false;
let newBundleId = '';
let isLaunchingTextEdit = false;
let testDriveError = '';
let isCompletingOnboarding = false;
let onboardingError = '';

$: slides = allSlides.filter((slide) => slide.id !== 'accessibility' || isMacOS);
$: humanizerStatus = $humanizer.status;
$: humanizerVerdict = humanizerStatus?.verdict;
$: humanizerOptions = humanizerVerdict?.options ?? [];
$: if (!selectedTier && humanizerOptions.length) {
	selectedTier =
		humanizerOptions.find((option) => option.installed && option.id === humanizerStatus?.active)?.id ??
		humanizerOptions.find((option) => option.installed)?.id ??
		humanizerOptions.find((option) => option.recommended)?.id ??
		'';
}
$: selectedOption = humanizerOptions.find((option) => option.id === selectedTier);
$: selectedIsActive = Boolean(selectedOption?.installed && selectedOption.id === humanizerStatus?.active);
$: humanizerBusy = humanizerStatus?.downloading ?? null;
$: humanizerInstalled = humanizerOptions.some((option) => option.installed);
/** The Humanizer slide speaks to this computer: its words change with what the check finds. */
$: current = humanizerSlide(slides[step], humanizerVerdict);
$: textEditIntegration = integrations.find((item) => item.bundle_id === 'com.apple.TextEdit');
$: accessibilityReady =
	(!isMacOS || accessibilityStatus === 'Granted') && !isPreparingService && !setupError;
$: existingBundleIds = integrations.map((item) => item.bundle_id);
$: nextDisabled = !canAdvance(slides[step].id, accessibilityReady, isMacOS);

onMount(() => {
	// Use the native platform in Tauri; user-agent detection is only for browser previews.
	isMacOS = isTauri() ? platform() === 'macos' : navigator.userAgent.includes('Macintosh');
	void prepareService();
	void loadIntegrations();
	void refreshHumanizer();
	let stopWatchingHumanizer: (() => void) | undefined;
	void watchHumanizer().then((stop) => (stopWatchingHumanizer = stop));

	// Watch for the permission switch being turned on, so nobody has to press a recheck button.
	const watcher = window.setInterval(() => void watchAccessibility(), 1000);
	window.addEventListener('focus', watchAccessibility);
	return () => {
		window.clearInterval(watcher);
		window.removeEventListener('focus', watchAccessibility);
		stopWatchingHumanizer?.();
	};
});

let isWatchingAccessibility = false;

/** Quietly polls macOS; once access is on, starts the service and moves to the next step. */
async function watchAccessibility() {
	if (
		isWatchingAccessibility ||
		isPreparingService ||
		!isMacOS ||
		accessibilityReady ||
		slides[step]?.id !== 'accessibility'
	) {
		return;
	}

	isWatchingAccessibility = true;
	try {
		if ((await Client.getAccessibilityPermissionStatus()) !== 'Granted') return;
		await prepareService();
		if (accessibilityReady && slides[step]?.id === 'accessibility') {
			// Let the "granted" message show for a moment before moving on.
			await new Promise((resolve) => setTimeout(resolve, 900));
			if (slides[step]?.id === 'accessibility') step++;
		}
	} catch (error) {
		console.error('Unable to check Accessibility permission.', error);
	} finally {
		isWatchingAccessibility = false;
	}
}

/** Gate service startup on Welcome outside macOS, or on Accessibility on macOS; choosing apps and the trial are optional. */
function humanizerSlide(slide: OnboardingSlide, verdict: typeof humanizerVerdict): OnboardingSlide {
	if (slide.id !== 'humanizer' || !verdict || verdict.level !== 'Unavailable') return slide;

	return {
		...slide,
		title: "Humanizer isn't available on this computer",
		lede: `${verdict.detail}\n\nEverything else in Gramflow works as normal. Click Next to continue.`,
	};
}

function canAdvance(
	slide: OnboardingSlide['id'],
	accessReady: boolean,
	needsPermission: boolean,
) {
	if (slide === 'welcome') return needsPermission || accessReady;
	if (slide === 'accessibility') return accessReady;
	return accessReady;
}

async function loadIntegrations() {
	isLoadingIntegrations = true;
	integrationsError = '';

	try {
		integrations = await Client.getIntegrations();
	} catch (error) {
		integrationsError = `Unable to load integrations: ${error}`;
	} finally {
		isLoadingIntegrations = false;
	}
}

async function setAppEnabled(bundleId: string, enabled: boolean) {
	if (isSavingApps) return;
	isSavingApps = true;
	integrationsError = '';

	try {
		await Client.setIntegrationEnabled(bundleId, enabled);
		await loadIntegrations();
	} catch (error) {
		integrationsError = `Unable to update the app: ${error}`;
	} finally {
		isSavingApps = false;
	}
}

async function addApp(bundleId: string) {
	const id = bundleId.trim();
	if (!id || isSavingApps || existingBundleIds.includes(id)) return;
	isSavingApps = true;
	integrationsError = '';

	try {
		await Client.addIntegration(id);
		await loadIntegrations();
		closeAppPicker();
	} catch (error) {
		integrationsError = `Unable to add the app: ${error}`;
	} finally {
		isSavingApps = false;
	}
}

function closeAppPicker() {
	appPickerOpen = false;
	newBundleId = '';
}

/** Make sure TextEdit is watched so the test drive works even if the user turned it off. */
async function ensureTextEditEnabled() {
	if (textEditIntegration?.enabled) return;
	if (textEditIntegration) {
		await Client.setIntegrationEnabled('com.apple.TextEdit', true);
	} else {
		await Client.addIntegration('com.apple.TextEdit');
	}
	await loadIntegrations();
}

async function prepareExtension() {
	if (isPreparingExtension) return;
	isPreparingExtension = true;
	extensionError = '';

	try {
		extensionFolder = await Client.prepareBrowserExtension();
	} catch (error) {
		extensionError = `${error}`;
	} finally {
		isPreparingExtension = false;
	}
}

async function prepareFirefox() {
	extensionError = '';

	try {
		firefoxFolder = await Client.prepareFirefoxExtension();
		await Client.openBrowserExtensionsPage('firefox');
	} catch (error) {
		extensionError = `${error}`;
	}
}

async function openExtensionsPage(browser: BrowserId) {
	extensionError = '';

	try {
		await Client.openBrowserExtensionsPage(browser);
	} catch (error) {
		extensionError = `${error}`;
	}
}

async function launchTextEditForTestDrive() {
	if (!accessibilityReady || isLaunchingTextEdit) return;
	isLaunchingTextEdit = true;
	testDriveError = '';

	try {
		await ensureTextEditEnabled();
		await Client.launchApp('com.apple.TextEdit');
	} catch (error) {
		testDriveError = `Unable to launch TextEdit: ${error}`;
	} finally {
		isLaunchingTextEdit = false;
	}
}

/** Persist completion only after mandatory setup; leave the deck only when the save succeeds. */
/** Switches to the chosen model when it is already downloaded. */
async function useSelectedModel() {
	if (!selectedTier) return;
	try {
		await Client.humanizerSetActiveModel(selectedTier);
		await refreshHumanizer();
	} catch (error) {
		onboardingError = `${error}`;
	}
}

/** Remembers the choice so Settings still offers setup, then moves on. Nothing is downloaded. */
async function skipHumanizer() {
	if (humanizerStatus && !humanizerInstalled) {
		try {
			await Client.humanizerSaveSettings({ ...humanizerStatus.settings, skipped: true });
		} catch (error) {
			console.error('Unable to save the Humanizer choice.', error);
		}
	}
	step++;
}

async function completeOnboarding() {
	if (!accessibilityReady || isCompletingOnboarding) return;
	isCompletingOnboarding = true;
	onboardingError = '';

	try {
		await Client.setOnboardingCompleted(true);
		onComplete();
	} catch (error) {
		onboardingError = `Unable to complete onboarding: ${error}`;
	} finally {
		isCompletingOnboarding = false;
	}
}

/** Start Gramflow on every platform, checking or requesting Accessibility permission only on macOS. */
async function prepareService(request = false) {
	isPreparingService = true;
	setupError = '';
	accessibilityStatus = null;

	try {
		if (isMacOS) {
			if (request) {
				accessibilityStatus = await Client.requestAccessibilityPermission();
				hasRequestedAccessibility = true;
			} else {
				accessibilityStatus = await Client.getAccessibilityPermissionStatus();
			}
		}

		if (
			(!isMacOS || accessibilityStatus === 'Granted') &&
			!(await Client.startHighlighterService())
		) {
			throw new Error('The Gramflow service did not start. Please try again.');
		}
	} catch (error) {
		setupError = `Unable to ${isMacOS ? 'set up Accessibility' : 'start Gramflow'}: ${error}`;
	} finally {
		isPreparingService = false;
	}
}
</script>

<div class="onboarding-shell">
  <SlideDeck
    fullBleed
    title={current.title}
    lede={current.lede}
    slideProgress={step / (slides.length - 1)}
    {nextDisabled}
    onFinish={completeOnboarding}
    finishDisabled={isCompletingOnboarding || !accessibilityReady}
    finishLabel={isCompletingOnboarding ? 'Finishing...' : 'Finish'}
    onBack={() => {
      if (!isCompletingOnboarding) step--;
    }}
    onNext={() => step++}
  >
    {#if slides[step].id === 'welcome' && !isMacOS}
      {#if isPreparingService}
        <div class="onboarding-actions">
          <p role="status">Starting Gramflow...</p>
        </div>
      {:else if setupError}
        <div class="onboarding-actions">
          <p role="alert">{setupError}</p>
          <Button on:click={() => prepareService()}>Retry Starting Gramflow</Button>
        </div>
      {/if}
    {:else if slides[step].id === 'accessibility'}
      <div class="onboarding-actions">
        <p role="status">
          {#if isPreparingService}
            Checking Accessibility access...
          {:else if accessibilityReady}
            Accessibility access granted. Continuing...
          {:else if accessibilityStatus === 'Unsupported'}
            Accessibility setup is unavailable on this platform. Setup cannot continue here.
          {:else if hasRequestedAccessibility && accessibilityStatus === 'NotGranted'}
            Switch Gramflow on in System Settings → Privacy &amp; Security → Accessibility. Gramflow will notice and continue by itself.
          {/if}
        </p>
        {#if setupError}
          <p role="alert">{setupError}</p>
        {/if}
        <Button
          disabled={isPreparingService || accessibilityReady || accessibilityStatus === 'Unsupported'}
          on:click={() => prepareService(!hasRequestedAccessibility && accessibilityStatus !== 'Granted')}
        >
          {#if isPreparingService}
            Checking...
          {:else if accessibilityReady}
            Granted
          {:else if accessibilityStatus === 'Unsupported'}
            Unavailable
          {:else if hasRequestedAccessibility || accessibilityStatus === 'Granted'}
            Open System Settings again
          {:else}
            Open System Settings
          {/if}
        </Button>

        {#if isMacOS && !accessibilityReady && !isPreparingService}
          <details class="help-details" open={hasRequestedAccessibility && accessibilityStatus === 'NotGranted'}>
            <summary>Already turned it on, but Gramflow can't tell?</summary>
            <p>
              macOS sometimes keeps an old Gramflow entry that no longer matches this version. Reset it,
              then grant access again:
            </p>
            <ol>
              <li>Quit Gramflow from its menu-bar icon.</li>
              <li>Open Terminal and run this command:</li>
            </ol>
            <div class="command-row">
              <code>{RESET_COMMAND}</code>
              <Button type="button" color="white" size="sm" on:click={copyResetCommand}>
                {copiedResetCommand ? 'Copied' : 'Copy'}
              </Button>
            </div>
            <ol start="3">
              <li>Open Gramflow again and click Open System Settings.</li>
              <li>Switch Gramflow on. Gramflow notices and continues by itself.</li>
            </ol>
          </details>
        {/if}
      </div>
    {:else if slides[step].id === 'apps'}
      <div class="onboarding-actions onboarding-apps">
        <p role="status">
          {#if isLoadingIntegrations}
            Loading your apps...
          {:else if isSavingApps}
            Saving...
          {/if}
        </p>
        {#if integrationsError}
          <p role="alert">{integrationsError}</p>
          <Button color="white" disabled={isLoadingIntegrations || isSavingApps} on:click={loadIntegrations}>
            Retry Loading Apps
          </Button>
        {/if}
        <ul class="onboarding-app-list">
          {#each integrations as app (app.bundle_id)}
            <li class="onboarding-app">
              <AppIcon bundleId={app.bundle_id} name={app.display_name} />
              <strong>{app.display_name}</strong>
              <Toggle
                appearance="settings"
                checked={app.enabled}
                disabled={isLoadingIntegrations || isSavingApps}
                aria-label={`Check writing in ${app.display_name}`}
                on:click={() => setAppEnabled(app.bundle_id, !app.enabled)}
              />
            </li>
          {/each}
        </ul>
        <div class="onboarding-app-actions">
          <Button color="white" disabled={isLoadingIntegrations || isSavingApps} on:click={() => (appPickerOpen = true)}>
            Add application...
          </Button>
        </div>
      </div>
    {:else if slides[step].id === 'browser'}
      <div class="onboarding-actions">
        <div class="onboarding-browser-columns">
        <section class="onboarding-browser-group">
          <h3>Chrome, Edge, Brave and Opera</h3>
          <ol class="onboarding-steps">
            <li>
              <Button disabled={isPreparingExtension} on:click={prepareExtension}>
                {isPreparingExtension ? 'Preparing...' : extensionFolder ? 'Show the folder again' : 'Get the extension folder'}
              </Button>
            </li>
            <li>
              Open your browser's extensions page and switch on <strong>Developer mode</strong>.
              <span class="onboarding-browser-buttons">
                <Button color="white" size="sm" on:click={() => openExtensionsPage('chrome')}>Chrome</Button>
                <Button color="white" size="sm" on:click={() => openExtensionsPage('edge')}>Edge</Button>
                <Button color="white" size="sm" on:click={() => openExtensionsPage('brave')}>Brave</Button>
                <Button color="white" size="sm" on:click={() => openExtensionsPage('opera')}>Opera</Button>
              </span>
            </li>
            <li>Click <strong>Load unpacked</strong> and choose the <strong>Gramflow Browser Extension</strong> folder{extensionFolder ? ' in Documents' : ''}.</li>
          </ol>
        </section>
        <section class="onboarding-browser-group">
          <h3>Firefox</h3>
          <ol class="onboarding-steps">
            <li>
              <Button color="white" size="sm" on:click={prepareFirefox}>Set up Firefox</Button>
            </li>
            <li>On the page that opens, click <strong>Load Temporary Add-on…</strong> and choose <strong>manifest.json</strong> inside the <strong>Gramflow Firefox Extension</strong> folder{firefoxFolder ? ' in Documents' : ''}.</li>
            <li>Firefox forgets the extension when it quits, so repeat step 2 after restarting it.</li>
          </ol>
        </section>
        </div>
        {#if extensionError}
          <p role="alert">{extensionError}</p>
        {/if}
      </div>
    {:else if slides[step].id === 'humanizer'}
      <div class="onboarding-actions onboarding-apps">
        {#if $humanizer.loading}
          <p role="status">Checking this computer...</p>
        {:else if humanizerVerdict && humanizerVerdict.level !== 'Unavailable'}
          <p role="status">
            {humanizerVerdict.headline}.
            {humanizerVerdict.detail}
          </p>
          <ul class="onboarding-app-list">
            {#each humanizerOptions as option (option.id)}
              {@const blocked = !option.fits_memory || (!option.fits_disk && !option.installed)}
              <li class="onboarding-app">
                <label class="grow">
                  <input
                    type="radio"
                    name="humanizer-tier"
                    value={option.id}
                    bind:group={selectedTier}
                    disabled={blocked || humanizerBusy !== null}
                  />
                  <strong>{option.label}</strong>
                  {#if option.recommended}<span class="pill amber">Recommended</span>{/if}
                  {#if option.installed}<span class="pill">Installed</span>{/if}
                  <p>
                    {formatGb(option.size_bytes)} download
                    {#if !option.fits_memory} · needs {option.min_ram_gb} GB of memory
                    {:else if !option.fits_disk && !option.installed} · not enough free disk space
                    {:else} · {option.blurb}{/if}
                  </p>
                </label>
              </li>
            {/each}
          </ul>
          {#if humanizerBusy && $humanizer.progress}
            <p role="status">
              Downloading… {formatGb($humanizer.progress.downloaded)} of {formatGb($humanizer.progress.total)}.
              You can keep going; the download continues in the background.
            </p>
          {:else if humanizerBusy}
            <p role="status">Starting the download…</p>
          {:else if humanizerInstalled}
            <p role="status">Humanizer is ready. Find it in Settings → Humanizer.</p>
          {/if}
          {#if $humanizer.error}
            <p role="alert">{$humanizer.error}</p>
          {/if}
          <div class="onboarding-app-actions">
            {#if humanizerBusy}
              <Button color="white" on:click={cancelHumanizerDownload}>Cancel download</Button>
            {:else}
              {#if selectedIsActive}
                <Button disabled>Ready to use</Button>
              {:else if selectedOption?.installed}
                <Button on:click={useSelectedModel}>Use this model</Button>
              {:else}
                <Button disabled={!selectedTier} on:click={() => startHumanizerDownload(selectedTier)}>Download</Button>
              {/if}
              <Button color="white" on:click={skipHumanizer}>Skip for now</Button>
            {/if}
          </div>
        {:else if $humanizer.error}
          <p role="alert">{$humanizer.error}</p>
          <Button color="white" on:click={refreshHumanizer}>Try again</Button>
        {/if}
      </div>
    {:else if slides[step].id === 'test-drive'}
      <div class="onboarding-actions">
        {#if testDriveError}
          <p role="alert">{testDriveError}</p>
        {/if}
        <Button disabled={isLaunchingTextEdit} on:click={launchTextEditForTestDrive}>
          {isLaunchingTextEdit ? 'Launching...' : 'Launch TextEdit'}
        </Button>
      </div>
    {:else if slides[step].id === 'ready'}
      <div class="onboarding-actions">
        {#if onboardingError}
          <p role="alert">{onboardingError}</p>
        {/if}
      </div>
    {/if}
  </SlideDeck>
</div>

{#if appPickerOpen}
  <AppPickerModal
    bind:bundleId={newBundleId}
    {existingBundleIds}
    isSaving={isSavingApps}
    close={closeAppPicker}
    add={addApp}
  />
{/if}
