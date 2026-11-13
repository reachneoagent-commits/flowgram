<script lang="ts">
import { Button, IconButton, Panel, Select, SettingRow, Toggle, TrashIcon } from 'components';
import { onMount } from 'svelte';
import { Client, type HumanizerOption, type HumanizerSettings } from '$lib/client';
import {
	cancelHumanizerDownload,
	formatGb,
	humanizer,
	refreshHumanizer,
	startHumanizerDownload,
	watchHumanizer,
} from '$lib/humanizer';
import ConfirmDialog from '../components/ConfirmDialog.svelte';

const IDLE_CHOICES = [
	{ value: '5', label: '5 minutes' },
	{ value: '10', label: '10 minutes' },
	{ value: '30', label: '30 minutes' },
	{ value: '60', label: '1 hour' },
	{ value: '0', label: 'Never' },
];

let pendingDelete: HumanizerOption | null = null;
let pendingRollback: HumanizerOption | null = null;
let isChecking = false;
let checkMessage = '';
let actionError = '';
let isSaving = false;
const isMac = typeof navigator !== 'undefined' && navigator.userAgent.includes('Macintosh');
const shortcutLabel = isMac ? 'Control + Command + H' : 'Control + Alt + H';

onMount(() => {
	void refreshHumanizer();
	let stop: (() => void) | undefined;
	void watchHumanizer().then((unlisten) => (stop = unlisten));
	return () => stop?.();
});

$: status = $humanizer.status;
$: verdict = status?.verdict;
$: available = verdict !== undefined && verdict.level !== 'Unavailable';
$: installedCount = verdict?.options.filter((option) => option.installed).length ?? 0;
$: busyDownload = status?.downloading ?? null;

function levelLabel(level: string) {
	return level === 'Ready' ? 'Ready' : level === 'Limited' ? 'Limited' : 'Not available';
}

function whyDisabled(option: HumanizerOption): string {
	if (!option.fits_memory) return `Needs ${option.min_ram_gb} GB of memory.`;
	if (!option.fits_disk && !option.installed) return 'Not enough free disk space.';
	return '';
}

async function run(action: () => Promise<void>) {
	actionError = '';
	try {
		await action();
	} catch (error) {
		actionError = `${error}`;
	} finally {
		await refreshHumanizer();
	}
}

async function saveSettings(patch: Partial<HumanizerSettings>) {
	if (!status || isSaving) return;
	isSaving = true;
	await run(() => Client.humanizerSaveSettings({ ...status.settings, ...patch }));
	isSaving = false;
}

function timeAgo(seconds: number | null): string {
	if (!seconds) return 'never';
	const minutes = Math.max(0, Math.round((Date.now() / 1000 - seconds) / 60));
	if (minutes < 1) return 'just now';
	if (minutes < 60) return `${minutes} minute${minutes === 1 ? '' : 's'} ago`;
	const hours = Math.round(minutes / 60);
	if (hours < 48) return `${hours} hour${hours === 1 ? '' : 's'} ago`;
	return `${Math.round(hours / 24)} days ago`;
}

function labelFor(tier: string): string {
	return verdict?.options.find((option) => option.id === tier)?.label ?? tier;
}

const PHASES: Record<string, string> = {
	downloading: 'Downloading the new version',
	testing: 'Testing the new version',
	installing: 'Installing',
};

async function checkNow() {
	if (isChecking) return;
	isChecking = true;
	checkMessage = '';
	try {
		const found = await Client.humanizerCheckUpdates();
		checkMessage = found.length ? '' : 'Humanizer is up to date.';
	} catch (error) {
		checkMessage = `${error}`;
	} finally {
		isChecking = false;
		await refreshHumanizer();
	}
}

async function applyUpdate(tier: string) {
	actionError = '';
	try {
		await Client.humanizerApplyUpdate(tier);
	} catch (error) {
		actionError = `${error}`;
	} finally {
		await refreshHumanizer();
	}
}

async function confirmRollback() {
	const option = pendingRollback;
	pendingRollback = null;
	if (option) await run(() => Client.humanizerRollback(option.id));
}

async function confirmDelete() {
	const option = pendingDelete;
	pendingDelete = null;
	if (option) await run(() => Client.humanizerDeleteModel(option.id));
}
</script>

<section>
  {#if $humanizer.loading}
    <p class="result-summary" role="status">Checking this computer...</p>
  {:else if verdict && status}
    <div class="stanza">
      <div class="eyebrow">This computer</div>
      <Panel>
        <div class="app-row">
          <div class="grow">
            <strong>{verdict.headline}</strong>
            <p>
              {verdict.detail || 'Rewrites run on this computer. Nothing you write leaves it.'}
            </p>
            <p>
              {Math.round(verdict.facts.total_ram_gb)} GB memory · {Math.round(verdict.facts.free_disk_gb)} GB free disk
            </p>
          </div>
          <span class="pill" class:amber={verdict.level !== 'Unavailable'}>{levelLabel(verdict.level)}</span>
        </div>
      </Panel>
      {#if available && !status.engine_installed}
        <p class="result-summary" role="alert">
          The rewriting engine isn't included in this build, so models can be managed but not used yet.
        </p>
      {/if}
    </div>

    {#if available}
      <div class="divider"></div>
      <div class="stanza">
        <div class="eyebrow">Models</div>
        <p class="section-copy">
          Download the size that suits this computer. You can keep more than one and switch any time.
          Models are stored on this computer and can be deleted whenever you like.
        </p>
        <Panel>
          {#each verdict.options as option (option.id)}
            {@const blocked = whyDisabled(option)}
            {@const isActive = status.active === option.id}
            {@const isDownloading = busyDownload === option.id}
            <div class="app-row">
              <div class="grow">
                <strong>{option.label}</strong>
                {#if option.recommended}<span class="pill amber">Recommended</span>{/if}
                {#if isActive}<span class="pill">In use</span>{/if}
                <p>{formatGb(option.size_bytes)} · {option.blurb}</p>
                {#if blocked}
                  <p>{blocked}</p>
                {:else if isDownloading && $humanizer.progress}
                  <div class="progress" role="progressbar" aria-label={`Downloading ${option.label}`} aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(($humanizer.progress.downloaded / Math.max($humanizer.progress.total, 1)) * 100)}>
                    <div style:width={`${Math.min(100, ($humanizer.progress.downloaded / Math.max($humanizer.progress.total, 1)) * 100)}%`}></div>
                  </div>
                  <p>{formatGb($humanizer.progress.downloaded)} of {formatGb($humanizer.progress.total)}</p>
                {:else if isDownloading}
                  <p>Starting the download...</p>
                {/if}
              </div>
              {#if isDownloading}
                <Button color="white" size="sm" on:click={cancelHumanizerDownload}>Cancel</Button>
              {:else if option.installed}
                {#if !isActive}
                  <Button color="white" size="sm" on:click={() => run(() => Client.humanizerSetActiveModel(option.id))}>Use this one</Button>
                {/if}
                <IconButton
                  danger
                  disabled={busyDownload !== null}
                  aria-label={`Delete the ${option.label} model`}
                  on:click={() => (pendingDelete = option)}
                >
                  <TrashIcon className="control-icon" />
                </IconButton>
              {:else}
                <Button
                  color={option.recommended && installedCount === 0 ? 'primary' : 'white'}
                  size="sm"
                  disabled={Boolean(blocked) || busyDownload !== null}
                  on:click={() => startHumanizerDownload(option.id)}
                >
                  Download
                </Button>
              {/if}
            </div>
          {/each}
        </Panel>
        {#if $humanizer.error}
          <p class="result-summary" role="alert">{$humanizer.error}</p>
        {/if}
        {#if actionError}
          <p class="result-summary" role="alert">{actionError}</p>
        {/if}
      </div>


      {#if installedCount > 0}
        <div class="divider"></div>
        <div class="stanza">
          <div class="eyebrow">Updates</div>
          <p class="section-copy">
            Gramflow checks a list of Humanizer versions we've reviewed and approved. Nothing is installed without your click, and the version you have is kept so you can go back.
          </p>
          <div class="rows">
            <SettingRow top>
              <strong>Check for updates automatically</strong>
              <p>About once a day. It only looks at our approved list.</p>
              <Toggle slot="control" appearance="settings"
                checked={status.settings.auto_check_updates}
                aria-label="Check for updates automatically"
                disabled={isSaving}
                on:click={() => saveSettings({ auto_check_updates: !status.settings.auto_check_updates })}
               />
            </SettingRow>
            <div class="inline-row">
              <Button color="white" size="sm" disabled={isChecking || busyDownload !== null} on:click={checkNow}>
                {isChecking ? 'Checking...' : 'Check now'}
              </Button>
              <span>Last checked {timeAgo(status.settings.last_update_check)}.</span>
            </div>
            {#if checkMessage}
              <p class="result-summary" role="status">{checkMessage}</p>
            {:else if status.update_error}
              <p class="result-summary" role="alert">{status.update_error}</p>
            {/if}
          </div>

          {#if status.updates.length}
            <Panel>
              {#each status.updates as update (update.tier)}
                {@const working = $humanizer.updatePhase?.tier === update.tier || busyDownload === update.tier}
                <div class="app-row">
                  <div class="grow">
                    <strong>{labelFor(update.tier)}</strong>
                    <span class="pill amber">Update available</span>
                    <p>
                      {update.from_version} → {update.to_version} · released {update.released} · {formatGb(update.size_bytes)} download
                    </p>
                    <p>{update.notes}</p>
                    {#if update.needs_newer_app}
                      <p>This version needs a newer Gramflow. Update the app first.</p>
                    {:else if working}
                      <p role="status">{PHASES[$humanizer.updatePhase?.phase ?? 'downloading'] ?? 'Working'}...</p>
                      {#if $humanizer.progress && $humanizer.progress.tier === update.tier && $humanizer.updatePhase?.phase !== 'testing'}
                        <div class="progress" role="progressbar" aria-label={`Downloading the ${labelFor(update.tier)} update`} aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(($humanizer.progress.downloaded / Math.max($humanizer.progress.total, 1)) * 100)}>
                          <div style:width={`${Math.min(100, ($humanizer.progress.downloaded / Math.max($humanizer.progress.total, 1)) * 100)}%`}></div>
                        </div>
                      {/if}
                    {/if}
                  </div>
                  {#if working}
                    <Button color="white" size="sm" disabled={$humanizer.updatePhase?.phase === 'installing'} on:click={cancelHumanizerDownload}>Cancel</Button>
                  {:else}
                    <Button size="sm" disabled={update.needs_newer_app || busyDownload !== null} on:click={() => applyUpdate(update.tier)}>Update</Button>
                  {/if}
                </div>
              {/each}
            </Panel>
          {/if}

          {#each verdict.options.filter((option) => status.rollback.includes(option.id)) as option (option.id)}
            <div class="inline-row">
              <Button color="white" size="sm" disabled={busyDownload !== null} on:click={() => (pendingRollback = option)}>
                Go back to the previous {option.label} version
              </Button>
            </div>
          {/each}
        </div>
      {/if}

      {#if installedCount > 0}
        <div class="divider"></div>
        <div class="stanza">
          <div class="eyebrow">Use it</div>
          <p class="section-copy">
            Select text in any app and press <kbd>{shortcutLabel}</kbd>. Gramflow rewrites it and offers to replace your selection. You can also paste a draft in the Humanize tab of the editor.
          </p>
          <div class="inline-row">
            <Button size="sm" on:click={() => run(Client.humanizerOpenWindow)}>Open Humanize in the editor</Button>
          </div>
        </div>

        <div class="divider"></div>
        <div class="stanza">
          <div class="eyebrow">Options</div>
          <div class="rows">
            <SettingRow top>
              <strong>Warn when numbers change</strong>
              <p>Flag a rewrite that drops or adds a number, so facts don't slip through.</p>
              <Toggle slot="control" appearance="settings"
                checked={status.settings.warn_numbers}
                aria-label="Warn when numbers change"
                disabled={isSaving}
                on:click={() => saveSettings({ warn_numbers: !status.settings.warn_numbers })}
               />
            </SettingRow>
            <div class="inline-row">
              <label for="humanizer-idle">Free memory after:</label>
              <Select
                unstyled
                id="humanizer-idle"
                class="select wide"
                disabled={isSaving}
                value={String(status.settings.idle_minutes)}
                on:change={(event) => saveSettings({ idle_minutes: Number((event.detail.currentTarget as HTMLSelectElement).value) })}
              >
                {#each IDLE_CHOICES as choice}
                  <option value={choice.value}>{choice.label}</option>
                {/each}
              </Select>
              <span>of no use</span>
            </div>
            <div class="inline-row">
              <Button color="white" size="sm" on:click={() => run(Client.humanizerOpenModelsFolder)}>Show models folder</Button>
              {#if status.engine_running}
                <Button color="white" size="sm" on:click={() => run(Client.humanizerStopEngine)}>Free memory now</Button>
              {/if}
            </div>
          </div>
        </div>
      {/if}
    {/if}
  {:else}
    <p class="result-summary" role="alert">{$humanizer.error || "Couldn't check this computer."}</p>
    <div class="inline-row">
      <Button color="white" size="sm" on:click={refreshHumanizer}>Try again</Button>
    </div>
  {/if}
</section>

{#if pendingRollback}
  <ConfirmDialog
    title={`Go back to the previous ${pendingRollback.label} version?`}
    message="Gramflow switches to the version you had before the last update. You can switch back again afterwards."
    confirmLabel="Go back"
    cancelLabel="Keep current"
    onCancel={() => (pendingRollback = null)}
    onConfirm={confirmRollback}
  />
{/if}

{#if pendingDelete}
  <ConfirmDialog
    title={`Delete the ${pendingDelete.label} model?`}
    message={`This frees ${formatGb(pendingDelete.size_bytes)} of disk space. You can download it again later.${installedCount === 1 ? ' Humanizer stops working until you download a model.' : ''}`}
    confirmLabel="Delete"
    cancelLabel="Keep"
    onCancel={() => (pendingDelete = null)}
    onConfirm={confirmDelete}
  />
{/if}

<style>
  .progress {
    height: 6px;
    margin-top: 8px;
    border-radius: 3px;
    background: var(--settings-line-strong);
    overflow: hidden;
  }
  .progress > div {
    height: 100%;
    background: var(--settings-accent);
    transition: width 0.25s linear;
  }
  @media (prefers-reduced-motion: reduce) {
    .progress > div {
      transition: none;
    }
  }
</style>
