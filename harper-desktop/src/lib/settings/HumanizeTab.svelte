<script lang="ts">
import { Button, Select } from 'components';
import { onMount } from 'svelte';
import { Client } from '$lib/client';
import { formatGb, humanizer, refreshHumanizer } from '$lib/humanizer';
import HumanizeWorkspace from './components/HumanizeWorkspace.svelte';

let switching = false;
let error = '';

onMount(() => {
	void refreshHumanizer();
});

$: status = $humanizer.status;
$: installed = status?.verdict.options.filter((option) => option.installed) ?? [];
$: active = installed.find((option) => option.id === status?.active);

async function switchModel(id: string) {
	if (switching || id === status?.active) return;
	switching = true;
	error = '';
	try {
		await Client.humanizerSetActiveModel(id);
	} catch (failure) {
		error = `${failure}`;
	} finally {
		switching = false;
		await refreshHumanizer();
	}
}
</script>

<div class="hz-tab">
  <main aria-label="Humanize">
    <header class="hz-bar">
      <div class="hz-bar-model">
        {#if active}
          <span class="hz-badge">{status?.engine_running ? 'Loaded' : 'Ready'}</span>
          <strong>{active.label}</strong>
          <span class="hz-bar-size">{formatGb(active.size_bytes)}</span>
        {:else}
          <span class="hz-badge hz-badge-off">Not set up</span>
        {/if}
      </div>
      <div class="hz-bar-actions">
        {#if installed.length > 1}
          <label for="humanize-model" class="hz-bar-label">Model</label>
          <Select
            unstyled
            id="humanize-model"
            class="hz-select"
            disabled={switching}
            value={status?.active ?? ''}
            on:change={(event) => switchModel((event.detail.currentTarget as HTMLSelectElement).value)}
          >
            {#each installed as option}
              <option value={option.id}>{option.label}</option>
            {/each}
          </Select>
        {/if}
        {#if status?.engine_running}
          <Button color="white" size="sm" on:click={() => Client.humanizerStopEngine().then(refreshHumanizer)}>Free memory</Button>
        {/if}
        <Button color="white" size="sm" on:click={() => Client.humanizerOpenSettings()}>Models and settings</Button>
      </div>
    </header>
    {#if error}
      <p class="hz-error" role="alert">{error}</p>
    {/if}
    <HumanizeWorkspace tall onSetup={() => Client.humanizerOpenSettings()} />
  </main>
</div>

<style>
  .hz-tab {
    --hz-ink: #2b2b2b;
    --hz-ink-2: #6b6b63;
    --hz-bg: #f5f5f0;
    box-sizing: border-box;
    min-height: 100%;
    padding: 20px 32px 28px;
    background: var(--hz-bg);
    color: var(--hz-ink);
  }
  :global(.dark) .hz-tab {
    --hz-ink: #eef1dc;
    --hz-ink-2: #a9ad96;
    --hz-bg: #15160e;
  }
  /* The editor's own stylesheet loads after the shared buttons' and wins the background in dark
     mode, which turns "white" buttons into bright blocks. Put the dark background back. */
  :global(.dark .hz-tab button[class*="dark:bg-gray-800"]) {
    background-color: var(--color-gray-800);
  }
  :global(.dark .hz-tab button[class*="dark:bg-gray-800"]:hover:not(:disabled)) {
    background-color: var(--color-gray-700);
  }
  :global(.hz-select) {
    padding: 6px 28px 6px 10px;
    border: 1px solid color-mix(in srgb, currentColor 22%, transparent);
    border-radius: 8px;
    background-color: color-mix(in srgb, currentColor 6%, transparent);
    color: inherit;
    font: inherit;
    font-size: 13px;
  }
  .hz-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }
  .hz-bar-model,
  .hz-bar-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .hz-bar-size,
  .hz-bar-label {
    color: var(--hz-ink-2);
    font-size: 12px;
  }
  .hz-badge {
    padding: 2px 8px;
    border-radius: 4px;
    background: rgba(184, 230, 46, 0.25);
    color: var(--hz-ink);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .hz-badge-off {
    background: rgba(128, 128, 118, 0.25);
  }
  .hz-error {
    margin: 12px 0 0;
    color: var(--hz-ink-2);
    font-size: 13px;
  }
</style>
